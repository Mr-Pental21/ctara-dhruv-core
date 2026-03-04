//! Diagnostics and warning types for time conversion fallbacks.

use crate::delta_t::{DeltaTModel, DeltaTSegment};

/// Warning emitted when fallback logic is used.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeWarning {
    /// UTC epoch is beyond LSK leap-second coverage; last value was frozen.
    LskFutureFrozen {
        utc_seconds: f64,
        last_entry_utc_seconds: f64,
        used_delta_at_seconds: f64,
    },
    /// UTC epoch is before LSK leap-second coverage.
    LskPreRangeFallback {
        utc_seconds: f64,
        first_entry_utc_seconds: f64,
    },
    /// UTC epoch is beyond EOP coverage; last DUT1 value was frozen.
    EopFutureFrozen {
        mjd: f64,
        last_entry_mjd: f64,
        used_dut1_seconds: f64,
    },
    /// UTC epoch is before EOP coverage; fallback DUT1 was used.
    EopPreRangeFallback {
        mjd: f64,
        first_entry_mjd: f64,
        used_dut1_seconds: f64,
    },
    /// Delta-T polynomial model was used.
    DeltaTModelUsed {
        model: DeltaTModel,
        segment: DeltaTSegment,
        assumed_dut1_seconds: f64,
    },
}

/// Source used for `TT-UTC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtUtcSource {
    /// Computed from LSK leap seconds (`DELTA_AT + 32.184`).
    LskDeltaAt,
    /// Computed from Delta-T model (`(TT-UT1) + DUT1`).
    DeltaTModel,
}

/// Diagnostics produced by UTC->TDB conversion.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDiagnostics {
    pub warnings: Vec<TimeWarning>,
    pub tt_minus_utc_s: f64,
    pub source: TtUtcSource,
}

impl TimeDiagnostics {
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl std::fmt::Display for TimeWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LskFutureFrozen {
                utc_seconds,
                last_entry_utc_seconds,
                used_delta_at_seconds,
            } => write!(
                f,
                "UTC {utc_seconds:.3}s is beyond LSK coverage (last {last_entry_utc_seconds:.3}s); using frozen DELTA_AT={used_delta_at_seconds:.0}s. Update naif0012.tls."
            ),
            Self::LskPreRangeFallback {
                utc_seconds,
                first_entry_utc_seconds,
            } => write!(
                f,
                "UTC {utc_seconds:.3}s is before LSK coverage (first {first_entry_utc_seconds:.3}s); using fallback model."
            ),
            Self::EopFutureFrozen {
                mjd,
                last_entry_mjd,
                used_dut1_seconds,
            } => write!(
                f,
                "MJD {mjd:.3} is beyond EOP coverage (last {last_entry_mjd:.3}); using frozen DUT1={used_dut1_seconds:.6}s. Update finals2000A.all."
            ),
            Self::EopPreRangeFallback {
                mjd,
                first_entry_mjd,
                used_dut1_seconds,
            } => write!(
                f,
                "MJD {mjd:.3} is before EOP coverage (first {first_entry_mjd:.3}); using DUT1 fallback={used_dut1_seconds:.6}s."
            ),
            Self::DeltaTModelUsed {
                model,
                segment,
                assumed_dut1_seconds,
            } => write!(
                f,
                "Delta-T fallback used (model={model:?}, segment={segment:?}); assumed DUT1={assumed_dut1_seconds:.6}s."
            ),
        }
    }
}
