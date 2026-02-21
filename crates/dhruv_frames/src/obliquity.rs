//! Obliquity of the ecliptic constants and polynomials.
//!
//! The J2000.0 value is from the IAU 1976 precession model, which is the
//! standard used by DE planetary ephemeris kernels.

use std::f64::consts::PI;

/// Mean obliquity of the ecliptic at J2000.0 (IAU 1976), in radians.
///
/// 23 deg 26' 21.448" = 84381.448" = 23.4392911111... deg
pub const OBLIQUITY_J2000_RAD: f64 = 23.439_291_111_1 * PI / 180.0;

/// Mean obliquity of the ecliptic at J2000.0, in degrees.
pub const OBLIQUITY_J2000_DEG: f64 = 23.439_291_111_1;

/// Cosine of J2000 obliquity (precomputed for rotation matrix).
pub const COS_OBL: f64 = 0.917_482_062_069_258_9;

/// Sine of J2000 obliquity (precomputed for rotation matrix).
pub const SIN_OBL: f64 = 0.397_777_155_931_735_8;

/// Mean obliquity of the ecliptic at epoch `t`, in arcseconds.
///
/// IAU 2006 polynomial (Hilton et al. 2006).
/// `t` = Julian centuries of TDB since J2000.0.
///
/// At T=0 returns 84381.406", which is the IAU 2006 J2000 value.
/// (The IAU 1976 constant [`OBLIQUITY_J2000_RAD`] uses 84381.448" — a 0.042"
/// difference between the two models.)
pub fn mean_obliquity_of_date_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    84381.406 - 46.836_769 * t - 0.000_183_1 * t2 + 0.002_003_40 * t3
        - 0.000_000_576 * t4
        - 0.000_000_043_4 * t5
}

/// Mean obliquity of the ecliptic at epoch `t`, in radians.
///
/// Same as [`mean_obliquity_of_date_arcsec`] converted to radians.
pub fn mean_obliquity_of_date_rad(t: f64) -> f64 {
    (mean_obliquity_of_date_arcsec(t) / 3600.0).to_radians()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precomputed_trig_matches() {
        let cos_check = OBLIQUITY_J2000_RAD.cos();
        let sin_check = OBLIQUITY_J2000_RAD.sin();
        assert!((COS_OBL - cos_check).abs() < 1e-15);
        assert!((SIN_OBL - sin_check).abs() < 1e-15);
    }

    #[test]
    fn obliquity_of_date_at_j2000() {
        // IAU 2006 value at T=0 is 84381.406"
        let eps = mean_obliquity_of_date_arcsec(0.0);
        assert!((eps - 84381.406).abs() < 1e-6, "eps(0) = {eps}");
    }

    #[test]
    fn obliquity_rad_consistent_with_arcsec() {
        let t = 0.5;
        let arcsec = mean_obliquity_of_date_arcsec(t);
        let rad = mean_obliquity_of_date_rad(t);
        assert!((rad - (arcsec / 3600.0).to_radians()).abs() < 1e-15);
    }

    #[test]
    fn obliquity_decreases_over_time() {
        // Obliquity is decreasing in the current era (~47"/century)
        let eps_now = mean_obliquity_of_date_arcsec(0.25); // ~2025
        let eps_j2000 = mean_obliquity_of_date_arcsec(0.0);
        assert!(
            eps_now < eps_j2000,
            "eps should decrease from J2000 to 2025"
        );
    }
}
