//! Convenience wrapper for the ctara-dhruv ephemeris engine.
//!
//! Provides a global singleton engine and high-level functions that accept
//! UTC dates directly, removing the need to manually manage `Engine` handles,
//! convert UTC→TDB, or transform Cartesian→spherical.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use std::path::PathBuf;
//! use dhruv_rs::*;
//!
//! let config = EngineConfig::with_single_spk(
//!     PathBuf::from("kernels/data/de442s.bsp"),
//!     PathBuf::from("kernels/data/naif0012.tls"),
//!     256,
//!     true,
//! );
//! init(config).expect("engine init");
//!
//! let date: UtcDate = "2024-03-20T12:00:00Z".parse().unwrap();
//! let lon = longitude(Body::Mars, Observer::Body(Body::Earth), date).unwrap();
//! println!("Mars ecliptic longitude: {:.4}°", lon.to_degrees());
//! ```

pub mod convenience;
pub mod date;
pub mod error;
pub mod global;

// Primary re-exports — users should only need `use dhruv_rs::*`
pub use convenience::{
    ayana, longitude, masa, nakshatra, nakshatra28, next_amavasya, next_purnima, next_sankranti,
    position, position_full, prev_amavasya, prev_purnima, prev_sankranti, query, query_batch,
    rashi, sidereal_longitude, varsha,
};
pub use date::UtcDate;
pub use error::DhruvError;
pub use global::{init, is_initialized};

// Re-export core types so callers don't need to depend on dhruv_core directly.
pub use dhruv_core::{Body, EngineConfig, Frame, Observer, StateVector};
pub use dhruv_frames::{SphericalCoords, SphericalState};

// Re-export vedic types used by the convenience functions.
pub use dhruv_vedic_base::{
    Ayana as AyanaKind, AyanamshaSystem, Dms, Masa as MasaName,
    Nakshatra as NakshatraName, Nakshatra28 as Nakshatra28Name, Nakshatra28Info, NakshatraInfo,
    Rashi as RashiName, RashiInfo, Samvatsara as SamvatsaraName, deg_to_dms,
};

// Re-export search result types used by convenience functions.
pub use dhruv_search::lunar_phase_types::LunarPhaseEvent;
pub use dhruv_search::panchang_types::{AyanaInfo, MasaInfo, VarshaInfo};
pub use dhruv_search::sankranti_types::{SankrantiConfig, SankrantiEvent};
