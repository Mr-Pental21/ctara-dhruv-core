//! Time-scale conversion functions: UTC ↔ TAI ↔ TT ↔ TDB.
//!
//! All internal representations use f64 seconds past J2000.0.
//!
//! Reference: NAIF Time Required Reading, IAU 1991 recommendations.
//! Implementation is original.

use crate::lsk::LskData;

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
/// Uses the NAIF one-term approximation:
/// ```text
/// M = M0 + M1 * TT_s
/// E = M + EB * sin(M)
/// TDB = TT + K * sin(E)
/// ```
///
/// Accuracy: ~30 microseconds vs the full relativistic treatment.
pub fn tt_to_tdb(tt_s: f64, lsk: &LskData) -> f64 {
    let m = lsk.m0 + lsk.m1 * tt_s;
    let e = m + lsk.eb * m.sin();
    tt_s + lsk.k * e.sin()
}

/// Convert TDB seconds past J2000 to TT seconds past J2000.
///
/// Inverts the TT→TDB formula. Since the correction is tiny (~1.6ms),
/// using TDB as proxy for TT in computing M introduces negligible error.
pub fn tdb_to_tt(tdb_s: f64, lsk: &LskData) -> f64 {
    let m = lsk.m0 + lsk.m1 * tdb_s;
    let e = m + lsk.eb * m.sin();
    tdb_s - lsk.k * e.sin()
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
}
