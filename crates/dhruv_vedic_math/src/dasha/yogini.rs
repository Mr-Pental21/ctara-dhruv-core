//! Yogini dasha engine (8 yoginis, 36-year cycle).
//!
//! Implements 6 computation tiers parallel to the nakshatra engine,
//! but using DashaEntity::Yogini entities instead of DashaEntity::Graha.

use crate::error::VedicError;

use super::balance::nakshatra_birth_balance;
use super::query::find_active_period;
use super::subperiod::{
    BirthChainLink, JD_MATCH_EPS, birth_chain, complete_level_anchored,
    generate_children_maybe_anchored,
};
use super::types::{
    DashaEntity, DashaHierarchy, DashaLevel, DashaPeriod, DashaSnapshot, DashaSystem,
    MAX_DASHA_LEVEL, MAX_PERIODS_PER_LEVEL,
};
use super::variation::{DashaVariationConfig, SubPeriodMethod};
use super::yogini_data::YoginiDashaConfig;

// ── Tier 0: Level-0 (Mahadasha) generation ───────────────────────────

/// Generate all level-0 (mahadasha) periods for Yogini dasha.
///
/// A single 36-year cycle by default; whole-cycle repetition can be
/// requested through `variation.cycles` or `variation.min_span_years`
/// (see [`DashaVariationConfig`]). `order` runs globally across cycles,
/// so `cycle = (order - 1) / sequence_len + 1`.
pub fn yogini_level0(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &YoginiDashaConfig,
    variation: &DashaVariationConfig,
) -> Vec<DashaPeriod> {
    let nak_idx = {
        let lon = crate::util::normalize_360(moon_sidereal_lon);
        (lon / crate::nakshatra::NAKSHATRA_SPAN_27).floor() as u8
    }
    .min(26);

    let start_yogini_idx = config.starting_yogini_idx(nak_idx) as usize;
    let entry_period = config.entry_period_days(nak_idx);
    let (_nak, balance_days, _frac) = nakshatra_birth_balance(moon_sidereal_lon, entry_period);

    let n = config.yogini_sequence.len();
    // First cycle covers the birth balance plus the remaining n-1 full periods.
    let first_cycle_span_days =
        balance_days + (config.total_period_days - config.periods_days[start_yogini_idx]);
    let cycle_count =
        variation.effective_cycle_count(1, first_cycle_span_days, config.total_period_days);
    let total_entries = (n * cycle_count as usize).min(MAX_PERIODS_PER_LEVEL);
    let mut periods = Vec::with_capacity(total_entries);
    let mut cursor = birth_jd;

    for offset in 0..total_entries {
        let seq_idx = (start_yogini_idx + offset) % n;
        let entity = config.yogini_sequence[seq_idx];
        let full_period = config.periods_days[seq_idx];

        let duration = if offset == 0 {
            balance_days
        } else {
            full_period
        };

        let end = cursor + duration;
        periods.push(DashaPeriod {
            entity,
            start_jd: cursor,
            end_jd: end,
            level: DashaLevel::Mahadasha,
            order: (offset as u16) + 1,
            parent_idx: 0,
        });
        cursor = end;
    }

    periods
}

/// Get the level-0 period for a specific yogini entity.
pub fn yogini_level0_entity(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &YoginiDashaConfig,
    variation: &DashaVariationConfig,
    entity: super::types::DashaEntity,
) -> Option<DashaPeriod> {
    let periods = yogini_level0(birth_jd, moon_sidereal_lon, config, variation);
    periods.into_iter().find(|p| p.entity == entity)
}

// ── Tier 1: Single child period ──────────────────────────────────────

/// Calculate one specific entity's sub-period within a parent.
///
/// `birth_jd` / `moon_sidereal_lon` identify the chart so the birth-clipped
/// period at each level gets children anchored on its full span (see
/// [`yogini_children`]).
#[allow(clippy::too_many_arguments)]
pub fn yogini_child_period(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    parent: &DashaPeriod,
    child_entity: DashaEntity,
    config: &YoginiDashaConfig,
    method: SubPeriodMethod,
    variation: &DashaVariationConfig,
) -> Option<DashaPeriod> {
    let child_level = parent.level.child_level()?;
    let children = yogini_children(
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
pub fn yogini_children(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    parent: &DashaPeriod,
    config: &YoginiDashaConfig,
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
/// its full span, as in [`yogini_children`].
pub fn yogini_complete_level(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    parent_level: &[DashaPeriod],
    config: &YoginiDashaConfig,
    child_level: DashaLevel,
    method: SubPeriodMethod,
    variation: &DashaVariationConfig,
) -> Result<Vec<DashaPeriod>, VedicError> {
    let parent_depth = (child_level as u8)
        .checked_sub(1)
        .ok_or(VedicError::InvalidInput(
            "child_level must be deeper than Mahadasha",
        ))?;
    let chain = yogini_birth_chain(birth_jd, moon_sidereal_lon, config, variation, parent_depth);
    let anchor = chain.get(parent_depth as usize);
    complete_level_inner(parent_level, config, child_level, method, anchor).map(|(v, _)| v)
}

fn complete_level_inner(
    parent_level: &[DashaPeriod],
    config: &YoginiDashaConfig,
    child_level: DashaLevel,
    method: SubPeriodMethod,
    anchor: Option<&BirthChainLink>,
) -> Result<(Vec<DashaPeriod>, Option<BirthChainLink>), VedicError> {
    let n = config.yogini_sequence.len();
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
pub fn yogini_hierarchy(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &YoginiDashaConfig,
    max_level: u8,
    variation: &DashaVariationConfig,
) -> Result<DashaHierarchy, VedicError> {
    let max_level = max_level.min(MAX_DASHA_LEVEL);
    let level0 = yogini_level0(birth_jd, moon_sidereal_lon, config, variation);
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
        system: DashaSystem::Yogini,
        birth_jd,
        levels,
    })
}

// ── Tier 5: Snapshot-only path ───────────────────────────────────────

/// Find active periods at query_jd without materializing full hierarchy.
pub fn yogini_snapshot(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &YoginiDashaConfig,
    query_jd: f64,
    max_level: u8,
    variation: &DashaVariationConfig,
) -> DashaSnapshot {
    let max_level = max_level.min(MAX_DASHA_LEVEL);
    let level0 = yogini_level0(birth_jd, moon_sidereal_lon, config, variation);
    let mut active_periods: Vec<DashaPeriod> = Vec::with_capacity((max_level + 1) as usize);

    let active_idx = match find_active_period(&level0, query_jd) {
        Some(idx) => idx,
        None => {
            return DashaSnapshot {
                system: DashaSystem::Yogini,
                query_jd,
                periods: active_periods,
            };
        }
    };
    active_periods.push(level0[active_idx]);

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
        system: DashaSystem::Yogini,
        query_jd,
        periods: active_periods,
    }
}

// ── Birth chain ──────────────────────────────────────────────────────

/// Chain link for the first mahadasha: the balance period and its full length.
fn level0_link(level0: &[DashaPeriod], config: &YoginiDashaConfig) -> Option<BirthChainLink> {
    let first = *level0.first()?;
    let pos = config
        .yogini_sequence
        .iter()
        .position(|&e| e == first.entity)?;
    Some(BirthChainLink {
        period: first,
        full_duration_days: config.periods_days[pos],
    })
}

fn yogini_birth_chain(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    config: &YoginiDashaConfig,
    variation: &DashaVariationConfig,
    upto_level: u8,
) -> Vec<BirthChainLink> {
    let level0 = yogini_level0(birth_jd, moon_sidereal_lon, config, variation);
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

fn anchor_for(
    birth_jd: f64,
    moon_sidereal_lon: f64,
    parent: &DashaPeriod,
    config: &YoginiDashaConfig,
    variation: &DashaVariationConfig,
) -> Option<BirthChainLink> {
    if (parent.start_jd - birth_jd).abs() >= JD_MATCH_EPS {
        return None;
    }
    let chain = yogini_birth_chain(
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
    use crate::dasha::types::{DAYS_PER_YEAR, DashaEntity};
    use crate::dasha::yogini_data::yogini_config;

    #[test]
    fn yogini_level0_at_ardra() {
        // Moon at Ardra start (5 * 13.333 = 66.667 deg) → Mangala yogini
        let cfg = yogini_config();
        let birth_jd = 2451545.0;
        let ardra_start = 5.0 * crate::nakshatra::NAKSHATRA_SPAN_27;
        let periods = yogini_level0(
            birth_jd,
            ardra_start,
            &cfg,
            &DashaVariationConfig::default(),
        );

        assert_eq!(periods.len(), 8);
        assert_eq!(periods[0].entity, DashaEntity::Yogini(0)); // Mangala
        let mangala_years = periods[0].duration_days() / DAYS_PER_YEAR;
        assert!(
            (mangala_years - 1.0).abs() < 0.01,
            "Mangala should be 1y, got {mangala_years}"
        );
    }

    #[test]
    fn yogini_total_span_36_at_nak_start() {
        let cfg = yogini_config();
        let birth_jd = 2451545.0;
        let ardra_start = 5.0 * crate::nakshatra::NAKSHATRA_SPAN_27;
        let periods = yogini_level0(
            birth_jd,
            ardra_start,
            &cfg,
            &DashaVariationConfig::default(),
        );

        let total_days: f64 = periods.iter().map(|p| p.duration_days()).sum();
        let total_years = total_days / DAYS_PER_YEAR;
        assert!(
            (total_years - 36.0).abs() < 1e-6,
            "Total should be 36y at nak start, got {total_years}"
        );
    }

    #[test]
    fn yogini_hierarchy_depth_2() {
        let cfg = yogini_config();
        let var = DashaVariationConfig::default();
        let h = yogini_hierarchy(2451545.0, 100.0, &cfg, 2, &var).unwrap();

        assert_eq!(h.system, DashaSystem::Yogini);
        assert_eq!(h.levels.len(), 3);
        assert_eq!(h.levels[0].len(), 8); // 8 mahadashas
        // 8*8 antardashas minus the 4 that elapsed before birth (the first
        // mahadasha is anchored on its full span).
        assert_eq!(h.levels[1].len(), 60);
        // Each retained antardasha has 8 children except the birth-clipped
        // one, which drops its pre-birth children too.
        let clipped = h.levels[2].iter().filter(|p| p.parent_idx == 0).count();
        assert!((1..=8).contains(&clipped));
        assert_eq!(h.levels[2].len(), 59 * 8 + clipped);
    }

    #[test]
    fn yogini_snapshot_matches_hierarchy() {
        let cfg = yogini_config();
        let var = DashaVariationConfig::default();
        let birth_jd = 2451545.0;
        let moon = 200.0;
        let query_jd = birth_jd + 1000.0;

        let h = yogini_hierarchy(birth_jd, moon, &cfg, 2, &var).unwrap();
        let snap = yogini_snapshot(birth_jd, moon, &cfg, query_jd, 2, &var);

        assert_eq!(snap.periods.len(), 3);
        for (level, snap_period) in snap.periods.iter().enumerate() {
            let active_in_h = h.levels[level]
                .iter()
                .find(|p| p.start_jd <= query_jd && query_jd < p.end_jd)
                .expect("should find active period in hierarchy");
            assert_eq!(snap_period.entity, active_in_h.entity);
        }
    }

    #[test]
    fn yogini_adjacent_no_gaps() {
        let cfg = yogini_config();
        let periods = yogini_level0(2451545.0, 150.0, &cfg, &DashaVariationConfig::default());
        for i in 1..periods.len() {
            assert!(
                (periods[i].start_jd - periods[i - 1].end_jd).abs() < 1e-10,
                "gap between periods {} and {}",
                i - 1,
                i
            );
        }
    }

    #[test]
    fn yogini_children_sum_to_parent() {
        let cfg = yogini_config();
        let var = DashaVariationConfig::default();
        let periods = yogini_level0(2451545.0, 0.0, &cfg, &var);
        let parent = &periods[0];
        let children = yogini_children(
            2451545.0,
            0.0,
            parent,
            &cfg,
            SubPeriodMethod::ProportionalFromParent,
            &var,
        );

        assert_eq!(children.len(), 8);
        assert!((children.last().unwrap().end_jd - parent.end_jd).abs() < 1e-10);
        assert!((children[0].start_jd - parent.start_jd).abs() < 1e-10);
    }

    #[test]
    fn yogini_balance_period_children_anchored() {
        // Moon mid-Ashwini: Ashwini -> yogini index 3 (Bhramari, 4y), 2y left.
        let cfg = yogini_config();
        let var = DashaVariationConfig::default();
        let birth_jd = 2451545.0;
        let moon = crate::nakshatra::NAKSHATRA_SPAN_27 / 2.0;
        let periods = yogini_level0(birth_jd, moon, &cfg, &var);
        let parent = &periods[0];
        assert!((parent.duration_days() / DAYS_PER_YEAR - 2.0).abs() < 1e-6);
        let children = yogini_children(
            birth_jd,
            moon,
            parent,
            &cfg,
            SubPeriodMethod::ProportionalFromParent,
            &var,
        );
        // Full 4y split 4/36 Bhramari .444, 5/36 Bhadrika .556, 6/36 Ulka
        // .667, 7/36 Siddha .778, 8/36 Sankata .889 (cum 3.333) -> birth at
        // 2y elapsed is inside Siddha (1.667..2.444).
        assert!(children.len() < 8);
        assert!((children[0].start_jd - parent.start_jd).abs() < 1e-10);
        assert!((children.last().unwrap().end_jd - parent.end_jd).abs() < 1e-10);
        let first_years = children[0].duration_days() / DAYS_PER_YEAR;
        assert!(
            (first_years - (2.444444 - 2.0)).abs() < 1e-4,
            "{first_years}"
        );
        // Hierarchy and snapshot agree with the anchored children.
        let h = yogini_hierarchy(birth_jd, moon, &cfg, 1, &var).unwrap();
        assert_eq!(h.levels[1].len(), 64 - (8 - children.len()));
        let snap = yogini_snapshot(birth_jd, moon, &cfg, birth_jd + 1.0, 1, &var);
        assert_eq!(snap.periods[1].entity, children[0].entity);
        assert!((snap.periods[1].end_jd - children[0].end_jd).abs() < 1e-9);
    }

    #[test]
    fn yogini_min_span_years_repeats_whole_cycles() {
        let cfg = yogini_config();
        let birth_jd = 2451545.0;
        let ardra_start = 5.0 * crate::nakshatra::NAKSHATRA_SPAN_27;
        let var = DashaVariationConfig {
            min_span_years: Some(100.0),
            ..DashaVariationConfig::default()
        };
        let periods = yogini_level0(birth_jd, ardra_start, &cfg, &var);

        // 36y cycles from a nakshatra start: 36 + 36 + 36 = 108 >= 100 → 3 cycles.
        assert_eq!(periods.len(), 24);
        let span_years = (periods.last().unwrap().end_jd - birth_jd) / DAYS_PER_YEAR;
        assert!(span_years >= 100.0, "span {span_years} should cover target");
        // Rotation repeats: entry 8 is the same yogini as entry 0, full period.
        assert_eq!(periods[8].entity, periods[0].entity);
        assert!((periods[8].start_jd - periods[7].end_jd).abs() < 1e-10);
        // Orders are global and contiguous.
        assert_eq!(periods[23].order, 24);
    }

    #[test]
    fn yogini_explicit_cycles_in_hierarchy() {
        let cfg = yogini_config();
        let var = DashaVariationConfig {
            cycles: Some(2),
            ..DashaVariationConfig::default()
        };
        let h = yogini_hierarchy(2451545.0, 100.0, &cfg, 1, &var).unwrap();
        assert_eq!(h.levels[0].len(), 16);
        // 16*8 minus the 4 antardashas that elapsed before birth.
        assert_eq!(h.levels[1].len(), 124);
    }

    #[test]
    fn yogini_children_start_from_parent_and_wrap() {
        let cfg = yogini_config();
        let parent = DashaPeriod {
            entity: DashaEntity::Yogini(7),
            start_jd: 2451545.0,
            end_jd: 2451545.0 + 320.0,
            level: DashaLevel::Mahadasha,
            order: 1,
            parent_idx: 0,
        };
        // Birth is elsewhere, so this parent is not on the birth chain.
        let children = yogini_children(
            2440000.0,
            0.0,
            &parent,
            &cfg,
            SubPeriodMethod::ProportionalFromParent,
            &DashaVariationConfig::default(),
        );

        assert_eq!(children.len(), 8);
        assert_eq!(children[0].entity, DashaEntity::Yogini(7));
        assert_eq!(children[1].entity, DashaEntity::Yogini(0));
        assert_eq!(children[2].entity, DashaEntity::Yogini(1));
        assert_eq!(children[7].entity, DashaEntity::Yogini(6));
    }
}
