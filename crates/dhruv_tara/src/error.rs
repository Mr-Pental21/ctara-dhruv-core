//! Error types for the `dhruv_tara` crate.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur in fixed star computations.
#[derive(Debug, Clone, PartialEq)]
pub enum TaraError {
    /// Star not found in catalog.
    StarNotFound(String),
    /// Catalog file could not be loaded or parsed.
    CatalogLoad(String),
    /// Earth position/velocity is required for Apparent tier or parallax.
    EarthStateRequired,
}

impl Display for TaraError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StarNotFound(id) => write!(f, "star not found in catalog: {id}"),
            Self::CatalogLoad(msg) => write!(f, "catalog load error: {msg}"),
            Self::EarthStateRequired => {
                write!(f, "earth state required for Apparent tier or parallax")
            }
        }
    }
}

impl Error for TaraError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_star_not_found() {
        let e = TaraError::StarNotFound("Spica".into());
        assert!(e.to_string().contains("Spica"));
    }

    #[test]
    fn display_catalog_load() {
        let e = TaraError::CatalogLoad("bad json".into());
        assert!(e.to_string().contains("bad json"));
    }

    #[test]
    fn display_earth_state_required() {
        let e = TaraError::EarthStateRequired;
        assert!(e.to_string().contains("earth state required"));
    }
}
