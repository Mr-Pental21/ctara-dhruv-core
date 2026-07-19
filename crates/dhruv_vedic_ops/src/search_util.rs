//! Crate-level time-conversion policy and UTC->TDB helpers.

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
