//! Galactic reference points (Galactic Center, Anti-Center).
//!
//! Fixed ICRS directions with NO proper motion. Positions are derived from
//! the IAU 2000 galactic coordinate system definition.
//!
//! Sources:
//! - IAU 2000 galactic coordinate system (Liu, Zhu & Zhang 2011, A&A 526, A16):
//!   NGP: α = 192.85948°, δ = 27.12825°
//!   θ₀ = 122.93192° (position angle of GNP from equatorial north, measured east)
//! - Galactic Center: l=0°, b=0° → ICRS via inverse rotation matrix.

/// ICRS direction of the Galactic Center (unit vector).
///
/// Derived from IAU 2000 definition: l=0°, b=0° transformed to ICRS.
/// α_GC ≈ 266.405°, δ_GC ≈ −28.936°
///
/// The exact ICRS coordinates are computed from the IAU rotation matrix
/// (see `galactic_to_icrs` below).
pub fn galactic_center_icrs() -> [f64; 3] {
    galactic_to_icrs(0.0, 0.0)
}

/// ICRS direction of the Galactic Anti-Center (unit vector).
///
/// l=180°, b=0° → ICRS.
pub fn galactic_anticenter_icrs() -> [f64; 3] {
    galactic_to_icrs(180.0, 0.0)
}

/// Convert galactic coordinates (l, b) in degrees to an ICRS unit direction vector.
///
/// Uses the IAU 2000 galactic-to-equatorial rotation matrix.
///
/// The rotation from galactic (l, b) to ICRS (x, y, z) is:
///   [x_icrs]   [R]^T   [cos(b)cos(l)]
///   [y_icrs] =       × [cos(b)sin(l)]
///   [z_icrs]          [sin(b)       ]
///
/// where R is the equatorial-to-galactic rotation matrix built from the
/// IAU pole coordinates.
fn galactic_to_icrs(l_deg: f64, b_deg: f64) -> [f64; 3] {
    let l = l_deg.to_radians();
    let b = b_deg.to_radians();

    let (sin_l, cos_l) = l.sin_cos();
    let (sin_b, cos_b) = b.sin_cos();

    // Galactic Cartesian
    let g = [cos_b * cos_l, cos_b * sin_l, sin_b];

    // R^T × g  (R is the equatorial→galactic matrix, R^T is galactic→equatorial)
    // The rotation matrix R (ICRS→Galactic) rows are:
    //   Row 1: direction to GC (l=0, b=0) in ICRS
    //   Row 2: direction to l=90°, b=0° in ICRS
    //   Row 3: direction to NGP (b=90°) in ICRS
    //
    // So R^T columns are these same vectors → R^T × g is the ICRS vector.

    [
        R_T[0][0] * g[0] + R_T[0][1] * g[1] + R_T[0][2] * g[2],
        R_T[1][0] * g[0] + R_T[1][1] * g[1] + R_T[1][2] * g[2],
        R_T[2][0] * g[0] + R_T[2][1] * g[1] + R_T[2][2] * g[2],
    ]
}

// IAU 2000 galactic pole and origin parameters
// NGP: α_p = 192.85948°, δ_p = 27.12825°
// Position angle of GNP: θ₀ = 122.93192°
//
// The equatorial→galactic rotation matrix is:
//   R = R3(-(90°+θ₀)) · R1(90°-δ_p) · R3(α_p)
//       (but we need its transpose for galactic→equatorial)
//
// Pre-computed from the standard formula:

/// Transpose of the equatorial→galactic rotation matrix (galactic→equatorial).
///
/// Source: Murray (1989), as used in the Hipparcos Catalogue (ESA SP-1200).
/// The equatorial→galactic matrix R has rows that are the galactic basis
/// vectors expressed in ICRS. R^T[i][j] = R[j][i].
///
/// R rows (ICRS→Galactic):
///   Row 0 (GC direction):  [-0.0548755604162154, -0.8734370902348850, -0.4838350155487132]
///   Row 1 (l=90° dir):     [+0.4941094278755837, -0.4448296299600118, +0.7469822445802868]
///   Row 2 (NGP direction): [-0.8676661489811610, -0.1980763734312016, +0.4559837761750669]
const R_T: [[f64; 3]; 3] = [
    // Column 0 of R = Row 0/1/2 column 0
    [
        -0.054_875_560_416_215_4,
        0.494_109_427_875_583_7,
        -0.867_666_148_981_161,
    ],
    // Column 1 of R
    [
        -0.873_437_090_234_885,
        -0.444_829_629_960_011_8,
        -0.198_076_373_431_201_6,
    ],
    // Column 2 of R
    [
        -0.483_835_015_548_713_2,
        0.746_982_244_580_286_8,
        0.455_983_776_175_066_9,
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn galactic_center_direction() {
        let gc = galactic_center_icrs();
        // GC is at approximately α ≈ 266.4°, δ ≈ -28.9°
        let r = (gc[0] * gc[0] + gc[1] * gc[1] + gc[2] * gc[2]).sqrt();
        assert!((r - 1.0).abs() < 1e-12, "should be unit vector, r={r}");

        // Convert to RA/Dec
        let ra = gc[1].atan2(gc[0]).to_degrees().rem_euclid(360.0);
        let dec = (gc[2] / r).asin().to_degrees();

        assert!(
            (ra - 266.405).abs() < 0.01,
            "GC RA: {ra}° (expected ~266.405°)"
        );
        assert!(
            (dec - (-28.936)).abs() < 0.01,
            "GC Dec: {dec}° (expected ~-28.936°)"
        );
    }

    #[test]
    fn galactic_anticenter_direction() {
        let gac = galactic_anticenter_icrs();
        let r = (gac[0] * gac[0] + gac[1] * gac[1] + gac[2] * gac[2]).sqrt();
        assert!((r - 1.0).abs() < 1e-12);

        // Anti-center: α ≈ 86.4°, δ ≈ +28.9°
        let ra = gac[1].atan2(gac[0]).to_degrees().rem_euclid(360.0);
        let dec = (gac[2] / r).asin().to_degrees();

        assert!((ra - 86.4).abs() < 0.1, "GAC RA: {ra}° (expected ~86.4°)");
        assert!(
            (dec - 28.9).abs() < 0.1,
            "GAC Dec: {dec}° (expected ~28.9°)"
        );
    }

    #[test]
    fn gc_and_anticenter_are_antipodal() {
        let gc = galactic_center_icrs();
        let gac = galactic_anticenter_icrs();
        // They should point in opposite directions
        let dot = gc[0] * gac[0] + gc[1] * gac[1] + gc[2] * gac[2];
        assert!(
            (dot - (-1.0)).abs() < 1e-12,
            "GC · GAC = {dot} (expected -1)"
        );
    }

    #[test]
    fn ngp_direction() {
        // l=0, b=90 should give NGP
        let ngp = galactic_to_icrs(0.0, 90.0);
        let ra = ngp[1].atan2(ngp[0]).to_degrees().rem_euclid(360.0);
        let dec = ngp[2].asin().to_degrees();
        assert!(
            (ra - 192.859).abs() < 0.01,
            "NGP RA: {ra}° (expected ~192.86°)"
        );
        assert!(
            (dec - 27.128).abs() < 0.01,
            "NGP Dec: {dec}° (expected ~27.13°)"
        );
    }

    #[test]
    fn rotation_matrix_orthogonal() {
        // R^T should be orthogonal: R^T · R = I
        for i in 0..3 {
            for j in 0..3 {
                let mut dot = 0.0;
                for k in 0..3 {
                    dot += R_T[i][k] * R_T[j][k];
                }
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (dot - expected).abs() < 1e-9,
                    "R_T orthogonality [{i}][{j}]: {dot} (expected {expected})"
                );
            }
        }
    }
}
