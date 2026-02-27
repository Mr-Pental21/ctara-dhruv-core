//! Space motion vector propagation for fixed stars.
//!
//! Algorithm: Butkevich & Lindegren (2014), A&A 570, A62.
//! Also: Hipparcos Volume 1, Section 1.5.
//!
//! Converts 6 astrometric parameters (α, δ, ϖ, μα*, μδ, vr) to
//! 3D Cartesian position + velocity, then linearly propagates.

use std::f64::consts::PI;

/// Milliarcseconds to radians.
const MAS_TO_RAD: f64 = PI / (180.0 * 3600.0 * 1000.0);

/// km/s to AU/Julian year.
const KM_S_TO_AU_YR: f64 = 1.0 / 4.740_470_446;

/// AU in km (IAU 2012).
pub const AU_KM: f64 = 149_597_870.7;

/// Equatorial position in ICRS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquatorialPosition {
    /// Right ascension in degrees [0, 360).
    pub ra_deg: f64,
    /// Declination in degrees [-90, 90].
    pub dec_deg: f64,
    /// Distance in AU.
    pub distance_au: f64,
}

/// Convert 6 astrometric parameters to Cartesian position (AU) and velocity (AU/yr).
///
/// # Arguments
/// * `ra_deg` — right ascension at reference epoch (degrees)
/// * `dec_deg` — declination at reference epoch (degrees)
/// * `parallax_mas` — parallax (milliarcseconds); zero/negative → 1e6 AU
/// * `pm_ra_mas_yr` — proper motion in RA (μα* = μα cos δ) (mas/yr)
/// * `pm_dec_mas_yr` — proper motion in Dec (mas/yr)
/// * `rv_km_s` — radial velocity (km/s); 0.0 if unknown
///
/// Returns `(position_au, velocity_au_yr)`.
fn astrometric_to_cartesian(
    ra_deg: f64,
    dec_deg: f64,
    parallax_mas: f64,
    pm_ra_mas_yr: f64,
    pm_dec_mas_yr: f64,
    rv_km_s: f64,
) -> ([f64; 3], [f64; 3]) {
    let alpha = ra_deg.to_radians();
    let delta = dec_deg.to_radians();
    let (sin_a, cos_a) = alpha.sin_cos();
    let (sin_d, cos_d) = delta.sin_cos();

    // Unit vector p (direction to star)
    let p = [cos_a * cos_d, sin_a * cos_d, sin_d];

    // Tangent vectors for proper motion decomposition:
    // q = ∂p/∂α (east), r = ∂p/∂δ (north)
    let q = [-sin_a, cos_a, 0.0];
    let r = [-cos_a * sin_d, -sin_a * sin_d, cos_d];

    // Distance from parallax: d(AU) = 1/ϖ(rad)
    let parallax_rad = parallax_mas * MAS_TO_RAD;
    let distance_au = if parallax_rad > 0.0 {
        1.0 / parallax_rad
    } else {
        1e10 // effectively infinite
    };

    // Position (AU)
    let pos = [p[0] * distance_au, p[1] * distance_au, p[2] * distance_au];

    // Convert proper motion from mas/yr to rad/yr
    let mu_alpha_rad = pm_ra_mas_yr * MAS_TO_RAD; // already μα*
    let mu_delta_rad = pm_dec_mas_yr * MAS_TO_RAD;

    // Radial velocity in AU/yr
    let vr_au_yr = rv_km_s * KM_S_TO_AU_YR;

    // Velocity (AU/yr) = r·vr + d·(μα*·q + μδ·r)
    let vel = [
        p[0] * vr_au_yr + distance_au * (mu_alpha_rad * q[0] + mu_delta_rad * r[0]),
        p[1] * vr_au_yr + distance_au * (mu_alpha_rad * q[1] + mu_delta_rad * r[1]),
        p[2] * vr_au_yr + distance_au * (mu_alpha_rad * q[2] + mu_delta_rad * r[2]),
    ];

    (pos, vel)
}

/// Convert Cartesian position (AU) back to equatorial coordinates.
fn cartesian_to_equatorial(pos: &[f64; 3]) -> EquatorialPosition {
    let x = pos[0];
    let y = pos[1];
    let z = pos[2];
    let r = (x * x + y * y + z * z).sqrt();

    if r == 0.0 {
        return EquatorialPosition {
            ra_deg: 0.0,
            dec_deg: 0.0,
            distance_au: 0.0,
        };
    }

    let dec = (z / r).asin();
    let ra = y.atan2(x);

    EquatorialPosition {
        ra_deg: if ra < 0.0 { ra + 2.0 * PI } else { ra }.to_degrees(),
        dec_deg: dec.to_degrees(),
        distance_au: r,
    }
}

/// Propagate a star's ICRS position from its reference epoch.
///
/// # Arguments
/// * `ra_deg` — right ascension at reference epoch (degrees)
/// * `dec_deg` — declination at reference epoch (degrees)
/// * `parallax_mas` — parallax (milliarcseconds)
/// * `pm_ra_mas_yr` — proper motion in RA (μα*) (mas/yr)
/// * `pm_dec_mas_yr` — proper motion in Dec (mas/yr)
/// * `rv_km_s` — radial velocity (km/s)
/// * `dt_years` — time difference from reference epoch (Julian years)
///
/// Returns the propagated equatorial position.
pub fn propagate_position(
    ra_deg: f64,
    dec_deg: f64,
    parallax_mas: f64,
    pm_ra_mas_yr: f64,
    pm_dec_mas_yr: f64,
    rv_km_s: f64,
    dt_years: f64,
) -> EquatorialPosition {
    let (pos0, vel) = astrometric_to_cartesian(
        ra_deg,
        dec_deg,
        parallax_mas,
        pm_ra_mas_yr,
        pm_dec_mas_yr,
        rv_km_s,
    );

    // Linear propagation: pos(t) = pos₀ + Δt × vel
    let pos = [
        pos0[0] + dt_years * vel[0],
        pos0[1] + dt_years * vel[1],
        pos0[2] + dt_years * vel[2],
    ];

    cartesian_to_equatorial(&pos)
}

/// Propagate and return the ICRS Cartesian position in AU.
///
/// Used internally for the ecliptic/sidereal pipeline.
pub fn propagate_cartesian_au(
    ra_deg: f64,
    dec_deg: f64,
    parallax_mas: f64,
    pm_ra_mas_yr: f64,
    pm_dec_mas_yr: f64,
    rv_km_s: f64,
    dt_years: f64,
) -> [f64; 3] {
    let (pos0, vel) = astrometric_to_cartesian(
        ra_deg,
        dec_deg,
        parallax_mas,
        pm_ra_mas_yr,
        pm_dec_mas_yr,
        rv_km_s,
    );

    [
        pos0[0] + dt_years * vel[0],
        pos0[1] + dt_years * vel[1],
        pos0[2] + dt_years * vel[2],
    ]
}

/// Convert an ICRS Cartesian position (AU) to an equatorial position.
pub fn cartesian_au_to_equatorial(pos: &[f64; 3]) -> EquatorialPosition {
    cartesian_to_equatorial(pos)
}

/// Apply parallax correction to a direction vector.
///
/// Shifts the direction from barycentric to geocentric by subtracting
/// Earth's position vector.
///
/// # Arguments
/// * `star_pos_au` — barycentric star position in AU
/// * `earth_pos_au` — barycentric Earth position in AU
///
/// Returns the geocentric direction as a unit vector.
pub fn apply_parallax(star_pos_au: &[f64; 3], earth_pos_au: &[f64; 3]) -> [f64; 3] {
    let geo = [
        star_pos_au[0] - earth_pos_au[0],
        star_pos_au[1] - earth_pos_au[1],
        star_pos_au[2] - earth_pos_au[2],
    ];
    let r = (geo[0] * geo[0] + geo[1] * geo[1] + geo[2] * geo[2]).sqrt();
    if r == 0.0 {
        return [0.0, 0.0, 0.0];
    }
    [geo[0] / r, geo[1] / r, geo[2] / r]
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEG_EPS: f64 = 1e-10;

    #[test]
    fn zero_dt_identity() {
        // Propagating with Δt=0 must return the input coordinates exactly.
        let ra = 201.298;
        let dec = -11.161;
        let result = propagate_position(ra, dec, 23.0, -42.0, -31.0, 1.0, 0.0);
        assert!(
            (result.ra_deg - ra).abs() < DEG_EPS,
            "ra: {} vs {}",
            result.ra_deg,
            ra
        );
        assert!(
            (result.dec_deg - dec).abs() < DEG_EPS,
            "dec: {} vs {}",
            result.dec_deg,
            dec
        );
    }

    #[test]
    fn zero_proper_motion_no_change() {
        let ra = 100.0;
        let dec = 30.0;
        // No proper motion, no radial velocity → position unchanged at any Δt
        let result = propagate_position(ra, dec, 10.0, 0.0, 0.0, 0.0, 100.0);
        assert!(
            (result.ra_deg - ra).abs() < DEG_EPS,
            "ra shifted: {} vs {}",
            result.ra_deg,
            ra
        );
        assert!(
            (result.dec_deg - dec).abs() < DEG_EPS,
            "dec shifted: {} vs {}",
            result.dec_deg,
            dec
        );
    }

    #[test]
    fn positive_pm_ra_moves_east() {
        let ra = 180.0;
        let dec = 0.0;
        // Large positive μα* → RA should increase
        let result = propagate_position(ra, dec, 100.0, 10000.0, 0.0, 0.0, 1.0);
        assert!(
            result.ra_deg > ra,
            "expected RA to increase, got {}",
            result.ra_deg
        );
    }

    #[test]
    fn positive_pm_dec_moves_north() {
        let ra = 0.0;
        let dec = 0.0;
        let result = propagate_position(ra, dec, 100.0, 0.0, 10000.0, 0.0, 1.0);
        assert!(
            result.dec_deg > dec,
            "expected Dec to increase, got {}",
            result.dec_deg
        );
    }

    #[test]
    fn negative_parallax_treated_as_infinite() {
        // Negative parallax should not cause panic or NaN
        let result = propagate_position(100.0, 30.0, -5.0, 10.0, -5.0, 0.0, 10.0);
        assert!(result.ra_deg.is_finite());
        assert!(result.dec_deg.is_finite());
        assert!(result.distance_au > 1e5); // effectively infinite
    }

    #[test]
    fn arcturus_large_pm() {
        // Arcturus: μα* ≈ -1093.45 mas/yr, μδ ≈ -1999.40 mas/yr
        // Total PM ≈ 2.28"/yr → over 100 years ≈ 228" ≈ 0.063°
        let ra = 213.9153;
        let dec = 19.1824;
        let plx = 88.83; // mas
        let result_fwd = propagate_position(ra, dec, plx, -1093.45, -1999.40, -5.19, 100.0);
        let result_bwd = propagate_position(ra, dec, plx, -1093.45, -1999.40, -5.19, -100.0);

        let dra_fwd = result_fwd.ra_deg - ra;
        let ddec_fwd = result_fwd.dec_deg - dec;
        let shift_fwd =
            (dra_fwd * dra_fwd * dec.to_radians().cos().powi(2) + ddec_fwd * ddec_fwd).sqrt();

        let dra_bwd = result_bwd.ra_deg - ra;
        let ddec_bwd = result_bwd.dec_deg - dec;
        let shift_bwd =
            (dra_bwd * dra_bwd * dec.to_radians().cos().powi(2) + ddec_bwd * ddec_bwd).sqrt();

        // Each should be ~0.06° (200" total PM per 100 yr)
        assert!((shift_fwd - 0.063).abs() < 0.02, "fwd shift: {shift_fwd}°");
        assert!((shift_bwd - 0.063).abs() < 0.02, "bwd shift: {shift_bwd}°");
    }

    #[test]
    fn parallax_correction_shifts_direction() {
        // Star at 10 AU along +x, Earth at 1 AU along +y
        let star = [10.0, 0.0, 0.0];
        let earth = [0.0, 1.0, 0.0];
        let dir = apply_parallax(&star, &earth);
        // Geocentric direction should point mostly along +x, slightly toward -y
        assert!(dir[0] > 0.9);
        assert!(dir[1] < 0.0);
    }
}
