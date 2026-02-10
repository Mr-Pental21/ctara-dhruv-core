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
}
