//! Birth balance calculation for nakshatra-based dasha systems.
//!
//! The birth balance is the remaining period of the first mahadasha at birth,
//! computed from the Moon's position within its nakshatra.

use crate::nakshatra::NAKSHATRA_SPAN_27;
use crate::util::normalize_360;

/// Compute nakshatra birth balance for a nakshatra-based dasha system.
///
/// Returns `(nakshatra_index, balance_days, elapsed_fraction)`:
/// - `nakshatra_index`: 0-based index (0=Ashwini..26=Revati) of the Moon's nakshatra
/// - `balance_days`: remaining days in the starting graha's period
/// - `elapsed_fraction`: fraction of nakshatra already traversed [0, 1)
pub fn nakshatra_birth_balance(
    moon_sidereal_lon: f64,
    entry_period_days: f64,
) -> (u8, f64, f64) {
    let lon = normalize_360(moon_sidereal_lon);
    let nak_idx = (lon / NAKSHATRA_SPAN_27).floor() as u8;
    let nak_idx = nak_idx.min(26);
    let position_in_nak = lon - (nak_idx as f64) * NAKSHATRA_SPAN_27;
    let elapsed_fraction = position_in_nak / NAKSHATRA_SPAN_27;
    let balance_days = entry_period_days * (1.0 - elapsed_fraction);
    (nak_idx, balance_days, elapsed_fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_at_start_of_nakshatra() {
        // Moon exactly at 0 deg (start of Ashwini)
        let (idx, balance, frac) = nakshatra_birth_balance(0.0, 2555.75);
        assert_eq!(idx, 0);
        assert!((balance - 2555.75).abs() < 1e-10);
        assert!(frac.abs() < 1e-10);
    }

    #[test]
    fn balance_at_midpoint() {
        // Moon at midpoint of Ashwini: 6.6667 deg
        let mid = NAKSHATRA_SPAN_27 / 2.0;
        let (idx, balance, frac) = nakshatra_birth_balance(mid, 2555.75);
        assert_eq!(idx, 0);
        assert!((frac - 0.5).abs() < 1e-10);
        assert!((balance - 2555.75 * 0.5).abs() < 1e-6);
    }

    #[test]
    fn balance_at_end_of_nakshatra() {
        // Moon just before end of Ashwini: almost 13.333 deg
        let near_end = NAKSHATRA_SPAN_27 - 0.001;
        let (idx, balance, _frac) = nakshatra_birth_balance(near_end, 2555.75);
        assert_eq!(idx, 0);
        assert!(balance < 1.0); // very small remaining balance
    }

    #[test]
    fn balance_rohini() {
        // Moon at 40 deg → Rohini (index 3), ~0 deg into Rohini
        // Rohini starts at 3 * 13.333 = 40.0 deg
        let (idx, balance, frac) = nakshatra_birth_balance(40.0, 3652.5);
        assert_eq!(idx, 3);
        assert!(frac.abs() < 1e-10);
        assert!((balance - 3652.5).abs() < 1e-10);
    }

    #[test]
    fn balance_wraps() {
        // Negative longitude wraps correctly
        let (idx, _, _) = nakshatra_birth_balance(-1.0, 1000.0);
        // -1 → 359 deg → Revati (index 26)
        assert_eq!(idx, 26);
    }
}
