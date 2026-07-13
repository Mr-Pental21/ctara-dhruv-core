//! Grahan (eclipse) computation: lunar shadow contacts and geographic solar visibility.
//!
//! Builds on the conjunction engine to find new/full moons, then applies
//! shadow geometry for classification, magnitude, and contact times.
//!
//! Chandra grahan algorithm:
//!   1. Find full moons (Sun-Moon opposition, 180 deg separation)
//!   2. Filter by ecliptic latitude threshold
//!   3. Compute Earth shadow radii using Danjon augmented method
//!   4. Classify by comparing Moon's angular distance to shadow radii
//!   5. Find contact times by bisection
//!
//! Surya grahan algorithm:
//!   1. Find new moons (Sun-Moon conjunction, 0 deg separation)
//!   2. Derive instantaneous Besselian elements from ephemeris vectors
//!   3. Intersect penumbral and central shadow cones with an oblate Earth
//!   4. Compute global path, limits, footprints, and topocentric visibility
//!   5. Find global and local contacts by bracketed root solving
//!
//! Sources: standard spherical astronomy (Meeus Ch. 54 for shadow geometry,
//! IAU 2015 nominal radii). See docs/clean_room_solar_eclipse_visibility.md.

use dhruv_core::{Body, Engine, Frame, Observer, Query};
use dhruv_frames::{
    cartesian_to_spherical, equation_of_equinoxes_and_true_obliquity, icrf_to_ecliptic,
    mean_obliquity_of_date_rad, nutation_iau2000b, precess_ecliptic_j2000_to_date,
};
use dhruv_time::{EopKernel, UtcTime, calendar_to_jd, gmst_rad};

use crate::conjunction::{next_conjunction, prev_conjunction, search_conjunctions};
use crate::conjunction_types::ConjunctionConfig;
use crate::error::SearchError;
use crate::grahan_types::{
    BesselianElements, ChandraGrahan, ChandraGrahanType, EclipseGeoPoint, GeoLocation,
    GrahanConfig, SuryaGrahan, SuryaGrahanFootprint, SuryaGrahanLocalCircumstances,
    SuryaGrahanPathPoint, SuryaGrahanType,
};

// ---------------------------------------------------------------------------
// Constants (IAU 2015 nominal values)
// ---------------------------------------------------------------------------

/// Earth equatorial radius in km (IAU 2015 Resolution B3).
const EARTH_RADIUS_KM: f64 = 6378.137;

/// Conventional inverse flattening used for geodetic eclipse coordinates.
const EARTH_INV_FLATTENING: f64 = 298.257_223_563;

const EARTH_POLAR_RADIUS_KM: f64 = EARTH_RADIUS_KM * (1.0 - 1.0 / EARTH_INV_FLATTENING);

/// Sun nominal radius in km (IAU 2015 Resolution B3).
const SUN_RADIUS_KM: f64 = 696_000.0;

/// Moon mean radius in km (IAU 2015).
const MOON_RADIUS_KM: f64 = 1737.4;

/// Danjon atmospheric enlargement factor for Earth's shadow.
/// The Earth's atmosphere causes the geometrical shadow to appear ~2% larger.
/// Published in Meeus, "Astronomical Algorithms", Ch. 54.
const DANJON_ENLARGEMENT: f64 = 1.02;

/// Ecliptic latitude threshold for grahan candidacy (degrees).
/// Generous threshold; exact geometry filters afterward.
const GRAHAN_LAT_THRESHOLD_DEG: f64 = 2.0;

/// Step size for new/full moon scan (days). Moon synodic period ~29.5 days,
/// so 0.5 day step safely brackets all crossings.
const MOON_STEP_DAYS: f64 = 0.5;

/// Bisection convergence for contact times (days). ~0.86 ms precision.
const CONTACT_CONVERGENCE_DAYS: f64 = 1e-8;

/// Maximum bisection iterations for contact times.
const CONTACT_MAX_ITER: u32 = 50;

// ---------------------------------------------------------------------------
// Internal geometry helpers
// ---------------------------------------------------------------------------

/// Query a body's ecliptic-of-date longitude, latitude (deg), and distance (km).
fn body_ecliptic_of_date(
    engine: &Engine,
    body: Body,
    jd_tdb: f64,
) -> Result<(f64, f64, f64), SearchError> {
    let query = Query {
        target: body,
        observer: Observer::Body(Body::Earth),
        frame: Frame::IcrfJ2000,
        epoch_tdb_jd: jd_tdb,
    };
    let state = engine.query(query)?;
    let ecl_j2000 = icrf_to_ecliptic(&state.position_km);
    let t = (jd_tdb - 2_451_545.0) / 36525.0;
    let ecl_date = precess_ecliptic_j2000_to_date(&ecl_j2000, t);
    let sph = cartesian_to_spherical(&ecl_date);
    Ok((sph.lon_deg.rem_euclid(360.0), sph.lat_deg, sph.distance_km))
}

/// Query Moon's ecliptic-of-date longitude, latitude (deg), and distance (km).
fn moon_ecliptic(engine: &Engine, jd_tdb: f64) -> Result<(f64, f64, f64), SearchError> {
    body_ecliptic_of_date(engine, Body::Moon, jd_tdb)
}

/// Apparent geocentric equatorial coordinates for a body at an epoch.
///
/// Converts the body's ecliptic-of-date (lon, lat) to right ascension and
/// declination on the true equator/equinox of date (IAU 2000B nutation in
/// longitude and obliquity applied). Returns degrees, RA in [0, 360).
/// Standard spherical rotation; see docs/clean_room_equatorial_output.md.
fn apparent_equatorial_deg(
    engine: &Engine,
    body: Body,
    jd_tdb: f64,
) -> Result<(f64, f64), SearchError> {
    let (lon_deg, lat_deg, _) = body_ecliptic_of_date(engine, body, jd_tdb)?;
    let t = (jd_tdb - 2_451_545.0) / 36525.0;
    let (dpsi_arcsec, deps_arcsec) = nutation_iau2000b(t);
    let eps = mean_obliquity_of_date_rad(t) + (deps_arcsec / 3600.0).to_radians();
    let lon = (lon_deg + dpsi_arcsec / 3600.0).to_radians();
    let lat = lat_deg.to_radians();
    let (sin_eps, cos_eps) = (eps.sin(), eps.cos());
    let ra = (lon.sin() * cos_eps - lat.tan() * sin_eps).atan2(lon.cos());
    let sin_dec = lat.sin() * cos_eps + lat.cos() * sin_eps * lon.sin();
    let dec = sin_dec.clamp(-1.0, 1.0).asin();
    Ok((ra.to_degrees().rem_euclid(360.0), dec.to_degrees()))
}

/// Query Sun's distance from Earth in km.
fn sun_distance(engine: &Engine, jd_tdb: f64) -> Result<f64, SearchError> {
    let query = Query {
        target: Body::Sun,
        observer: Observer::Body(Body::Earth),
        frame: Frame::IcrfJ2000,
        epoch_tdb_jd: jd_tdb,
    };
    let state = engine.query(query)?;
    let r = (state.position_km[0].powi(2)
        + state.position_km[1].powi(2)
        + state.position_km[2].powi(2))
    .sqrt();
    Ok(r)
}

/// Angular separation between Sun and Moon centers (degrees) at a given epoch.
/// Computed from their ICRF positions relative to Earth.
fn sun_moon_angular_separation(engine: &Engine, jd_tdb: f64) -> Result<f64, SearchError> {
    let sun_q = Query {
        target: Body::Sun,
        observer: Observer::Body(Body::Earth),
        frame: Frame::IcrfJ2000,
        epoch_tdb_jd: jd_tdb,
    };
    let moon_q = Query {
        target: Body::Moon,
        observer: Observer::Body(Body::Earth),
        frame: Frame::IcrfJ2000,
        epoch_tdb_jd: jd_tdb,
    };
    let sun_state = engine.query(sun_q)?;
    let moon_state = engine.query(moon_q)?;

    // Unit vectors
    let s = &sun_state.position_km;
    let m = &moon_state.position_km;
    let r_s = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
    let r_m = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt();

    if r_s < 1e-10 || r_m < 1e-10 {
        return Ok(0.0);
    }

    let dot = (s[0] * m[0] + s[1] * m[1] + s[2] * m[2]) / (r_s * r_m);
    let angle_rad = dot.clamp(-1.0, 1.0).acos();
    Ok(angle_rad.to_degrees())
}

/// Compute Earth shadow radii at Moon's distance using the Danjon method.
///
/// Returns (penumbral_radius_deg, umbral_radius_deg) as angular radii
/// on the sky at the Moon's distance.
///
/// The Danjon method enlarges the geometrical shadow by 2% to account
/// for Earth's atmosphere.
fn shadow_radii_deg(sun_dist_km: f64, moon_dist_km: f64) -> (f64, f64) {
    // Parallax of Sun and Moon
    let pi_sun = (EARTH_RADIUS_KM / sun_dist_km).asin();
    let pi_moon = (EARTH_RADIUS_KM / moon_dist_km).asin();

    // Angular semidiameter of the Sun as seen from Earth
    let s_sun = (SUN_RADIUS_KM / sun_dist_km).asin();

    // Penumbral shadow radius (projected at Moon's distance)
    let penumbral_rad = DANJON_ENLARGEMENT * (pi_moon + pi_sun + s_sun);
    // Umbral shadow radius
    let umbral_rad = DANJON_ENLARGEMENT * (pi_moon + pi_sun - s_sun);

    (penumbral_rad.to_degrees(), umbral_rad.to_degrees())
}

/// Moon's angular semidiameter in degrees.
fn moon_angular_radius_deg(moon_dist_km: f64) -> f64 {
    (MOON_RADIUS_KM / moon_dist_km).asin().to_degrees()
}

/// Angular distance of the Moon's center from the anti-solar point (shadow axis).
/// At full moon, this is approximately 180 - (Sun-Moon separation),
/// which gives the angular offset from the center of Earth's shadow.
fn moon_shadow_offset_deg(engine: &Engine, jd_tdb: f64) -> Result<f64, SearchError> {
    let sep = sun_moon_angular_separation(engine, jd_tdb)?;
    // At exact opposition sep = 180°. Shadow offset = 180° - sep.
    // The Moon's ecliptic latitude drives this offset.
    Ok((180.0 - sep).abs())
}

// ---------------------------------------------------------------------------
// Shared vector / terrestrial geometry for solar visibility
// ---------------------------------------------------------------------------

#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

#[inline]
fn unit(a: [f64; 3]) -> [f64; 3] {
    let n = norm(a);
    if n <= f64::EPSILON {
        [0.0, 0.0, 0.0]
    } else {
        scale(a, 1.0 / n)
    }
}

fn utc_jd(utc: UtcTime) -> f64 {
    calendar_to_jd(
        utc.year,
        utc.month,
        utc.day as f64
            + utc.hour as f64 / 24.0
            + utc.minute as f64 / 1440.0
            + utc.second / 86_400.0,
    )
}

fn gast_rad_for(engine: &Engine, eop: Option<&EopKernel>, jd_tdb: f64) -> f64 {
    let utc = UtcTime::from_jd_tdb(jd_tdb, engine.lsk());
    let jd_utc = utc_jd(utc);
    let jd_ut1 = eop
        .and_then(|kernel| kernel.utc_to_ut1_jd(jd_utc).ok())
        .unwrap_or(jd_utc);
    let t = (jd_tdb - 2_451_545.0) / 36_525.0;
    let (equation_of_equinoxes, _) = equation_of_equinoxes_and_true_obliquity(t);
    (gmst_rad(jd_ut1) + equation_of_equinoxes).rem_euclid(std::f64::consts::TAU)
}

/// ICRF J2000 vector to true equatorial/equinox-of-date.
fn icrf_to_true_equatorial_of_date(v: [f64; 3], jd_tdb: f64) -> [f64; 3] {
    let t = (jd_tdb - 2_451_545.0) / 36_525.0;
    let ecl_j2000 = icrf_to_ecliptic(&v);
    let ecl_date = precess_ecliptic_j2000_to_date(&ecl_j2000, t);
    let (dpsi_arcsec, deps_arcsec) = nutation_iau2000b(t);
    let dpsi = (dpsi_arcsec / 3600.0).to_radians();
    let (sd, cd) = dpsi.sin_cos();
    let true_ecl = [
        cd * ecl_date[0] - sd * ecl_date[1],
        sd * ecl_date[0] + cd * ecl_date[1],
        ecl_date[2],
    ];
    let eps = mean_obliquity_of_date_rad(t) + (deps_arcsec / 3600.0).to_radians();
    let (se, ce) = eps.sin_cos();
    [
        true_ecl[0],
        ce * true_ecl[1] - se * true_ecl[2],
        se * true_ecl[1] + ce * true_ecl[2],
    ]
}

fn sun_moon_true_vectors(
    engine: &Engine,
    jd_tdb: f64,
) -> Result<([f64; 3], [f64; 3]), SearchError> {
    let query = |target| Query {
        target,
        observer: Observer::Body(Body::Earth),
        frame: Frame::IcrfJ2000,
        epoch_tdb_jd: jd_tdb,
    };
    let sun = engine.query(query(Body::Sun))?.position_km;
    let moon = engine.query(query(Body::Moon))?.position_km;
    Ok((
        icrf_to_true_equatorial_of_date(sun, jd_tdb),
        icrf_to_true_equatorial_of_date(moon, jd_tdb),
    ))
}

#[derive(Debug, Clone, Copy)]
struct ShadowGeometry {
    moon: [f64; 3],
    q: [f64; 3],
    east: [f64; 3],
    north: [f64; 3],
    axis_plane: [f64; 3],
    sun_moon_distance: f64,
    moon_to_plane: f64,
}

fn shadow_geometry(engine: &Engine, jd_tdb: f64) -> Result<ShadowGeometry, SearchError> {
    let (sun, moon) = sun_moon_true_vectors(engine, jd_tdb)?;
    // q points from the Moon back toward the Sun. The physical shadow travels
    // from the Moon in the -q direction.
    let q = unit(sub(sun, moon));
    let mut east = unit(cross([0.0, 0.0, 1.0], q));
    if norm(east) < 0.5 {
        east = unit(cross([0.0, 1.0, 0.0], q));
    }
    let north = unit(cross(q, east));
    let moon_to_plane = dot(moon, q);
    let axis_plane = sub(moon, scale(q, moon_to_plane));
    Ok(ShadowGeometry {
        moon,
        q,
        east,
        north,
        axis_plane,
        sun_moon_distance: norm(sub(sun, moon)),
        moon_to_plane,
    })
}

/// Compute instantaneous Besselian elements from Dhruv ephemeris vectors.
pub fn besselian_elements_at(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
) -> Result<BesselianElements, SearchError> {
    let g = shadow_geometry(engine, jd_tdb)?;
    let gast = gast_rad_for(engine, eop, jd_tdb);
    let ra = g.q[1].atan2(g.q[0]).rem_euclid(std::f64::consts::TAU);
    let dec = g.q[2].clamp(-1.0, 1.0).asin();
    let tan_f1 = (SUN_RADIUS_KM + MOON_RADIUS_KM) / g.sun_moon_distance;
    let tan_f2 = (SUN_RADIUS_KM - MOON_RADIUS_KM) / g.sun_moon_distance;
    let penumbra_radius = MOON_RADIUS_KM + g.moon_to_plane * tan_f1;
    let signed_umbra_radius = MOON_RADIUS_KM - g.moon_to_plane * tan_f2;
    Ok(BesselianElements {
        jd_tdb,
        utc: UtcTime::from_jd_tdb(jd_tdb, engine.lsk()),
        x: dot(g.axis_plane, g.east) / EARTH_RADIUS_KM,
        y: dot(g.axis_plane, g.north) / EARTH_RADIUS_KM,
        d_deg: dec.to_degrees(),
        mu_deg: (gast - ra).to_degrees().rem_euclid(360.0),
        l1: penumbra_radius / EARTH_RADIUS_KM,
        // NASA-compatible sign: total/umbra negative, annular/antumbra positive.
        l2: -signed_umbra_radius / EARTH_RADIUS_KM,
        tan_f1,
        tan_f2,
    })
}

fn rotate_z(v: [f64; 3], angle: f64) -> [f64; 3] {
    let (s, c) = angle.sin_cos();
    [c * v[0] - s * v[1], s * v[0] + c * v[1], v[2]]
}

fn ecef_to_geodetic(v: [f64; 3]) -> EclipseGeoPoint {
    let a = EARTH_RADIUS_KM;
    let b = EARTH_POLAR_RADIUS_KM;
    let e2 = 1.0 - b * b / (a * a);
    let p = v[0].hypot(v[1]);
    let mut lat = v[2].atan2(p * (1.0 - e2));
    for _ in 0..8 {
        let sin_lat = lat.sin();
        let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();
        lat = (v[2] + e2 * n * sin_lat).atan2(p);
    }
    let lon = v[1].atan2(v[0]).to_degrees().rem_euclid(360.0);
    EclipseGeoPoint {
        latitude_deg: lat.to_degrees(),
        longitude_deg: if lon > 180.0 { lon - 360.0 } else { lon },
    }
}

fn geodetic_to_ecef(location: &GeoLocation) -> [f64; 3] {
    let a = EARTH_RADIUS_KM;
    let b = EARTH_POLAR_RADIUS_KM;
    let e2 = 1.0 - b * b / (a * a);
    let lat = location.latitude_rad();
    let lon = location.longitude_rad();
    let h = location.altitude_m / 1000.0;
    let n = a / (1.0 - e2 * lat.sin().powi(2)).sqrt();
    [
        (n + h) * lat.cos() * lon.cos(),
        (n + h) * lat.cos() * lon.sin(),
        (n * (1.0 - e2) + h) * lat.sin(),
    ]
}

fn ray_ellipsoid_intersections(origin: [f64; 3], direction: [f64; 3]) -> Vec<(f64, [f64; 3])> {
    let d = unit(direction);
    let a2 = EARTH_RADIUS_KM * EARTH_RADIUS_KM;
    let b2 = EARTH_POLAR_RADIUS_KM * EARTH_POLAR_RADIUS_KM;
    let qa = (d[0] * d[0] + d[1] * d[1]) / a2 + d[2] * d[2] / b2;
    let qb = 2.0 * ((origin[0] * d[0] + origin[1] * d[1]) / a2 + origin[2] * d[2] / b2);
    let qc =
        (origin[0] * origin[0] + origin[1] * origin[1]) / a2 + origin[2] * origin[2] / b2 - 1.0;
    let disc = qb * qb - 4.0 * qa * qc;
    if disc < 0.0 {
        return Vec::new();
    }
    let root = disc.sqrt();
    let t1 = (-qb - root) / (2.0 * qa);
    let t2 = (-qb + root) / (2.0 * qa);
    let mut hits = [t1, t2]
        .into_iter()
        .filter(|t| *t >= 0.0)
        .map(|t| (t, add(origin, scale(d, t))))
        .collect::<Vec<_>>();
    hits.sort_by(|a, b| a.0.total_cmp(&b.0));
    hits
}

fn ray_ellipsoid_intersection(origin: [f64; 3], direction: [f64; 3]) -> Option<[f64; 3]> {
    ray_ellipsoid_intersections(origin, direction)
        .into_iter()
        .next()
        .map(|(_, point)| point)
}

fn axis_ground_point(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
) -> Result<Option<EclipseGeoPoint>, SearchError> {
    let g = shadow_geometry(engine, jd_tdb)?;
    let hit = ray_ellipsoid_intersection(g.moon, scale(g.q, -1.0));
    Ok(hit.map(|p| ecef_to_geodetic(rotate_z(p, -gast_rad_for(engine, eop, jd_tdb)))))
}

#[derive(Debug, Clone, Copy)]
enum ShadowCone {
    Penumbra,
    Central,
}

fn shadow_boundary(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
    cone: ShadowCone,
    step_deg: u32,
) -> Result<Vec<EclipseGeoPoint>, SearchError> {
    let g = shadow_geometry(engine, jd_tdb)?;
    let (apex, tan_angle) = match cone {
        ShadowCone::Penumbra => {
            let distance = MOON_RADIUS_KM * g.sun_moon_distance / (SUN_RADIUS_KM + MOON_RADIUS_KM);
            (
                add(g.moon, scale(g.q, distance)),
                (SUN_RADIUS_KM + MOON_RADIUS_KM) / g.sun_moon_distance,
            )
        }
        ShadowCone::Central => {
            let distance = MOON_RADIUS_KM * g.sun_moon_distance / (SUN_RADIUS_KM - MOON_RADIUS_KM);
            (
                sub(g.moon, scale(g.q, distance)),
                (SUN_RADIUS_KM - MOON_RADIUS_KM) / g.sun_moon_distance,
            )
        }
    };
    let toward_earth = unit(scale(apex, -1.0));
    let axis_sign = if dot(toward_earth, g.q) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let axis = scale(g.q, axis_sign);
    let cos_angle = 1.0 / (1.0 + tan_angle * tan_angle).sqrt();
    let sin_angle = tan_angle * cos_angle;
    let gast = gast_rad_for(engine, eop, jd_tdb);
    let step = step_deg.clamp(1, 15) as usize;
    let mut points = Vec::with_capacity(360 / step + 1);
    for deg in (0..360).step_by(step) {
        let phi = (deg as f64).to_radians();
        let radial = add(scale(g.east, phi.cos()), scale(g.north, phi.sin()));
        let direction = add(scale(axis, cos_angle), scale(radial, sin_angle));
        let hit = ray_ellipsoid_intersections(apex, direction)
            .into_iter()
            .map(|(_, point)| point)
            .max_by(|a, b| dot(*a, g.q).total_cmp(&dot(*b, g.q)));
        if let Some(hit) = hit {
            points.push(ecef_to_geodetic(rotate_z(hit, -gast)));
        }
    }
    Ok(points)
}

fn bessel_penumbra_metric(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
) -> Result<f64, SearchError> {
    let b = besselian_elements_at(engine, eop, jd_tdb)?;
    Ok((b.x * b.x + b.y * b.y).sqrt() - (1.0 + b.l1))
}

fn minimize_scalar<F>(mut left: f64, mut right: f64, f: F) -> Result<f64, SearchError>
where
    F: Fn(f64) -> Result<f64, SearchError>,
{
    let phi = (5.0_f64.sqrt() - 1.0) * 0.5;
    let mut c = right - phi * (right - left);
    let mut d = left + phi * (right - left);
    let mut fc = f(c)?;
    let mut fd = f(d)?;
    for _ in 0..64 {
        if fc <= fd {
            right = d;
            d = c;
            fd = fc;
            c = right - phi * (right - left);
            fc = f(c)?;
        } else {
            left = c;
            c = d;
            fc = fd;
            d = left + phi * (right - left);
            fd = f(d)?;
        }
        if right - left < CONTACT_CONVERGENCE_DAYS {
            break;
        }
    }
    Ok((left + right) * 0.5)
}

fn root_bisection<F>(mut left: f64, mut right: f64, f: F) -> Result<Option<f64>, SearchError>
where
    F: Fn(f64) -> Result<f64, SearchError>,
{
    let mut fl = f(left)?;
    let fr = f(right)?;
    if fl == 0.0 {
        return Ok(Some(left));
    }
    if fr == 0.0 {
        return Ok(Some(right));
    }
    if fl * fr > 0.0 {
        return Ok(None);
    }
    for _ in 0..CONTACT_MAX_ITER {
        let mid = (left + right) * 0.5;
        let fm = f(mid)?;
        if fl * fm <= 0.0 {
            right = mid;
        } else {
            left = mid;
            fl = fm;
        }
        if right - left < CONTACT_CONVERGENCE_DAYS {
            break;
        }
    }
    Ok(Some((left + right) * 0.5))
}

fn surrounding_roots<F>(
    center: f64,
    half_window_days: f64,
    step_minutes: f64,
    f: F,
) -> Result<(Option<f64>, Option<f64>), SearchError>
where
    F: Fn(f64) -> Result<f64, SearchError> + Copy,
{
    let step = step_minutes / 1440.0;
    let mut previous_t = center - half_window_days;
    let mut previous_f = f(previous_t)?;
    let mut before = None;
    let mut after = None;
    let mut t = previous_t + step;
    while t <= center + half_window_days + step * 0.5 {
        let value = f(t)?;
        if previous_f * value <= 0.0
            && let Some(root) = root_bisection(previous_t, t, f)?
        {
            if root <= center {
                before = Some(root);
            } else if after.is_none() {
                after = Some(root);
            }
        }
        previous_t = t;
        previous_f = value;
        t += step;
    }
    Ok((before, after))
}

#[derive(Debug, Clone, Copy)]
struct LocalDiskGeometry {
    separation_rad: f64,
    sun_radius_rad: f64,
    moon_radius_rad: f64,
    sun_altitude_deg: f64,
    sun_azimuth_deg: f64,
}

fn local_disk_geometry(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
    location: &GeoLocation,
) -> Result<LocalDiskGeometry, SearchError> {
    let (sun, moon) = sun_moon_true_vectors(engine, jd_tdb)?;
    let gast = gast_rad_for(engine, eop, jd_tdb);
    let observer_ecef = geodetic_to_ecef(location);
    let observer_eq = rotate_z(observer_ecef, gast);
    let sun_topo = sub(sun, observer_eq);
    let moon_topo = sub(moon, observer_eq);
    let sun_distance = norm(sun_topo);
    let moon_distance = norm(moon_topo);
    let sun_u = unit(sun_topo);
    let moon_u = unit(moon_topo);
    let separation_rad = dot(sun_u, moon_u).clamp(-1.0, 1.0).acos();
    let sun_ecef = rotate_z(sun_u, -gast);
    let lat = location.latitude_rad();
    let lon = location.longitude_rad();
    let east = -lon.sin() * sun_ecef[0] + lon.cos() * sun_ecef[1];
    let north = -lat.sin() * lon.cos() * sun_ecef[0] - lat.sin() * lon.sin() * sun_ecef[1]
        + lat.cos() * sun_ecef[2];
    let up = lat.cos() * lon.cos() * sun_ecef[0]
        + lat.cos() * lon.sin() * sun_ecef[1]
        + lat.sin() * sun_ecef[2];
    Ok(LocalDiskGeometry {
        separation_rad,
        sun_radius_rad: (SUN_RADIUS_KM / sun_distance).asin(),
        moon_radius_rad: (MOON_RADIUS_KM / moon_distance).asin(),
        sun_altitude_deg: up.clamp(-1.0, 1.0).asin().to_degrees(),
        sun_azimuth_deg: east.atan2(north).to_degrees().rem_euclid(360.0),
    })
}

fn disk_magnitude(g: LocalDiskGeometry) -> f64 {
    ((g.sun_radius_rad + g.moon_radius_rad - g.separation_rad) / (2.0 * g.sun_radius_rad)).max(0.0)
}

fn disk_obscuration(g: LocalDiskGeometry) -> f64 {
    let sun_radius = g.sun_radius_rad;
    let moon_radius = g.moon_radius_rad;
    let d = g.separation_rad;
    if d >= sun_radius + moon_radius {
        return 0.0;
    }
    if d <= (moon_radius - sun_radius).abs() {
        return if moon_radius >= sun_radius {
            1.0
        } else {
            moon_radius.powi(2) / sun_radius.powi(2)
        };
    }
    let alpha = ((d * d + sun_radius.powi(2) - moon_radius.powi(2)) / (2.0 * d * sun_radius))
        .clamp(-1.0, 1.0)
        .acos();
    let beta = ((d * d + moon_radius.powi(2) - sun_radius.powi(2)) / (2.0 * d * moon_radius))
        .clamp(-1.0, 1.0)
        .acos();
    let area = sun_radius.powi(2) * alpha + moon_radius.powi(2) * beta
        - 0.5
            * ((-d + sun_radius + moon_radius)
                * (d + sun_radius - moon_radius)
                * (d - sun_radius + moon_radius)
                * (d + sun_radius + moon_radius))
                .sqrt();
    (area / (std::f64::consts::PI * sun_radius.powi(2))).clamp(0.0, 1.0)
}

fn local_type(g: LocalDiskGeometry) -> Option<SuryaGrahanType> {
    if g.separation_rad >= g.sun_radius_rad + g.moon_radius_rad {
        None
    } else if g.separation_rad < (g.moon_radius_rad - g.sun_radius_rad).abs() {
        if g.moon_radius_rad >= g.sun_radius_rad {
            Some(SuryaGrahanType::Total)
        } else {
            Some(SuryaGrahanType::Annular)
        }
    } else {
        Some(SuryaGrahanType::Partial)
    }
}

fn local_circumstances(
    engine: &Engine,
    eop: Option<&EopKernel>,
    location: GeoLocation,
    near_jd: f64,
) -> Result<SuryaGrahanLocalCircumstances, SearchError> {
    let maximum_jd = minimize_scalar(near_jd - 0.25, near_jd + 0.25, |jd| {
        Ok(local_disk_geometry(engine, eop, jd, &location)?.separation_rad)
    })?;
    let maximum = local_disk_geometry(engine, eop, maximum_jd, &location)?;
    let external = |jd| -> Result<f64, SearchError> {
        let g = local_disk_geometry(engine, eop, jd, &location)?;
        Ok(g.separation_rad - g.sun_radius_rad - g.moon_radius_rad)
    };
    let internal = |jd| -> Result<f64, SearchError> {
        let g = local_disk_geometry(engine, eop, jd, &location)?;
        Ok(g.separation_rad - (g.moon_radius_rad - g.sun_radius_rad).abs())
    };
    let (c1, c4) = surrounding_roots(maximum_jd, 0.3, 2.0, external)?;
    let typ = local_type(maximum);
    let (c2, c3) = if matches!(typ, Some(SuryaGrahanType::Total | SuryaGrahanType::Annular)) {
        surrounding_roots(maximum_jd, 0.08, 0.25, internal)?
    } else {
        (None, None)
    };
    let visibility_start = c1.unwrap_or(maximum_jd - 0.2);
    let visibility_end = c4.unwrap_or(maximum_jd + 0.2);
    let mut visible = false;
    let mut sample_jd = visibility_start;
    while sample_jd <= visibility_end + 1.0 / 2880.0 {
        let sample = local_disk_geometry(engine, eop, sample_jd, &location)?;
        if local_type(sample).is_some() && sample.sun_altitude_deg > -0.833 {
            visible = true;
            break;
        }
        sample_jd += 2.0 / 1440.0;
    }
    Ok(SuryaGrahanLocalCircumstances {
        location,
        visible,
        grahan_type: typ,
        maximum_jd: typ.map(|_| maximum_jd),
        maximum_utc: typ.map(|_| UtcTime::from_jd_tdb(maximum_jd, engine.lsk())),
        c1_jd: c1,
        c1_utc: c1.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        c2_jd: c2,
        c2_utc: c2.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        c3_jd: c3,
        c3_utc: c3.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        c4_jd: c4,
        c4_utc: c4.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        magnitude: if typ.is_some() {
            disk_magnitude(maximum)
        } else {
            0.0
        },
        obscuration: if typ.is_some() {
            disk_obscuration(maximum)
        } else {
            0.0
        },
        sun_altitude_deg: maximum.sun_altitude_deg,
        sun_azimuth_deg: maximum.sun_azimuth_deg,
        central_duration_seconds: match (c2, c3) {
            (Some(start), Some(end)) => (end - start) * 86_400.0,
            _ => 0.0,
        },
    })
}

fn validate_surya_inputs(
    location: Option<GeoLocation>,
    config: &GrahanConfig,
) -> Result<(), SearchError> {
    if !(1..=30).contains(&config.path_step_minutes) {
        return Err(SearchError::InvalidConfig(
            "path_step_minutes must be between 1 and 30",
        ));
    }
    if !(1..=15).contains(&config.boundary_step_deg) {
        return Err(SearchError::InvalidConfig(
            "boundary_step_deg must be between 1 and 15",
        ));
    }
    if let Some(location) = location {
        if !location.latitude_deg.is_finite() || !(-90.0..=90.0).contains(&location.latitude_deg) {
            return Err(SearchError::InvalidConfig(
                "location latitude must be finite and between -90 and 90",
            ));
        }
        if !location.longitude_deg.is_finite()
            || !(-180.0..=180.0).contains(&location.longitude_deg)
        {
            return Err(SearchError::InvalidConfig(
                "location longitude must be finite and between -180 and 180",
            ));
        }
        if !location.altitude_m.is_finite() {
            return Err(SearchError::InvalidConfig(
                "location altitude must be finite",
            ));
        }
    }
    Ok(())
}

fn haversine_km(a: EclipseGeoPoint, b: EclipseGeoPoint) -> f64 {
    let lat1 = a.latitude_deg.to_radians();
    let lat2 = b.latitude_deg.to_radians();
    let dlat = lat2 - lat1;
    let dlon = (b.longitude_deg - a.longitude_deg).to_radians();
    let h = (dlat * 0.5).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon * 0.5).sin().powi(2);
    2.0 * EARTH_RADIUS_KM * h.sqrt().asin()
}

fn closest_axis_surface_point(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
) -> Result<EclipseGeoPoint, SearchError> {
    if let Some(point) = axis_ground_point(engine, eop, jd_tdb)? {
        return Ok(point);
    }
    let g = shadow_geometry(engine, jd_tdb)?;
    let p = unit(g.axis_plane);
    let scale_to_surface = 1.0
        / ((p[0] * p[0] + p[1] * p[1]) / EARTH_RADIUS_KM.powi(2)
            + p[2] * p[2] / EARTH_POLAR_RADIUS_KM.powi(2))
        .sqrt();
    Ok(ecef_to_geodetic(rotate_z(
        scale(p, scale_to_surface),
        -gast_rad_for(engine, eop, jd_tdb),
    )))
}

fn path_point(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
    boundary_step_deg: u32,
) -> Result<Option<SuryaGrahanPathPoint>, SearchError> {
    let Some(center) = axis_ground_point(engine, eop, jd_tdb)? else {
        return Ok(None);
    };
    let boundary = shadow_boundary(engine, eop, jd_tdb, ShadowCone::Central, boundary_step_deg)?;
    if boundary.is_empty() {
        return Ok(None);
    }
    let northern_limit = boundary
        .iter()
        .copied()
        .max_by(|a, b| a.latitude_deg.total_cmp(&b.latitude_deg));
    let southern_limit = boundary
        .iter()
        .copied()
        .min_by(|a, b| a.latitude_deg.total_cmp(&b.latitude_deg));
    let width_km = boundary
        .iter()
        .map(|point| haversine_km(center, *point))
        .filter(|distance| *distance > 0.001)
        .min_by(f64::total_cmp)
        .unwrap_or(0.0)
        * 2.0;
    let location = GeoLocation::new(center.latitude_deg, center.longitude_deg, 0.0);
    let local = local_disk_geometry(engine, eop, jd_tdb, &location)?;
    let grahan_type = local_type(local).unwrap_or(SuryaGrahanType::Partial);
    Ok(Some(SuryaGrahanPathPoint {
        jd_tdb,
        utc: UtcTime::from_jd_tdb(jd_tdb, engine.lsk()),
        center,
        northern_limit,
        southern_limit,
        width_km,
        central_duration_seconds: 0.0,
        sun_altitude_deg: local.sun_altitude_deg,
        sun_azimuth_deg: local.sun_azimuth_deg,
        grahan_type,
    }))
}

fn sample_path_and_footprints(
    engine: &Engine,
    eop: Option<&EopKernel>,
    start_jd: f64,
    end_jd: f64,
    config: &GrahanConfig,
) -> Result<(Vec<SuryaGrahanPathPoint>, Vec<SuryaGrahanFootprint>), SearchError> {
    if !config.include_path {
        return Ok((Vec::new(), Vec::new()));
    }
    let step = config.path_step_minutes.clamp(1, 30) as f64 / 1440.0;
    let boundary_step = config.boundary_step_deg.clamp(1, 15);
    let mut path = Vec::new();
    let mut footprints = Vec::new();
    let mut jd = start_jd;
    while jd <= end_jd + step * 0.5 {
        let boundary = shadow_boundary(engine, eop, jd, ShadowCone::Penumbra, boundary_step)?;
        if !boundary.is_empty() {
            footprints.push(SuryaGrahanFootprint {
                jd_tdb: jd,
                utc: UtcTime::from_jd_tdb(jd, engine.lsk()),
                boundary,
            });
        }
        if let Some(point) = path_point(engine, eop, jd, boundary_step)? {
            path.push(point);
        }
        jd += step;
    }
    // Approximate local central duration from adjacent path-center speed.
    for index in 0..path.len() {
        let speed = if path.len() < 2 {
            0.0
        } else if index == 0 {
            haversine_km(path[0].center, path[1].center) / (step * 86_400.0)
        } else if index + 1 == path.len() {
            haversine_km(path[index - 1].center, path[index].center) / (step * 86_400.0)
        } else {
            haversine_km(path[index - 1].center, path[index + 1].center) / (2.0 * step * 86_400.0)
        };
        if speed > 1.0e-6 {
            path[index].central_duration_seconds = path[index].width_km / speed;
        }
    }
    Ok((path, footprints))
}

// ---------------------------------------------------------------------------
// Chandra grahan (lunar eclipses)
// ---------------------------------------------------------------------------

/// Classify a chandra grahan based on geometry.
fn classify_chandra(
    shadow_offset_deg: f64,
    moon_radius_deg: f64,
    umbral_radius_deg: f64,
    penumbral_radius_deg: f64,
) -> Option<ChandraGrahanType> {
    let moon_near_edge = shadow_offset_deg - moon_radius_deg;
    let moon_far_edge = shadow_offset_deg + moon_radius_deg;

    if moon_near_edge >= penumbral_radius_deg {
        // Moon entirely outside penumbra — no grahan
        None
    } else if moon_far_edge <= umbral_radius_deg {
        // Moon entirely inside umbra — total
        Some(ChandraGrahanType::Total)
    } else if moon_near_edge < umbral_radius_deg {
        // Moon partially inside umbra — partial
        Some(ChandraGrahanType::Partial)
    } else {
        // Moon in penumbra only
        Some(ChandraGrahanType::Penumbral)
    }
}

/// Find a contact time by bisecting when Moon's limb crosses a shadow boundary.
///
/// `boundary_radius_deg` is the shadow radius (umbral or penumbral).
/// `limb_sign`: -1.0 for near limb (inner), +1.0 for far limb (outer).
/// Searches between `t_a` and `t_b`.
fn find_chandra_contact(
    engine: &Engine,
    t_a: f64,
    t_b: f64,
    boundary_radius_deg: f64,
    limb_sign: f64,
) -> Result<f64, SearchError> {
    // f(t) = (shadow_offset + limb_sign * moon_radius) - boundary_radius
    // We look for f(t) = 0

    let f = |jd: f64| -> Result<f64, SearchError> {
        let offset = moon_shadow_offset_deg(engine, jd)?;
        let (_, _, moon_dist) = moon_ecliptic(engine, jd)?;
        let moon_r = moon_angular_radius_deg(moon_dist);
        Ok(offset + limb_sign * moon_r - boundary_radius_deg)
    };

    let mut ta = t_a;
    let mut tb = t_b;
    let mut fa = f(ta)?;

    for _ in 0..CONTACT_MAX_ITER {
        let tm = 0.5 * (ta + tb);
        let fm = f(tm)?;

        if fa * fm <= 0.0 {
            tb = tm;
        } else {
            ta = tm;
            fa = fm;
        }

        if (tb - ta).abs() < CONTACT_CONVERGENCE_DAYS {
            break;
        }
    }

    Ok(0.5 * (ta + tb))
}

/// Compute a single chandra grahan from a full moon event.
fn compute_chandra_grahan(
    engine: &Engine,
    full_moon_jd: f64,
    config: &GrahanConfig,
) -> Result<Option<ChandraGrahan>, SearchError> {
    // Get Moon's ecliptic latitude at full moon
    let (_, moon_lat, moon_dist) = moon_ecliptic(engine, full_moon_jd)?;

    // Quick filter
    if moon_lat.abs() > GRAHAN_LAT_THRESHOLD_DEG {
        return Ok(None);
    }

    let sun_dist = sun_distance(engine, full_moon_jd)?;
    let (penumbral_radius, umbral_radius) = shadow_radii_deg(sun_dist, moon_dist);
    let moon_radius = moon_angular_radius_deg(moon_dist);
    let shadow_offset = moon_shadow_offset_deg(engine, full_moon_jd)?;

    let grahan_type =
        match classify_chandra(shadow_offset, moon_radius, umbral_radius, penumbral_radius) {
            Some(t) => t,
            None => return Ok(None),
        };

    if !config.include_penumbral && grahan_type == ChandraGrahanType::Penumbral {
        return Ok(None);
    }

    // Compute magnitudes
    let umbral_magnitude = (umbral_radius - shadow_offset + moon_radius) / (2.0 * moon_radius);
    let penumbral_magnitude =
        (penumbral_radius - shadow_offset + moon_radius) / (2.0 * moon_radius);

    // Contact times — search window: ~6 hours around greatest grahan
    let half_window = 0.25; // 6 hours in days

    // P1: near limb enters penumbra (outer limb, going in)
    let p1_jd = find_chandra_contact(
        engine,
        full_moon_jd - half_window,
        full_moon_jd,
        penumbral_radius,
        1.0, // far limb crosses penumbra boundary
    )?;

    // P4: far limb exits penumbra
    let p4_jd = find_chandra_contact(
        engine,
        full_moon_jd,
        full_moon_jd + half_window,
        penumbral_radius,
        1.0,
    )?;

    // U1/U4: umbral contacts (only if partial or total)
    let (u1_jd, u4_jd) = if grahan_type != ChandraGrahanType::Penumbral {
        let u1 = find_chandra_contact(
            engine,
            full_moon_jd - half_window,
            full_moon_jd,
            umbral_radius,
            1.0,
        )?;
        let u4 = find_chandra_contact(
            engine,
            full_moon_jd,
            full_moon_jd + half_window,
            umbral_radius,
            1.0,
        )?;
        (Some(u1), Some(u4))
    } else {
        (None, None)
    };

    // U2/U3: totality contacts (only if total)
    let (u2_jd, u3_jd) = if grahan_type == ChandraGrahanType::Total {
        let u2 = find_chandra_contact(
            engine,
            full_moon_jd - half_window,
            full_moon_jd,
            umbral_radius,
            -1.0, // near limb crosses umbra boundary
        )?;
        let u3 = find_chandra_contact(
            engine,
            full_moon_jd,
            full_moon_jd + half_window,
            umbral_radius,
            -1.0,
        )?;
        (Some(u2), Some(u3))
    } else {
        (None, None)
    };

    let angular_sep = sun_moon_angular_separation(engine, full_moon_jd)?;
    let (moon_ra, moon_dec) = apparent_equatorial_deg(engine, Body::Moon, full_moon_jd)?;

    Ok(Some(ChandraGrahan {
        grahan_type,
        magnitude: umbral_magnitude,
        penumbral_magnitude,
        greatest_grahan_jd: full_moon_jd,
        greatest_grahan_utc: UtcTime::from_jd_tdb(full_moon_jd, engine.lsk()),
        p1_jd,
        p1_utc: UtcTime::from_jd_tdb(p1_jd, engine.lsk()),
        u1_jd,
        u1_utc: u1_jd.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        u2_jd,
        u2_utc: u2_jd.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        u3_jd,
        u3_utc: u3_jd.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        u4_jd,
        u4_utc: u4_jd.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        p4_jd,
        p4_utc: UtcTime::from_jd_tdb(p4_jd, engine.lsk()),
        moon_ecliptic_lat_deg: moon_lat,
        angular_separation_deg: angular_sep,
        moon_right_ascension_deg: moon_ra,
        moon_declination_deg: moon_dec,
    }))
}

/// Find the next chandra grahan (lunar eclipse) after `jd_tdb`.
pub fn next_chandra_grahan(
    engine: &Engine,
    jd_tdb: f64,
    config: &GrahanConfig,
) -> Result<Option<ChandraGrahan>, SearchError> {
    let moon_config = ConjunctionConfig::opposition(MOON_STEP_DAYS);
    let mut search_jd = jd_tdb;

    // Search up to ~2 years (enough for at least 2 grahan seasons)
    for _ in 0..50 {
        let full_moon = next_conjunction(engine, Body::Sun, Body::Moon, search_jd, &moon_config)?;
        let Some(fm) = full_moon else {
            return Ok(None);
        };

        if let Some(grahan) = compute_chandra_grahan(engine, fm.jd_tdb, config)? {
            return Ok(Some(grahan));
        }

        // Advance past this full moon
        search_jd = fm.jd_tdb + 1.0;
    }

    Ok(None)
}

/// Find the previous chandra grahan (lunar eclipse) before `jd_tdb`.
pub fn prev_chandra_grahan(
    engine: &Engine,
    jd_tdb: f64,
    config: &GrahanConfig,
) -> Result<Option<ChandraGrahan>, SearchError> {
    let moon_config = ConjunctionConfig::opposition(MOON_STEP_DAYS);
    let mut search_jd = jd_tdb;

    for _ in 0..50 {
        let full_moon = prev_conjunction(engine, Body::Sun, Body::Moon, search_jd, &moon_config)?;
        let Some(fm) = full_moon else {
            return Ok(None);
        };

        if let Some(grahan) = compute_chandra_grahan(engine, fm.jd_tdb, config)? {
            return Ok(Some(grahan));
        }

        search_jd = fm.jd_tdb - 1.0;
    }

    Ok(None)
}

/// Search for all chandra grahan in a time range.
pub fn search_chandra_grahan(
    engine: &Engine,
    jd_start: f64,
    jd_end: f64,
    config: &GrahanConfig,
) -> Result<Vec<ChandraGrahan>, SearchError> {
    if jd_end <= jd_start {
        return Err(SearchError::InvalidConfig("jd_end must be after jd_start"));
    }

    let moon_config = ConjunctionConfig::opposition(MOON_STEP_DAYS);
    let full_moons = search_conjunctions(
        engine,
        Body::Sun,
        Body::Moon,
        jd_start,
        jd_end,
        &moon_config,
    )?;

    let mut results = Vec::new();
    for fm in &full_moons {
        if let Some(grahan) = compute_chandra_grahan(engine, fm.jd_tdb, config)? {
            results.push(grahan);
        }
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// Surya grahan (solar eclipses — Earth ellipsoid and topocentric visibility)
// ---------------------------------------------------------------------------

/// Compute a solar eclipse from an ephemeris-derived new-moon candidate.
fn compute_surya_grahan(
    engine: &Engine,
    eop: Option<&EopKernel>,
    new_moon_jd: f64,
    location: Option<GeoLocation>,
    config: &GrahanConfig,
) -> Result<Option<SuryaGrahan>, SearchError> {
    let greatest_jd = minimize_scalar(new_moon_jd - 0.5, new_moon_jd + 0.5, |jd| {
        let b = besselian_elements_at(engine, eop, jd)?;
        Ok((b.x * b.x + b.y * b.y).sqrt())
    })?;
    let besselian = besselian_elements_at(engine, eop, greatest_jd)?;
    let rho = (besselian.x * besselian.x + besselian.y * besselian.y).sqrt();
    if rho >= 1.0 + besselian.l1 {
        return Ok(None);
    }

    let (c1_jd, c4_jd) = surrounding_roots(greatest_jd, 0.35, 2.0, |jd| {
        bessel_penumbra_metric(engine, eop, jd)
    })?;
    let central_cone_metric = |jd| -> Result<f64, SearchError> {
        let b = besselian_elements_at(engine, eop, jd)?;
        Ok((b.x * b.x + b.y * b.y).sqrt() - (1.0 + b.l2.abs()))
    };
    let central_location = axis_ground_point(engine, eop, greatest_jd)?;
    let central_reaches_earth = central_location.is_some()
        || !shadow_boundary(engine, eop, greatest_jd, ShadowCone::Central, 10)?.is_empty();
    let (c2_jd, c3_jd) = if central_reaches_earth {
        surrounding_roots(greatest_jd, 0.2, 1.0, central_cone_metric)?
    } else {
        (None, None)
    };
    let start = c1_jd.unwrap_or(greatest_jd - 0.2);
    let end = c4_jd.unwrap_or(greatest_jd + 0.2);
    let (path, footprints) = sample_path_and_footprints(engine, eop, start, end, config)?;

    let peak_point = match central_location {
        Some(point) => point,
        None => closest_axis_surface_point(engine, eop, greatest_jd)?,
    };
    let greatest_location = Some(peak_point);
    let peak_geo = GeoLocation::new(peak_point.latitude_deg, peak_point.longitude_deg, 0.0);
    let peak = local_disk_geometry(engine, eop, greatest_jd, &peak_geo)?;
    let mut grahan_type = if central_reaches_earth {
        if besselian.l2 < 0.0 {
            SuryaGrahanType::Total
        } else {
            SuryaGrahanType::Annular
        }
    } else {
        SuryaGrahanType::Partial
    };
    if let (Some(begin), Some(end)) = (c2_jd, c3_jd) {
        let mut has_total = false;
        let mut has_annular = false;
        let mut jd = begin;
        while jd <= end + 1.0 / 28_800.0 {
            if let Some(point) = axis_ground_point(engine, eop, jd)? {
                let location = GeoLocation::new(point.latitude_deg, point.longitude_deg, 0.0);
                match local_type(local_disk_geometry(engine, eop, jd, &location)?) {
                    Some(SuryaGrahanType::Total) => has_total = true,
                    Some(SuryaGrahanType::Annular) => has_annular = true,
                    _ => {}
                }
            }
            jd += 0.1 / 1440.0;
        }
        if has_total && has_annular {
            grahan_type = SuryaGrahanType::Hybrid;
        }
    }

    let (_, moon_lat, _) = moon_ecliptic(engine, greatest_jd)?;
    let min_sep = sun_moon_angular_separation(engine, greatest_jd)?;
    let (sun_ra, sun_dec) = apparent_equatorial_deg(engine, Body::Sun, greatest_jd)?;
    let local = match location {
        Some(point) => Some(local_circumstances(engine, eop, point, greatest_jd)?),
        None => None,
    };

    Ok(Some(SuryaGrahan {
        grahan_type,
        magnitude: disk_magnitude(peak),
        obscuration: disk_obscuration(peak),
        apparent_diameter_ratio: peak.moon_radius_rad / peak.sun_radius_rad,
        gamma: rho.copysign(moon_lat),
        greatest_grahan_jd: greatest_jd,
        greatest_grahan_utc: UtcTime::from_jd_tdb(greatest_jd, engine.lsk()),
        c1_jd,
        c1_utc: c1_jd.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        c2_jd,
        c2_utc: c2_jd.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        c3_jd,
        c3_utc: c3_jd.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        c4_jd,
        c4_utc: c4_jd.map(|jd| UtcTime::from_jd_tdb(jd, engine.lsk())),
        moon_ecliptic_lat_deg: moon_lat,
        angular_separation_deg: min_sep,
        sun_right_ascension_deg: sun_ra,
        sun_declination_deg: sun_dec,
        greatest_location,
        besselian,
        path,
        footprints,
        local,
    }))
}

/// Find the next solar eclipse after `jd_tdb`.
pub fn next_surya_grahan(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
    location: Option<GeoLocation>,
    config: &GrahanConfig,
) -> Result<Option<SuryaGrahan>, SearchError> {
    validate_surya_inputs(location, config)?;
    let moon_config = ConjunctionConfig::conjunction(MOON_STEP_DAYS);
    let mut search_jd = jd_tdb;

    for _ in 0..50 {
        let new_moon = next_conjunction(engine, Body::Sun, Body::Moon, search_jd, &moon_config)?;
        let Some(nm) = new_moon else {
            return Ok(None);
        };

        if let Some(grahan) = compute_surya_grahan(engine, eop, nm.jd_tdb, location, config)? {
            return Ok(Some(grahan));
        }

        search_jd = nm.jd_tdb + 1.0;
    }

    Ok(None)
}

/// Find the previous solar eclipse before `jd_tdb`.
pub fn prev_surya_grahan(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
    location: Option<GeoLocation>,
    config: &GrahanConfig,
) -> Result<Option<SuryaGrahan>, SearchError> {
    validate_surya_inputs(location, config)?;
    let moon_config = ConjunctionConfig::conjunction(MOON_STEP_DAYS);
    let mut search_jd = jd_tdb;

    for _ in 0..50 {
        let new_moon = prev_conjunction(engine, Body::Sun, Body::Moon, search_jd, &moon_config)?;
        let Some(nm) = new_moon else {
            return Ok(None);
        };

        if let Some(grahan) = compute_surya_grahan(engine, eop, nm.jd_tdb, location, config)? {
            return Ok(Some(grahan));
        }

        search_jd = nm.jd_tdb - 1.0;
    }

    Ok(None)
}

/// Search for all solar eclipses in a time range.
pub fn search_surya_grahan(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_start: f64,
    jd_end: f64,
    location: Option<GeoLocation>,
    config: &GrahanConfig,
) -> Result<Vec<SuryaGrahan>, SearchError> {
    validate_surya_inputs(location, config)?;
    if jd_end <= jd_start {
        return Err(SearchError::InvalidConfig("jd_end must be after jd_start"));
    }

    let moon_config = ConjunctionConfig::conjunction(MOON_STEP_DAYS);
    let new_moons = search_conjunctions(
        engine,
        Body::Sun,
        Body::Moon,
        jd_start,
        jd_end,
        &moon_config,
    )?;

    let mut results = Vec::new();
    for nm in &new_moons {
        if let Some(grahan) = compute_surya_grahan(engine, eop, nm.jd_tdb, location, config)? {
            results.push(grahan);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_radii_reasonable() {
        // Sun at ~1 AU, Moon at ~384400 km
        let (pen, umb) = shadow_radii_deg(149_597_870.7, 384_400.0);
        // Penumbral radius ~1.2-1.3 deg (pi_moon ~0.95 deg dominates)
        assert!(pen > 1.1 && pen < 1.4, "penumbral = {pen}");
        // Umbral radius ~0.65-0.75 deg (pi_moon - s_sun, Danjon enlarged)
        assert!(umb > 0.6 && umb < 0.8, "umbral = {umb}");
    }

    #[test]
    fn moon_angular_radius_typical() {
        let r = moon_angular_radius_deg(384_400.0);
        // ~0.26 deg
        assert!(r > 0.24 && r < 0.28, "moon angular radius = {r}");
    }

    #[test]
    fn classify_chandra_total() {
        // Moon center very close to shadow axis, small offset
        // near_edge = 0.1 - 0.26 = -0.16, far_edge = 0.1 + 0.26 = 0.36 < 0.70
        let result = classify_chandra(0.1, 0.26, 0.70, 1.25);
        assert_eq!(result, Some(ChandraGrahanType::Total));
    }

    #[test]
    fn classify_chandra_partial() {
        // Moon center near umbra boundary
        // near_edge = 0.55 - 0.26 = 0.29 < 0.70, but far_edge = 0.55 + 0.26 = 0.81 > 0.70
        let result = classify_chandra(0.55, 0.26, 0.70, 1.25);
        assert_eq!(result, Some(ChandraGrahanType::Partial));
    }

    #[test]
    fn classify_chandra_penumbral() {
        // Moon center outside umbra but inside penumbra
        // near_edge = 1.05 - 0.26 = 0.79 >= 0.70 (outside umbra)
        // far_edge = 1.05 + 0.26 = 1.31 > 1.25 but near_edge < 1.25 (inside penumbra)
        let result = classify_chandra(1.05, 0.26, 0.70, 1.25);
        assert_eq!(result, Some(ChandraGrahanType::Penumbral));
    }

    #[test]
    fn classify_chandra_none() {
        // Moon outside penumbra entirely (near edge > penumbral radius)
        let result = classify_chandra(1.6, 0.26, 0.70, 1.25);
        assert_eq!(result, None);
    }

    #[test]
    fn classify_surya_total() {
        let result = local_type(local_geometry(0.266, 0.270, 0.002));
        assert_eq!(result, Some(SuryaGrahanType::Total));
    }

    #[test]
    fn classify_surya_annular() {
        let result = local_type(local_geometry(0.266, 0.250, 0.002));
        assert_eq!(result, Some(SuryaGrahanType::Annular));
    }

    #[test]
    fn classify_surya_partial() {
        let result = local_type(local_geometry(0.266, 0.260, 0.30));
        assert_eq!(result, Some(SuryaGrahanType::Partial));
    }

    #[test]
    fn classify_surya_none() {
        let result = local_type(local_geometry(0.266, 0.260, 0.6));
        assert_eq!(result, None);
    }

    #[test]
    fn grahan_config_defaults() {
        let c = GrahanConfig::default();
        assert!(c.include_penumbral);
        assert!(c.include_peak_details);
        assert!(!c.include_path);
        assert_eq!(c.path_step_minutes, 1);
        assert_eq!(c.boundary_step_deg, 2);
    }

    fn local_geometry(sun_deg: f64, moon_deg: f64, separation_deg: f64) -> LocalDiskGeometry {
        LocalDiskGeometry {
            separation_rad: separation_deg.to_radians(),
            sun_radius_rad: sun_deg.to_radians(),
            moon_radius_rad: moon_deg.to_radians(),
            sun_altitude_deg: 45.0,
            sun_azimuth_deg: 180.0,
        }
    }

    #[test]
    fn danjon_enlargement_is_1_02() {
        assert!((DANJON_ENLARGEMENT - 1.02).abs() < 1e-10);
    }
}
