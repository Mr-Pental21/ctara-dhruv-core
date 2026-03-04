//! Julian Date ↔ calendar conversions.
//!
//! Algorithms from Meeus, "Astronomical Algorithms" (2nd ed.), chapter 7.
//! Implementation is original.

/// J2000.0 epoch as Julian Date (2000-Jan-01 12:00:00 TDB).
pub const J2000_JD: f64 = 2_451_545.0;

/// Seconds in one Julian day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// Calendar policy for historical dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarPolicy {
    /// Extend Gregorian calendar rules to all epochs.
    ProlepticGregorian,
    /// Use historical Gregorian cutover (1582-10-15).
    GregorianCutover1582,
}

impl Default for CalendarPolicy {
    fn default() -> Self {
        Self::ProlepticGregorian
    }
}

/// Two-part Julian Date representation (`jd = day + frac`) where
/// `frac` is kept near zero to reduce cancellation error.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoPartJd {
    pub day: f64,
    pub frac: f64,
}

impl TwoPartJd {
    pub fn from_jd(jd: f64) -> Self {
        let day = jd.floor();
        let frac = jd - day;
        Self { day, frac }
    }

    pub fn to_jd(self) -> f64 {
        self.day + self.frac
    }

    pub fn from_tdb_seconds(tdb_s: f64) -> Self {
        let days = tdb_s / SECONDS_PER_DAY;
        Self {
            day: J2000_JD,
            frac: days,
        }
        .normalized()
    }

    pub fn to_tdb_seconds(self) -> f64 {
        ((self.day - J2000_JD) + self.frac) * SECONDS_PER_DAY
    }

    pub fn normalized(self) -> Self {
        let carry = self.frac.floor();
        Self {
            day: self.day + carry,
            frac: self.frac - carry,
        }
    }
}

/// Convert a Gregorian calendar date to Julian Date.
///
/// `day` may be fractional (e.g. 1.5 = noon on the 1st).
/// Valid for dates after 1582-Oct-15 (Gregorian calendar adoption).
pub fn calendar_to_jd(year: i32, month: u32, day: f64) -> f64 {
    calendar_to_jd_with_policy(year, month, day, CalendarPolicy::default())
}

/// Convert a calendar date to Julian Date with explicit calendar policy.
pub fn calendar_to_jd_with_policy(year: i32, month: u32, day: f64, policy: CalendarPolicy) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };

    let a = y / 100;
    let b = match policy {
        CalendarPolicy::ProlepticGregorian => 2 - a + a / 4,
        CalendarPolicy::GregorianCutover1582 => {
            // Julian calendar before 1582-10-15.
            let is_gregorian = year > 1582
                || (year == 1582 && month > 10)
                || (year == 1582 && month == 10 && day >= 15.0);
            if is_gregorian { 2 - a + a / 4 } else { 0 }
        }
    };

    let jd_int = (365.25 * (y as f64 + 4716.0)).floor();
    let jd_month = (30.6001 * (m as f64 + 1.0)).floor();

    jd_int + jd_month + day + b as f64 - 1524.5
}

/// Convert a Julian Date to Gregorian calendar date.
///
/// Returns `(year, month, day)` where `day` is fractional.
pub fn jd_to_calendar(jd: f64) -> (i32, u32, f64) {
    jd_to_calendar_with_policy(jd, CalendarPolicy::default())
}

/// Convert Julian Date to calendar date with explicit calendar policy.
pub fn jd_to_calendar_with_policy(jd: f64, policy: CalendarPolicy) -> (i32, u32, f64) {
    let jd_plus = jd + 0.5;
    let z = jd_plus.floor() as i64;
    let f = jd_plus - z as f64;

    let a = match policy {
        CalendarPolicy::ProlepticGregorian => {
            let alpha = ((z as f64 - 1_867_216.25) / 36_524.25).floor() as i64;
            z + 1 + alpha - alpha / 4
        }
        CalendarPolicy::GregorianCutover1582 => {
            if z < 2_299_161 {
                z
            } else {
                let alpha = ((z as f64 - 1_867_216.25) / 36_524.25).floor() as i64;
                z + 1 + alpha - alpha / 4
            }
        }
    };

    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d) as f64 - (30.6001 * e as f64).floor() + f;
    let month = if e < 14 { e - 1 } else { e - 13 };
    let year = if month > 2 { c - 4716 } else { c - 4715 };

    (year as i32, month as u32, day)
}

/// Convert a Julian Date (TDB) to TDB seconds past J2000.0.
#[inline]
pub fn jd_to_tdb_seconds(jd: f64) -> f64 {
    TwoPartJd::from_jd(jd).to_tdb_seconds()
}

/// Convert TDB seconds past J2000.0 to Julian Date (TDB).
#[inline]
pub fn tdb_seconds_to_jd(tdb_s: f64) -> f64 {
    TwoPartJd::from_tdb_seconds(tdb_s).to_jd()
}

/// Month abbreviation lookup (1-indexed: month 1 = "JAN").
pub fn month_from_abbrev(abbrev: &str) -> Option<u32> {
    match abbrev.to_ascii_uppercase().as_str() {
        "JAN" => Some(1),
        "FEB" => Some(2),
        "MAR" => Some(3),
        "APR" => Some(4),
        "MAY" => Some(5),
        "JUN" => Some(6),
        "JUL" => Some(7),
        "AUG" => Some(8),
        "SEP" => Some(9),
        "OCT" => Some(10),
        "NOV" => Some(11),
        "DEC" => Some(12),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    #[test]
    fn j2000_epoch() {
        // 2000-Jan-01 12:00:00 = JD 2451545.0
        let jd = calendar_to_jd(2000, 1, 1.5);
        assert!((jd - J2000_JD).abs() < EPS, "J2000.0: got {jd}");
    }

    #[test]
    fn y2000_midnight() {
        // 2000-Jan-01 00:00:00 = JD 2451544.5
        let jd = calendar_to_jd(2000, 1, 1.0);
        assert!((jd - 2_451_544.5).abs() < EPS);
    }

    #[test]
    fn known_epoch_1972_jan_1() {
        // 1972-Jan-01 00:00 = JD 2441317.5
        let jd = calendar_to_jd(1972, 1, 1.0);
        assert!((jd - 2_441_317.5).abs() < EPS, "1972-Jan-01: got {jd}");
    }

    #[test]
    fn roundtrip_calendar_jd() {
        let cases = [
            (2000, 1, 1.5),
            (1972, 7, 1.0),
            (2024, 12, 15.75),
            (1969, 7, 20.0),
        ];
        for (y, m, d) in cases {
            let jd = calendar_to_jd(y, m, d);
            let (y2, m2, d2) = jd_to_calendar(jd);
            assert_eq!(y, y2, "year mismatch for ({y}, {m}, {d})");
            assert_eq!(m, m2, "month mismatch for ({y}, {m}, {d})");
            assert!((d - d2).abs() < EPS, "day mismatch for ({y}, {m}, {d})");
        }
    }

    #[test]
    fn tdb_seconds_at_j2000() {
        assert_eq!(jd_to_tdb_seconds(J2000_JD), 0.0);
    }

    #[test]
    fn tdb_seconds_roundtrip() {
        let jd = 2_460_000.5;
        let s = jd_to_tdb_seconds(jd);
        let jd2 = tdb_seconds_to_jd(s);
        assert!((jd - jd2).abs() < 1e-12);
    }

    #[test]
    fn calendar_policy_cutover_differs_pre_1582() {
        let jd_greg = calendar_to_jd_with_policy(1500, 3, 1.0, CalendarPolicy::ProlepticGregorian);
        let jd_cutover =
            calendar_to_jd_with_policy(1500, 3, 1.0, CalendarPolicy::GregorianCutover1582);
        assert!((jd_greg - jd_cutover).abs() > 0.5);
    }

    #[test]
    fn two_part_jd_roundtrip() {
        let jd = 2_451_545.123_456_789;
        let p = TwoPartJd::from_jd(jd);
        assert!((p.to_jd() - jd).abs() < 1e-12);
    }
}
