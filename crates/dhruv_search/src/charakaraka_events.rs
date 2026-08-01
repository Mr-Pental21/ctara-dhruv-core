//! Chara-karaka ranking-change events over a time range.
//!
//! Emits the exact moments the chara-karaka ranking changes for a scheme.
//! A ranking is piecewise constant in time; it can only change where one of
//! these root families has a zero:
//!
//! - **rashi ingress** of a ranked body: its degree-in-rashi resets, so its
//!   ranking key jumps;
//! - **pairwise degree-in-rashi crossing**: for two normally counted bodies
//!   `i`, `j` the keys tie when `L_i - L_j = 0 (mod 30)`. Rahu counts
//!   reversed (`effective = 30 - deg`), so a Rahu/classical tie is the SUM
//!   condition `L_Rahu + L_j = 0 (mod 30)` — the both-at-zero root of that
//!   lattice is spurious (effective 30 vs 0 is no tie) and is dropped by
//!   the actual-change check;
//! - **integer-degree bin boundary** (`MixedParashara` only): the 8↔7 mode
//!   predicate compares integer degrees-in-rashi of the classical bodies,
//!   so the mode can flip when any classical body crosses a 1° boundary.
//!
//! Every candidate root is verified by evaluating the full ranking just
//! before and just after it; only actual changes are emitted, and roots
//! closer together than the consolidation window collapse into one event.
//!
//! Longitudes come from the same computation as the per-moment
//! `charakaraka_for_date` (`graha_longitudes` with a config derived from
//! `SankrantiConfig`, honoring `node_mode`), so event snapshots agree with
//! the per-moment op by construction.
//!
//! Known floor: a double crossing that enters and leaves a lattice cell
//! entirely inside one scan-grid step (~6 h) is missed as a pair — the
//! excursion is bounded by slow-body station wobble (≲0.003°) or true-node
//! wobble (≲0.1°), and chain consistency of emitted events is preserved.

use dhruv_core::Engine;
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::{
    CharakarakaResult, CharakarakaRole, CharakarakaScheme, Graha, SAPTA_GRAHAS,
    charakarakas_from_longitudes,
};

use crate::error::SearchError;
use crate::jyotish::graha_longitudes;
use crate::jyotish_types::GrahaLongitudesConfig;
use crate::sankranti_types::SankrantiConfig;
use crate::search_util::{is_coverage_edge, normalize_to_pm180, utc_to_jd_tdb_with_eop};

/// Hard ceiling on events returned by a single `charakaraka_events` call.
pub const MAX_CHARAKARAKA_EVENTS: u32 = 50_000;

/// Scan-grid step in days. Chandra-involved family values move at most
/// ~16.5°/day, i.e. ≤ ~4.2° per step — well under the 180° unwrap limit,
/// and small enough that lattice crossings are enumerated exactly from the
/// unwrapped per-step delta.
const GRID_STEP_DAYS: f64 = 0.25;

/// Offset (days, ~0.43 s) on each side of a candidate root used to
/// evaluate the ranking before/after (mirrors `amsha_events`).
const PROBE_DAYS: f64 = 5e-6;

/// Candidate roots closer together than this merge into one event
/// (~1.7 s). Kept at ≥ 2× [`PROBE_DAYS`] so before/after probes of
/// neighboring events cannot interleave, which preserves the
/// `previous.after == next.before` chain invariant.
const CONSOLIDATION_DAYS: f64 = 2e-5;

/// Resume back-off applied to `next_from_utc` (~8.6 s) so a resumed sweep
/// re-brackets the first unemitted root.
const RESUME_BACKOFF_DAYS: f64 = 1e-4;

/// Chunk size for open-ended next/prev scans.
const NEXT_PREV_CHUNK_DAYS: f64 = 5.0;

/// Scan ceiling for next/prev. Ranking changes are Chandra-dominated and
/// occur every few hours to days; 60 days is far beyond the largest gap.
const NEXT_PREV_MAX_SCAN_DAYS: f64 = 60.0;

const ALL_ROLES: [CharakarakaRole; 9] = [
    CharakarakaRole::Atma,
    CharakarakaRole::Amatya,
    CharakarakaRole::Bhratri,
    CharakarakaRole::Matri,
    CharakarakaRole::Pitri,
    CharakarakaRole::Putra,
    CharakarakaRole::Gnati,
    CharakarakaRole::Dara,
    CharakarakaRole::MatriPutra,
];

/// What kind of root produced a ranking-change event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharakarakaEventTrigger {
    /// A pairwise effective-degree crossing (including Rahu sum crossings).
    DegreeCrossing,
    /// A ranked body entered a new rashi.
    RashiIngress,
    /// `MixedParashara` switched between the 8-karaka and 7-merged modes.
    SchemeModeChange,
}

impl CharakarakaEventTrigger {
    /// Stable numeric code (0, 1, 2 in declaration order).
    pub const fn code(self) -> u8 {
        match self {
            Self::DegreeCrossing => 0,
            Self::RashiIngress => 1,
            Self::SchemeModeChange => 2,
        }
    }
}

/// One chara-karaka ranking change.
///
/// `before`/`after` reuse the per-moment [`CharakarakaResult`] shape:
/// ordered entries carry role, graha, rank, longitude, degrees-in-rashi
/// and effective (Rahu-reversed) degrees, plus the scheme's effective
/// `used_eight_karakas` flag. The entry order is the documented ranking
/// order: effective degree desc, then raw degrees-in-rashi desc, then
/// graha index asc.
#[derive(Debug, Clone, PartialEq)]
pub struct CharakarakaChangeEvent {
    /// UTC time of the change.
    pub utc: UtcTime,
    /// JD(TDB) of the change.
    pub jd_tdb: f64,
    /// Root family that caused the change. When simultaneous roots merge,
    /// the priority is: mode flip observed → `SchemeModeChange`; any
    /// ingress root in the cluster → `RashiIngress`; else
    /// `DegreeCrossing`.
    pub trigger: CharakarakaEventTrigger,
    /// Ranking immediately before the change.
    pub before: CharakarakaResult,
    /// Ranking immediately after the change.
    pub after: CharakarakaResult,
    /// Roles whose assigned graha differs between `before` and `after`
    /// (a role present on only one side counts), sorted by role code.
    pub changed_roles: Vec<CharakarakaRole>,
}

/// Result of a `charakaraka_events` range sweep.
#[derive(Debug, Clone, PartialEq)]
pub struct CharakarakaEventsResult {
    /// Ranking-change events in ascending time order.
    pub events: Vec<CharakarakaChangeEvent>,
    /// True when the event budget was exhausted before `to_utc`.
    pub truncated: bool,
    /// Resume point when truncated: re-invoke with `from_utc =
    /// next_from_utc`. The seam event is re-found by the resumed sweep;
    /// consumers deduplicate on the event time.
    pub next_from_utc: Option<UtcTime>,
}

/// A root family: a scalar angle function crossing a fixed lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    /// `L_b` crossing multiples of 30°.
    Ingress(Graha),
    /// `L_i - L_j` crossing multiples of 30° (classical pair).
    Cross(Graha, Graha),
    /// `L_Rahu + L_j` crossing multiples of 30°.
    Sum(Graha),
    /// `L_b` crossing integer degrees (multiples of 30° excluded — those
    /// are `Ingress` roots). `MixedParashara` only.
    Bin(Graha),
}

impl Family {
    fn lattice_deg(self) -> f64 {
        match self {
            Family::Bin(_) => 1.0,
            _ => 30.0,
        }
    }

    fn value(self, lons: &[f64; 9]) -> f64 {
        match self {
            Family::Ingress(b) | Family::Bin(b) => lons[b.index() as usize],
            Family::Cross(i, j) => lons[i.index() as usize] - lons[j.index() as usize],
            Family::Sum(j) => {
                lons[Graha::Rahu.index() as usize] + lons[j.index() as usize]
            }
        }
    }
}

fn families_for_scheme(scheme: CharakarakaScheme) -> Vec<Family> {
    let include_rahu = matches!(
        scheme,
        CharakarakaScheme::Eight | CharakarakaScheme::MixedParashara
    );
    let mut families = Vec::new();
    for graha in SAPTA_GRAHAS {
        families.push(Family::Ingress(graha));
    }
    if include_rahu {
        families.push(Family::Ingress(Graha::Rahu));
    }
    for i in 0..SAPTA_GRAHAS.len() {
        for j in (i + 1)..SAPTA_GRAHAS.len() {
            families.push(Family::Cross(SAPTA_GRAHAS[i], SAPTA_GRAHAS[j]));
        }
    }
    if include_rahu {
        for graha in SAPTA_GRAHAS {
            families.push(Family::Sum(graha));
        }
    }
    if scheme == CharakarakaScheme::MixedParashara {
        for graha in SAPTA_GRAHAS {
            families.push(Family::Bin(graha));
        }
    }
    families
}

fn lons_config(aya_config: &SankrantiConfig) -> GrahaLongitudesConfig {
    GrahaLongitudesConfig::sidereal_with_model(
        aya_config.ayanamsha_system,
        aya_config.use_nutation,
        aya_config.precession_model,
        aya_config.reference_plane,
    )
    .with_outer_planets(false)
    .with_node_mode(aya_config.node_mode)
}

fn lons_at(
    engine: &Engine,
    jd_tdb: f64,
    config: &GrahaLongitudesConfig,
) -> Result<[f64; 9], SearchError> {
    Ok(graha_longitudes(engine, jd_tdb, config)?.longitudes)
}

fn ranking_at(
    engine: &Engine,
    jd_tdb: f64,
    config: &GrahaLongitudesConfig,
    scheme: CharakarakaScheme,
) -> Result<CharakarakaResult, SearchError> {
    let lons = lons_at(engine, jd_tdb, config)?;
    Ok(charakarakas_from_longitudes(&lons, scheme))
}

fn rankings_equal(a: &CharakarakaResult, b: &CharakarakaResult) -> bool {
    a.used_eight_karakas == b.used_eight_karakas
        && a.entries.len() == b.entries.len()
        && a.entries
            .iter()
            .zip(b.entries.iter())
            .all(|(x, y)| x.role == y.role && x.graha == y.graha)
}

fn changed_roles(before: &CharakarakaResult, after: &CharakarakaResult) -> Vec<CharakarakaRole> {
    let assignee = |result: &CharakarakaResult, role: CharakarakaRole| -> Option<Graha> {
        result
            .entries
            .iter()
            .find(|entry| entry.role == role)
            .map(|entry| entry.graha)
    };
    ALL_ROLES
        .iter()
        .copied()
        .filter(|&role| assignee(before, role) != assignee(after, role))
        .collect()
}

/// A candidate root: a time where some family crosses its lattice.
#[derive(Debug, Clone, Copy)]
struct Root {
    jd: f64,
    is_ingress: bool,
}

/// Enumerate the unwrapped lattice values crossed while moving from `g0`
/// by `delta` (shortest-path per-step delta, |delta| < 180).
fn lattice_targets(g0: f64, delta: f64, lattice: f64) -> Vec<f64> {
    let mut targets = Vec::new();
    if delta > 0.0 {
        let mut m = (g0 / lattice).floor() + 1.0;
        while m * lattice <= g0 + delta {
            targets.push(m * lattice);
            m += 1.0;
        }
    } else if delta < 0.0 {
        let mut m = (g0 / lattice).ceil() - 1.0;
        while m * lattice >= g0 + delta {
            targets.push(m * lattice);
            m -= 1.0;
        }
    }
    targets
}

/// True when an unwrapped lattice value lies on the 30° grid (an ingress
/// root, excluded from the `Bin` family).
fn on_rashi_grid(target: f64) -> bool {
    let r = target.rem_euclid(30.0);
    r < 1e-9 || r > 30.0 - 1e-9
}

/// Bisect the wrapped signed distance of `family`'s value to `target`
/// inside `[t_a, t_b]` (signs of the distance differ at the ends by
/// construction of the target enumeration).
fn bisect_root(
    engine: &Engine,
    lons_cfg: &GrahaLongitudesConfig,
    family: Family,
    target: f64,
    mut t_a: f64,
    mut t_b: f64,
    mut f_a: f64,
    aya_config: &SankrantiConfig,
) -> Result<f64, SearchError> {
    for _ in 0..aya_config.max_iterations {
        let t_mid = 0.5 * (t_a + t_b);
        let lons = lons_at(engine, t_mid, lons_cfg)?;
        let f_mid = normalize_to_pm180(family.value(&lons) - target);
        if f_a * f_mid <= 0.0 {
            t_b = t_mid;
        } else {
            t_a = t_mid;
            f_a = f_mid;
        }
        if (t_b - t_a).abs() < aya_config.convergence_days {
            break;
        }
    }
    Ok(0.5 * (t_a + t_b))
}

/// Collect all candidate roots in `[from_jd, to_jd]`, ascending.
fn collect_roots(
    engine: &Engine,
    lons_cfg: &GrahaLongitudesConfig,
    families: &[Family],
    from_jd: f64,
    to_jd: f64,
    aya_config: &SankrantiConfig,
) -> Result<Vec<Root>, SearchError> {
    let mut roots: Vec<Root> = Vec::new();
    let mut t_prev = from_jd;
    let mut lons_prev = lons_at(engine, t_prev, lons_cfg)?;
    while t_prev < to_jd {
        let t_curr = (t_prev + GRID_STEP_DAYS).min(to_jd);
        let lons_curr = lons_at(engine, t_curr, lons_cfg)?;
        for &family in families {
            let lattice = family.lattice_deg();
            let g0 = family.value(&lons_prev).rem_euclid(360.0);
            let g1 = family.value(&lons_curr).rem_euclid(360.0);
            let delta = normalize_to_pm180(g1 - g0);
            for target in lattice_targets(g0, delta, lattice) {
                if matches!(family, Family::Bin(_)) && on_rashi_grid(target) {
                    continue;
                }
                let f_a = normalize_to_pm180(g0 - target);
                let jd = bisect_root(
                    engine, lons_cfg, family, target, t_prev, t_curr, f_a, aya_config,
                )?;
                roots.push(Root {
                    jd,
                    is_ingress: matches!(family, Family::Ingress(_)),
                });
            }
        }
        t_prev = t_curr;
        lons_prev = lons_curr;
    }
    roots.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    Ok(roots)
}

/// Merge roots within [`CONSOLIDATION_DAYS`] into single candidates.
fn consolidate(roots: &[Root]) -> Vec<Root> {
    let mut out: Vec<Root> = Vec::new();
    for root in roots {
        match out.last_mut() {
            Some(last) if (root.jd - last.jd).abs() <= CONSOLIDATION_DAYS => {
                last.is_ingress |= root.is_ingress;
            }
            _ => out.push(*root),
        }
    }
    out
}

/// Evaluate a candidate root; `Ok(Some(event))` only on an actual change.
fn evaluate_candidate(
    engine: &Engine,
    lons_cfg: &GrahaLongitudesConfig,
    scheme: CharakarakaScheme,
    candidate: Root,
) -> Result<Option<CharakarakaChangeEvent>, SearchError> {
    let before = ranking_at(engine, candidate.jd - PROBE_DAYS, lons_cfg, scheme)?;
    let after = ranking_at(engine, candidate.jd + PROBE_DAYS, lons_cfg, scheme)?;
    if rankings_equal(&before, &after) {
        return Ok(None);
    }
    let trigger = if before.used_eight_karakas != after.used_eight_karakas {
        CharakarakaEventTrigger::SchemeModeChange
    } else if candidate.is_ingress {
        CharakarakaEventTrigger::RashiIngress
    } else {
        CharakarakaEventTrigger::DegreeCrossing
    };
    let changed = changed_roles(&before, &after);
    Ok(Some(CharakarakaChangeEvent {
        utc: UtcTime::from_jd_tdb(candidate.jd, engine.lsk()),
        jd_tdb: candidate.jd,
        trigger,
        before,
        after,
        changed_roles: changed,
    }))
}

/// Find every chara-karaka ranking change in `[from_utc, to_utc]`.
///
/// `max_events` caps the emitted events (`0` selects
/// [`MAX_CHARAKARAKA_EVENTS`]); on truncation `next_from_utc` gives the
/// resume point. Rankings are sidereal per `aya_config` (ayanamsha,
/// nutation, precession model, reference plane, and `node_mode` — the
/// same longitude computation as the per-moment charakaraka).
pub fn charakaraka_events(
    engine: &Engine,
    eop: &EopKernel,
    from_utc: &UtcTime,
    to_utc: &UtcTime,
    aya_config: &SankrantiConfig,
    scheme: CharakarakaScheme,
    max_events: u32,
) -> Result<CharakarakaEventsResult, SearchError> {
    aya_config
        .validate()
        .map_err(SearchError::InvalidConfig)?;
    let from_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), from_utc);
    let to_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), to_utc);
    if to_jd <= from_jd {
        return Err(SearchError::InvalidConfig("to_utc must be after from_utc"));
    }
    let cap = if max_events == 0 {
        MAX_CHARAKARAKA_EVENTS
    } else {
        max_events.min(MAX_CHARAKARAKA_EVENTS)
    };

    let lons_cfg = lons_config(aya_config);
    let families = families_for_scheme(scheme);

    let roots = collect_roots(engine, &lons_cfg, &families, from_jd, to_jd, aya_config)?;
    let candidates = consolidate(&roots);

    let mut events: Vec<CharakarakaChangeEvent> = Vec::new();
    let mut truncated = false;
    let mut next_from_utc = None;

    for candidate in candidates {
        let Some(event) = evaluate_candidate(engine, &lons_cfg, scheme, candidate)? else {
            continue;
        };
        if events.len() as u32 >= cap {
            truncated = true;
            next_from_utc = Some(UtcTime::from_jd_tdb(
                candidate.jd - RESUME_BACKOFF_DAYS,
                engine.lsk(),
            ));
            break;
        }
        events.push(event);
    }

    Ok(CharakarakaEventsResult {
        events,
        truncated,
        next_from_utc,
    })
}

/// First chara-karaka ranking change strictly after `at_utc`, or `None`
/// at the ephemeris coverage edge.
pub fn next_charakaraka_event(
    engine: &Engine,
    eop: &EopKernel,
    at_utc: &UtcTime,
    aya_config: &SankrantiConfig,
    scheme: CharakarakaScheme,
) -> Result<Option<CharakarakaChangeEvent>, SearchError> {
    aya_config
        .validate()
        .map_err(SearchError::InvalidConfig)?;
    let at_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), at_utc);
    let lons_cfg = lons_config(aya_config);
    let families = families_for_scheme(scheme);

    let mut chunk_start = at_jd;
    while chunk_start < at_jd + NEXT_PREV_MAX_SCAN_DAYS {
        let chunk_end = chunk_start + NEXT_PREV_CHUNK_DAYS;
        let roots = match collect_roots(
            engine, &lons_cfg, &families, chunk_start, chunk_end, aya_config,
        ) {
            Ok(roots) => roots,
            Err(err) if is_coverage_edge(&err) => return Ok(None),
            Err(err) => return Err(err),
        };
        for candidate in consolidate(&roots) {
            if candidate.jd <= at_jd {
                continue;
            }
            let evaluated = match evaluate_candidate(engine, &lons_cfg, scheme, candidate) {
                Ok(evaluated) => evaluated,
                Err(err) if is_coverage_edge(&err) => return Ok(None),
                Err(err) => return Err(err),
            };
            if let Some(event) = evaluated {
                return Ok(Some(event));
            }
        }
        chunk_start = chunk_end;
    }
    Ok(None)
}

/// Last chara-karaka ranking change strictly before `at_utc`, or `None`
/// at the ephemeris coverage edge.
pub fn prev_charakaraka_event(
    engine: &Engine,
    eop: &EopKernel,
    at_utc: &UtcTime,
    aya_config: &SankrantiConfig,
    scheme: CharakarakaScheme,
) -> Result<Option<CharakarakaChangeEvent>, SearchError> {
    aya_config
        .validate()
        .map_err(SearchError::InvalidConfig)?;
    let at_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), at_utc);
    let lons_cfg = lons_config(aya_config);
    let families = families_for_scheme(scheme);

    let mut chunk_end = at_jd;
    while chunk_end > at_jd - NEXT_PREV_MAX_SCAN_DAYS {
        let chunk_start = chunk_end - NEXT_PREV_CHUNK_DAYS;
        let roots = match collect_roots(
            engine, &lons_cfg, &families, chunk_start, chunk_end, aya_config,
        ) {
            Ok(roots) => roots,
            Err(err) if is_coverage_edge(&err) => return Ok(None),
            Err(err) => return Err(err),
        };
        for candidate in consolidate(&roots).into_iter().rev() {
            if candidate.jd >= at_jd {
                continue;
            }
            let evaluated = match evaluate_candidate(engine, &lons_cfg, scheme, candidate) {
                Ok(evaluated) => evaluated,
                Err(err) if is_coverage_edge(&err) => return Ok(None),
                Err(err) => return Err(err),
            };
            if let Some(event) = evaluated {
                return Ok(Some(event));
            }
        }
        chunk_end = chunk_start;
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lattice_targets_forward_single() {
        let targets = lattice_targets(29.0, 2.0, 30.0);
        assert_eq!(targets, vec![30.0]);
    }

    #[test]
    fn lattice_targets_backward_single() {
        let targets = lattice_targets(0.5, -1.0, 30.0);
        assert_eq!(targets, vec![0.0]);
    }

    #[test]
    fn lattice_targets_no_crossing() {
        assert!(lattice_targets(10.0, 5.0, 30.0).is_empty());
        assert!(lattice_targets(10.0, 0.0, 30.0).is_empty());
    }

    #[test]
    fn lattice_targets_multiple_bins() {
        let targets = lattice_targets(10.2, 3.5, 1.0);
        assert_eq!(targets, vec![11.0, 12.0, 13.0]);
    }

    #[test]
    fn lattice_targets_backward_bins() {
        let targets = lattice_targets(10.2, -2.5, 1.0);
        assert_eq!(targets, vec![10.0, 9.0, 8.0]);
    }

    #[test]
    fn lattice_targets_exact_start_excluded() {
        // A value exactly on the lattice is not re-reported as a crossing
        // in either direction (the previous grid interval already found it
        // as its inclusive end).
        assert!(lattice_targets(30.0, 2.0, 30.0).is_empty());
        assert!(lattice_targets(30.0, -2.0, 30.0).is_empty());
        // Crossing at the inclusive end of the span is reported.
        assert_eq!(lattice_targets(28.5, 1.5, 30.0), vec![30.0]);
    }

    #[test]
    fn rashi_grid_detection() {
        assert!(on_rashi_grid(30.0));
        assert!(on_rashi_grid(0.0));
        assert!(on_rashi_grid(330.0));
        assert!(!on_rashi_grid(29.0));
        assert!(!on_rashi_grid(31.0));
    }

    #[test]
    fn family_counts_per_scheme() {
        assert_eq!(families_for_scheme(CharakarakaScheme::Eight).len(), 36);
        assert_eq!(
            families_for_scheme(CharakarakaScheme::SevenNoPitri).len(),
            28
        );
        assert_eq!(
            families_for_scheme(CharakarakaScheme::SevenPkMergedMk).len(),
            28
        );
        assert_eq!(
            families_for_scheme(CharakarakaScheme::MixedParashara).len(),
            43
        );
    }

    #[test]
    fn consolidation_merges_close_roots() {
        let roots = vec![
            Root {
                jd: 100.0,
                is_ingress: false,
            },
            Root {
                jd: 100.0 + CONSOLIDATION_DAYS / 2.0,
                is_ingress: true,
            },
            Root {
                jd: 100.5,
                is_ingress: false,
            },
        ];
        let merged = consolidate(&roots);
        assert_eq!(merged.len(), 2);
        assert!(merged[0].is_ingress);
        assert!(!merged[1].is_ingress);
    }

    #[test]
    fn changed_roles_diffs_by_role() {
        let lons_a: [f64; 9] = [29.0, 28.0, 27.0, 26.0, 25.0, 24.0, 23.0, 29.0, 15.0];
        let mut lons_b = lons_a;
        // Swap Surya and Chandra ranks.
        lons_b[0] = 28.0;
        lons_b[1] = 29.0;
        let before = charakarakas_from_longitudes(&lons_a, CharakarakaScheme::SevenNoPitri);
        let after = charakarakas_from_longitudes(&lons_b, CharakarakaScheme::SevenNoPitri);
        let changed = changed_roles(&before, &after);
        assert_eq!(
            changed,
            vec![CharakarakaRole::Atma, CharakarakaRole::Amatya]
        );
    }

    #[test]
    fn changed_roles_covers_mode_flip() {
        let lons_seven: [f64; 9] = [29.5, 28.4, 27.3, 26.2, 25.1, 24.0, 23.9, 29.0, 15.0];
        let mut lons_eight = lons_seven;
        // Force an integer-degree tie (Chandra and Mangal both in bin 27).
        lons_eight[1] = 27.9;
        lons_eight[2] = 27.2;
        let before =
            charakarakas_from_longitudes(&lons_seven, CharakarakaScheme::MixedParashara);
        let after =
            charakarakas_from_longitudes(&lons_eight, CharakarakaScheme::MixedParashara);
        assert!(!before.used_eight_karakas);
        assert!(after.used_eight_karakas);
        let changed = changed_roles(&before, &after);
        // Matri and Putra exist only in the 8 set, MatriPutra only in the
        // merged 7 set — all three must be flagged. Pitri exists in both
        // and keeps Guru here, so it must NOT be flagged.
        assert!(changed.contains(&CharakarakaRole::Matri));
        assert!(changed.contains(&CharakarakaRole::Putra));
        assert!(changed.contains(&CharakarakaRole::MatriPutra));
        assert!(!changed.contains(&CharakarakaRole::Pitri));
    }

    #[test]
    fn rankings_equal_ignores_degrees() {
        let lons_a: [f64; 9] = [29.0, 28.0, 27.0, 26.0, 25.0, 24.0, 23.0, 29.0, 15.0];
        let mut lons_b = lons_a;
        for lon in lons_b.iter_mut() {
            *lon += 0.1;
        }
        let a = charakarakas_from_longitudes(&lons_a, CharakarakaScheme::Eight);
        let b = charakarakas_from_longitudes(&lons_b, CharakarakaScheme::Eight);
        assert!(rankings_equal(&a, &b));
    }

    #[test]
    fn trigger_codes_are_stable() {
        assert_eq!(CharakarakaEventTrigger::DegreeCrossing.code(), 0);
        assert_eq!(CharakarakaEventTrigger::RashiIngress.code(), 1);
        assert_eq!(CharakarakaEventTrigger::SchemeModeChange.code(), 2);
    }
}
