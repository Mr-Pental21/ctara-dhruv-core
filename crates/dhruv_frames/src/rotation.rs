//! Frame rotation between ICRF/J2000 and Ecliptic J2000.
//!
//! The rotation is a single-axis rotation about the X axis by the
//! J2000 obliquity of the ecliptic.

use crate::obliquity::{COS_OBL, SIN_OBL};

/// Rotate a 3-vector from ICRF/J2000 equatorial to Ecliptic J2000.
///
/// The rotation matrix (X-axis rotation by obliquity ε):
/// ```text
/// | 1    0        0       |
/// | 0    cos(ε)   sin(ε)  |
/// | 0   -sin(ε)   cos(ε)  |
/// ```
#[inline]
pub fn icrf_to_ecliptic(v: &[f64; 3]) -> [f64; 3] {
    [
        v[0],
        COS_OBL * v[1] + SIN_OBL * v[2],
        -SIN_OBL * v[1] + COS_OBL * v[2],
    ]
}

/// Rotate a 3-vector from Ecliptic J2000 to ICRF/J2000 equatorial.
///
/// This is the transpose of the ICRF→Ecliptic matrix.
#[inline]
pub fn ecliptic_to_icrf(v: &[f64; 3]) -> [f64; 3] {
    [
        v[0],
        COS_OBL * v[1] - SIN_OBL * v[2],
        SIN_OBL * v[1] + COS_OBL * v[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-14;

    #[test]
    fn roundtrip_icrf_ecliptic() {
        let v = [1.0e8, -5.0e7, 3.0e7];
        let ecl = icrf_to_ecliptic(&v);
        let back = ecliptic_to_icrf(&ecl);
        for i in 0..3 {
            assert!(
                (v[i] - back[i]).abs() < EPS * v[i].abs().max(1.0),
                "axis {i}: {:.15e} != {:.15e}",
                v[i],
                back[i]
            );
        }
    }

    #[test]
    fn x_axis_unchanged() {
        let v = [1.0, 0.0, 0.0];
        let ecl = icrf_to_ecliptic(&v);
        assert!((ecl[0] - 1.0).abs() < EPS);
        assert!(ecl[1].abs() < EPS);
        assert!(ecl[2].abs() < EPS);
    }

    #[test]
    fn magnitude_preserved() {
        let v: [f64; 3] = [1.234e8, -5.678e7, 9.012e6];
        let r_orig = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let ecl = icrf_to_ecliptic(&v);
        let r_ecl = (ecl[0] * ecl[0] + ecl[1] * ecl[1] + ecl[2] * ecl[2]).sqrt();
        assert!((r_orig - r_ecl).abs() < EPS * r_orig);
    }
}
