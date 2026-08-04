//! Generic zero-crossing search utility.
//!
//! Provides a reusable coarse-scan + bisection algorithm that finds where
//! a scalar function crosses zero. Used by conjunction, sankranti, and
//! other search modules.

use crate::error::SearchError;
use dhruv_core::Engine;
use dhruv_time::{
    EopKernel, TimeConversionPolicy, TimeWarning, UtcTime, calendar_to_jd, jd_to_tdb_seconds,
    tdb_seconds_to_jd,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, RwLock};

static TIME_POLICY: LazyLock<RwLock<TimeConversionPolicy>> =
    LazyLock::new(|| RwLock::new(TimeConversionPolicy::default()));
static LSK_PRE_RANGE_WARNED: AtomicBool = AtomicBool::new(false);
static LSK_FUTURE_WARNED: AtomicBool = AtomicBool::new(false);
static EOP_PRE_RANGE_WARNED: AtomicBool = AtomicBool::new(false);
static EOP_FUTURE_WARNED: AtomicBool = AtomicBool::new(false);
static DELTA_T_MODEL_WARNED: AtomicBool = AtomicBool::new(false);

/// Set crate-level UTC->TDB conversion policy used by date-driven search APIs.
pub fn set_time_conversion_policy(policy: TimeConversionPolicy) {
    match TIME_POLICY.write() {
        Ok(mut guard) => *guard = policy,
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            *guard = policy;
        }
    }
    dhruv_vedic_base::set_time_conversion_policy(policy);
}

/// Current crate-level UTC->TDB conversion policy.
pub fn time_conversion_policy() -> TimeConversionPolicy {
    match TIME_POLICY.read() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

/// Convert UTC date/time to JD(TDB) using current crate-level time policy.
pub(crate) fn utc_to_jd_tdb(engine: &Engine, utc: &UtcTime) -> f64 {
    utc_to_jd_tdb_with_eop(engine, None, utc)
}

/// Convert UTC date/time to JD(TDB) using current crate-level time policy and
/// optional EOP DUT1 support.
pub(crate) fn utc_to_jd_tdb_with_eop(
    engine: &Engine,
    eop: Option<&EopKernel>,
    utc: &UtcTime,
) -> f64 {
    let day_frac = utc.day as f64
        + utc.hour as f64 / 24.0
        + utc.minute as f64 / 1440.0
        + utc.second / 86_400.0;
    let jd_utc = calendar_to_jd(utc.year, utc.month, day_frac);
    let utc_s = jd_to_tdb_seconds(jd_utc);
    let out = engine
        .lsk()
        .utc_to_tdb_with_policy_and_eop(utc_s, eop, time_conversion_policy());
    for w in &out.diagnostics.warnings {
        emit_warning_once(w);
    }
    tdb_seconds_to_jd(out.tdb_seconds)
}

fn emit_warning_once(warning: &TimeWarning) {
    match warning {
        TimeWarning::LskPreRangeFallback { .. } => {
            if !LSK_PRE_RANGE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("Warning: {warning}");
            }
        }
        TimeWarning::LskFutureFrozen { .. } => {
            if !LSK_FUTURE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("Warning: {warning}");
            }
        }
        TimeWarning::EopPreRangeFallback { .. } => {
            if !EOP_PRE_RANGE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("Warning: {warning}");
            }
        }
        TimeWarning::EopFutureFrozen { .. } => {
            if !EOP_FUTURE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("Warning: {warning}");
            }
        }
        TimeWarning::DeltaTModelUsed { .. } => {
            if !DELTA_T_MODEL_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("Warning: {warning}");
            }
        }
    }
}

/// True when the error is the ephemeris coverage edge — the only engine
/// error that legitimately terminates an open-ended next/prev scan as
/// "no event found". All other engine errors must propagate.
pub(crate) fn is_coverage_edge(err: &SearchError) -> bool {
    matches!(
        err,
        SearchError::Engine(dhruv_core::EngineError::EpochOutOfRange { .. })
    )
}

/// Cursor nudge past a found event when resuming a forward scan (~86 ms).
pub(crate) const FORWARD_EPSILON_DAYS: f64 = 1e-6;
/// Cursor nudge past a found event when resuming a backward scan (~86 ms).
pub(crate) const BACKWARD_EPSILON_DAYS: f64 = 1e-6;
/// Residual ceiling for accepting a fixed-longitude root: rejects
/// boundary false positives introduced by the end-of-range clamp.
pub(crate) const FIXED_LONGITUDE_EVENT_TOLERANCE_DEG: f64 = 1e-3;

/// Normalize an angle to [-180, +180].
pub(crate) fn normalize_to_pm180(deg: f64) -> f64 {
    let mut d = deg % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d <= -180.0 {
        d += 360.0;
    }
    d
}

/// Check if a sign change is a genuine zero crossing vs a wrap-around discontinuity.
pub(crate) fn is_genuine_crossing(f_a: f64, f_b: f64) -> bool {
    f_a * f_b < 0.0 && (f_a - f_b).abs() < 270.0
}

/// Generic zero-crossing finder using coarse scan + bisection.
///
/// Scans from `jd_start` with the given `step` (positive = forward, negative = backward),
/// evaluating `f(t)` at each point. When a genuine sign change is detected, refines
/// via bisection to find the precise crossing time.
///
/// Returns the JD of the crossing, or `None` if no crossing is found within `max_steps`.
pub(crate) fn find_zero_crossing(
    f: &dyn Fn(f64) -> Result<f64, SearchError>,
    jd_start: f64,
    step: f64,
    max_steps: usize,
    max_iterations: u32,
    convergence_days: f64,
) -> Result<Option<f64>, SearchError> {
    let mut f_prev = f(jd_start)?;
    let mut t_prev = jd_start;

    for _ in 0..max_steps {
        let t_curr = t_prev + step;
        let f_curr = f(t_curr)?;

        if is_genuine_crossing(f_prev, f_curr) {
            // Ensure t_a < t_b for bisection
            let (mut t_a, mut f_a, mut t_b, _f_b) = if t_prev < t_curr {
                (t_prev, f_prev, t_curr, f_curr)
            } else {
                (t_curr, f_curr, t_prev, f_prev)
            };

            for _ in 0..max_iterations {
                let t_mid = 0.5 * (t_a + t_b);
                let f_mid = f(t_mid)?;

                if f_a * f_mid <= 0.0 {
                    t_b = t_mid;
                } else {
                    t_a = t_mid;
                    f_a = f_mid;
                }

                if (t_b - t_a).abs() < convergence_days {
                    break;
                }
            }

            return Ok(Some(0.5 * (t_a + t_b)));
        }

        t_prev = t_curr;
        f_prev = f_curr;
    }

    Ok(None)
}

/// Normalize a signed angular residual to (-period/2, +period/2].
pub(crate) fn normalize_to_half_period(deg: f64, period_deg: f64) -> f64 {
    let mut d = deg % period_deg;
    let half = period_deg / 2.0;
    if d > half {
        d -= period_deg;
    } else if d <= -half {
        d += period_deg;
    }
    d
}

/// Sign change that is a genuine periodic zero crossing rather than the
/// wrap-around discontinuity of the half-period normalization.
pub(crate) fn is_periodic_crossing(f_a: f64, f_b: f64, period_deg: f64) -> bool {
    f_a * f_b < 0.0 && (f_a - f_b).abs() < period_deg * 0.75
}

/// Coarse scan + bisection on a periodic residual function `f` (already
/// normalized to ±period/2), stepping by the signed `step` from `jd_start`.
///
/// The first sample's failure propagates; hitting the ephemeris coverage
/// edge on a later scan sample ends the scan with `Ok(None)` so window
/// sweeps near kernel edges yield partial results. Other engine errors
/// propagate, as do errors during bisection refinement (those samples lie
/// between two successfully evaluated brackets).
pub(crate) fn find_periodic_return(
    jd_start: f64,
    step: f64,
    max_steps: usize,
    max_iterations: u32,
    convergence_days: f64,
    period_deg: f64,
    f: &dyn Fn(f64) -> Result<f64, SearchError>,
) -> Result<Option<f64>, SearchError> {
    let mut f_prev = f(jd_start)?;
    let mut t_prev = jd_start;
    for _ in 0..max_steps {
        let t_curr = t_prev + step;
        let f_curr = match f(t_curr) {
            Ok(value) => value,
            Err(e) if is_coverage_edge(&e) => return Ok(None),
            Err(e) => return Err(e),
        };
        if is_periodic_crossing(f_prev, f_curr, period_deg) {
            let (mut t_a, mut f_a, mut t_b) = if t_prev < t_curr {
                (t_prev, f_prev, t_curr)
            } else {
                (t_curr, f_curr, t_prev)
            };
            for _ in 0..max_iterations {
                let t_mid = 0.5 * (t_a + t_b);
                let f_mid = f(t_mid)?;
                if f_a * f_mid <= 0.0 {
                    t_b = t_mid;
                } else {
                    t_a = t_mid;
                    f_a = f_mid;
                }
                if (t_b - t_a).abs() < convergence_days {
                    break;
                }
            }
            return Ok(Some(0.5 * (t_a + t_b)));
        }
        t_prev = t_curr;
        f_prev = f_curr;
    }
    Ok(None)
}

/// Find where `longitude_fn` reaches `target_deg` (mod `period_deg`)
/// scanning from `start_jd` toward `end_jd` in the given direction.
///
/// Samples beyond `end_jd` are clamped to a nonzero residual so the scan
/// cannot converge outside the window; a root whose residual exceeds
/// [`FIXED_LONGITUDE_EVENT_TOLERANCE_DEG`] (a boundary false positive) is
/// rejected. Inherits [`find_periodic_return`]'s coverage-edge tolerance.
#[allow(clippy::too_many_arguments)]
pub(crate) fn find_fixed_longitude_event(
    start_jd: f64,
    end_jd: f64,
    target_deg: f64,
    step_size_days: f64,
    max_iterations: u32,
    convergence_days: f64,
    longitude_fn: &dyn Fn(f64) -> Result<f64, SearchError>,
    period_deg: f64,
    direction: crate::conjunction_types::SearchDirection,
) -> Result<Option<f64>, SearchError> {
    use crate::conjunction_types::SearchDirection;
    let step = match direction {
        SearchDirection::Forward => step_size_days,
        SearchDirection::Backward => -step_size_days,
    };
    let total_days = (end_jd - start_jd).abs();
    let max_steps = (total_days / step_size_days).ceil() as usize + 1;
    let candidate = find_periodic_return(
        start_jd,
        step,
        max_steps,
        max_iterations,
        convergence_days,
        period_deg,
        &|jd| {
            if matches!(direction, SearchDirection::Forward) && jd > end_jd + FORWARD_EPSILON_DAYS {
                return Ok(1.0);
            }
            if matches!(direction, SearchDirection::Backward) && jd < end_jd - BACKWARD_EPSILON_DAYS
            {
                return Ok(1.0);
            }
            let longitude = longitude_fn(jd)?;
            Ok(normalize_to_half_period(longitude - target_deg, period_deg))
        },
    )?;
    let Some(event_jd) = candidate else {
        return Ok(None);
    };
    let residual_deg =
        normalize_to_half_period(longitude_fn(event_jd)? - target_deg, period_deg).abs();
    if residual_deg <= FIXED_LONGITUDE_EVENT_TOLERANCE_DEG {
        Ok(Some(event_jd))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conjunction_types::SearchDirection;

    #[test]
    fn normalize_basic() {
        assert!((normalize_to_pm180(0.0) - 0.0).abs() < 1e-10);
        assert!((normalize_to_pm180(180.0) - 180.0).abs() < 1e-10);
        assert!((normalize_to_pm180(270.0) - (-90.0)).abs() < 1e-10);
        assert!((normalize_to_pm180(360.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn genuine_crossing_positive() {
        assert!(is_genuine_crossing(5.0, -3.0));
        assert!(is_genuine_crossing(-10.0, 10.0));
    }

    #[test]
    fn wraparound_rejected() {
        assert!(!is_genuine_crossing(170.0, -170.0));
        assert!(!is_genuine_crossing(-170.0, 170.0));
    }

    #[test]
    fn find_linear_zero() {
        // f(x) = x - 10.3, zero at x=10.3 (off-grid so sign change is detected)
        let f = |t: f64| -> Result<f64, SearchError> { Ok(t - 10.3) };
        let result = find_zero_crossing(&f, 0.0, 1.0, 100, 50, 1e-10).unwrap();
        assert!(result.is_some());
        let t = result.unwrap();
        assert!((t - 10.3).abs() < 1e-8, "got {t}");
    }

    #[test]
    fn find_no_crossing() {
        // f(x) = x + 10, never crosses zero in [0..50]
        let f = |t: f64| -> Result<f64, SearchError> { Ok(t + 10.0) };
        let result = find_zero_crossing(&f, 0.0, 1.0, 50, 50, 1e-10).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn find_backward_crossing() {
        // f(x) = x - 5.7, zero at x=5.7, search backward from x=10
        let f = |t: f64| -> Result<f64, SearchError> { Ok(t - 5.7) };
        let result = find_zero_crossing(&f, 10.0, -1.0, 100, 50, 1e-10).unwrap();
        assert!(result.is_some());
        let t = result.unwrap();
        assert!((t - 5.7).abs() < 1e-8, "got {t}");
    }

    #[test]
    fn fixed_longitude_search_rejects_boundary_false_positive() {
        let result = find_fixed_longitude_event(
            0.0,
            1.0,
            20.0,
            2.0,
            32,
            1e-8,
            &|jd| Ok(10.0 + jd),
            360.0,
            SearchDirection::Forward,
        )
        .expect("search should not error");

        assert!(
            result.is_none(),
            "expected no event within tolerance {FIXED_LONGITUDE_EVENT_TOLERANCE_DEG}"
        );
    }

    #[test]
    fn fixed_longitude_search_finds_linear_root() {
        // longitude 10 + jd reaches 20.3 at jd = 10.3 (off-grid so the
        // sign change is detected between samples).
        let result = find_fixed_longitude_event(
            0.0,
            30.0,
            20.3,
            2.0,
            60,
            1e-10,
            &|jd| Ok(10.0 + jd),
            360.0,
            SearchDirection::Forward,
        )
        .expect("search should not error");
        let jd = result.expect("root expected");
        assert!((jd - 10.3).abs() < 1e-8, "got {jd}");
    }

    fn coverage_edge_error() -> SearchError {
        SearchError::Engine(dhruv_core::EngineError::EpochOutOfRange { epoch_tdb_jd: 0.0 })
    }

    #[test]
    fn periodic_return_first_sample_coverage_error_propagates() {
        let f = |_jd: f64| -> Result<f64, SearchError> { Err(coverage_edge_error()) };
        let result = find_periodic_return(0.0, 1.0, 10, 32, 1e-8, 360.0, &f);
        assert!(result.is_err(), "first-sample failure must propagate");
    }

    #[test]
    fn periodic_return_mid_scan_coverage_edge_ends_scan() {
        // Valid for jd < 5, coverage edge afterwards; no crossing before it.
        let f = |jd: f64| -> Result<f64, SearchError> {
            if jd < 5.0 {
                Ok(10.0)
            } else {
                Err(coverage_edge_error())
            }
        };
        let result = find_periodic_return(0.0, 1.0, 100, 32, 1e-8, 360.0, &f)
            .expect("coverage edge mid-scan must not error");
        assert!(result.is_none());
    }
}
