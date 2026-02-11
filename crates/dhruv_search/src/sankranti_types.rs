//! Types for Sankranti search results.

use dhruv_time::UtcTime;
use dhruv_vedic_base::{AyanamshaSystem, Rashi};

/// Configuration for Sankranti search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SankrantiConfig {
    /// Ayanamsha system for sidereal longitude.
    pub ayanamsha_system: AyanamshaSystem,
    /// Whether to apply nutation correction to the ayanamsha.
    pub use_nutation: bool,
    /// Coarse scan step size in days (default: 1.0).
    pub step_size_days: f64,
    /// Maximum bisection iterations (default: 50).
    pub max_iterations: u32,
    /// Convergence threshold in days (default: 1e-8).
    pub convergence_days: f64,
}

impl SankrantiConfig {
    /// Create with specified ayanamsha system and nutation flag, default search parameters.
    pub fn new(ayanamsha_system: AyanamshaSystem, use_nutation: bool) -> Self {
        Self {
            ayanamsha_system,
            use_nutation,
            step_size_days: 1.0,
            max_iterations: 50,
            convergence_days: 1e-8,
        }
    }

    /// Default configuration with Lahiri ayanamsha.
    pub fn default_lahiri() -> Self {
        Self {
            ayanamsha_system: AyanamshaSystem::Lahiri,
            use_nutation: false,
            step_size_days: 1.0,
            max_iterations: 50,
            convergence_days: 1e-8,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.step_size_days <= 0.0 {
            return Err("step_size_days must be positive");
        }
        if self.max_iterations == 0 {
            return Err("max_iterations must be > 0");
        }
        if self.convergence_days <= 0.0 {
            return Err("convergence_days must be positive");
        }
        Ok(())
    }
}

/// A Sankranti event: the Sun entering a new rashi.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SankrantiEvent {
    /// UTC time of the event.
    pub utc: UtcTime,
    /// The rashi the Sun is entering.
    pub rashi: Rashi,
    /// 0-based rashi index (0=Mesha .. 11=Meena).
    pub rashi_index: u8,
    /// Sun's sidereal longitude at the boundary (degrees, ~N*30).
    pub sun_sidereal_longitude_deg: f64,
    /// Sun's tropical longitude at the event (degrees).
    pub sun_tropical_longitude_deg: f64,
}
