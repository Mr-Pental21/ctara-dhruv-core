//! IAU 2006 general precession in ecliptic longitude and full 3D ecliptic
//! precession matrix (J2000 ↔ ecliptic-of-date).
//!
//! The general precession p_A measures the accumulated westward motion of
//! the vernal equinox along the ecliptic since J2000.0. This is the
//! foundational quantity for computing ayanamsha at any epoch.
//!
//! The full 3D ecliptic precession rotation uses the IAU 2006 parameters
//! π_A (inclination) and Π_A (node longitude) from Capitaine et al. 2003.
//!
//! Sources:
//! - Capitaine, Wallace & Chapront 2003, A&A 412, 567-586, Table 1.
//! - IERS Conventions 2010, Chapter 5, Table 5.1.
//! Public domain (IAU standard).

/// IAU 2006 general precession in ecliptic longitude, in arcseconds.
///
/// # Arguments
/// * `t` — Julian centuries of TDB since J2000.0: `(JD_TDB - 2451545.0) / 36525.0`
///
/// # Returns
/// Accumulated precession in arcseconds. Positive means the equinox has
/// moved westward (tropical longitudes of stars have increased).
///
/// The dominant linear term is ~5028.80″/century ≈ 1.3969°/century.
pub fn general_precession_longitude_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    5028.796195 * t + 1.1054348 * t2 + 0.00007964 * t3 - 0.000023857 * t4 - 0.0000000383 * t5
}

/// IAU 2006 general precession in ecliptic longitude, in degrees.
///
/// Same as [`general_precession_longitude_arcsec`] but converted to degrees.
pub fn general_precession_longitude_deg(t: f64) -> f64 {
    general_precession_longitude_arcsec(t) / 3600.0
}

/// Inclination of the ecliptic of date to the J2000 ecliptic, in arcseconds.
///
/// π_A from IERS Conventions 2010, Table 5.1 (Capitaine et al. 2003).
/// `t` = Julian centuries of TDB since J2000.0.
pub fn ecliptic_inclination_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    46.998_973 * t - 0.033_492_6 * t2 - 0.000_125_59 * t3
        + 0.000_000_113 * t4 - 0.000_000_002_2 * t5
}

/// Longitude of the ascending node of the ecliptic of date on the J2000
/// ecliptic, in arcseconds.
///
/// Π_A from IERS Conventions 2010, Table 5.1 (Capitaine et al. 2003).
/// `t` = Julian centuries of TDB since J2000.0.
pub fn ecliptic_node_longitude_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    629_546.793_6 + 3289.478_9 * t + 0.606_22 * t2
        - 0.000_83 * t3 - 0.000_01 * t4 - 0.000_000_01 * t5
}

/// Time derivative of the general precession in ecliptic longitude, in deg/day.
///
/// d(p_A)/dt evaluated at epoch `t` (Julian centuries since J2000.0).
/// Informational: used for documenting the ~50"/yr correction magnitude.
/// Velocity transforms use finite-differencing rather than this scalar.
pub fn general_precession_rate_deg_per_day(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    // d(p_A)/dt in arcsec/century
    let rate = 5028.796_195
        + 2.0 * 1.105_434_8 * t
        + 3.0 * 0.000_079_64 * t2
        - 4.0 * 0.000_023_857 * t3
        - 5.0 * 0.000_000_038_3 * t4;
    rate / 3600.0 / 36525.0
}

/// Precess a 3-vector from J2000 ecliptic to ecliptic-of-date.
///
/// Applies the full IAU 2006 ecliptic precession rotation matrix:
/// `P = R3(-(Π_A + p_A)) · R1(π_A) · R3(Π_A)`
///
/// `t` = Julian centuries of TDB since J2000.0.
/// Returns identity at T=0.
pub fn precess_ecliptic_j2000_to_date(v: &[f64; 3], t: f64) -> [f64; 3] {
    if t.abs() < 1e-15 {
        return *v;
    }

    let pi_a = (ecliptic_inclination_arcsec(t) / 3600.0).to_radians();
    let cap_pi_a = (ecliptic_node_longitude_arcsec(t) / 3600.0).to_radians();
    let p_a = (general_precession_longitude_arcsec(t) / 3600.0).to_radians();

    // R3(Π_A) applied to v
    let (s1, c1) = cap_pi_a.sin_cos();
    let x1 = c1 * v[0] + s1 * v[1];
    let y1 = -s1 * v[0] + c1 * v[1];
    let z1 = v[2];

    // R1(π_A) applied
    let (s2, c2) = pi_a.sin_cos();
    let x2 = x1;
    let y2 = c2 * y1 + s2 * z1;
    let z2 = -s2 * y1 + c2 * z1;

    // R3(-(Π_A + p_A)) applied
    let (s3, c3) = (-(cap_pi_a + p_a)).sin_cos();
    [c3 * x2 + s3 * y2, -s3 * x2 + c3 * y2, z2]
}

/// Precess a 3-vector from ecliptic-of-date back to J2000 ecliptic.
///
/// Applies P^{-1} = P^T (P is orthogonal):
/// `P^{-1} = R3(-Π_A) · R1(-π_A) · R3(Π_A + p_A)`
///
/// Used for round-trip tests and any consumer that needs J2000 coordinates
/// from of-date input.
pub fn precess_ecliptic_date_to_j2000(v: &[f64; 3], t: f64) -> [f64; 3] {
    if t.abs() < 1e-15 {
        return *v;
    }

    let pi_a = (ecliptic_inclination_arcsec(t) / 3600.0).to_radians();
    let cap_pi_a = (ecliptic_node_longitude_arcsec(t) / 3600.0).to_radians();
    let p_a = (general_precession_longitude_arcsec(t) / 3600.0).to_radians();

    // R3(Π_A + p_A) applied to v
    let (s1, c1) = (cap_pi_a + p_a).sin_cos();
    let x1 = c1 * v[0] + s1 * v[1];
    let y1 = -s1 * v[0] + c1 * v[1];
    let z1 = v[2];

    // R1(-π_A) applied
    let (s2, c2) = (-pi_a).sin_cos();
    let x2 = x1;
    let y2 = c2 * y1 + s2 * z1;
    let z2 = -s2 * y1 + c2 * z1;

    // R3(-Π_A) applied
    let (s3, c3) = (-cap_pi_a).sin_cos();
    [c3 * x2 + s3 * y2, -s3 * x2 + c3 * y2, z2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_at_j2000() {
        assert_eq!(general_precession_longitude_arcsec(0.0), 0.0);
    }

    #[test]
    fn one_century_approx() {
        let p = general_precession_longitude_arcsec(1.0);
        // 5028.796195 + 1.1054348 + 0.00007964 - ... ≈ 5029.90
        assert!((p - 5029.90).abs() < 1.0, "p_A(1.0) = {p}");
    }

    #[test]
    fn negative_century() {
        let p = general_precession_longitude_arcsec(-1.0);
        assert!(p < 0.0, "p_A(-1.0) should be negative, got {p}");
    }

    #[test]
    fn rate_per_year() {
        // 1 year = 0.01 century
        let p = general_precession_longitude_arcsec(0.01);
        // ~50.29" per year
        assert!((p - 50.29).abs() < 0.1, "p_A(0.01) = {p}");
    }

    #[test]
    fn deg_conversion_consistent() {
        let t = 0.5;
        let arcsec = general_precession_longitude_arcsec(t);
        let deg = general_precession_longitude_deg(t);
        assert!((deg - arcsec / 3600.0).abs() < 1e-15);
    }

    // ---------- ecliptic inclination / node ----------

    #[test]
    fn ecliptic_inclination_zero_at_j2000() {
        // π_A(0) should be 0 by definition (no precession at J2000)
        assert_eq!(ecliptic_inclination_arcsec(0.0), 0.0);
    }

    #[test]
    fn ecliptic_inclination_one_century() {
        // Leading term: ~46.999"/century
        let pi = ecliptic_inclination_arcsec(1.0);
        assert!((pi - 47.0).abs() < 1.0, "π_A(1.0) = {pi}");
    }

    #[test]
    fn ecliptic_node_longitude_at_j2000() {
        // Π_A(0) ≈ 629546.7936" ≈ 174.87°
        let node = ecliptic_node_longitude_arcsec(0.0);
        assert!((node - 629_546.793_6).abs() < 1e-6, "Π_A(0) = {node}");
    }

    // ---------- precession matrix ----------

    #[test]
    fn precess_j2000_to_date_identity_at_t0() {
        let v = [1.0_f64, 0.5, -0.3];
        let out = precess_ecliptic_j2000_to_date(&v, 0.0);
        for i in 0..3 {
            assert!((out[i] - v[i]).abs() < 1e-15, "component {i}: {}", out[i]);
        }
    }

    #[test]
    fn precess_date_to_j2000_identity_at_t0() {
        let v = [1.0_f64, 0.5, -0.3];
        let out = precess_ecliptic_date_to_j2000(&v, 0.0);
        for i in 0..3 {
            assert!((out[i] - v[i]).abs() < 1e-15, "component {i}: {}", out[i]);
        }
    }

    #[test]
    fn precess_round_trip() {
        // forward then inverse must recover original to machine precision
        let v = [0.8_f64, 0.5, 0.1];
        for &t in &[0.5_f64, 1.0, -1.0, 5.0] {
            let fwd = precess_ecliptic_j2000_to_date(&v, t);
            let back = precess_ecliptic_date_to_j2000(&fwd, t);
            for i in 0..3 {
                assert!(
                    (back[i] - v[i]).abs() < 1e-12,
                    "t={t} component {i}: got {}, expected {}",
                    back[i],
                    v[i]
                );
            }
        }
    }

    #[test]
    fn precess_unit_length_preserved() {
        // Rotation must preserve vector length
        let v = [0.6_f64, 0.8, 0.0];
        let len_in = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        for &t in &[1.0_f64, -1.0, 5.0] {
            let out = precess_ecliptic_j2000_to_date(&v, t);
            let len_out = (out[0] * out[0] + out[1] * out[1] + out[2] * out[2]).sqrt();
            assert!(
                (len_out - len_in).abs() < 1e-13,
                "t={t}: |v|={len_in}, |Pv|={len_out}"
            );
        }
    }

    #[test]
    fn precess_j2000_x_axis_rotates_by_p_a() {
        // At T=1, (1,0,0) should rotate by ~p_A in longitude
        let v = [1.0_f64, 0.0, 0.0];
        let out = precess_ecliptic_j2000_to_date(&v, 1.0);
        let lon_in = 0.0_f64;
        let lon_out = out[1].atan2(out[0]).to_degrees();
        let p_a = general_precession_longitude_deg(1.0);
        let diff = (lon_out - lon_in - p_a).abs() % 360.0;
        // Allow 1° tolerance (higher-order terms from π_A, Π_A)
        assert!(diff.min(360.0 - diff) < 1.0, "lon shift={:.4}°, p_A={:.4}°", lon_out - lon_in, p_a);
    }
}
