//! Cartesian ↔ Spherical coordinate conversion.

use std::f64::consts::PI;

/// Spherical coordinates: longitude, latitude, distance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphericalCoords {
    /// Longitude in radians, range [0, 2π).
    /// Measured in the x-y plane from +x toward +y.
    pub lon_rad: f64,
    /// Latitude in radians, range [-π/2, π/2].
    /// Elevation above the x-y plane.
    pub lat_rad: f64,
    /// Distance from origin in km.
    pub distance_km: f64,
}

impl SphericalCoords {
    /// Longitude in degrees, range [0, 360).
    pub fn lon_deg(&self) -> f64 {
        self.lon_rad.to_degrees()
    }

    /// Latitude in degrees, range [-90, 90].
    pub fn lat_deg(&self) -> f64 {
        self.lat_rad.to_degrees()
    }
}

/// Convert Cartesian `[x, y, z]` (km) to spherical coordinates.
///
/// Longitude is measured in the x-y plane from +x toward +y.
/// Latitude is elevation above the x-y plane.
pub fn cartesian_to_spherical(xyz: &[f64; 3]) -> SphericalCoords {
    let x = xyz[0];
    let y = xyz[1];
    let z = xyz[2];

    let r = (x * x + y * y + z * z).sqrt();

    if r == 0.0 {
        return SphericalCoords {
            lon_rad: 0.0,
            lat_rad: 0.0,
            distance_km: 0.0,
        };
    }

    let lon = y.atan2(x);
    let lat = (z / r).asin();

    SphericalCoords {
        lon_rad: if lon < 0.0 { lon + 2.0 * PI } else { lon },
        lat_rad: lat,
        distance_km: r,
    }
}

/// Convert spherical coordinates back to Cartesian `[x, y, z]` (km).
pub fn spherical_to_cartesian(s: &SphericalCoords) -> [f64; 3] {
    let cos_lat = s.lat_rad.cos();
    [
        s.distance_km * cos_lat * s.lon_rad.cos(),
        s.distance_km * cos_lat * s.lon_rad.sin(),
        s.distance_km * s.lat_rad.sin(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    #[test]
    fn along_x_axis() {
        let s = cartesian_to_spherical(&[1.0e8, 0.0, 0.0]);
        assert!((s.lon_rad - 0.0).abs() < EPS);
        assert!((s.lat_rad - 0.0).abs() < EPS);
        assert!((s.distance_km - 1.0e8).abs() < EPS);
    }

    #[test]
    fn along_y_axis() {
        let s = cartesian_to_spherical(&[0.0, 1.0e8, 0.0]);
        assert!((s.lon_rad - PI / 2.0).abs() < EPS);
        assert!((s.lat_rad - 0.0).abs() < EPS);
    }

    #[test]
    fn along_negative_x() {
        let s = cartesian_to_spherical(&[-1.0e8, 0.0, 0.0]);
        assert!((s.lon_rad - PI).abs() < EPS);
    }

    #[test]
    fn along_z_axis() {
        let s = cartesian_to_spherical(&[0.0, 0.0, 1.0e8]);
        assert!((s.lat_rad - PI / 2.0).abs() < EPS);
        assert!((s.distance_km - 1.0e8).abs() < EPS);
    }

    #[test]
    fn roundtrip() {
        let xyz = [1.234e8, -5.678e7, 3.456e7];
        let s = cartesian_to_spherical(&xyz);
        let back = spherical_to_cartesian(&s);
        for i in 0..3 {
            assert!(
                (xyz[i] - back[i]).abs() < EPS * xyz[i].abs().max(1.0),
                "axis {i}: {:.10e} != {:.10e}",
                xyz[i],
                back[i]
            );
        }
    }

    #[test]
    fn zero_vector() {
        let s = cartesian_to_spherical(&[0.0, 0.0, 0.0]);
        assert_eq!(s.distance_km, 0.0);
    }

    #[test]
    fn longitude_always_positive() {
        // Negative x, negative y → third quadrant → lon in [π, 3π/2)
        let s = cartesian_to_spherical(&[-1.0, -1.0, 0.0]);
        assert!(s.lon_rad >= 0.0 && s.lon_rad < 2.0 * PI);
    }
}
