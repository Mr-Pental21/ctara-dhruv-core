//! Configuration types for fixed star position queries.

/// Accuracy tier for star position computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaraAccuracy {
    /// Geometric/catalog place (barycentric ICRS). Default.
    /// Includes: space motion propagation, frame rotation, precession.
    /// Optionally includes parallax if `apply_parallax` is set.
    Astrometric,
    /// Apparent place (as seen from Earth).
    /// Adds: annual aberration, gravitational light deflection, nutation.
    /// Requires `EarthState` to be provided.
    Apparent,
}

/// Configuration for star position queries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaraConfig {
    /// Accuracy tier.
    pub accuracy: TaraAccuracy,
    /// Whether to apply parallax correction (shifts position based on Earth's
    /// position relative to the star). Requires `EarthState`.
    pub apply_parallax: bool,
}

impl Default for TaraConfig {
    fn default() -> Self {
        Self {
            accuracy: TaraAccuracy::Astrometric,
            apply_parallax: false,
        }
    }
}

/// Earth's barycentric state in ICRS, provided by the caller.
///
/// The caller obtains these from the ephemeris engine (e.g., `dhruv_core`)
/// and passes them in. This keeps `dhruv_tara` independent of `dhruv_core`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EarthState {
    /// ICRS barycentric position of Earth in AU.
    pub position_au: [f64; 3],
    /// ICRS barycentric velocity of Earth in AU/day.
    pub velocity_au_day: [f64; 3],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = TaraConfig::default();
        assert_eq!(cfg.accuracy, TaraAccuracy::Astrometric);
        assert!(!cfg.apply_parallax);
    }
}
