use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::DhruvError;

/// UTC calendar date with sub-second precision.
///
/// Used as input to convenience functions. Supports construction via
/// [`UtcDate::new`] or parsing ISO 8601 strings via [`FromStr`].
///
/// ```
/// use dhruv_rs::UtcDate;
/// let d: UtcDate = "2024-03-20T12:00:00Z".parse().unwrap();
/// assert_eq!(d.year, 2024);
/// assert_eq!(d.month, 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UtcDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub min: u32,
    pub sec: f64,
}

impl UtcDate {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: f64) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            min,
            sec,
        }
    }
}

impl Display for UtcDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let whole = self.sec as u32;
        let frac = self.sec - whole as f64;
        if frac.abs() < 1e-9 {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                self.year, self.month, self.day, self.hour, self.min, whole
            )
        } else {
            write!(
                f,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:09.6}Z",
                self.year, self.month, self.day, self.hour, self.min, self.sec
            )
        }
    }
}

/// Parse ISO 8601 subset: `YYYY-MM-DDTHH:MM:SS[.f]Z`
///
/// Supports fractional seconds. The trailing `Z` is required.
impl FromStr for UtcDate {
    type Err = DhruvError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || DhruvError::DateParse(format!("invalid ISO 8601 date: {s}"));

        let bytes = s.as_bytes();

        // Minimum: "YYYY-MM-DDTHH:MM:SSZ" = 20 chars
        if bytes.len() < 20 {
            return Err(err());
        }

        // Must end with 'Z'
        if bytes[bytes.len() - 1] != b'Z' {
            return Err(err());
        }

        // Fixed separators
        if bytes[4] != b'-' || bytes[7] != b'-' || bytes[13] != b':' || bytes[16] != b':' {
            return Err(err());
        }

        // 'T' or ' ' separator between date and time
        if bytes[10] != b'T' && bytes[10] != b' ' {
            return Err(err());
        }

        let year: i32 = parse_int(&s[0..4]).ok_or_else(err)?;
        let month: u32 = parse_uint(&s[5..7]).ok_or_else(err)?;
        let day: u32 = parse_uint(&s[8..10]).ok_or_else(err)?;
        let hour: u32 = parse_uint(&s[11..13]).ok_or_else(err)?;
        let min: u32 = parse_uint(&s[14..16]).ok_or_else(err)?;

        // Seconds: everything between index 17 and the trailing 'Z'
        let sec_str = &s[17..bytes.len() - 1];
        let sec: f64 = sec_str.parse().map_err(|_| err())?;

        if !(1..=12).contains(&month) {
            return Err(err());
        }
        if !(1..=31).contains(&day) {
            return Err(err());
        }
        if hour > 23 || min > 59 || sec < 0.0 || sec >= 61.0 {
            return Err(err());
        }

        Ok(UtcDate {
            year,
            month,
            day,
            hour,
            min,
            sec,
        })
    }
}

fn parse_int(s: &str) -> Option<i32> {
    s.parse().ok()
}

fn parse_uint(s: &str) -> Option<u32> {
    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_iso() {
        let d: UtcDate = "2024-03-20T12:00:00Z".parse().unwrap();
        assert_eq!(d.year, 2024);
        assert_eq!(d.month, 3);
        assert_eq!(d.day, 20);
        assert_eq!(d.hour, 12);
        assert_eq!(d.min, 0);
        assert!((d.sec - 0.0).abs() < 1e-12);
    }

    #[test]
    fn parse_fractional_seconds() {
        let d: UtcDate = "2024-03-20T12:30:45.123Z".parse().unwrap();
        assert_eq!(d.hour, 12);
        assert_eq!(d.min, 30);
        assert!((d.sec - 45.123).abs() < 1e-9);
    }

    #[test]
    fn parse_space_separator() {
        let d: UtcDate = "2024-03-20 12:00:00Z".parse().unwrap();
        assert_eq!(d.year, 2024);
    }

    #[test]
    fn parse_negative_year() {
        let d: UtcDate = "-500-06-15T00:00:00Z".parse().unwrap();
        assert_eq!(d.year, -500);
    }

    #[test]
    fn rejects_missing_z() {
        assert!("2024-03-20T12:00:00".parse::<UtcDate>().is_err());
    }

    #[test]
    fn rejects_too_short() {
        assert!("2024Z".parse::<UtcDate>().is_err());
    }

    #[test]
    fn rejects_invalid_month() {
        assert!("2024-13-20T12:00:00Z".parse::<UtcDate>().is_err());
    }

    #[test]
    fn rejects_invalid_day() {
        assert!("2024-03-00T12:00:00Z".parse::<UtcDate>().is_err());
    }

    #[test]
    fn rejects_invalid_hour() {
        assert!("2024-03-20T25:00:00Z".parse::<UtcDate>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        let d = UtcDate::new(2024, 3, 20, 12, 0, 0.0);
        let s = d.to_string();
        let d2: UtcDate = s.parse().unwrap();
        assert_eq!(d, d2);
    }

    #[test]
    fn new_constructor() {
        let d = UtcDate::new(2000, 1, 1, 12, 0, 0.0);
        assert_eq!(d.year, 2000);
        assert_eq!(d.month, 1);
    }
}
