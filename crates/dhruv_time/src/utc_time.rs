//! UTC calendar date/time with sub-second precision.
//!
//! Provides `UtcTime`, the canonical UTC representation used throughout
//! the engine. Conversion to/from JD TDB requires a [`LeapSecondKernel`].

use crate::LeapSecondKernel;
use crate::error::TimeError;
use crate::julian::{calendar_to_jd, jd_to_calendar, jd_to_tdb_seconds, tdb_seconds_to_jd};

/// UTC calendar date with sub-second precision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UtcTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: f64,
}

impl UtcTime {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: f64) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Construct a UTC instant with strict range checks.
    ///
    /// If `lsk` is provided, `23:59:60` is only accepted on legal leap-second days.
    pub fn try_new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: f64,
        lsk: Option<&LeapSecondKernel>,
    ) -> Result<Self, TimeError> {
        let candidate = Self::new(year, month, day, hour, minute, second);
        candidate.validate(lsk)?;
        Ok(candidate)
    }

    /// Validate calendar/time fields and optional leap-second legality.
    pub fn validate(&self, lsk: Option<&LeapSecondKernel>) -> Result<(), TimeError> {
        if !(1..=12).contains(&self.month) {
            return Err(TimeError::InvalidUtc(format!(
                "month {} out of range 1..12",
                self.month
            )));
        }
        let dim = days_in_month(self.year, self.month);
        if self.day == 0 || self.day > dim {
            return Err(TimeError::InvalidUtc(format!(
                "day {} out of range for {:04}-{:02}",
                self.day, self.year, self.month
            )));
        }
        if self.hour > 23 {
            return Err(TimeError::InvalidUtc(format!(
                "hour {} out of range 0..23",
                self.hour
            )));
        }
        if self.minute > 59 {
            return Err(TimeError::InvalidUtc(format!(
                "minute {} out of range 0..59",
                self.minute
            )));
        }
        if !self.second.is_finite() || self.second < 0.0 {
            return Err(TimeError::InvalidUtc(
                "second must be finite and >= 0".to_string(),
            ));
        }
        if self.second < 60.0 {
            return Ok(());
        }
        if self.second >= 61.0 {
            return Err(TimeError::InvalidUtc(format!(
                "second {} out of range (must be < 61)",
                self.second
            )));
        }
        if self.hour != 23 || self.minute != 59 {
            return Err(TimeError::InvalidUtc(
                "second=60 is only legal at 23:59:60".to_string(),
            ));
        }
        let Some(lsk) = lsk else {
            return Err(TimeError::InvalidUtc(
                "second=60 requires leap-second kernel validation".to_string(),
            ));
        };
        if !lsk
            .data()
            .is_leap_second_day(self.year, self.month, self.day)
        {
            return Err(TimeError::InvalidUtc(format!(
                "{:04}-{:02}-{:02} is not a leap-second day in loaded LSK",
                self.year, self.month, self.day
            )));
        }
        Ok(())
    }

    /// Normalize overflowing/underflowing time-of-day across UTC day boundaries.
    ///
    /// This is leap-second aware: days with a legal insertion are treated as
    /// 86401-second days.
    pub fn normalize_with_lsk(&self, lsk: &LeapSecondKernel) -> Result<Self, TimeError> {
        let mut y = self.year;
        let mut m = self.month;
        let mut d = self.day;
        let mut sod = self.hour as f64 * 3600.0 + self.minute as f64 * 60.0 + self.second;

        // Allow unvalidated input here; normalize first.
        for _ in 0..8 {
            let day_len = if lsk.data().is_leap_second_day(y, m, d) {
                86_401.0
            } else {
                86_400.0
            };
            if sod < 0.0 {
                let (py, pm, pd) = prev_day(y, m, d);
                y = py;
                m = pm;
                d = pd;
                sod += if lsk.data().is_leap_second_day(y, m, d) {
                    86_401.0
                } else {
                    86_400.0
                };
                continue;
            }
            if sod >= day_len {
                sod -= day_len;
                let (ny, nm, nd) = next_day(y, m, d);
                y = ny;
                m = nm;
                d = nd;
                continue;
            }
            // In range for this day.
            let (hour, minute, second) = if day_len > 86_400.0 && sod >= 86_400.0 {
                (23u32, 59u32, 60.0 + (sod - 86_400.0))
            } else {
                let h = (sod / 3600.0).floor() as u32;
                let rem = sod - h as f64 * 3600.0;
                let mi = (rem / 60.0).floor() as u32;
                let se = rem - mi as f64 * 60.0;
                (h, mi, se)
            };
            let out = Self::new(y, m, d, hour, minute, second);
            out.validate(Some(lsk))?;
            return Ok(out);
        }

        Err(TimeError::InvalidUtc(
            "normalization did not converge".to_string(),
        ))
    }

    /// Convert to Julian Date TDB using leap-second data.
    pub fn to_jd_tdb(&self, lsk: &LeapSecondKernel) -> f64 {
        let day_frac = self.day as f64
            + self.hour as f64 / 24.0
            + self.minute as f64 / 1440.0
            + self.second / 86_400.0;
        let jd = calendar_to_jd(self.year, self.month, day_frac);
        // jd is UTC JD; convert to TDB seconds then back to JD
        let utc_s = jd_to_tdb_seconds(jd);
        let tdb_s = lsk.utc_to_tdb(utc_s);
        tdb_seconds_to_jd(tdb_s)
    }

    /// Convert from Julian Date TDB back to UTC calendar.
    pub fn from_jd_tdb(jd_tdb: f64, lsk: &LeapSecondKernel) -> Self {
        let tdb_s = jd_to_tdb_seconds(jd_tdb);
        let utc_s = lsk.tdb_to_utc(tdb_s);
        let utc_jd = tdb_seconds_to_jd(utc_s);
        let (year, month, day_frac) = jd_to_calendar(utc_jd);
        let day = day_frac.floor() as u32;
        let frac = day_frac.fract();
        let total_seconds = frac * 86_400.0;
        let hour = (total_seconds / 3600.0).floor() as u32;
        let minute = ((total_seconds % 3600.0) / 60.0).floor() as u32;
        let second = total_seconds % 60.0;
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0) && ((year % 100 != 0) || (year % 400 == 0))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn next_day(year: i32, month: u32, day: u32) -> (i32, u32, u32) {
    let jd = calendar_to_jd(year, month, day as f64) + 1.0;
    let (y, m, df) = jd_to_calendar(jd);
    (y, m, df.floor() as u32)
}

fn prev_day(year: i32, month: u32, day: u32) -> (i32, u32, u32) {
    let jd = calendar_to_jd(year, month, day as f64) - 1.0;
    let (y, m, df) = jd_to_calendar(jd);
    (y, m, df.floor() as u32)
}

impl std::fmt::Display for UtcTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let whole = self.second as u32;
        let frac = self.second - whole as f64;
        if frac.abs() < 1e-9 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                self.year, self.month, self.day, self.hour, self.minute, whole
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:09.6}Z",
                self.year, self.month, self.day, self.hour, self.minute, self.second
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constructor() {
        let t = UtcTime::new(2024, 3, 20, 12, 30, 45.5);
        assert_eq!(t.year, 2024);
        assert_eq!(t.month, 3);
        assert_eq!(t.day, 20);
        assert_eq!(t.hour, 12);
        assert_eq!(t.minute, 30);
        assert!((t.second - 45.5).abs() < 1e-12);
    }

    #[test]
    fn display_whole_seconds() {
        let t = UtcTime::new(2024, 1, 15, 0, 0, 0.0);
        assert_eq!(t.to_string(), "2024-01-15T00:00:00Z");
    }

    #[test]
    fn display_fractional_seconds() {
        let t = UtcTime::new(2024, 1, 15, 12, 30, 45.123);
        let s = t.to_string();
        assert!(s.contains("12:30:"), "got: {s}");
    }

    fn test_lsk() -> LeapSecondKernel {
        let content = r#"
\begindata
DELTET/DELTA_T_A = 32.184
DELTET/K         = 1.657D-3
DELTET/EB        = 1.671D-2
DELTET/M         = ( 6.239996 1.99096871D-7 )
DELTET/DELTA_AT  = ( 10, @1972-JAN-1
                     11, @1972-JUL-1
                     37, @2017-JAN-1 )
\begintext
"#;
        LeapSecondKernel::parse(content).unwrap()
    }

    #[test]
    fn try_new_rejects_illegal_calendar() {
        let e = UtcTime::try_new(2024, 2, 30, 0, 0, 0.0, None).unwrap_err();
        assert!(matches!(e, TimeError::InvalidUtc(_)));
    }

    #[test]
    fn try_new_rejects_second_60_without_lsk() {
        let e = UtcTime::try_new(2016, 12, 31, 23, 59, 60.0, None).unwrap_err();
        assert!(matches!(e, TimeError::InvalidUtc(_)));
    }

    #[test]
    fn try_new_accepts_known_leap_second() {
        let lsk = test_lsk();
        let t = UtcTime::try_new(2016, 12, 31, 23, 59, 60.0, Some(&lsk)).unwrap();
        assert_eq!(t.second, 60.0);
    }

    #[test]
    fn normalize_rolls_over_leap_second_day() {
        let lsk = test_lsk();
        let t = UtcTime::new(2016, 12, 31, 23, 59, 61.2)
            .normalize_with_lsk(&lsk)
            .unwrap();
        assert_eq!(t.year, 2017);
        assert_eq!(t.month, 1);
        assert_eq!(t.day, 1);
        assert_eq!(t.hour, 0);
        assert_eq!(t.minute, 0);
        assert!((t.second - 0.2).abs() < 1e-9);
    }
}
