//! Time-scale conversion functions: UTC ↔ TAI ↔ TT ↔ TDB.
//!
//! All internal representations use f64 seconds past J2000.0.
//!
//! Reference: NAIF Time Required Reading, IAU 1991 recommendations.
//! Implementation is original.

use crate::delta_t::{
    DeltaTModel, SmhFutureParabolaFamily, delta_t_seconds_with_model,
    smh_asymptotic_delta_t_seconds_for_jd_with_family,
};
use crate::diagnostics::{TimeDiagnostics, TimeWarning, TtUtcSource};
use crate::eop::{EopData, EopLookupOptions};
use crate::julian::{jd_to_tdb_seconds, tdb_seconds_to_jd};
use crate::lsk::LskData;

/// Options for hybrid UTC->TDB conversion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeConversionOptions {
    /// Emit fallback warnings in diagnostics.
    pub warn_on_fallback: bool,
    /// Delta-T model to use when outside LSK leap coverage.
    pub delta_t_model: DeltaTModel,
    /// For UTC epochs beyond EOP coverage, freeze DUT1 at last known value.
    pub freeze_future_dut1: bool,
    /// DUT1 fallback to use before EOP range (seconds).
    pub pre_range_dut1: f64,
    /// Future Delta-T transition strategy after leap-table coverage.
    pub future_delta_t_transition: FutureDeltaTTransition,
    /// Transition window (years) for bridge strategy.
    pub future_transition_years: f64,
    /// SMH future parabola-family selector used when post-EOP asymptotic
    /// fallback is active for `Smh2016WithPre720Quadratic` under bridge
    /// transition strategy.
    pub smh_future_family: SmhFutureParabolaFamily,
}

impl Default for TimeConversionOptions {
    fn default() -> Self {
        Self {
            warn_on_fallback: true,
            delta_t_model: DeltaTModel::default(),
            freeze_future_dut1: true,
            pre_range_dut1: 0.0,
            future_delta_t_transition: FutureDeltaTTransition::default(),
            future_transition_years: 100.0,
            smh_future_family: SmhFutureParabolaFamily::default(),
        }
    }
}

/// Future Delta-T transition behavior for UTC epochs after leap-table coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FutureDeltaTTransition {
    /// Legacy frozen-compatible behavior:
    /// `TT-UTC = last DELTA_AT + DELTA_T_A`.
    LegacyTtUtcBlend,
    /// Bridge from modern endpoint to selected asymptotic model over the
    /// configured transition window.
    BridgeFromModernEndpoint,
}

impl Default for FutureDeltaTTransition {
    fn default() -> Self {
        Self::LegacyTtUtcBlend
    }
}

/// UTC->TDB conversion policy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeConversionPolicy {
    /// Existing behavior: always use LSK leap-second lookup.
    StrictLsk,
    /// Hybrid behavior: use Delta-T fallback outside LSK coverage.
    HybridDeltaT(TimeConversionOptions),
}

impl Default for TimeConversionPolicy {
    fn default() -> Self {
        Self::HybridDeltaT(TimeConversionOptions::default())
    }
}

/// Result of UTC->TDB conversion with diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct UtcToTdbResult {
    pub tdb_seconds: f64,
    pub diagnostics: TimeDiagnostics,
}

/// Look up delta_AT (cumulative leap seconds) for a UTC epoch.
///
/// Uses binary search on the sorted leap-second table.
/// Returns 0.0 for epochs before the first entry (pre-1972).
pub fn lookup_delta_at(utc_seconds: f64, lsk: &LskData) -> f64 {
    let table = &lsk.leap_seconds;
    if table.is_empty() {
        return 0.0;
    }

    // Binary search for the last entry where epoch <= utc_seconds.
    match table.binary_search_by(|&(_, epoch)| epoch.partial_cmp(&utc_seconds).unwrap()) {
        Ok(i) => table[i].0,
        Err(0) => 0.0, // before first leap second
        Err(i) => table[i - 1].0,
    }
}

/// Convert UTC seconds past J2000 to TAI seconds past J2000.
pub fn utc_to_tai(utc_s: f64, lsk: &LskData) -> f64 {
    utc_s + lookup_delta_at(utc_s, lsk)
}

/// Convert TAI seconds past J2000 to TT (Terrestrial Time) seconds past J2000.
///
/// TT = TAI + 32.184 s (exact by IAU definition).
pub fn tai_to_tt(tai_s: f64, lsk: &LskData) -> f64 {
    tai_s + lsk.delta_t_a
}

/// Convert TT seconds past J2000 to TDB (Barycentric Dynamical Time) seconds past J2000.
///
/// Uses the NAIF one-term formulation with iterative Kepler solve:
/// ```text
/// M = M0 + M1 * TT_s
/// E = solve( E = M + EB * sin(E) )
/// TDB = TT + K * sin(E)
/// ```
///
/// Accuracy: ~30 microseconds vs the full relativistic treatment.
pub fn tt_to_tdb(tt_s: f64, lsk: &LskData) -> f64 {
    let m = lsk.m0 + lsk.m1 * tt_s;
    let e = solve_kepler(m, lsk.eb);
    tt_s + lsk.k * e.sin()
}

/// Convert TDB seconds past J2000 to TT seconds past J2000.
///
/// Inverts the TT→TDB formula. Since the correction is tiny (~1.6ms),
/// using TDB as proxy for TT in computing M introduces negligible error.
pub fn tdb_to_tt(tdb_s: f64, lsk: &LskData) -> f64 {
    // Solve fixed-point: TT = TDB - K*sin(E(TT)).
    let mut tt = tdb_s;
    for _ in 0..4 {
        let m = lsk.m0 + lsk.m1 * tt;
        let e = solve_kepler(m, lsk.eb);
        tt = tdb_s - lsk.k * e.sin();
    }
    tt
}

fn solve_kepler(m: f64, e: f64) -> f64 {
    // Fixed-point solve of E = M + e*sin(E).
    let mut ecc_anom = m;
    for _ in 0..4 {
        ecc_anom = m + e * ecc_anom.sin();
    }
    ecc_anom
}

/// Convert TT seconds past J2000 to TAI seconds past J2000.
pub fn tt_to_tai(tt_s: f64, lsk: &LskData) -> f64 {
    tt_s - lsk.delta_t_a
}

/// Full forward conversion: UTC seconds past J2000 → TDB seconds past J2000.
pub fn utc_to_tdb(utc_s: f64, lsk: &LskData) -> f64 {
    let tai = utc_to_tai(utc_s, lsk);
    let tt = tai_to_tt(tai, lsk);
    tt_to_tdb(tt, lsk)
}

/// Full forward conversion with configurable fallback policy and diagnostics.
pub fn utc_to_tdb_with_policy(
    utc_s: f64,
    lsk: &LskData,
    policy: TimeConversionPolicy,
) -> UtcToTdbResult {
    utc_to_tdb_with_policy_and_eop(utc_s, lsk, None, policy)
}

/// Full forward conversion with configurable fallback policy, diagnostics,
/// and optional EOP DUT1 support for Delta-T fallback branches.
pub fn utc_to_tdb_with_policy_and_eop(
    utc_s: f64,
    lsk: &LskData,
    eop: Option<&EopData>,
    policy: TimeConversionPolicy,
) -> UtcToTdbResult {
    match policy {
        TimeConversionPolicy::StrictLsk => {
            let delta_at = lookup_delta_at(utc_s, lsk);
            let tt_minus_utc = delta_at + lsk.delta_t_a;
            UtcToTdbResult {
                tdb_seconds: utc_to_tdb(utc_s, lsk),
                diagnostics: TimeDiagnostics {
                    warnings: Vec::new(),
                    tt_minus_utc_s: tt_minus_utc,
                    source: TtUtcSource::LskDeltaAt,
                },
            }
        }
        TimeConversionPolicy::HybridDeltaT(options) => {
            let table = &lsk.leap_seconds;
            let (tt_minus_utc, source, warnings) = if table.is_empty() {
                let (delta_t, segment) =
                    delta_t_seconds_with_model(tdb_seconds_to_jd(utc_s), options.delta_t_model);
                let (dut1, mut dut1_warnings) = fallback_dut1_seconds(utc_s, eop, options);
                if options.warn_on_fallback {
                    dut1_warnings.push(TimeWarning::DeltaTModelUsed {
                        model: options.delta_t_model,
                        segment,
                        assumed_dut1_seconds: dut1,
                    });
                }
                (delta_t + dut1, TtUtcSource::DeltaTModel, dut1_warnings)
            } else {
                let mut warnings = Vec::new();
                let (_, first_epoch) = table[0];
                let (last_delta_at, last_epoch) = table[table.len() - 1];

                if utc_s < first_epoch {
                    let (delta_t, segment) =
                        delta_t_seconds_with_model(tdb_seconds_to_jd(utc_s), options.delta_t_model);
                    let (dut1, mut dut1_warnings) = fallback_dut1_seconds(utc_s, eop, options);
                    warnings.append(&mut dut1_warnings);
                    if options.warn_on_fallback {
                        warnings.push(TimeWarning::LskPreRangeFallback {
                            utc_seconds: utc_s,
                            first_entry_utc_seconds: first_epoch,
                        });
                        warnings.push(TimeWarning::DeltaTModelUsed {
                            model: options.delta_t_model,
                            segment,
                            assumed_dut1_seconds: dut1,
                        });
                    }
                    (delta_t + dut1, TtUtcSource::DeltaTModel, warnings)
                } else if utc_s > last_epoch {
                    match options.future_delta_t_transition {
                        FutureDeltaTTransition::LegacyTtUtcBlend => {
                            if options.warn_on_fallback {
                                warnings.push(TimeWarning::LskFutureFrozen {
                                    utc_seconds: utc_s,
                                    last_entry_utc_seconds: last_epoch,
                                    used_delta_at_seconds: last_delta_at,
                                });
                            }
                            (
                                last_delta_at + lsk.delta_t_a,
                                TtUtcSource::LskDeltaAt,
                                warnings,
                            )
                        }
                        FutureDeltaTTransition::BridgeFromModernEndpoint => {
                            let transition_anchor_utc_s = eop
                                .and_then(|d| d.prediction_end_mjd())
                                .map(|mjd| jd_to_tdb_seconds(mjd + 2_400_000.5))
                                .map(|s| s.max(last_epoch))
                                .unwrap_or(last_epoch);

                            let (delta_t, segment) = delta_t_with_bridge_transition(
                                utc_s,
                                transition_anchor_utc_s,
                                options.delta_t_model,
                                options.smh_future_family,
                                options.future_transition_years,
                            );
                            let (dut1, mut dut1_warnings) =
                                fallback_dut1_seconds(utc_s, eop, options);
                            warnings.append(&mut dut1_warnings);
                            let tt_minus_utc = delta_t + dut1;
                            if options.warn_on_fallback {
                                warnings.push(TimeWarning::DeltaTModelUsed {
                                    model: options.delta_t_model,
                                    segment,
                                    assumed_dut1_seconds: dut1,
                                });
                            }
                            (tt_minus_utc, TtUtcSource::DeltaTModel, warnings)
                        }
                    }
                } else {
                    let delta_at = lookup_delta_at(utc_s, lsk);
                    (delta_at + lsk.delta_t_a, TtUtcSource::LskDeltaAt, warnings)
                }
            };

            let tt = utc_s + tt_minus_utc;
            UtcToTdbResult {
                tdb_seconds: tt_to_tdb(tt, lsk),
                diagnostics: TimeDiagnostics {
                    warnings,
                    tt_minus_utc_s: tt_minus_utc,
                    source,
                },
            }
        }
    }
}

fn delta_t_with_bridge_transition(
    utc_s: f64,
    transition_anchor_utc_s: f64,
    delta_t_model: DeltaTModel,
    smh_future_family: SmhFutureParabolaFamily,
    transition_years: f64,
) -> (f64, crate::delta_t::DeltaTSegment) {
    let jd_eval = tdb_seconds_to_jd(utc_s);

    if delta_t_model != DeltaTModel::Smh2016WithPre720Quadratic {
        return delta_t_seconds_with_model(jd_eval, delta_t_model);
    }

    if utc_s <= transition_anchor_utc_s {
        return delta_t_seconds_with_model(jd_eval, delta_t_model);
    }

    let (asym_eval, segment) =
        smh_asymptotic_delta_t_seconds_for_jd_with_family(jd_eval, smh_future_family);

    if transition_years <= 0.0 {
        return (asym_eval, segment);
    }

    let jd_anchor = tdb_seconds_to_jd(transition_anchor_utc_s);
    let (modern_end, _) = delta_t_seconds_with_model(jd_anchor, delta_t_model);
    let (asym_end, _) =
        smh_asymptotic_delta_t_seconds_for_jd_with_family(jd_anchor, smh_future_family);
    let window_seconds = transition_years * 365.25 * 86_400.0;
    let alpha = ((utc_s - transition_anchor_utc_s) / window_seconds).clamp(0.0, 1.0);
    let bridged = asym_eval + (modern_end - asym_end) * (1.0 - alpha);
    (bridged, segment)
}

fn fallback_dut1_seconds(
    utc_s: f64,
    eop: Option<&EopData>,
    options: TimeConversionOptions,
) -> (f64, Vec<TimeWarning>) {
    let Some(eop_data) = eop else {
        return (options.pre_range_dut1, Vec::new());
    };

    let jd_utc = tdb_seconds_to_jd(utc_s);
    let mjd_utc = jd_utc - 2_400_000.5;
    match eop_data.dut1_at_mjd_with_options(
        mjd_utc,
        EopLookupOptions {
            freeze_future_dut1: options.freeze_future_dut1,
            pre_range_dut1: options.pre_range_dut1,
            warn_on_fallback: options.warn_on_fallback,
        },
    ) {
        Ok(out) => (out.dut1_seconds, out.warnings),
        Err(_) => (options.pre_range_dut1, Vec::new()),
    }
}

/// Full inverse conversion: TDB seconds past J2000 → UTC seconds past J2000.
///
/// Uses iteration because the leap-second lookup depends on UTC,
/// which is what we're solving for. Converges in 2-3 iterations.
pub fn tdb_to_utc(tdb_s: f64, lsk: &LskData) -> f64 {
    let tt = tdb_to_tt(tdb_s, lsk);
    let tai = tt_to_tai(tt, lsk);

    // Iteratively solve for UTC: tai = utc + delta_at(utc)
    let mut utc = tai; // initial guess (off by leap seconds)
    for _ in 0..3 {
        let delta = lookup_delta_at(utc, lsk);
        utc = tai - delta;
    }
    utc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eop::EopData;
    use crate::julian::{calendar_to_jd, jd_to_tdb_seconds};
    use crate::lsk::parse_lsk;

    fn test_lsk() -> LskData {
        // Minimal LSK for testing.
        let content = r#"
\begindata
DELTET/DELTA_T_A       =   32.184
DELTET/K               =    1.657D-3
DELTET/EB              =    1.671D-2
DELTET/M               = (  6.239996   1.99096871D-7  )
DELTET/DELTA_AT        = ( 10,   @1972-JAN-1
                           37,   @2017-JAN-1  )
\begintext
"#;
        parse_lsk(content).unwrap()
    }

    #[test]
    fn delta_at_before_1972_is_zero() {
        let lsk = test_lsk();
        let utc = -1.0e10; // well before 1972
        assert_eq!(lookup_delta_at(utc, &lsk), 0.0);
    }

    #[test]
    fn delta_at_after_2017_is_37() {
        let lsk = test_lsk();
        let utc = 1.0e9; // well after 2017
        assert!((lookup_delta_at(utc, &lsk) - 37.0).abs() < 1e-10);
    }

    #[test]
    fn utc_to_tdb_at_j2000() {
        let lsk = test_lsk();
        // At J2000.0 (2000-Jan-01 12:00:00 TDB), UTC seconds = 0 would be
        // approximate. The exact relation is:
        // TDB ≈ UTC + 32 (leap secs at 2000) + 32.184 + tiny TDB correction
        let utc_s = 0.0;
        let tdb_s = utc_to_tdb(utc_s, &lsk);
        // Should be roughly 10 + 32.184 ≈ 42.184 (our test LSK only has 10s before 2017)
        // With the TDB correction (~1.6ms max), should be close to 42.184.
        let expected_approx = 10.0 + 32.184;
        assert!(
            (tdb_s - expected_approx).abs() < 0.01,
            "got {tdb_s}, expected ~{expected_approx}"
        );
    }

    #[test]
    fn tdb_utc_roundtrip() {
        let lsk = test_lsk();
        let original_utc = 5.0e8; // some epoch after 2017
        let tdb = utc_to_tdb(original_utc, &lsk);
        let recovered_utc = tdb_to_utc(tdb, &lsk);
        assert!(
            (original_utc - recovered_utc).abs() < 1e-9,
            "roundtrip error: {:.3e} s",
            (original_utc - recovered_utc).abs()
        );
    }

    #[test]
    fn tdb_correction_magnitude() {
        let lsk = test_lsk();
        // The TDB-TT correction should be at most ~1.66 ms.
        let tt = 0.0;
        let tdb = tt_to_tdb(tt, &lsk);
        let correction = (tdb - tt).abs();
        assert!(
            correction < 0.002,
            "TDB-TT correction {correction} exceeds 2ms"
        );
    }

    #[test]
    fn hybrid_future_freeze_warns() {
        let lsk = test_lsk();
        let utc = 1.0e9;
        let out = utc_to_tdb_with_policy(
            utc,
            &lsk,
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions::default()),
        );
        assert!(out.diagnostics.has_warnings());
        assert_eq!(out.diagnostics.source, TtUtcSource::LskDeltaAt);
    }

    #[test]
    fn hybrid_pre_range_uses_delta_t() {
        let lsk = test_lsk();
        let utc = -1.0e10;
        let out = utc_to_tdb_with_policy(
            utc,
            &lsk,
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions::default()),
        );
        assert!(out.diagnostics.has_warnings());
        assert_eq!(out.diagnostics.source, TtUtcSource::DeltaTModel);
    }

    #[test]
    fn strict_and_hybrid_match_for_lsk_covered_modern_date() {
        let lsk = test_lsk();
        let jd_utc = calendar_to_jd(2024, 6, 15.0);
        let utc_s = jd_to_tdb_seconds(jd_utc);

        let strict = utc_to_tdb_with_policy(utc_s, &lsk, TimeConversionPolicy::StrictLsk);
        let hybrid = utc_to_tdb_with_policy(
            utc_s,
            &lsk,
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions::default()),
        );

        assert!(
            (strict.tdb_seconds - hybrid.tdb_seconds).abs() < 1e-12,
            "covered-date mismatch strict={} hybrid={}",
            strict.tdb_seconds,
            hybrid.tdb_seconds
        );
        assert_eq!(hybrid.diagnostics.source, TtUtcSource::LskDeltaAt);
    }

    #[test]
    fn bridge_future_switches_to_delta_t_model() {
        let lsk = test_lsk();
        let utc = 1.0e9;
        let out = utc_to_tdb_with_policy(
            utc,
            &lsk,
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: true,
                delta_t_model: DeltaTModel::default(),
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::BridgeFromModernEndpoint,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::default(),
            }),
        );
        assert_eq!(out.diagnostics.source, TtUtcSource::DeltaTModel);
    }

    #[test]
    fn bridge_transition_blends_at_anchor() {
        let lsk = test_lsk();
        let (_, last_epoch) = lsk.leap_seconds[lsk.leap_seconds.len() - 1];
        let utc_s = last_epoch + 1.0;
        let out = utc_to_tdb_with_policy(
            utc_s,
            &lsk,
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::default(),
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::BridgeFromModernEndpoint,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::default(),
            }),
        );
        let (expected, _segment) = delta_t_with_bridge_transition(
            utc_s,
            last_epoch,
            DeltaTModel::default(),
            SmhFutureParabolaFamily::default(),
            100.0,
        );
        assert!(
            (out.diagnostics.tt_minus_utc_s - expected).abs() < 1e-6,
            "expected bridge TT-UTC near anchor; got {} vs {}",
            out.diagnostics.tt_minus_utc_s,
            expected
        );
    }

    #[test]
    fn bridge_uses_eop_prediction_end_as_transition_anchor() {
        let lsk = test_lsk();
        let eop_snippet = {
            let mk = |mjd: f64, flag: char, dut1: f64| {
                let mut line = vec![b' '; 70];
                let mjd_s = format!("{mjd:8.2}");
                line[7..15].copy_from_slice(mjd_s.as_bytes());
                line[57] = flag as u8;
                let dut1_s = format!("{dut1:10.7}");
                line[58..68].copy_from_slice(dut1_s.as_bytes());
                String::from_utf8(line).unwrap()
            };
            vec![mk(61000.0, 'I', 0.10), mk(62000.0, 'P', 0.12)].join("\n")
        };
        let eop = EopData::parse_finals(&eop_snippet).unwrap();
        // Between LSK end (~2017) and EOP prediction end (MJD 62000), bridge
        // should remain on the modern model branch, anchored at EOP end.
        let utc_s = jd_to_tdb_seconds(2_400_000.5 + 61500.0);
        let out = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::default(),
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::BridgeFromModernEndpoint,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::default(),
            }),
        );
        let jd_utc = tdb_seconds_to_jd(utc_s);
        let (delta_t, _segment) = delta_t_seconds_with_model(jd_utc, DeltaTModel::default());
        let dut1 = 0.11; // midpoint between MJD 61000 (0.10) and 62000 (0.12)
        let expected = delta_t + dut1;
        assert!(
            (out.diagnostics.tt_minus_utc_s - expected).abs() < 1e-6,
            "expected model-branch TT-UTC before EOP prediction end; got {} vs {}",
            out.diagnostics.tt_minus_utc_s,
            expected
        );
    }

    #[test]
    fn smh_future_uses_asymptotic_branch_after_transition_window() {
        let lsk = test_lsk();
        let eop_snippet = {
            let mk = |mjd: f64, flag: char, dut1: f64| {
                let mut line = vec![b' '; 70];
                let mjd_s = format!("{mjd:8.2}");
                line[7..15].copy_from_slice(mjd_s.as_bytes());
                line[57] = flag as u8;
                let dut1_s = format!("{dut1:10.7}");
                line[58..68].copy_from_slice(dut1_s.as_bytes());
                String::from_utf8(line).unwrap()
            };
            vec![mk(61000.0, 'I', 0.10), mk(62000.0, 'P', 0.12)].join("\n")
        };
        let eop = EopData::parse_finals(&eop_snippet).unwrap();
        let utc_s = jd_to_tdb_seconds(2_400_000.5 + 65000.0); // well after prediction tail
        let out = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::BridgeFromModernEndpoint,
                future_transition_years: 1.0,
                smh_future_family: SmhFutureParabolaFamily::default(),
            }),
        );
        let jd_utc = tdb_seconds_to_jd(utc_s);
        let (asym_dt, _seg) = crate::delta_t::smh_asymptotic_delta_t_seconds_for_jd_with_family(
            jd_utc,
            SmhFutureParabolaFamily::default(),
        );
        // Since transition window is only 1 year and date is far beyond anchor,
        // TT-UTC should be fully on asymptotic branch plus frozen DUT1.
        let expected = asym_dt + 0.12;
        assert!(
            (out.diagnostics.tt_minus_utc_s - expected).abs() < 1e-6,
            "expected asymptotic TT-UTC {} got {}",
            expected,
            out.diagnostics.tt_minus_utc_s
        );
    }

    #[test]
    fn smh_future_family_selection_changes_post_eop_asymptotic_value() {
        let lsk = test_lsk();
        let eop_snippet = {
            let mk = |mjd: f64, flag: char, dut1: f64| {
                let mut line = vec![b' '; 70];
                let mjd_s = format!("{mjd:8.2}");
                line[7..15].copy_from_slice(mjd_s.as_bytes());
                line[57] = flag as u8;
                let dut1_s = format!("{dut1:10.7}");
                line[58..68].copy_from_slice(dut1_s.as_bytes());
                String::from_utf8(line).unwrap()
            };
            vec![mk(61000.0, 'I', 0.10), mk(62000.0, 'P', 0.12)].join("\n")
        };
        let eop = EopData::parse_finals(&eop_snippet).unwrap();
        let utc_s = jd_to_tdb_seconds(2_400_000.5 + 65000.0);

        let out_c20 = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::BridgeFromModernEndpoint,
                future_transition_years: 1.0,
                smh_future_family: SmhFutureParabolaFamily::ConstantCMinus20,
            }),
        );
        let out_c17 = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::BridgeFromModernEndpoint,
                future_transition_years: 1.0,
                smh_future_family: SmhFutureParabolaFamily::ConstantCMinus17p52,
            }),
        );

        // Same epoch and DUT1: c=-17.52 should be exactly +2.48s vs c=-20.
        let d = out_c17.diagnostics.tt_minus_utc_s - out_c20.diagnostics.tt_minus_utc_s;
        assert!((d - 2.48).abs() < 1e-9, "expected +2.48s shift, got {d}");
    }

    #[test]
    fn legacy_strategy_ignores_future_family() {
        let lsk = test_lsk();
        let eop_snippet = {
            let mk = |mjd: f64, flag: char, dut1: f64| {
                let mut line = vec![b' '; 70];
                let mjd_s = format!("{mjd:8.2}");
                line[7..15].copy_from_slice(mjd_s.as_bytes());
                line[57] = flag as u8;
                let dut1_s = format!("{dut1:10.7}");
                line[58..68].copy_from_slice(dut1_s.as_bytes());
                String::from_utf8(line).unwrap()
            };
            vec![mk(61000.0, 'I', 0.10), mk(62000.0, 'P', 0.12)].join("\n")
        };
        let eop = EopData::parse_finals(&eop_snippet).unwrap();
        let utc_s = jd_to_tdb_seconds(2_400_000.5 + 65000.0);

        let out_addendum = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::LegacyTtUtcBlend,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::Addendum2020Piecewise,
            }),
        );
        let out_stephenson = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::LegacyTtUtcBlend,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::Stephenson1997,
            }),
        );
        assert!(
            (out_addendum.diagnostics.tt_minus_utc_s - out_stephenson.diagnostics.tt_minus_utc_s)
                .abs()
                < 1e-12,
            "future family must not change frozen-future TT-UTC"
        );
        assert_eq!(out_addendum.diagnostics.source, TtUtcSource::LskDeltaAt);
        assert_eq!(out_stephenson.diagnostics.source, TtUtcSource::LskDeltaAt);
    }

    #[test]
    fn future_blend_reaches_selected_model_after_100_years() {
        let lsk = test_lsk();
        let eop_snippet = {
            let mk = |mjd: f64, flag: char, dut1: f64| {
                let mut line = vec![b' '; 70];
                let mjd_s = format!("{mjd:8.2}");
                line[7..15].copy_from_slice(mjd_s.as_bytes());
                line[57] = flag as u8;
                let dut1_s = format!("{dut1:10.7}");
                line[58..68].copy_from_slice(dut1_s.as_bytes());
                String::from_utf8(line).unwrap()
            };
            vec![mk(61000.0, 'I', 0.10), mk(62000.0, 'P', 0.12)].join("\n")
        };
        let eop = EopData::parse_finals(&eop_snippet).unwrap();
        let anchor_utc_s = jd_to_tdb_seconds(2_400_000.5 + 62000.0);
        let utc_s = anchor_utc_s + 100.0 * 365.25 * 86_400.0 + 10.0;

        let out = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::BridgeFromModernEndpoint,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::Stephenson1997,
            }),
        );

        let jd_utc = tdb_seconds_to_jd(utc_s);
        let (asym_dt, _) = crate::delta_t::smh_asymptotic_delta_t_seconds_for_jd_with_family(
            jd_utc,
            SmhFutureParabolaFamily::Stephenson1997,
        );
        let expected = asym_dt + 0.12;
        assert!(
            (out.diagnostics.tt_minus_utc_s - expected).abs() < 1e-6,
            "expected full model TT-UTC after 100y blend window: {} vs {}",
            out.diagnostics.tt_minus_utc_s,
            expected
        );
        assert_eq!(out.diagnostics.source, TtUtcSource::DeltaTModel);
    }

    #[test]
    fn future_blend_reaches_stephenson2016_after_100_years() {
        let lsk = test_lsk();
        let eop_snippet = {
            let mk = |mjd: f64, flag: char, dut1: f64| {
                let mut line = vec![b' '; 70];
                let mjd_s = format!("{mjd:8.2}");
                line[7..15].copy_from_slice(mjd_s.as_bytes());
                line[57] = flag as u8;
                let dut1_s = format!("{dut1:10.7}");
                line[58..68].copy_from_slice(dut1_s.as_bytes());
                String::from_utf8(line).unwrap()
            };
            vec![mk(61000.0, 'I', 0.10), mk(62000.0, 'P', 0.12)].join("\n")
        };
        let eop = EopData::parse_finals(&eop_snippet).unwrap();
        let anchor_utc_s = jd_to_tdb_seconds(2_400_000.5 + 62000.0);
        let utc_s = anchor_utc_s + 100.0 * 365.25 * 86_400.0 + 10.0;

        let out = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::BridgeFromModernEndpoint,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::Stephenson2016,
            }),
        );

        let jd_utc = tdb_seconds_to_jd(utc_s);
        let (asym_dt, _) = crate::delta_t::smh_asymptotic_delta_t_seconds_for_jd_with_family(
            jd_utc,
            SmhFutureParabolaFamily::Stephenson2016,
        );
        let expected = asym_dt + 0.12;
        assert!(
            (out.diagnostics.tt_minus_utc_s - expected).abs() < 1e-6,
            "expected full Stephenson2016 TT-UTC after 100y blend window: {} vs {}",
            out.diagnostics.tt_minus_utc_s,
            expected
        );
        assert_eq!(out.diagnostics.source, TtUtcSource::DeltaTModel);
    }

    #[test]
    fn legacy_strategy_ignores_stephenson2016() {
        let lsk = test_lsk();
        let eop_snippet = {
            let mk = |mjd: f64, flag: char, dut1: f64| {
                let mut line = vec![b' '; 70];
                let mjd_s = format!("{mjd:8.2}");
                line[7..15].copy_from_slice(mjd_s.as_bytes());
                line[57] = flag as u8;
                let dut1_s = format!("{dut1:10.7}");
                line[58..68].copy_from_slice(dut1_s.as_bytes());
                String::from_utf8(line).unwrap()
            };
            vec![mk(61000.0, 'I', 0.10), mk(62000.0, 'P', 0.12)].join("\n")
        };
        let eop = EopData::parse_finals(&eop_snippet).unwrap();
        let utc_s = jd_to_tdb_seconds(2_400_000.5 + 65000.0);

        let out_addendum = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::LegacyTtUtcBlend,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::Addendum2020Piecewise,
            }),
        );
        let out_steph2016 = utc_to_tdb_with_policy_and_eop(
            utc_s,
            &lsk,
            Some(&eop),
            TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: false,
                delta_t_model: DeltaTModel::Smh2016WithPre720Quadratic,
                freeze_future_dut1: true,
                pre_range_dut1: 0.0,
                future_delta_t_transition: FutureDeltaTTransition::LegacyTtUtcBlend,
                future_transition_years: 100.0,
                smh_future_family: SmhFutureParabolaFamily::Stephenson2016,
            }),
        );
        assert!(
            (out_addendum.diagnostics.tt_minus_utc_s - out_steph2016.diagnostics.tt_minus_utc_s)
                .abs()
                < 1e-12
        );
        assert_eq!(out_addendum.diagnostics.source, TtUtcSource::LskDeltaAt);
        assert_eq!(out_steph2016.diagnostics.source, TtUtcSource::LskDeltaAt);
    }
}
