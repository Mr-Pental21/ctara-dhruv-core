//! Types for panchang classification results.

use dhruv_time::UtcTime;
use dhruv_vedic_base::{Ayana, Masa, Samvatsara};

/// Masa (lunar month) classification result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MasaInfo {
    /// The masa (lunar month).
    pub masa: Masa,
    /// Whether this is an adhika (intercalary) month.
    pub adhika: bool,
    /// Start of the masa (previous new moon).
    pub start: UtcTime,
    /// End of the masa (next new moon).
    pub end: UtcTime,
}

/// Ayana (solstice period) classification result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AyanaInfo {
    /// The ayana (Uttarayana or Dakshinayana).
    pub ayana: Ayana,
    /// Start of this ayana period (Sankranti).
    pub start: UtcTime,
    /// End of this ayana period (next Sankranti).
    pub end: UtcTime,
}

/// Varsha (year in 60-year cycle) classification result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VarshaInfo {
    /// The samvatsara name.
    pub samvatsara: Samvatsara,
    /// Order in the 60-year cycle (1-60).
    pub order: u8,
    /// Start of the Vedic year (Chaitra Pratipada).
    pub start: UtcTime,
    /// End of the Vedic year (next Chaitra Pratipada).
    pub end: UtcTime,
}
