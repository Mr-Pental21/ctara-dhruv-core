//! Types for lunar phase search results.

use dhruv_time::UtcTime;

/// Lunar phase type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LunarPhase {
    /// Amavasya (new moon): Sun-Moon conjunction (0 deg separation).
    NewMoon,
    /// Purnima (full moon): Sun-Moon opposition (180 deg separation).
    FullMoon,
}

impl LunarPhase {
    pub const fn name(self) -> &'static str {
        match self {
            Self::NewMoon => "Amavasya",
            Self::FullMoon => "Purnima",
        }
    }
}

/// A lunar phase event with UTC time and body longitudes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LunarPhaseEvent {
    /// UTC time of the event.
    pub utc: UtcTime,
    /// Phase type (new moon or full moon).
    pub phase: LunarPhase,
    /// Tropical ecliptic longitude of the Moon at the event (degrees).
    pub moon_longitude_deg: f64,
    /// Tropical ecliptic longitude of the Sun at the event (degrees).
    pub sun_longitude_deg: f64,
}
