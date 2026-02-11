//! Celestial event search engine: conjunctions, oppositions, aspects, and eclipses.
//!
//! This crate provides:
//! - General-purpose conjunction/separation engine for any body pair
//! - Lunar eclipse computation (penumbral, partial, total)
//! - Solar eclipse computation (geocentric and topocentric)

pub mod conjunction;
pub mod conjunction_types;
pub mod eclipse;
pub mod eclipse_types;
pub mod error;

pub use conjunction::{next_conjunction, prev_conjunction, search_conjunctions};
pub use conjunction_types::{ConjunctionConfig, ConjunctionEvent, SearchDirection};
pub use eclipse::{
    next_lunar_eclipse, next_solar_eclipse, prev_lunar_eclipse, prev_solar_eclipse,
    search_lunar_eclipses, search_solar_eclipses,
};
pub use eclipse_types::{
    EclipseConfig, GeoLocation, LunarEclipse, LunarEclipseType, SolarEclipse, SolarEclipseType,
};
pub use error::SearchError;
