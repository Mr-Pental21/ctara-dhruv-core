//! General transit-body selector: ephemeris bodies plus computed lunar nodes.
//!
//! `dhruv_core::Body` intentionally excludes computed points, so search
//! operations that also accept Rahu/Ketu use this wrapper enum. It carries
//! the same wire codes and names everywhere (gochar events, ingress,
//! conjunction, motion).

use std::fmt;

use dhruv_core::Body;
use dhruv_vedic_base::LunarNode;

/// Transit code for Rahu (lunar ascending node).
pub const TRANSIT_CODE_RAHU: i32 = 10_007;
/// Transit code for Ketu (lunar descending node).
pub const TRANSIT_CODE_KETU: i32 = 10_008;

/// A body that search operations can track: an ephemeris body or a lunar node.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitBody {
    Body(Body),
    Rahu,
    Ketu,
}

/// Prints like the wrapped body (`Sun`, `Rahu`) so name-based wire
/// serialization is identical for plain bodies and nodes.
impl fmt::Debug for TransitBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Body(body) => write!(f, "{body:?}"),
            Self::Rahu => write!(f, "Rahu"),
            Self::Ketu => write!(f, "Ketu"),
        }
    }
}

impl TransitBody {
    pub const fn code(self) -> i32 {
        match self {
            Self::Body(body) => body.code(),
            Self::Rahu => TRANSIT_CODE_RAHU,
            Self::Ketu => TRANSIT_CODE_KETU,
        }
    }

    pub const fn from_code(code: i32) -> Option<Self> {
        match code {
            TRANSIT_CODE_RAHU => Some(Self::Rahu),
            TRANSIT_CODE_KETU => Some(Self::Ketu),
            _ => match Body::from_code(code) {
                Some(body) => Some(Self::Body(body)),
                None => None,
            },
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Body(body) => match body {
                Body::Sun => "Sun",
                Body::Mercury => "Mercury",
                Body::Venus => "Venus",
                Body::Earth => "Earth",
                Body::Moon => "Moon",
                Body::Mars => "Mars",
                Body::Jupiter => "Jupiter",
                Body::Saturn => "Saturn",
                Body::Uranus => "Uranus",
                Body::Neptune => "Neptune",
                Body::Pluto => "Pluto",
            },
            Self::Rahu => "Rahu",
            Self::Ketu => "Ketu",
        }
    }

    pub const fn lunar_node(self) -> Option<LunarNode> {
        match self {
            Self::Rahu => Some(LunarNode::Rahu),
            Self::Ketu => Some(LunarNode::Ketu),
            Self::Body(_) => None,
        }
    }

    pub const fn body(self) -> Option<Body> {
        match self {
            Self::Body(body) => Some(body),
            Self::Rahu | Self::Ketu => None,
        }
    }

    /// Default coarse-scan step for rashi-ingress search, in days.
    ///
    /// Chosen so a single step never spans more than a fraction of one
    /// rashi (30 deg) at the body's fastest geocentric motion. Matches the
    /// per-body steps used by gochar transit search, except the nodes use a
    /// tighter 1-day step (the osculating node swings ~0.6 deg/day near
    /// cusps).
    pub const fn default_ingress_step_days(self) -> f64 {
        match self {
            Self::Body(Body::Moon) => 0.25,
            Self::Body(Body::Mercury | Body::Venus) => 0.5,
            Self::Body(Body::Sun | Body::Mars) => 1.0,
            Self::Body(Body::Jupiter | Body::Saturn) => 2.0,
            Self::Body(Body::Uranus | Body::Neptune | Body::Pluto) => 5.0,
            // True node can swing ~0.6 deg/day either way near cusps.
            Self::Rahu | Self::Ketu => 1.0,
            Self::Body(_) => 1.0,
        }
    }

    /// Scan ceiling for next/prev ingress search (any-rashi target), in days.
    ///
    /// Upper bound on the wait until the body's next rashi boundary crossing,
    /// including retrograde loitering (values from the bodies' slowest
    /// per-rashi transit durations, with margin). The Sun keeps the legacy
    /// 400-day ceiling.
    pub const fn ingress_max_scan_days(self) -> f64 {
        match self {
            Self::Body(Body::Moon) => 40.0,
            Self::Body(Body::Sun) => 400.0,
            Self::Body(Body::Mercury) => 500.0,
            Self::Body(Body::Venus) => 700.0,
            Self::Body(Body::Mars) => 1_500.0,
            Self::Body(Body::Jupiter) => 1_500.0,
            Self::Body(Body::Saturn) => 2_000.0,
            Self::Body(Body::Uranus) => 4_000.0,
            Self::Body(Body::Neptune) => 7_000.0,
            Self::Body(Body::Pluto) => 13_000.0,
            Self::Rahu | Self::Ketu => 800.0,
            Self::Body(_) => 400.0,
        }
    }

    /// Mean geocentric longitude rate in deg/day (signed; nodes regress).
    ///
    /// Derived from standard J2000 sidereal orbital periods. Inferior
    /// planets share the Sun's mean geocentric rate (they oscillate about
    /// it). Used only to bound conjunction scan windows.
    pub(crate) const fn mean_rate_deg_per_day(self) -> f64 {
        match self {
            Self::Body(Body::Moon) => 13.1764,
            Self::Body(Body::Sun | Body::Mercury | Body::Venus | Body::Earth) => 0.9856,
            Self::Body(Body::Mars) => 0.5240,
            Self::Body(Body::Jupiter) => 0.0831,
            Self::Body(Body::Saturn) => 0.0335,
            Self::Body(Body::Uranus) => 0.0117,
            Self::Body(Body::Neptune) => 0.0060,
            Self::Body(Body::Pluto) => 0.0040,
            Self::Rahu | Self::Ketu => -0.0529,
        }
    }
}

impl From<Body> for TransitBody {
    fn from(value: Body) -> Self {
        Self::Body(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        for code in [10, 199, 299, 301, 399, 499, 599, 699, 799, 899, 999] {
            let tb = TransitBody::from_code(code).unwrap();
            assert_eq!(tb.code(), code);
            assert!(tb.body().is_some());
        }
        assert_eq!(
            TransitBody::from_code(TRANSIT_CODE_RAHU),
            Some(TransitBody::Rahu)
        );
        assert_eq!(
            TransitBody::from_code(TRANSIT_CODE_KETU),
            Some(TransitBody::Ketu)
        );
        assert_eq!(TransitBody::from_code(12_345), None);
    }

    #[test]
    fn debug_matches_inner_body() {
        assert_eq!(format!("{:?}", TransitBody::Body(Body::Sun)), "Sun");
        assert_eq!(format!("{:?}", TransitBody::Rahu), "Rahu");
        assert_eq!(format!("{:?}", TransitBody::Ketu), "Ketu");
    }

    #[test]
    fn node_helpers() {
        assert_eq!(TransitBody::Rahu.lunar_node(), Some(LunarNode::Rahu));
        assert_eq!(TransitBody::Ketu.lunar_node(), Some(LunarNode::Ketu));
        assert_eq!(TransitBody::Body(Body::Sun).lunar_node(), None);
        assert_eq!(TransitBody::Rahu.body(), None);
    }

    #[test]
    fn ingress_defaults_are_positive() {
        for code in [
            10, 199, 299, 301, 499, 599, 699, 799, 899, 999, 10_007, 10_008,
        ] {
            let tb = TransitBody::from_code(code).unwrap();
            assert!(tb.default_ingress_step_days() > 0.0);
            assert!(tb.ingress_max_scan_days() >= 40.0);
        }
    }
}
