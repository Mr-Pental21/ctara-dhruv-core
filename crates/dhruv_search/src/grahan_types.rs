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
#[derive(Debug, Clone, PartialEq)]
pub struct GrahanConfig {
    /// Include penumbral-only chandra grahan in results. Default: true.
    pub include_penumbral: bool,
    /// Include ecliptic latitude and angular separation at peak. Default: true.
    pub include_peak_details: bool,
    /// Include sampled geographic path and shadow-footprint geometry for
    /// Surya grahan. Default: false, keeping summary searches inexpensive.
    pub include_path: bool,
    /// Sampling cadence for geographic path products. Range: 1..=30 minutes.
    pub path_step_minutes: u32,
    /// Maximum base angular sampling of instantaneous shadow-cone boundary
    /// rings. Tangent regions are subdivided adaptively. Range: 1..=15 degrees.
    pub boundary_step_deg: u32,
    /// Include the per-event geographic grid of local circumstances for
    /// Surya grahan. Default: false.
    pub include_local_grid: bool,
    /// Grid spacing in degrees for `local_grid` and the isoline field
    /// sampling. Values outside [0.5, 10] are clamped. Default: 2.0.
    pub local_grid_step_deg: f64,
    /// Include visibility/duration/magnitude isoline rings for Surya grahan.
    /// Default: false.
    pub include_isolines: bool,
    /// Visible-duration isoline levels as fractions of the global C1–C4
    /// span. Values outside (0, 1) are dropped; the list is sorted,
    /// deduplicated, and capped at 16 entries. Default: [0.25, 0.5, 0.75].
    pub duration_isoline_fractions: Vec<f64>,
    /// Local maximum-magnitude isoline levels. Values outside (0, 1.5] are
    /// dropped; the list is sorted, deduplicated, and capped at 16 entries.
    /// Default: [0.25, 0.5, 0.75, 1.0].
    pub magnitude_isoline_levels: Vec<f64>,
    /// Include the swept central (umbral/antumbral) corridor outline for
    /// Surya grahan. Default: false.
    pub include_central_corridor: bool,
    /// Include instantaneous penumbral footprints at the event's own
    /// contact moments (C1/C2/greatest/C3/C4) for Surya grahan.
    /// Default: false.
    pub include_contact_footprints: bool,
    /// Include instantaneous umbral/antumbral shadow outlines at every
    /// path timestamp and at the C2/greatest/C3 moments for Surya grahan.
    /// Default: false.
    pub include_umbra_footprints: bool,
    /// Instantaneous iso-magnitude contour levels. When non-empty, every
    /// sampled footprint and contact footprint carries `magnitude_rings` at
    /// these levels. Values outside (0, 1.5] are dropped; the list is
    /// sorted, deduplicated, and capped at 16 entries. Default: empty.
    pub instantaneous_magnitude_levels: Vec<f64>,
}

impl Default for GrahanConfig {
    fn default() -> Self {
        Self {
            include_penumbral: true,
            include_peak_details: true,
            include_path: false,
            path_step_minutes: 1,
            boundary_step_deg: 2,
            include_local_grid: false,
            local_grid_step_deg: 2.0,
            include_isolines: false,
            duration_isoline_fractions: vec![0.25, 0.5, 0.75],
            magnitude_isoline_levels: vec![0.25, 0.5, 0.75, 1.0],
            include_central_corridor: false,
            include_contact_footprints: false,
            include_umbra_footprints: false,
            instantaneous_magnitude_levels: Vec::new(),
        }
    }
}

impl GrahanConfig {
    /// Effective grid step after clamping to the supported [0.5, 10] range.
    /// Non-finite values fall back to the default spacing.
    pub fn effective_local_grid_step_deg(&self) -> f64 {
        if self.local_grid_step_deg.is_finite() {
            self.local_grid_step_deg.clamp(0.5, 10.0)
        } else {
            2.0
        }
    }

    /// Sanitized duration isoline fractions: finite, in (0, 1), sorted,
    /// deduplicated, at most 16.
    pub fn effective_duration_isoline_fractions(&self) -> Vec<f64> {
        sanitize_levels(&self.duration_isoline_fractions, 0.0, 1.0, false)
    }

    /// Sanitized magnitude isoline levels: finite, in (0, 1.5], sorted,
    /// deduplicated, at most 16.
    pub fn effective_magnitude_isoline_levels(&self) -> Vec<f64> {
        sanitize_levels(&self.magnitude_isoline_levels, 0.0, 1.5, true)
    }

    /// Sanitized instantaneous magnitude levels: finite, in (0, 1.5],
    /// sorted, deduplicated, at most 16.
    pub fn effective_instantaneous_magnitude_levels(&self) -> Vec<f64> {
        sanitize_levels(&self.instantaneous_magnitude_levels, 0.0, 1.5, true)
    }

    /// The configuration actually applied after clamping and sanitizing.
    /// Responses echo this so callers can build cache keys against the
    /// effective values rather than the raw request.
    pub fn effective(&self) -> GrahanConfig {
        GrahanConfig {
            include_penumbral: self.include_penumbral,
            include_peak_details: self.include_peak_details,
            include_path: self.include_path,
            path_step_minutes: self.path_step_minutes.clamp(1, 30),
            boundary_step_deg: self.boundary_step_deg.clamp(1, 15),
            include_local_grid: self.include_local_grid,
            local_grid_step_deg: self.effective_local_grid_step_deg(),
            include_isolines: self.include_isolines,
            duration_isoline_fractions: self.effective_duration_isoline_fractions(),
            magnitude_isoline_levels: self.effective_magnitude_isoline_levels(),
            include_central_corridor: self.include_central_corridor,
            include_contact_footprints: self.include_contact_footprints,
            include_umbra_footprints: self.include_umbra_footprints,
            instantaneous_magnitude_levels: self.effective_instantaneous_magnitude_levels(),
        }
    }
}

fn sanitize_levels(levels: &[f64], min: f64, max: f64, max_inclusive: bool) -> Vec<f64> {
    let mut sanitized: Vec<f64> = levels
        .iter()
        .copied()
        .filter(|value| {
            value.is_finite()
                && *value > min
                && if max_inclusive {
                    *value <= max
                } else {
                    *value < max
                }
        })
        .collect();
    sanitized.sort_by(f64::total_cmp);
    sanitized.dedup();
    sanitized.truncate(16);
    sanitized
}

/// Geographic coordinate on the reference Earth ellipsoid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EclipseGeoPoint {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
}

/// Instantaneous Besselian elements derived from the loaded ephemeris.
///
/// `x`, `y`, `l1`, and `l2` use Earth equatorial radii. `l2` is negative
/// for an umbral (total) cone and positive for an antumbral (annular) cone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BesselianElements {
    pub jd_tdb: f64,
    pub utc: UtcTime,
    pub x: f64,
    pub y: f64,
    pub d_deg: f64,
    pub mu_deg: f64,
    pub l1: f64,
    pub l2: f64,
    pub tan_f1: f64,
    pub tan_f2: f64,
}

/// One timestamped sample along a total/annular/hybrid ground path. The
/// northern and southern limits describe the local corridor around `center`.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaGrahanPathPoint {
    pub jd_tdb: f64,
    pub utc: UtcTime,
    pub center: EclipseGeoPoint,
    pub northern_limit: Option<EclipseGeoPoint>,
    pub southern_limit: Option<EclipseGeoPoint>,
    pub width_km: f64,
    pub central_duration_seconds: f64,
    pub sun_altitude_deg: f64,
    pub sun_azimuth_deg: f64,
    pub grahan_type: SuryaGrahanType,
}

/// One instantaneous iso-magnitude contour ring: the closed curve where the
/// eclipse magnitude at this moment equals `level`, clipped by the
/// terminator like the visibility products. Same ring contract as
/// `SuryaIsolineRing`.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaMagnitudeRing {
    pub level: f64,
    pub boundary: Vec<EclipseGeoPoint>,
    pub contains_pole: Option<PoleSide>,
}

/// Boundary of the instantaneous penumbral footprint on Earth: the region
/// with a partial phase in progress and the Sun up, clipped by the
/// day/night terminator (a shadow is only observable on the day side) and
/// closed along the terminator arc where truncated. The vertices form one
/// ordered closed ring; the final coordinate repeats the first.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaGrahanFootprint {
    pub jd_tdb: f64,
    pub utc: UtcTime,
    pub boundary: Vec<EclipseGeoPoint>,
    /// Set when the shadow region bounded by this ring contains a
    /// geographic pole; decided on the sphere by the geometry producer.
    pub contains_pole: Option<PoleSide>,
    /// Instantaneous iso-magnitude contours at this timestamp, ordered by
    /// level. Empty unless `GrahanConfig::instantaneous_magnitude_levels`
    /// is non-empty; levels the moment's maximum magnitude does not reach
    /// are omitted.
    pub magnitude_rings: Vec<SuryaMagnitudeRing>,
}

/// The event contact a contact-moment footprint belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuryaContactKind {
    C1,
    C2,
    Greatest,
    C3,
    C4,
}

/// Instantaneous penumbral footprint at one of the event's own contact
/// moments. Only contacts the event actually has are returned (no C2/C3
/// for partial events). At exact C1/C4 tangency the penumbra-ellipsoid
/// intersection degenerates toward a point: the entry is still returned,
/// but its `boundary` may be empty; consumers should fall back to the
/// nearest sampled footprint in that case.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaContactFootprint {
    pub contact: SuryaContactKind,
    pub jd_tdb: f64,
    pub utc: UtcTime,
    /// Closed, ordered, antimeridian-safe ring (Change 4 contract); may be
    /// empty at exact tangency.
    pub boundary: Vec<EclipseGeoPoint>,
    pub contains_pole: Option<PoleSide>,
    /// Instantaneous iso-magnitude contours at this contact, ordered by
    /// level. Empty unless `GrahanConfig::instantaneous_magnitude_levels`
    /// is non-empty; unreached levels are omitted.
    pub magnitude_rings: Vec<SuryaMagnitudeRing>,
}

/// Instantaneous umbral/antumbral shadow outline at one moment: the true
/// shape of the central shadow on the ground, strongly elongated near the
/// corridor ends where the shadow strikes at grazing incidence. Clipped by
/// the day/night terminator (near the central contacts totality happens at
/// sunrise/sunset, so the oval ends exactly on the terminator, flush with
/// the corridor's rounded end caps).
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaUmbraFootprint {
    pub jd_tdb: f64,
    pub utc: UtcTime,
    /// `Total` (umbra) or `Annular` (antumbra) at this moment.
    pub grahan_type: SuryaGrahanType,
    /// Closed, ordered, antimeridian-safe ring (Change 4 contract).
    pub boundary: Vec<EclipseGeoPoint>,
    pub contains_pole: Option<PoleSide>,
}

/// Which geographic pole a closed ring encloses, if any. Decided on the
/// sphere from the ring's longitude winding, never re-derived planar-side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoleSide {
    North,
    South,
}

/// One closed, ordered, non-self-intersecting boundary ring on the
/// reference ellipsoid. The final vertex repeats the first. Vertices are
/// ordered continuously so a renderer can unwrap longitudes across the
/// antimeridian (a consecutive longitude jump greater than 180 degrees
/// marks one seam crossing).
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaIsolineRing {
    pub boundary: Vec<EclipseGeoPoint>,
    /// Set when the enclosed region contains a geographic pole; such rings
    /// wind fully around in longitude.
    pub contains_pole: Option<PoleSide>,
}

/// One visible sample of the per-event local-circumstance grid.
///
/// Samples lie at grid-cell centers: `lat = -90 + (i + 0.5) * step`,
/// `lon = -180 + (j + 0.5) * step`. Only locations that see at least one
/// Sun-up partial phase are emitted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuryaLocalGridSample {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    /// Local maximum eclipse magnitude (fraction of the Sun's diameter).
    pub magnitude: f64,
    /// Fraction of the solar disk area obscured at local maximum.
    pub obscuration: f64,
    /// Time of local maximum (JD TDB), same convention as the per-location
    /// `local` circumstances (not Sun-up clipped).
    pub maximum_jd: f64,
    pub maximum_utc: UtcTime,
    /// First visible partial contact (Sun-up clipped), JD TDB.
    pub first_contact_jd: f64,
    pub first_contact_utc: UtcTime,
    /// Last visible partial contact (Sun-up clipped), JD TDB.
    pub last_contact_jd: f64,
    pub last_contact_utc: UtcTime,
    /// Measure of times in [C1, C4] with a partial phase in progress and
    /// the Sun risen (summed across split intervals).
    pub visible_duration_seconds: f64,
}

/// Closed rings of equal visible duration, tagged with the requested
/// fraction of the global C1–C4 span.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaDurationIsoline {
    pub fraction: f64,
    pub rings: Vec<SuryaIsolineRing>,
}

/// Closed rings of equal local maximum magnitude.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaMagnitudeIsoline {
    pub level: f64,
    pub rings: Vec<SuryaIsolineRing>,
}

/// Isoline products for one Surya grahan event.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaIsolines {
    /// Level-0 curve(s) enclosing every location that sees any Sun-up
    /// partial phase. The night-side gap can split this into several rings.
    pub visibility_boundary: Vec<SuryaIsolineRing>,
    pub duration_isolines: Vec<SuryaDurationIsoline>,
    pub magnitude_isolines: Vec<SuryaMagnitudeIsoline>,
}

/// One swept central-corridor segment: the closed outline of the ground
/// area touched by the umbral (total) or antumbral (annular) shadow.
/// Hybrid events return separate annular and total segments that meet at
/// their transition points; plain central events return one segment.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaCorridorSegment {
    /// `Total` or `Annular` for the shadow that sweeps this segment.
    pub grahan_type: SuryaGrahanType,
    pub rings: Vec<SuryaIsolineRing>,
}

/// Swept central corridor for one Surya grahan event.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaCentralCorridor {
    /// Segments ordered along the path by first central contact.
    pub segments: Vec<SuryaCorridorSegment>,
}

/// Whether and how the central (umbral/antumbral) shadow reaches Earth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuryaCentrality {
    /// The shadow axis intersects the Earth: a center line exists.
    Full,
    /// The shadow cone grazes the Earth but the center line misses it;
    /// limits are one-sided and the swept corridor still closes.
    Partial,
    /// The central shadow never reaches Earth (partial-only event).
    None,
}

/// Location-specific solar-eclipse circumstances.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaGrahanLocalCircumstances {
    pub location: GeoLocation,
    pub visible: bool,
    pub grahan_type: Option<SuryaGrahanType>,
    pub maximum_jd: Option<f64>,
    pub maximum_utc: Option<UtcTime>,
    pub c1_jd: Option<f64>,
    pub c1_utc: Option<UtcTime>,
    pub c2_jd: Option<f64>,
    pub c2_utc: Option<UtcTime>,
    pub c3_jd: Option<f64>,
    pub c3_utc: Option<UtcTime>,
    pub c4_jd: Option<f64>,
    pub c4_utc: Option<UtcTime>,
    pub magnitude: f64,
    pub obscuration: f64,
    pub sun_altitude_deg: f64,
    pub sun_azimuth_deg: f64,
    pub central_duration_seconds: f64,
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
    /// Moon's apparent geocentric right ascension at greatest grahan, in
    /// degrees [0, 360) (equinox of date, IAU 2000B nutation applied).
    pub moon_right_ascension_deg: f64,
    /// Moon's apparent geocentric declination at greatest grahan, in degrees.
    pub moon_declination_deg: f64,
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

/// Global solar-eclipse event with optional map and local circumstances.
#[derive(Debug, Clone, PartialEq)]
pub struct SuryaGrahan {
    /// Grahan classification.
    pub grahan_type: SuryaGrahanType,
    /// Standard eclipse magnitude: fraction of the Sun's diameter covered at
    /// the geographic point of greatest eclipse.
    pub magnitude: f64,
    /// Fraction of the solar disk area obscured at greatest eclipse, [0, 1].
    pub obscuration: f64,
    /// Apparent Moon/Sun diameter ratio at greatest eclipse.
    pub apparent_diameter_ratio: f64,
    /// Signed minimum shadow-axis distance from the geocenter, in Earth
    /// equatorial radii (north positive).
    pub gamma: f64,
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
    /// Sun's apparent geocentric right ascension at greatest grahan, in
    /// degrees [0, 360) (equinox of date, IAU 2000B nutation applied).
    pub sun_right_ascension_deg: f64,
    /// Sun's apparent geocentric declination at greatest grahan, in degrees.
    pub sun_declination_deg: f64,
    /// Geographic point of greatest eclipse when the penumbra reaches Earth.
    pub greatest_location: Option<EclipseGeoPoint>,
    /// Besselian elements at greatest eclipse.
    pub besselian: BesselianElements,
    /// Timestamped central line and limits. Empty for partial eclipses or
    /// when `GrahanConfig::include_path` is false.
    pub path: Vec<SuryaGrahanPathPoint>,
    /// Instantaneous penumbral boundary rings sampled through the event.
    /// Empty when `GrahanConfig::include_path` is false.
    pub footprints: Vec<SuryaGrahanFootprint>,
    /// Optional circumstances for the request's geographic location.
    pub local: Option<SuryaGrahanLocalCircumstances>,
    /// Whether the central shadow reaches Earth and, if so, whether the
    /// center line does.
    pub centrality: SuryaCentrality,
    /// Geographic grid of local circumstances. Empty unless
    /// `GrahanConfig::include_local_grid` is true.
    pub local_grid: Vec<SuryaLocalGridSample>,
    /// Visibility/duration/magnitude isolines. None unless
    /// `GrahanConfig::include_isolines` is true.
    pub isolines: Option<SuryaIsolines>,
    /// Swept central corridor. None unless
    /// `GrahanConfig::include_central_corridor` is true or the event has no
    /// central shadow contact (centrality `None`).
    pub central_corridor: Option<SuryaCentralCorridor>,
    /// Penumbral footprints at the event's own contact moments. Empty
    /// unless `GrahanConfig::include_contact_footprints` is true.
    pub contact_footprints: Vec<SuryaContactFootprint>,
    /// Instantaneous umbral/antumbral outlines at every path timestamp and
    /// the C2/greatest/C3 moments. Empty unless
    /// `GrahanConfig::include_umbra_footprints` is true or the central
    /// shadow never reaches Earth.
    pub umbra_footprints: Vec<SuryaUmbraFootprint>,
}
