//! Canonical non-search operation APIs shared across wrappers and frontends.

use dhruv_core::Engine;
use dhruv_frames::SphericalCoords;
use dhruv_tara::{
    EarthState, EquatorialPosition, TaraCatalog, TaraConfig, TaraError, TaraId,
    position_ecliptic_with_config, position_equatorial_with_config, sidereal_longitude_with_config,
};
use dhruv_time::EopKernel;
use dhruv_vedic_base::{
    AyanamshaSystem, LunarNode, NodeMode, ayanamsha_deg, ayanamsha_mean_deg, ayanamsha_true_deg,
    jd_tdb_to_centuries, lunar_node_deg, lunar_node_deg_for_epoch,
};

use crate::error::SearchError;

/// High-level query modes used across operation APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryMode {
    /// Find the first result after a timestamp.
    Next,
    /// Find the first result before a timestamp.
    Prev,
    /// Find all results inside an interval.
    Range,
    /// Evaluate at one explicit date/time.
    AtDate,
}

/// Ayanamsha computation mode selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AyanamshaMode {
    /// Mean ayanamsha model (no nutation term).
    Mean,
    /// True ayanamsha from explicit delta-psi arcseconds.
    True,
    /// Unified ayanamsha (`use_nutation` flag controls mean/true behavior).
    Unified,
}

/// Canonical ayanamsha operation request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AyanamshaOperation {
    /// Ayanamsha system.
    pub system: AyanamshaSystem,
    /// Computation mode selector.
    pub mode: AyanamshaMode,
    /// Epoch as JD TDB.
    pub at_jd_tdb: f64,
    /// Nutation inclusion flag used by `Unified` mode.
    pub use_nutation: bool,
    /// Delta-psi arcseconds used by `True` mode.
    pub delta_psi_arcsec: f64,
}

/// Execute an ayanamsha operation request.
pub fn ayanamsha(op: &AyanamshaOperation) -> Result<f64, SearchError> {
    let t = jd_tdb_to_centuries(op.at_jd_tdb);
    let deg = match op.mode {
        AyanamshaMode::Mean => ayanamsha_mean_deg(op.system, t),
        AyanamshaMode::True => ayanamsha_true_deg(op.system, t, op.delta_psi_arcsec),
        AyanamshaMode::Unified => ayanamsha_deg(op.system, t, op.use_nutation),
    };
    Ok(deg)
}

/// Lunar-node backend selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeBackend {
    /// Analytic backend (`lunar_node_deg`) that does not use engine states.
    Analytic,
    /// Engine-backed backend (`lunar_node_deg_for_epoch`).
    Engine,
}

/// Canonical lunar-node operation request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeOperation {
    /// Rahu or Ketu selector.
    pub node: LunarNode,
    /// Mean or true node model.
    pub mode: NodeMode,
    /// Backend selector.
    pub backend: NodeBackend,
    /// Epoch as JD TDB.
    pub at_jd_tdb: f64,
}

/// Execute a lunar-node operation request.
pub fn lunar_node(engine: &Engine, op: &NodeOperation) -> Result<f64, SearchError> {
    match op.backend {
        NodeBackend::Analytic => {
            let t = jd_tdb_to_centuries(op.at_jd_tdb);
            Ok(lunar_node_deg(op.node, t, op.mode))
        }
        NodeBackend::Engine => Ok(lunar_node_deg_for_epoch(
            engine,
            op.node,
            op.at_jd_tdb,
            op.mode,
        )?),
    }
}

pub use dhruv_search::operations::{
    PANCHANG_INCLUDE_ALL, PANCHANG_INCLUDE_ALL_CALENDAR, PANCHANG_INCLUDE_ALL_CORE,
    PANCHANG_INCLUDE_AYANA, PANCHANG_INCLUDE_GHATIKA, PANCHANG_INCLUDE_HORA,
    PANCHANG_INCLUDE_KARANA, PANCHANG_INCLUDE_LOCATION_DEPENDENT,
    PANCHANG_INCLUDE_LOCATION_INDEPENDENT, PANCHANG_INCLUDE_MASA, PANCHANG_INCLUDE_NAKSHATRA,
    PANCHANG_INCLUDE_TITHI, PANCHANG_INCLUDE_VAAR, PANCHANG_INCLUDE_VARSHA, PANCHANG_INCLUDE_YOGA,
    PanchangOperation, PanchangResult, panchang_include_bits,
};

/// Execute a panchang operation request.
///
/// Delegates to the canonical implementation in `dhruv_search`; only
/// elements selected in `include_mask` are computed.
pub fn panchang(
    engine: &Engine,
    eop: &EopKernel,
    op: &PanchangOperation,
) -> Result<PanchangResult, SearchError> {
    Ok(dhruv_search::operations::panchang(engine, eop, op)?)
}

/// Tara output selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaraOutputKind {
    /// ICRS equatorial position (RA/Dec/distance AU).
    Equatorial,
    /// Ecliptic-of-date spherical coordinates.
    Ecliptic,
    /// Sidereal longitude in degrees.
    Sidereal,
}

/// Canonical tara operation request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaraOperation {
    /// Tara identifier.
    pub star: TaraId,
    /// Output selector.
    pub output: TaraOutputKind,
    /// Epoch as JD TDB.
    pub at_jd_tdb: f64,
    /// Ayanamsha in degrees (used by sidereal output).
    pub ayanamsha_deg: f64,
    /// Fixed-star computation configuration.
    pub config: TaraConfig,
    /// Optional Earth state for apparent/parallax modes.
    pub earth_state: Option<EarthState>,
}

/// Canonical tara operation response.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaraResult {
    /// Equatorial output.
    Equatorial(EquatorialPosition),
    /// Ecliptic output.
    Ecliptic(SphericalCoords),
    /// Sidereal longitude in degrees.
    Sidereal(f64),
}

/// Execute a tara operation request.
pub fn tara(catalog: &TaraCatalog, op: &TaraOperation) -> Result<TaraResult, TaraError> {
    match op.output {
        TaraOutputKind::Equatorial => Ok(TaraResult::Equatorial(position_equatorial_with_config(
            catalog,
            op.star,
            op.at_jd_tdb,
            &op.config,
            op.earth_state.as_ref(),
        )?)),
        TaraOutputKind::Ecliptic => Ok(TaraResult::Ecliptic(position_ecliptic_with_config(
            catalog,
            op.star,
            op.at_jd_tdb,
            &op.config,
            op.earth_state.as_ref(),
        )?)),
        TaraOutputKind::Sidereal => Ok(TaraResult::Sidereal(sidereal_longitude_with_config(
            catalog,
            op.star,
            op.at_jd_tdb,
            op.ayanamsha_deg,
            &op.config,
            op.earth_state.as_ref(),
        )?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_mode_at_date_is_stable() {
        assert_eq!(QueryMode::AtDate, QueryMode::AtDate);
    }

    #[test]
    fn ayanamsha_mode_is_stable() {
        let op = AyanamshaOperation {
            system: AyanamshaSystem::Lahiri,
            mode: AyanamshaMode::Mean,
            at_jd_tdb: 2_451_545.0,
            use_nutation: false,
            delta_psi_arcsec: 0.0,
        };
        assert!(ayanamsha(&op).is_ok());
    }

    #[test]
    fn node_backend_is_stable() {
        assert_eq!(NodeBackend::Analytic, NodeBackend::Analytic);
        assert_eq!(NodeBackend::Engine, NodeBackend::Engine);
    }

    #[test]
    fn panchang_include_mask_is_stable() {
        assert_eq!(PANCHANG_INCLUDE_ALL_CORE, 0x7f);
        assert_eq!(PANCHANG_INCLUDE_ALL_CALENDAR, 0x380);
        assert_eq!(PANCHANG_INCLUDE_ALL, 0x3ff);
    }
}
