//! Time-scale conversions (UTC/TAI/TT/TDB) and leap-second support.
//!
//! This crate provides:
//! - Julian Date ↔ calendar conversions
//! - LSK (Leapseconds Kernel) file parsing
//! - UTC → TAI → TT → TDB conversion chain (and inverse)
//! - An `Epoch` type for type-safe TDB epoch handling

pub mod error;
pub mod julian;
pub mod lsk;
pub mod scales;
pub mod sidereal;

use std::path::Path;

pub use error::TimeError;
pub use julian::{
    calendar_to_jd, jd_to_calendar, jd_to_tdb_seconds, tdb_seconds_to_jd, J2000_JD,
    SECONDS_PER_DAY,
};
pub use lsk::LskData;

/// A loaded leap-second kernel, ready for time conversions.
#[derive(Debug, Clone)]
pub struct LeapSecondKernel {
    data: LskData,
}

impl LeapSecondKernel {
    /// Load an LSK file from a path.
    pub fn load(path: &Path) -> Result<Self, TimeError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse an LSK from its text content.
    pub fn parse(content: &str) -> Result<Self, TimeError> {
        let data = lsk::parse_lsk(content)?;
        Ok(Self { data })
    }

    /// Access the parsed LSK data.
    pub fn data(&self) -> &LskData {
        &self.data
    }

    /// Convert UTC seconds past J2000 to TDB seconds past J2000.
    pub fn utc_to_tdb(&self, utc_s: f64) -> f64 {
        scales::utc_to_tdb(utc_s, &self.data)
    }

    /// Convert TDB seconds past J2000 to UTC seconds past J2000.
    pub fn tdb_to_utc(&self, tdb_s: f64) -> f64 {
        scales::tdb_to_utc(tdb_s, &self.data)
    }
}

/// A TDB epoch represented as seconds past J2000.0.
///
/// This is the primary time type used throughout the engine.
/// It wraps an `f64` providing type safety and convenient conversions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Epoch {
    tdb_seconds: f64,
}

impl Epoch {
    /// Create an epoch from TDB seconds past J2000.0.
    pub fn from_tdb_seconds(s: f64) -> Self {
        Self { tdb_seconds: s }
    }

    /// Create an epoch from a Julian Date in TDB.
    pub fn from_jd_tdb(jd: f64) -> Self {
        Self {
            tdb_seconds: jd_to_tdb_seconds(jd),
        }
    }

    /// Create an epoch from a UTC calendar date using an LSK for leap seconds.
    pub fn from_utc(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: f64,
        lsk: &LeapSecondKernel,
    ) -> Self {
        let day_frac = day as f64 + hour as f64 / 24.0 + min as f64 / 1440.0 + sec / 86_400.0;
        let jd = calendar_to_jd(year, month, day_frac);
        let utc_s = jd_to_tdb_seconds(jd); // Note: this is UTC seconds past J2000, not TDB
        let tdb_s = lsk.utc_to_tdb(utc_s);
        Self {
            tdb_seconds: tdb_s,
        }
    }

    /// TDB seconds past J2000.0.
    pub fn as_tdb_seconds(self) -> f64 {
        self.tdb_seconds
    }

    /// Julian Date in TDB.
    pub fn as_jd_tdb(self) -> f64 {
        tdb_seconds_to_jd(self.tdb_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_from_jd_roundtrip() {
        let jd = 2_460_000.5;
        let epoch = Epoch::from_jd_tdb(jd);
        assert!((epoch.as_jd_tdb() - jd).abs() < 1e-12);
    }

    #[test]
    fn epoch_j2000_is_zero() {
        let epoch = Epoch::from_jd_tdb(J2000_JD);
        assert_eq!(epoch.as_tdb_seconds(), 0.0);
    }

    /// Integration test with real LSK file.
    #[test]
    fn load_real_lsk() {
        let lsk_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../kernels/data/naif0012.tls");

        if !lsk_path.exists() {
            eprintln!("Skipping: LSK not found at {}", lsk_path.display());
            return;
        }

        let lsk = LeapSecondKernel::load(&lsk_path).expect("should load naif0012.tls");

        // naif0012.tls should have 28 leap second entries (10s in 1972 through 37s in 2017).
        assert!(
            lsk.data().leap_seconds.len() >= 28,
            "expected >= 28 leap seconds, got {}",
            lsk.data().leap_seconds.len()
        );

        // Last entry should be 37s.
        let last = lsk.data().leap_seconds.last().unwrap();
        assert!(
            (last.0 - 37.0).abs() < 1e-10,
            "last leap second value: {}",
            last.0
        );
    }

    #[test]
    fn utc_tdb_roundtrip_with_real_lsk() {
        let lsk_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../kernels/data/naif0012.tls");

        if !lsk_path.exists() {
            eprintln!("Skipping: LSK not found");
            return;
        }

        let lsk = LeapSecondKernel::load(&lsk_path).unwrap();

        // 2024-Jun-15 00:00:00 UTC
        let utc_jd = calendar_to_jd(2024, 6, 15.0);
        let utc_s = jd_to_tdb_seconds(utc_jd);

        let tdb_s = lsk.utc_to_tdb(utc_s);
        let recovered_utc_s = lsk.tdb_to_utc(tdb_s);

        assert!(
            (utc_s - recovered_utc_s).abs() < 1e-9,
            "roundtrip error: {:.3e} s",
            (utc_s - recovered_utc_s).abs()
        );
    }
}
