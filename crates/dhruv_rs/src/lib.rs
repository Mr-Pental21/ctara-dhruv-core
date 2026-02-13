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
    all_rise_set_events, arudha_padas, ashtakavarga, ayana, ayanamsha, bhavas, core_bindus,
    drishti, ghatika, graha_longitudes, graha_positions, hora, karana, lagna, longitude, lunar_node,
    masa, mc, moon_nakshatra, nakshatra, nakshatra28, next_amavasya, next_chandra_grahan,
    next_conjunction, next_max_speed, next_purnima, next_sankranti, next_specific_sankranti,
    next_stationary, next_surya_grahan, nutation, panchang, position, position_full,
    prev_amavasya, prev_chandra_grahan, prev_conjunction, prev_max_speed, prev_purnima,
    prev_sankranti, prev_specific_sankranti, prev_stationary, prev_surya_grahan, query,
    query_batch, ramc, rashi, search_amavasyas, search_chandra_grahan, search_conjunctions,
    search_max_speed, search_purnimas, search_sankrantis, search_stationary, search_surya_grahan,
    sidereal_longitude, special_lagnas, sphutas, sunrise, sunset, tithi, upagrahas, vaar, varsha,
    yoga,
};
pub use date::UtcDate;
pub use error::DhruvError;
pub use global::{init, is_initialized};

// Re-export core types so callers don't need to depend on dhruv_core directly.
pub use dhruv_core::{Body, EngineConfig, Frame, Observer, StateVector};
pub use dhruv_frames::{SphericalCoords, SphericalState};

// Re-export vedic types used by the convenience functions.
pub use dhruv_search::{DrishtiConfig, DrishtiResult, GrahaLongitudes};
pub use dhruv_vedic_base::riseset_types::GeoLocation;
pub use dhruv_vedic_base::{
    AllSpecialLagnas, AllUpagrahas, ArudhaPada, ArudhaResult, AshtakavargaResult,
    BhinnaAshtakavarga, Graha, SarvaAshtakavarga, SpecialLagna, Sphuta, SphutalInputs, Upagraha,
};
pub use dhruv_vedic_base::{
    Ayana as AyanaKind, AyanamshaSystem, Dms, Hora as HoraLord, Karana as KaranaName,
    Masa as MasaName, Nakshatra as NakshatraName, Nakshatra28 as Nakshatra28Name, Nakshatra28Info,
    NakshatraInfo, Paksha, Rashi as RashiName, RashiInfo, Samvatsara as SamvatsaraName,
    Tithi as TithiName, Vaar as VaarName, Yoga as YogaName, deg_to_dms,
};
pub use dhruv_vedic_base::{DrishtiEntry, GrahaDrishtiMatrix};
pub use dhruv_vedic_base::{
    BhavaConfig, BhavaResult, BhavaSystem, Bhava, LunarNode, NodeMode,
};
pub use dhruv_vedic_base::riseset_types::{RiseSetConfig, RiseSetEvent, RiseSetResult, SunLimb};

// Re-export EopKernel for sunrise-based panchang functions.
pub use dhruv_time::EopKernel;

// Re-export search result types used by convenience functions.
pub use dhruv_search::lunar_phase_types::LunarPhaseEvent;
pub use dhruv_search::panchang_types::{
    AyanaInfo, GhatikaInfo, HoraInfo, KaranaInfo, MasaInfo, PanchangInfo, TithiInfo, VaarInfo,
    VarshaInfo, YogaInfo,
};
pub use dhruv_search::sankranti_types::{SankrantiConfig, SankrantiEvent};
pub use dhruv_search::conjunction_types::{ConjunctionConfig, ConjunctionEvent};
pub use dhruv_search::grahan_types::{
    ChandraGrahan, ChandraGrahanType, GrahanConfig, SuryaGrahan, SuryaGrahanType,
};
pub use dhruv_search::stationary_types::{
    MaxSpeedEvent, MaxSpeedType, StationType, StationaryConfig, StationaryEvent,
};
