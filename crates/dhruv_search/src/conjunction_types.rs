//! Types for conjunction, opposition, and aspect search.

use dhruv_time::UtcTime;
use dhruv_vedic_base::NodeMode;

use crate::transit_body::TransitBody;

/// Configuration for a conjunction/aspect search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConjunctionConfig {
    /// Target ecliptic longitude separation in degrees, range [0, 360).
    /// 0 = conjunction, 180 = opposition, 90 = square, 120 = trine, etc.
    pub target_separation_deg: f64,
    /// Coarse scan step size in days.
    /// Use 0.5 for Moon pairs, 1.0 for inner planets, 2.0 for outer planets.
    pub step_size_days: f64,
    /// Maximum bisection iterations (default 50).
    pub max_iterations: u32,
    /// Convergence threshold in days (default 1e-8, ~0.86 ms).
    pub convergence_days: f64,
    /// Lunar-node model used when either body is Rahu/Ketu
    /// (default: true/osculating node, matching the rest of the engine).
    pub node_mode: NodeMode,
}

impl ConjunctionConfig {
    /// Search for a conjunction (0 deg separation).
    pub fn conjunction(step_size_days: f64) -> Self {
        Self {
            target_separation_deg: 0.0,
            step_size_days,
            max_iterations: 50,
            convergence_days: 1e-8,
            node_mode: NodeMode::default(),
        }
    }

    /// Search for an opposition (180 deg separation).
    pub fn opposition(step_size_days: f64) -> Self {
        Self {
            target_separation_deg: 180.0,
            ..Self::conjunction(step_size_days)
        }
    }

    /// Search for a specific aspect angle.
    pub fn aspect(target_deg: f64, step_size_days: f64) -> Self {
        Self {
            target_separation_deg: target_deg,
            ..Self::conjunction(step_size_days)
        }
    }

    /// Validate the configuration.
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if !self.target_separation_deg.is_finite()
            || self.target_separation_deg < 0.0
            || self.target_separation_deg >= 360.0
        {
            return Err("target_separation_deg must be in [0, 360)");
        }
        if !self.step_size_days.is_finite() || self.step_size_days <= 0.0 {
            return Err("step_size_days must be positive");
        }
        if self.max_iterations == 0 {
            return Err("max_iterations must be > 0");
        }
        if !self.convergence_days.is_finite() || self.convergence_days <= 0.0 {
            return Err("convergence_days must be positive");
        }
        Ok(())
    }
}

/// Search direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// Result of a conjunction/aspect event search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConjunctionEvent {
    /// Event time as Julian Date (TDB).
    pub jd_tdb: f64,
    /// Event time as structured Gregorian UTC.
    pub utc: UtcTime,
    /// Actual ecliptic longitude separation at peak, in degrees [0, 360).
    pub actual_separation_deg: f64,
    /// Body 1 ecliptic longitude in degrees [0, 360).
    pub body1_longitude_deg: f64,
    /// Body 2 ecliptic longitude in degrees [0, 360).
    pub body2_longitude_deg: f64,
    /// Body 1 ecliptic latitude in degrees (0 for lunar nodes).
    pub body1_latitude_deg: f64,
    /// Body 2 ecliptic latitude in degrees (0 for lunar nodes).
    pub body2_latitude_deg: f64,
    /// Which bodies were involved.
    pub body1: TransitBody,
    pub body2: TransitBody,
    /// The target separation angle this event matched (equals the config's
    /// single target, or one of the requested angles in multi-angle mode).
    pub target_separation_deg: f64,
    /// Body 1 sidereal longitude in degrees, when a sidereal
    /// (sankranti) config was supplied with the request.
    pub body1_sidereal_longitude_deg: Option<f64>,
    /// Body 2 sidereal longitude in degrees (see above).
    pub body2_sidereal_longitude_deg: Option<f64>,
    /// Body 1 sidereal rashi index 0-11 (see above).
    pub body1_rashi_index: Option<u8>,
    /// Body 2 sidereal rashi index 0-11 (see above).
    pub body2_rashi_index: Option<u8>,
}
