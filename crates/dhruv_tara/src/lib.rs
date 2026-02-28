//! Fixed star positions with proper motion propagation.
//!
//! Provides position computation for 122 Vedic and astronomical reference stars
//! using HGCA ICRS J2016.0 astrometric data (Brandt 2021, ApJS 254, 42).
//! Supports equatorial, ecliptic, and sidereal coordinate output with optional
//! apparent-place corrections.

pub mod apparent;
pub mod catalog;
pub mod config;
pub mod error;
pub mod galactic;
pub mod position;
pub mod propagation;
pub mod tara_id;

pub use apparent::{apply_aberration, apply_light_deflection};
pub use catalog::{TaraCatalog, TaraEntry};
pub use config::{EarthState, TaraAccuracy, TaraConfig};
pub use error::TaraError;
pub use galactic::{galactic_anticenter_icrs, galactic_center_icrs};
pub use position::{position_ecliptic, position_equatorial, sidereal_longitude};
pub use position::{
    position_ecliptic_with_config, position_equatorial_with_config, sidereal_longitude_with_config,
};
pub use propagation::{EquatorialPosition, propagate_position};
pub use tara_id::{TaraCategory, TaraId};
