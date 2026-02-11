//! Types for eclipse computation.

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

/// Eclipse search configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EclipseConfig {
    /// Include penumbral-only lunar eclipses in results. Default: true.
    pub include_penumbral: bool,
    /// Include ecliptic latitude and angular separation at peak. Default: true.
    pub include_peak_details: bool,
}

impl Default for EclipseConfig {
    fn default() -> Self {
        Self {
            include_penumbral: true,
            include_peak_details: true,
        }
    }
}

/// Lunar eclipse type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LunarEclipseType {
    /// Moon passes through Earth's penumbral shadow only.
    Penumbral,
    /// Part of the Moon enters Earth's umbral shadow.
    Partial,
    /// Moon is entirely within Earth's umbral shadow.
    Total,
}

/// Lunar eclipse event with contact times and magnitudes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LunarEclipse {
    /// Eclipse classification.
    pub eclipse_type: LunarEclipseType,
    /// Umbral magnitude: fraction of Moon's diameter covered by umbra.
    /// Negative for penumbral-only eclipses.
    pub magnitude: f64,
    /// Penumbral magnitude: fraction of Moon's diameter in penumbra.
    pub penumbral_magnitude: f64,
    /// Time of greatest eclipse (JD TDB).
    pub greatest_eclipse_jd: f64,
    /// P1: First penumbral contact (JD TDB).
    pub p1_jd: f64,
    /// U1: First umbral contact (JD TDB). None for penumbral-only.
    pub u1_jd: Option<f64>,
    /// U2: Start of totality (JD TDB). None unless total.
    pub u2_jd: Option<f64>,
    /// U3: End of totality (JD TDB). None unless total.
    pub u3_jd: Option<f64>,
    /// U4: Last umbral contact (JD TDB). None for penumbral-only.
    pub u4_jd: Option<f64>,
    /// P4: Last penumbral contact (JD TDB).
    pub p4_jd: f64,
    /// Moon's ecliptic latitude at greatest eclipse, in degrees.
    pub moon_ecliptic_lat_deg: f64,
    /// Angular separation between Moon center and shadow axis at greatest eclipse, in degrees.
    pub angular_separation_deg: f64,
}

/// Solar eclipse type classification (geocentric).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolarEclipseType {
    /// Moon covers part of the Sun.
    Partial,
    /// Moon is smaller (farther) than the Sun; ring of sunlight visible.
    Annular,
    /// Moon completely covers the Sun.
    Total,
    /// Eclipse transitions between annular and total along the path.
    Hybrid,
}

/// Geocentric solar eclipse event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolarEclipse {
    /// Eclipse classification.
    pub eclipse_type: SolarEclipseType,
    /// Magnitude: ratio of apparent Moon diameter to Sun diameter at greatest eclipse.
    pub magnitude: f64,
    /// Time of greatest eclipse (JD TDB).
    pub greatest_eclipse_jd: f64,
    /// C1: First external contact (JD TDB). Moon's limb first touches Sun's limb.
    pub c1_jd: Option<f64>,
    /// C2: First internal contact (JD TDB). None for partial eclipses.
    pub c2_jd: Option<f64>,
    /// C3: Last internal contact (JD TDB). None for partial eclipses.
    pub c3_jd: Option<f64>,
    /// C4: Last external contact (JD TDB). Moon's limb last touches Sun's limb.
    pub c4_jd: Option<f64>,
    /// Moon's ecliptic latitude at greatest eclipse, in degrees.
    pub moon_ecliptic_lat_deg: f64,
    /// Angular separation between Sun and Moon centers at greatest eclipse, in degrees.
    pub angular_separation_deg: f64,
}
