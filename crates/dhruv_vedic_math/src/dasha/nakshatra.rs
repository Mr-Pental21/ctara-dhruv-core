//! Generic nakshatra-based dasha engine (serves all 10 nakshatra systems).
//!
//! Implements 6 computation tiers:
//! - Tier 0: Level-0 (mahadasha) generation
//! - Tier 1: Single child period
//! - Tier 2: All children of one parent
//! - Tier 3: Complete level from parent level
//! - Tier 4: Full hierarchy (levels 0..N)
//! - Tier 5: Snapshot-only path (no full materialization)

use crate::error::VedicError;

use super::balance::group_birth_balance;
use super::nakshatra_data::NakshatraDashaConfig;
use super::query::find_active_period;
use super::subperiod::{
    BirthChainLink, JD_MATCH_EPS, birth_chain, complete_level_anchored,
    generate_children_maybe_anchored,
};
use super::types::{
    DashaEntity, DashaHierarchy, DashaLevel, DashaPeriod, DashaSnapshot, MAX_DASHA_LEVEL,
    MAX_PERIODS_PER_LEVEL,
};
use super::variation::{DashaVariationConfig, SubPeriodMethod};

// ── Tier 0: Level-0 (Mahadasha) generation ───────────────────────────

/// Generate all level-0 (mahadasha) periods from birth inputs.
///
/// The number of whole-cycle repetitions defaults to the system's
/// `cycle_count` and can be overridden through `variation.cycles` or
/// `variation.min_span_years` (see [`DashaVariationConfig`]). Each cycle
/// repeats the full sequence in the same rotation; `order` runs globally
/// across cycles, so `cycle = (order - 1) / sequence_len + 1`.
pub fn nakshatra_level0(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &NakshatraDashaConfig,
    variation: &DashaVariationConfig,
) -> Vec<DashaPeriod> {
    let (nak_idx, elapsed_fraction, map) =
        birth_nakshatra_position(moon_sidereal_lon, config, variation);
    let start_graha_idx = map[nak_idx] as usize;
    let balance_days = group_birth_balance(
        nak_idx,
        elapsed_fraction,
        map,
        config.periods_days[start_graha_idx],
    );

    let n = config.graha_sequence.len();
    // First cycle covers the birth balance plus the remaining n-1 full periods.
    let first_cycle_span_days =
        balance_days + (config.total_period_days - config.periods_days[start_graha_idx]);
    let cycle_count = variation.effective_cycle_count(
        config.cycle_count,
        first_cycle_span_days,
        config.total_period_days,
    );
    let total_entries = (n * cycle_count as usize).min(MAX_PERIODS_PER_LEVEL);
    let mut periods = Vec::with_capacity(total_entries);
    let mut cursor = birth_jd;

    for cycle_offset in 0..total_entries {
        let seq_idx = (start_graha_idx + cycle_offset) % n;
        let graha = config.graha_sequence[seq_idx];
        let full_period = config.periods_days[seq_idx];

        let duration = if cycle_offset == 0 {
            balance_days
        } else {
            full_period
        };

        let end = cursor + duration;
        periods.push(DashaPeriod {
            entity: DashaEntity::Graha(graha),
            start_jd: cursor,
            end_jd: end,
            level: DashaLevel::Mahadasha,
            order: (cycle_offset as u16) + 1,
            parent_idx: 0,
        });
        cursor = end;
    }

    periods
}

/// Get the level-0 period for a specific entity.
pub fn nakshatra_level0_entity(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &NakshatraDashaConfig,
    variation: &DashaVariationConfig,
    entity: DashaEntity,
) -> Option<DashaPeriod> {
    let periods = nakshatra_level0(birth_jd, moon_sidereal_lon, config, variation);
    periods.into_iter().find(|p| p.entity == entity)
}

// ── Tier 1: Single child period ──────────────────────────────────────

/// Calculate one specific entity's sub-period within a parent period.
///
/// `birth_jd` / `moon_sidereal_lon` identify the chart so that the
/// birth-clipped period at each level gets children anchored on its full
/// span (see [`nakshatra_children`]).
#[allow(clippy::too_many_arguments)]
pub fn nakshatra_child_period(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    parent: &DashaPeriod,
    child_entity: DashaEntity,
    config: &NakshatraDashaConfig,
    method: SubPeriodMethod,
    variation: &DashaVariationConfig,
) -> Option<DashaPeriod> {
    let child_level = parent.level.child_level()?;
    let children = nakshatra_children(
        birth_jd,
        moon_sidereal_lon,
        parent,
        config,
        method,
        variation,
    );
    children
        .into_iter()
        .find(|c| c.entity == child_entity && c.level == child_level)
}

// ── Tier 2: All children of one parent ───────────────────────────────

/// Calculate all child periods for a single parent period.
///
/// When `parent` is the birth-clipped period of its level (it starts at
/// `birth_jd`), its children are laid out over the parent's full span,
/// children that ended before birth are dropped and the straddling one is
/// clipped to start at birth. Any other parent is split normally.
pub fn nakshatra_children(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    parent: &DashaPeriod,
    config: &NakshatraDashaConfig,
    method: SubPeriodMethod,
    variation: &DashaVariationConfig,
) -> Vec<DashaPeriod> {
    let child_level = match parent.level.child_level() {
        Some(l) => l,
        None => return Vec::new(),
    };
    let anchor = anchor_for(birth_jd, moon_sidereal_lon, parent, config, variation);
    let seq = config.entity_sequence();
    generate_children_maybe_anchored(
        parent,
        &seq,
        config.total_period_days,
        child_level,
        0,
        method,
        anchor.as_ref(),
    )
    .0
}

// ── Tier 3: Complete level from parent level ─────────────────────────

/// Calculate complete level N given all periods at level N-1.
///
/// The birth-clipped parent (if present in `parent_level`) is expanded on
/// its full span, as in [`nakshatra_children`].
pub fn nakshatra_complete_level(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    parent_level: &[DashaPeriod],
    config: &NakshatraDashaConfig,
    child_level: DashaLevel,
    method: SubPeriodMethod,
    variation: &DashaVariationConfig,
) -> Result<Vec<DashaPeriod>, VedicError> {
    let parent_depth = (child_level as u8)
        .checked_sub(1)
        .ok_or(VedicError::InvalidInput(
            "child_level must be deeper than Mahadasha",
        ))?;
    let chain = nakshatra_birth_chain(birth_jd, moon_sidereal_lon, config, variation, parent_depth);
    let anchor = chain.get(parent_depth as usize);
    complete_level_inner(parent_level, config, child_level, method, anchor).map(|(v, _)| v)
}

fn complete_level_inner(
    parent_level: &[DashaPeriod],
    config: &NakshatraDashaConfig,
    child_level: DashaLevel,
    method: SubPeriodMethod,
    anchor: Option<&BirthChainLink>,
) -> Result<(Vec<DashaPeriod>, Option<BirthChainLink>), VedicError> {
    let n = config.graha_sequence.len();
    if parent_level.len() * n > MAX_PERIODS_PER_LEVEL {
        return Err(VedicError::InvalidInput(
            "dasha level would exceed MAX_PERIODS_PER_LEVEL",
        ));
    }
    let seq = config.entity_sequence();
    Ok(complete_level_anchored(
        parent_level,
        &seq,
        config.total_period_days,
        child_level,
        method,
        anchor,
    ))
}

// ── Tier 4: Full hierarchy (levels 0..N) ─────────────────────────────

/// Calculate birth balance, then all levels from 0 to max_level.
pub fn nakshatra_hierarchy(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &NakshatraDashaConfig,
    max_level: u8,
    variation: &DashaVariationConfig,
) -> Result<DashaHierarchy, VedicError> {
    let max_level = max_level.min(MAX_DASHA_LEVEL);
    let level0 = nakshatra_level0(birth_jd, moon_sidereal_lon, config, variation);
    let mut anchor = level0_link(&level0, config);
    let mut levels: Vec<Vec<DashaPeriod>> = vec![level0];

    for depth in 1..=max_level {
        let child_level = match DashaLevel::from_u8(depth) {
            Some(l) => l,
            None => break,
        };
        let method = variation.method_for_level(depth - 1, config.default_method);
        let parent = &levels[(depth - 1) as usize];
        let (children, next) =
            complete_level_inner(parent, config, child_level, method, anchor.as_ref())?;
        anchor = next;
        levels.push(children);
    }

    Ok(DashaHierarchy {
        system: config.system,
        birth_jd,
        levels,
    })
}

// ── Tier 5: Snapshot-only path ───────────────────────────────────────

/// Find active periods at query_jd without materializing full hierarchy.
///
/// Generates only the chain of active periods from level 0 to max_level.
/// O(depth * sequence_length) instead of O(sequence_length^depth).
pub fn nakshatra_snapshot(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &NakshatraDashaConfig,
    query_jd: f64,
    max_level: u8,
    variation: &DashaVariationConfig,
) -> DashaSnapshot {
    let max_level = max_level.min(MAX_DASHA_LEVEL);
    let level0 = nakshatra_level0(birth_jd, moon_sidereal_lon, config, variation);
    let mut active_periods: Vec<DashaPeriod> = Vec::with_capacity((max_level + 1) as usize);

    // Find active mahadasha
    let active_idx = match find_active_period(&level0, query_jd) {
        Some(idx) => idx,
        None => {
            return DashaSnapshot {
                system: config.system,
                query_jd,
                periods: active_periods,
            };
        }
    };
    active_periods.push(level0[active_idx]);

    // Drill down through levels, following the birth chain while the active
    // period is the birth-clipped one.
    let seq = config.entity_sequence();
    let mut current_parent = level0[active_idx];
    let mut anchor = level0_link(&level0, config).filter(|l| l.matches(&current_parent));
    for depth in 1..=max_level {
        let child_level = match current_parent.level.child_level() {
            Some(l) => l,
            None => break,
        };
        let method = variation.method_for_level(depth - 1, config.default_method);
        let (children, next) = generate_children_maybe_anchored(
            &current_parent,
            &seq,
            config.total_period_days,
            child_level,
            0,
            method,
            anchor.as_ref(),
        );
        match find_active_period(&children, query_jd) {
            Some(idx) => {
                active_periods.push(children[idx]);
                current_parent = children[idx];
                anchor = next.filter(|l| l.matches(&current_parent));
            }
            None => break,
        }
    }

    DashaSnapshot {
        system: config.system,
        query_jd,
        periods: active_periods,
    }
}

// ── Birth position and birth chain ───────────────────────────────────

/// Moon's nakshatra index, fraction elapsed within it, and the applicable
/// nakshatra→graha map: the 28-scheme (unequal arcs around Abhijit) when the
/// system supports it and `variation.use_abhijit` is set, else the 27-scheme.
fn birth_nakshatra_position<'a>(
    moon_sidereal_lon: f64,
    config: &'a NakshatraDashaConfig,
    variation: &DashaVariationConfig,
) -> (usize, f64, &'a [u8]) {
    let lon = crate::util::normalize_360(moon_sidereal_lon);
    match (&config.nakshatra28_to_graha_idx, variation.use_abhijit) {
        (Some(map28), true) => {
            let info = crate::nakshatra::nakshatra28_from_longitude(lon);
            let idx = (info.nakshatra_index as usize).min(27);
            let (start, end) = crate::nakshatra::nakshatra_28_boundaries()[idx];
            let frac = ((lon - start) / (end - start)).clamp(0.0, 1.0);
            (idx, frac, &map28[..])
        }
        _ => {
            let span = crate::nakshatra::NAKSHATRA_SPAN_27;
            let idx = ((lon / span).floor() as usize).min(26);
            let frac = ((lon - idx as f64 * span) / span).clamp(0.0, 1.0);
            (idx, frac, &config.nakshatra_to_graha_idx[..])
        }
    }
}

/// Chain link for the first mahadasha: the balance period and its full length.
fn level0_link(level0: &[DashaPeriod], config: &NakshatraDashaConfig) -> Option<BirthChainLink> {
    let first = *level0.first()?;
    let pos = config
        .graha_sequence
        .iter()
        .position(|&g| DashaEntity::Graha(g) == first.entity)?;
    Some(BirthChainLink {
        period: first,
        full_duration_days: config.periods_days[pos],
    })
}

/// Birth chain (clipped period per level with its full length) down to
/// `upto_level` inclusive.
fn nakshatra_birth_chain(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &NakshatraDashaConfig,
    variation: &DashaVariationConfig,
    upto_level: u8,
) -> Vec<BirthChainLink> {
    let level0 = nakshatra_level0(birth_jd, moon_sidereal_lon, config, variation);
    let Some(link0) = level0_link(&level0, config) else {
        return Vec::new();
    };
    let seq = config.entity_sequence();
    birth_chain(
        link0,
        &seq,
        config.total_period_days,
        upto_level,
        &|depth| variation.method_for_level(depth, config.default_method),
    )
}

/// The chain link for `parent` when it is the birth-clipped period of its
/// level, else None.
fn anchor_for(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    parent: &DashaPeriod,
    config: &NakshatraDashaConfig,
    variation: &DashaVariationConfig,
) -> Option<BirthChainLink> {
    if (parent.start_jd - birth_jd).abs() >= JD_MATCH_EPS {
        return None;
    }
    let chain = nakshatra_birth_chain(
        birth_jd,
        moon_sidereal_lon,
        config,
        variation,
        parent.level as u8,
    );
    chain.into_iter().find(|l| l.matches(parent))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::nakshatra_data::vimshottari_config;
    use crate::dasha::types::DAYS_PER_YEAR;
    use crate::graha::Graha;

    #[test]
    fn vimshottari_ashwini_0_deg() {
        // Moon at 0 deg (Ashwini start) → Ketu mahadasha, full 7y, no balance deduction
        let cfg = vimshottari_config();
        let birth_jd = 2451545.0; // J2000
        let periods = nakshatra_level0(birth_jd, 0.0, &cfg, &DashaVariationConfig::default());

        assert_eq!(periods.len(), 9);
        assert_eq!(periods[0].entity, DashaEntity::Graha(Graha::Ketu));
        let ketu_years = periods[0].duration_days() / DAYS_PER_YEAR;
        assert!((ketu_years - 7.0).abs() < 1e-6);

        // Total should be 120 years
        let total_days: f64 = periods.iter().map(|p| p.duration_days()).sum();
        let total_years = total_days / DAYS_PER_YEAR;
        assert!((total_years - 120.0).abs() < 1e-6);
    }

    #[test]
    fn vimshottari_rohini_40_deg() {
        // Moon at 40 deg = start of Rohini → Chandra mahadasha with full balance
        let cfg = vimshottari_config();
        let birth_jd = 2451545.0;
        let periods = nakshatra_level0(birth_jd, 40.0, &cfg, &DashaVariationConfig::default());

        assert_eq!(periods[0].entity, DashaEntity::Graha(Graha::Chandra));
        // Rohini starts at exactly 40.0 deg, so full 10y balance
        let chandra_years = periods[0].duration_days() / DAYS_PER_YEAR;
        assert!((chandra_years - 10.0).abs() < 0.01);
    }

    #[test]
    fn vimshottari_partial_balance() {
        // Moon at 46.667 deg (mid-Rohini) → Chandra with ~5y balance
        let cfg = vimshottari_config();
        let birth_jd = 2451545.0;
        let mid_rohini = 40.0 + crate::nakshatra::NAKSHATRA_SPAN_27 / 2.0;
        let periods =
            nakshatra_level0(birth_jd, mid_rohini, &cfg, &DashaVariationConfig::default());

        assert_eq!(periods[0].entity, DashaEntity::Graha(Graha::Chandra));
        let chandra_years = periods[0].duration_days() / DAYS_PER_YEAR;
        assert!((chandra_years - 5.0).abs() < 0.1);
    }

    #[test]
    fn vimshottari_adjacent_periods_no_gaps() {
        let cfg = vimshottari_config();
        let periods = nakshatra_level0(2451545.0, 100.0, &cfg, &DashaVariationConfig::default());
        for i in 1..periods.len() {
            assert!(
                (periods[i].start_jd - periods[i - 1].end_jd).abs() < 1e-10,
                "gap between periods {} and {}",
                i - 1,
                i
            );
        }
    }

    const PROP: SubPeriodMethod = SubPeriodMethod::ProportionalFromParent;

    #[test]
    fn vimshottari_children_count() {
        let cfg = vimshottari_config();
        let var = DashaVariationConfig::default();
        let periods = nakshatra_level0(2451545.0, 0.0, &cfg, &var);
        let children = nakshatra_children(2451545.0, 0.0, &periods[0], &cfg, PROP, &var);
        assert_eq!(children.len(), 9);
        // First child should be same entity as parent (Ketu)
        assert_eq!(children[0].entity, DashaEntity::Graha(Graha::Ketu));
    }

    #[test]
    fn vimshottari_children_sum_to_parent() {
        let cfg = vimshottari_config();
        let var = DashaVariationConfig::default();
        let periods = nakshatra_level0(2451545.0, 0.0, &cfg, &var);
        let parent = &periods[0];
        let children = nakshatra_children(2451545.0, 0.0, parent, &cfg, PROP, &var);

        // Last child end == parent end
        assert!((children.last().unwrap().end_jd - parent.end_jd).abs() < 1e-10);
        // First child start == parent start
        assert!((children[0].start_jd - parent.start_jd).abs() < 1e-10);
    }

    #[test]
    fn balance_period_children_are_anchored_on_full_span() {
        // Moon mid-Rohini: Chandra mahadasha, 5y of 10y remain. Antardashas on
        // the full 10y: Chandra .833, Mangal .583, Rahu 1.5, Guru 1.333,
        // Shani 1.583 (4.25..5.833y) -> birth falls inside Shani.
        let cfg = vimshottari_config();
        let var = DashaVariationConfig::default();
        let birth_jd = 2451545.0;
        let moon = 40.0 + crate::nakshatra::NAKSHATRA_SPAN_27 / 2.0;
        let periods = nakshatra_level0(birth_jd, moon, &cfg, &var);
        let parent = &periods[0];
        let children = nakshatra_children(birth_jd, moon, parent, &cfg, PROP, &var);

        assert_eq!(
            children.len(),
            5,
            "Shani, Buddh, Ketu, Shukra, Surya remain"
        );
        assert_eq!(children[0].entity, DashaEntity::Graha(Graha::Shani));
        assert_eq!(
            children[0].order, 5,
            "order keeps the full-sequence position"
        );
        assert!((children[0].start_jd - parent.start_jd).abs() < 1e-10);
        let shani_years = children[0].duration_days() / DAYS_PER_YEAR;
        assert!(
            (shani_years - (5.8333333 - 5.0)).abs() < 1e-4,
            "{shani_years}"
        );
        // Buddh gets its full share of the 10y mahadasha.
        let buddh_years = children[1].duration_days() / DAYS_PER_YEAR;
        assert!((buddh_years - 17.0 / 120.0 * 10.0).abs() < 1e-9);
        assert!((children.last().unwrap().end_jd - parent.end_jd).abs() < 1e-10);
        // Children off the birth chain are untouched.
        let next = nakshatra_children(birth_jd, moon, &periods[1], &cfg, PROP, &var);
        assert_eq!(next.len(), 9);
        assert_eq!(next[0].entity, periods[1].entity);
    }

    #[test]
    fn anchoring_is_consistent_across_tiers() {
        let cfg = vimshottari_config();
        let var = DashaVariationConfig::default();
        let birth_jd = 2451545.0;
        let moon = 40.0 + crate::nakshatra::NAKSHATRA_SPAN_27 / 2.0;
        let h = nakshatra_hierarchy(birth_jd, moon, &cfg, 2, &var).unwrap();
        // 4 antardashas elapsed before birth.
        assert_eq!(h.levels[1].len(), 81 - 4);
        // Level 1 == children of each mahadasha.
        let mut expected = Vec::new();
        for p in &h.levels[0] {
            expected.extend(nakshatra_children(birth_jd, moon, p, &cfg, PROP, &var));
        }
        assert_eq!(h.levels[1].len(), expected.len());
        for (a, b) in h.levels[1].iter().zip(&expected) {
            assert_eq!(a.entity, b.entity);
            assert!((a.start_jd - b.start_jd).abs() < 1e-9);
            assert!((a.end_jd - b.end_jd).abs() < 1e-9);
        }
        // Complete level reproduces hierarchy level 2.
        let l2 = nakshatra_complete_level(
            birth_jd,
            moon,
            &h.levels[1],
            &cfg,
            DashaLevel::Pratyantardasha,
            PROP,
            &var,
        )
        .unwrap();
        assert_eq!(l2.len(), h.levels[2].len());
        // The clipped pratyantardasha at birth is itself anchored: its first
        // child starts at birth and does not necessarily carry order 1.
        assert!((h.levels[2][0].start_jd - birth_jd).abs() < 1e-9);
        // Snapshot at and just after birth matches the hierarchy.
        for q in [birth_jd, birth_jd + 10.0, birth_jd + 3000.0] {
            let snap = nakshatra_snapshot(birth_jd, moon, &cfg, q, 2, &var);
            assert_eq!(snap.periods.len(), 3);
            for (level, sp) in snap.periods.iter().enumerate() {
                let hp = h.levels[level]
                    .iter()
                    .find(|p| p.start_jd <= q && q < p.end_jd)
                    .unwrap();
                assert_eq!(sp.entity, hp.entity);
                assert!((sp.start_jd - hp.start_jd).abs() < 1e-9);
                assert!((sp.end_jd - hp.end_jd).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn ashtottari_group_balance() {
        use crate::dasha::nakshatra_data::ashtottari_config;
        let cfg = ashtottari_config();
        let span = crate::nakshatra::NAKSHATRA_SPAN_27;
        let no_abhijit = DashaVariationConfig {
            use_abhijit: false,
            ..DashaVariationConfig::default()
        };
        let years = |moon: f64, var: &DashaVariationConfig| {
            let p = nakshatra_level0(2451545.0, moon, &cfg, var);
            (p[0].entity, p[0].duration_days() / DAYS_PER_YEAR)
        };
        // Ardra start: Surya owns Ardra..Ashlesha (4 x 1.5y) -> full 6y.
        let (e, y) = years(5.0 * span, &no_abhijit);
        assert_eq!(e, DashaEntity::Graha(Graha::Surya));
        assert!((y - 6.0).abs() < 1e-9, "{y}");
        // Pushya start: Pushya + Ashlesha remain -> 3y.
        let (_, y) = years(7.0 * span, &no_abhijit);
        assert!((y - 3.0).abs() < 1e-9, "{y}");
        // Mid-Ashlesha: half a share -> 0.75y.
        let (_, y) = years(8.5 * span, &no_abhijit);
        assert!((y - 0.75).abs() < 1e-9, "{y}");
        // Ashlesha now belongs to Surya (was Chandra in the old map).
        let (e, _) = years(8.5 * span, &no_abhijit);
        assert_eq!(e, DashaEntity::Graha(Graha::Surya));
    }

    #[test]
    fn ashtottari_abhijit_uses_28_arcs_and_4_shares() {
        use crate::dasha::nakshatra_data::ashtottari_config;
        let cfg = ashtottari_config();
        let with = DashaVariationConfig::default(); // use_abhijit = true
        let without = DashaVariationConfig {
            use_abhijit: false,
            ..DashaVariationConfig::default()
        };
        let moon = 278.0; // inside Abhijit [276.667, 280.889)
        let p28 = nakshatra_level0(2451545.0, moon, &cfg, &with);
        let p27 = nakshatra_level0(2451545.0, moon, &cfg, &without);
        assert_eq!(p28[0].entity, DashaEntity::Graha(Graha::Shani));
        assert_eq!(p27[0].entity, DashaEntity::Graha(Graha::Shani));
        // 28-scheme: share 2.5y, Abhijit arc 4deg13'20", Shravana still ahead.
        let abhijit_span = (280.0 + 53.0 / 60.0 + 20.0 / 3600.0) - (276.0 + 40.0 / 60.0);
        let frac28 = (moon - (276.0 + 40.0 / 60.0)) / abhijit_span;
        let want28 = 2.5 * (1.0 - frac28) + 2.5;
        let got28 = p28[0].duration_days() / DAYS_PER_YEAR;
        assert!((got28 - want28).abs() < 1e-9, "{got28} vs {want28}");
        // 27-scheme: U.Ashadha (266.667..280), share 10/3, Shravana ahead.
        let span = crate::nakshatra::NAKSHATRA_SPAN_27;
        let frac27 = (moon - 20.0 * span) / span;
        let want27 = (10.0 / 3.0) * (1.0 - frac27) + 10.0 / 3.0;
        let got27 = p27[0].duration_days() / DAYS_PER_YEAR;
        assert!((got27 - want27).abs() < 1e-9, "{got27} vs {want27}");
        // Totals still 108y from the group start.
        let total: f64 = p28.iter().map(|p| p.duration_days()).sum::<f64>() / DAYS_PER_YEAR;
        assert!(total < 108.0 && total > 100.0);
        // Abhijit flag is ignored by systems without a 28-scheme map.
        let v = vimshottari_config();
        let a = nakshatra_level0(2451545.0, moon, &v, &with);
        let b = nakshatra_level0(2451545.0, moon, &v, &without);
        assert!((a[0].end_jd - b[0].end_jd).abs() < 1e-12);
    }

    #[test]
    fn vimshottari_rahu_children_start_from_rahu_and_wrap() {
        let cfg = vimshottari_config();
        let parent = DashaPeriod {
            entity: DashaEntity::Graha(Graha::Rahu),
            start_jd: 2451545.0,
            end_jd: 2451545.0 + 360.0,
            level: DashaLevel::Mahadasha,
            order: 1,
            parent_idx: 0,
        };
        // Birth is elsewhere, so this parent is not on the birth chain.
        let children = nakshatra_children(
            2440000.0,
            0.0,
            &parent,
            &cfg,
            PROP,
            &DashaVariationConfig::default(),
        );

        assert_eq!(children.len(), 9);
        assert_eq!(children[0].entity, DashaEntity::Graha(Graha::Rahu));
        assert_eq!(children[1].entity, DashaEntity::Graha(Graha::Guru));
        assert_eq!(children[2].entity, DashaEntity::Graha(Graha::Shani));
        assert_eq!(children[8].entity, DashaEntity::Graha(Graha::Mangal));
    }

    #[test]
    fn vimshottari_hierarchy_level_counts() {
        let cfg = vimshottari_config();
        let var = DashaVariationConfig::default();
        let h = nakshatra_hierarchy(2451545.0, 0.0, &cfg, 2, &var).unwrap();

        assert_eq!(h.levels.len(), 3); // 0, 1, 2
        assert_eq!(h.levels[0].len(), 9); // 9 mahadashas
        assert_eq!(h.levels[1].len(), 81); // 9*9 antardashas
        assert_eq!(h.levels[2].len(), 729); // 9*9*9 pratyantardashas
    }

    #[test]
    fn vimshottari_snapshot_matches_hierarchy() {
        let cfg = vimshottari_config();
        let var = DashaVariationConfig::default();
        let birth_jd = 2451545.0;
        let moon = 100.0;
        let query_jd = birth_jd + 1000.0; // ~2.7 years after birth

        let h = nakshatra_hierarchy(birth_jd, moon, &cfg, 2, &var).unwrap();
        let snap = nakshatra_snapshot(birth_jd, moon, &cfg, query_jd, 2, &var);

        assert_eq!(snap.periods.len(), 3);
        // Verify snapshot matches hierarchy by checking entities match
        for (level, snap_period) in snap.periods.iter().enumerate() {
            let active_in_h = h.levels[level]
                .iter()
                .find(|p| p.start_jd <= query_jd && query_jd < p.end_jd)
                .expect("should find active period in hierarchy");
            assert_eq!(snap_period.entity, active_in_h.entity);
            assert!((snap_period.start_jd - active_in_h.start_jd).abs() < 1e-6);
        }
    }

    #[test]
    fn vimshottari_level0_entity_lookup() {
        let cfg = vimshottari_config();
        let birth_jd = 2451545.0;
        let result = nakshatra_level0_entity(
            birth_jd,
            0.0,
            &cfg,
            &DashaVariationConfig::default(),
            DashaEntity::Graha(Graha::Ketu),
        );
        assert!(result.is_some());
        let period = result.unwrap();
        assert_eq!(period.entity, DashaEntity::Graha(Graha::Ketu));
    }

    #[test]
    fn vimshottari_explicit_cycles() {
        let cfg = vimshottari_config();
        let var = DashaVariationConfig {
            cycles: Some(2),
            ..DashaVariationConfig::default()
        };
        let periods = nakshatra_level0(2451545.0, 0.0, &cfg, &var);

        assert_eq!(periods.len(), 18);
        // Second cycle repeats the same rotation with full periods.
        assert_eq!(periods[9].entity, periods[0].entity);
        assert_eq!(periods[17].entity, periods[8].entity);
        // Orders are global and contiguous.
        assert_eq!(periods[17].order, 18);
        // No gaps across the cycle boundary.
        assert!((periods[9].start_jd - periods[8].end_jd).abs() < 1e-10);
        // Second-cycle first entry is the full period (7y Ketu), not the balance.
        let years = periods[9].duration_days() / DAYS_PER_YEAR;
        assert!((years - 7.0).abs() < 1e-6);
    }

    #[test]
    fn vimshottari_min_span_years() {
        let cfg = vimshottari_config();
        // Moon mid-Rohini: balance cuts the first cycle short of 120y, so
        // 200y target needs 2 cycles.
        let mid_rohini = 40.0 + crate::nakshatra::NAKSHATRA_SPAN_27 / 2.0;
        let var = DashaVariationConfig {
            min_span_years: Some(200.0),
            ..DashaVariationConfig::default()
        };
        let periods = nakshatra_level0(2451545.0, mid_rohini, &cfg, &var);

        assert_eq!(periods.len(), 18);
        let birth_jd = 2451545.0;
        let span_years = (periods.last().unwrap().end_jd - birth_jd) / DAYS_PER_YEAR;
        assert!(span_years >= 200.0, "span {span_years} should cover target");
    }

    #[test]
    fn vimshottari_min_span_within_default_is_unchanged() {
        let cfg = vimshottari_config();
        let var = DashaVariationConfig {
            min_span_years: Some(100.0),
            ..DashaVariationConfig::default()
        };
        let periods = nakshatra_level0(2451545.0, 0.0, &cfg, &var);
        // 120y first cycle already covers 100y.
        assert_eq!(periods.len(), 9);
    }

    #[test]
    fn cycles_propagate_to_hierarchy_and_snapshot() {
        let cfg = vimshottari_config();
        let var = DashaVariationConfig {
            cycles: Some(2),
            ..DashaVariationConfig::default()
        };
        let h = nakshatra_hierarchy(2451545.0, 0.0, &cfg, 1, &var).unwrap();
        assert_eq!(h.levels[0].len(), 18);
        assert_eq!(h.levels[1].len(), 162);

        // Snapshot inside the second cycle finds active periods.
        let query_jd = 2451545.0 + 130.0 * DAYS_PER_YEAR;
        let snap = nakshatra_snapshot(2451545.0, 0.0, &cfg, query_jd, 1, &var);
        assert_eq!(snap.periods.len(), 2);
        assert!(snap.periods[0].order > 9, "active mahadasha is in cycle 2");
    }

    #[test]
    fn max_periods_cap() {
        // Level 4 of a 9-entity system: 9^5 = 59049, under cap
        // But level 5 would be 9^6 = 531441, over cap
        // We test that level 4 works fine
        let cfg = vimshottari_config();
        let var = DashaVariationConfig::default();
        let h = nakshatra_hierarchy(2451545.0, 0.0, &cfg, 4, &var).unwrap();
        assert_eq!(h.levels.len(), 5); // levels 0-4
    }
}
