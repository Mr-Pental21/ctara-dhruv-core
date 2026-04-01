//! Types for grahan (eclipse) computation.

use dhruv_time::UtcTime;

/// Geographic location on Earth's surface.
///
/// Identical fields to `dhruv_vedic_base::GeoLocation` but defined
/// independently to avoid a dependency on the vedic crate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoLocation {
    /// Geodetic latitude in degrees, north positive. Range: [-90, 90].
    pub latitude_deg: f64,
    /// Geodetic longitude in degrees, east positive. Range: [-180, 180].
    pub longitude_deg: f64,
    /// Altitude above mean sea level in meters.
    pub altitude_m: f64,
}

impl GeoLocation {
    pub fn new(latitude_deg: f64, longitude_deg: f64, altitude_m: f64) -> Self {
        Self {
            latitude_deg,
            longitude_deg,
            altitude_m,
        }
    }

    pub fn latitude_rad(&self) -> f64 {
        self.latitude_deg.to_radians()
    }

    pub fn longitude_rad(&self) -> f64 {
        self.longitude_deg.to_radians()
    }
}

/// Grahan search configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrahanConfig {
    /// Include penumbral-only chandra grahan in results. Default: true.
    pub include_penumbral: bool,
    /// Include ecliptic latitude and angular separation at peak. Default: true.
    pub include_peak_details: bool,
}

impl Default for GrahanConfig {
    fn default() -> Self {
        Self {
            include_penumbral: true,
            include_peak_details: true,
        }
    }
}

/// Chandra grahan (lunar eclipse) type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChandraGrahanType {
    /// Moon passes through Earth's penumbral shadow only.
    Penumbral,
    /// Part of the Moon enters Earth's umbral shadow.
    Partial,
    /// Moon is entirely within Earth's umbral shadow.
    Total,
}

/// Chandra grahan (lunar eclipse) event with contact times and magnitudes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChandraGrahan {
    /// Grahan classification.
    pub grahan_type: ChandraGrahanType,
    /// Umbral magnitude: fraction of Moon's diameter covered by umbra.
    /// Negative for penumbral-only grahan.
    pub magnitude: f64,
    /// Penumbral magnitude: fraction of Moon's diameter in penumbra.
    pub penumbral_magnitude: f64,
    /// Time of greatest grahan (JD TDB).
    pub greatest_grahan_jd: f64,
    /// Time of greatest grahan as structured Gregorian UTC.
    pub greatest_grahan_utc: UtcTime,
    /// P1: First penumbral contact (JD TDB).
    pub p1_jd: f64,
    /// P1: First penumbral contact as structured Gregorian UTC.
    pub p1_utc: UtcTime,
    /// U1: First umbral contact (JD TDB). None for penumbral-only.
    pub u1_jd: Option<f64>,
    /// U1: First umbral contact as structured Gregorian UTC. None for penumbral-only.
    pub u1_utc: Option<UtcTime>,
    /// U2: Start of totality (JD TDB). None unless total.
    pub u2_jd: Option<f64>,
    /// U2: Start of totality as structured Gregorian UTC. None unless total.
    pub u2_utc: Option<UtcTime>,
    /// U3: End of totality (JD TDB). None unless total.
    pub u3_jd: Option<f64>,
    /// U3: End of totality as structured Gregorian UTC. None unless total.
    pub u3_utc: Option<UtcTime>,
    /// U4: Last umbral contact (JD TDB). None for penumbral-only.
    pub u4_jd: Option<f64>,
    /// U4: Last umbral contact as structured Gregorian UTC. None for penumbral-only.
    pub u4_utc: Option<UtcTime>,
    /// P4: Last penumbral contact (JD TDB).
    pub p4_jd: f64,
    /// P4: Last penumbral contact as structured Gregorian UTC.
    pub p4_utc: UtcTime,
    /// Moon's ecliptic latitude at greatest grahan, in degrees.
    pub moon_ecliptic_lat_deg: f64,
    /// Angular separation between Moon center and shadow axis at greatest grahan, in degrees.
    pub angular_separation_deg: f64,
}

/// Surya grahan (solar eclipse) type classification (geocentric).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuryaGrahanType {
    /// Moon covers part of the Sun.
    Partial,
    /// Moon is smaller (farther) than the Sun; ring of sunlight visible.
    Annular,
    /// Moon completely covers the Sun.
    Total,
    /// Grahan transitions between annular and total along the path.
    Hybrid,
}

/// Geocentric surya grahan (solar eclipse) event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuryaGrahan {
    /// Grahan classification.
    pub grahan_type: SuryaGrahanType,
    /// Magnitude: ratio of apparent Moon diameter to Sun diameter at greatest grahan.
    pub magnitude: f64,
    /// Time of greatest grahan (JD TDB).
    pub greatest_grahan_jd: f64,
    /// Time of greatest grahan as structured Gregorian UTC.
    pub greatest_grahan_utc: UtcTime,
    /// C1: First external contact (JD TDB). Moon's limb first touches Sun's limb.
    pub c1_jd: Option<f64>,
    /// C1 as structured Gregorian UTC. None if absent.
    pub c1_utc: Option<UtcTime>,
    /// C2: First internal contact (JD TDB). None for partial grahan.
    pub c2_jd: Option<f64>,
    /// C2 as structured Gregorian UTC. None if absent.
    pub c2_utc: Option<UtcTime>,
    /// C3: Last internal contact (JD TDB). None for partial grahan.
    pub c3_jd: Option<f64>,
    /// C3 as structured Gregorian UTC. None if absent.
    pub c3_utc: Option<UtcTime>,
    /// C4: Last external contact (JD TDB). Moon's limb last touches Sun's limb.
    pub c4_jd: Option<f64>,
    /// C4 as structured Gregorian UTC. None if absent.
    pub c4_utc: Option<UtcTime>,
    /// Moon's ecliptic latitude at greatest grahan, in degrees.
    pub moon_ecliptic_lat_deg: f64,
    /// Angular separation between Sun and Moon centers at greatest grahan, in degrees.
    pub angular_separation_deg: f64,
}
