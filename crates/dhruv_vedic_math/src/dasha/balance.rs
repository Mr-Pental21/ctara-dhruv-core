//! Birth balance calculation for dasha systems.
//!
//! - Nakshatra-based: computed from Moon's position within its nakshatra.
//! - Rashi-based: computed from lagna's position within its rashi.

use crate::nakshatra::NAKSHATRA_SPAN_27;
use crate::util::normalize_360;

/// Compute nakshatra birth balance for a nakshatra-based dasha system.
///
/// Returns `(nakshatra_index, balance_days, elapsed_fraction)`:
/// - `nakshatra_index`: 0-based index (0=Ashwini..26=Revati) of the Moon's nakshatra
/// - `balance_days`: remaining days in the starting graha's period
/// - `elapsed_fraction`: fraction of nakshatra already traversed [0, 1)
pub fn nakshatra_birth_balance(moon_sidereal_lon: f64, entry_period_days: f64) -> (u8, f64, f64) {
    let lon = normalize_360(moon_sidereal_lon);
    let nak_idx = (lon / NAKSHATRA_SPAN_27).floor() as u8;
    let nak_idx = nak_idx.min(26);
    let position_in_nak = lon - (nak_idx as f64) * NAKSHATRA_SPAN_27;
    let elapsed_fraction = position_in_nak / NAKSHATRA_SPAN_27;
    let balance_days = entry_period_days * (1.0 - elapsed_fraction);
    (nak_idx, balance_days, elapsed_fraction)
}

/// Group-aware birth balance for nakshatra-based systems whose graha years
/// are shared by a contiguous run of nakshatras (Ashtottari, Shodsottari, ...).
///
/// - `nak_idx`: the Moon's nakshatra index in the scheme `map` describes
///   (27 or 28 entries).
/// - `elapsed_fraction`: fraction of that nakshatra already traversed [0, 1).
/// - `map`: nakshatra → graha-sequence index. The group is the maximal run of
///   consecutive entries (wrapping past the last index) equal to `map[nak_idx]`.
/// - `full_period_days`: the graha's full period.
///
/// Each nakshatra of the group owns `share = full / group_size`. The balance is
/// the remainder of the Moon's nakshatra plus every later nakshatra of the group:
/// `share × (1 − elapsed) + share × remaining_after`. For maps where no two
/// consecutive nakshatras share a graha (Vimshottari) this reduces to
/// `full × (1 − elapsed)`.
pub fn group_birth_balance(
    nak_idx: usize,
    elapsed_fraction: f64,
    map: &[u8],
    full_period_days: f64,
) -> f64 {
    let n = map.len();
    if n == 0 {
        return full_period_days * (1.0 - elapsed_fraction);
    }
    let nak_idx = nak_idx.min(n - 1);
    let graha = map[nak_idx];

    // Walk backwards to count group members before the Moon's nakshatra.
    let mut before = 0usize;
    while before < n - 1 && map[(nak_idx + n - before - 1) % n] == graha {
        before += 1;
    }
    // Walk forwards to count group members after it.
    let mut after = 0usize;
    while before + after < n - 1 && map[(nak_idx + after + 1) % n] == graha {
        after += 1;
    }

    let group_size = (before + after + 1) as f64;
    let share = full_period_days / group_size;
    share * (1.0 - elapsed_fraction) + share * after as f64
}

/// Compute rashi birth balance for a rashi-based dasha system.
///
/// Returns `(balance_days, elapsed_fraction)`:
/// - `balance_days`: remaining days in the starting rashi's period
/// - `elapsed_fraction`: fraction of rashi already traversed [0, 1)
///
/// The lagna's position within its rashi determines how much of the first
/// mahadasha period has elapsed.
pub fn rashi_birth_balance(lagna_sidereal_lon: f64, entry_period_days: f64) -> (f64, f64) {
    let lon = normalize_360(lagna_sidereal_lon);
    let rashi_idx = (lon / 30.0).floor() as u8;
    let rashi_idx = rashi_idx.min(11);
    let position_in_rashi = lon - (rashi_idx as f64) * 30.0;
    let elapsed_fraction = position_in_rashi / 30.0;
    let balance_days = entry_period_days * (1.0 - elapsed_fraction);
    (balance_days, elapsed_fraction)
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
    fn group_balance_single_member_groups_match_plain_balance() {
        // Vimshottari-style map: no two consecutive nakshatras share a graha.
        let map: Vec<u8> = (0..27).map(|i| (i % 9) as u8).collect();
        let full = 3652.5;
        let got = group_birth_balance(3, 0.25, &map, full);
        assert!((got - full * 0.75).abs() < 1e-9);
    }

    #[test]
    fn group_balance_adds_remaining_group_members() {
        // Groups of 4,3 from index 5; Rahu (6) wraps 25,26,0,1.
        let map: [u8; 27] = [
            6, 6, 7, 7, 7, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6,
        ];
        let full = 6.0; // years, Surya
        // First of 4 at its start: whole period.
        assert!((group_birth_balance(5, 0.0, &map, full) - 6.0).abs() < 1e-12);
        // Third of 4, halfway: 0.5 share + 1 share.
        assert!((group_birth_balance(7, 0.5, &map, full) - 2.25).abs() < 1e-12);
        // Last of 4, at its end: nothing left.
        assert!(group_birth_balance(8, 1.0, &map, full).abs() < 1e-12);
        // Wrapping group: Revati (26) is 2nd of Rahu's 4 -> 2 shares + remainder.
        let rahu = 12.0;
        assert!((group_birth_balance(26, 0.0, &map, rahu) - 9.0).abs() < 1e-12);
        assert!((group_birth_balance(1, 0.5, &map, rahu) - 1.5).abs() < 1e-12);
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

    // ── Rashi birth balance tests ──

    #[test]
    fn rashi_balance_at_rashi_start() {
        // Lagna at 0 deg (start of Mesha) → full period
        let (balance, frac) = rashi_birth_balance(0.0, 2555.75);
        assert!((balance - 2555.75).abs() < 1e-10);
        assert!(frac.abs() < 1e-10);
    }

    #[test]
    fn rashi_balance_at_midpoint() {
        // Lagna at 15 deg (midpoint of Mesha) → half period
        let (balance, frac) = rashi_birth_balance(15.0, 2555.75);
        assert!((frac - 0.5).abs() < 1e-10);
        assert!((balance - 2555.75 * 0.5).abs() < 1e-6);
    }

    #[test]
    fn rashi_balance_at_end() {
        // Lagna near end of Mesha: 29.999 deg → tiny balance
        let (balance, _frac) = rashi_birth_balance(29.999, 2555.75);
        assert!(balance < 1.0);
    }

    #[test]
    fn rashi_balance_second_sign() {
        // Lagna at 30 deg (start of Vrishabha) → full period
        let (balance, frac) = rashi_birth_balance(30.0, 3000.0);
        assert!(frac.abs() < 1e-10);
        assert!((balance - 3000.0).abs() < 1e-10);
    }

    #[test]
    fn rashi_balance_wraps() {
        // Negative longitude wraps correctly
        let (balance, _frac) = rashi_birth_balance(-1.0, 1000.0);
        // -1 → 359 deg → Meena (index 11), position 29 deg → small balance
        assert!(balance < 100.0);
    }
}
