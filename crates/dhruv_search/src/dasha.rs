//! Dasha orchestration: bridges the ephemeris engine with the pure-math
//! dasha computation in dhruv_vedic_base.
//!
//! Provides two top-level entry points:
//! - `dasha_hierarchy_for_birth`: computes full hierarchy (levels 0..N)
//! - `dasha_snapshot_at`: finds active periods at a query time (efficient)

use dhruv_core::Engine;
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::dasha::{
    DashaHierarchy, DashaSnapshot, DashaSystem, DashaVariationConfig,
    nakshatra_hierarchy, nakshatra_snapshot, vimshottari_config,
};
use dhruv_vedic_base::riseset_types::{GeoLocation, RiseSetConfig};
use dhruv_vedic_base::BhavaConfig;

use crate::error::SearchError;
use crate::panchang::moon_sidereal_longitude_at;
use crate::sankranti_types::SankrantiConfig;

/// Compute the Moon's sidereal longitude for dasha birth balance.
fn moon_sidereal_lon(
    engine: &Engine,
    _eop: &EopKernel,
    utc: &UtcTime,
    aya_config: &SankrantiConfig,
) -> Result<f64, SearchError> {
    let jd_tdb = utc.to_jd_tdb(engine.lsk());
    moon_sidereal_longitude_at(engine, jd_tdb, aya_config)
}

/// Convert UtcTime to JD UTC (calendar only, no TDB).
fn utc_to_jd_utc(utc: &UtcTime) -> f64 {
    let y = utc.year as f64;
    let m = utc.month as f64;
    let d =
        utc.day as f64 + utc.hour as f64 / 24.0 + utc.minute as f64 / 1440.0 + utc.second / 86400.0;

    let (y2, m2) = if m <= 2.0 {
        (y - 1.0, m + 12.0)
    } else {
        (y, m)
    };
    let a = (y2 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    (365.25 * (y2 + 4716.0)).floor() + (30.6001 * (m2 + 1.0)).floor() + d + b - 1524.5
}

/// Dispatch to the correct dasha engine for a given system.
///
/// Currently only Vimshottari is implemented (Phase 18a).
/// Remaining systems will be added in sub-phases 18b-18d.
fn dispatch_hierarchy(
    system: DashaSystem,
    birth_jd: f64,
    moon_sid_lon: f64,
    max_level: u8,
    variation: &DashaVariationConfig,
) -> Result<DashaHierarchy, SearchError> {
    match system {
        DashaSystem::Vimshottari => {
            let cfg = vimshottari_config();
            nakshatra_hierarchy(birth_jd, moon_sid_lon, &cfg, max_level, variation)
                .map_err(SearchError::from)
        }
        _ => Err(SearchError::InvalidConfig("dasha system not yet implemented")),
    }
}

/// Dispatch to the correct snapshot engine for a given system.
fn dispatch_snapshot(
    system: DashaSystem,
    birth_jd: f64,
    moon_sid_lon: f64,
    query_jd: f64,
    max_level: u8,
    variation: &DashaVariationConfig,
) -> Result<DashaSnapshot, SearchError> {
    match system {
        DashaSystem::Vimshottari => {
            let cfg = vimshottari_config();
            Ok(nakshatra_snapshot(
                birth_jd, moon_sid_lon, &cfg, query_jd, max_level, variation,
            ))
        }
        _ => Err(SearchError::InvalidConfig("dasha system not yet implemented")),
    }
}

/// Compute full hierarchy for a birth chart.
pub fn dasha_hierarchy_for_birth(
    engine: &Engine,
    eop: &EopKernel,
    birth_utc: &UtcTime,
    _location: &GeoLocation,
    system: DashaSystem,
    max_level: u8,
    _bhava_config: &BhavaConfig,
    _riseset_config: &RiseSetConfig,
    aya_config: &SankrantiConfig,
    variation: &DashaVariationConfig,
) -> Result<DashaHierarchy, SearchError> {
    let birth_jd = utc_to_jd_utc(birth_utc);
    let moon_sid_lon = moon_sidereal_lon(engine, eop, birth_utc, aya_config)?;
    dispatch_hierarchy(system, birth_jd, moon_sid_lon, max_level, variation)
}

/// Find active periods at a specific time.
///
/// Snapshot-only path: does NOT materialize full hierarchy. Efficient for deep levels.
pub fn dasha_snapshot_at(
    engine: &Engine,
    eop: &EopKernel,
    birth_utc: &UtcTime,
    query_utc: &UtcTime,
    _location: &GeoLocation,
    system: DashaSystem,
    max_level: u8,
    _bhava_config: &BhavaConfig,
    _riseset_config: &RiseSetConfig,
    aya_config: &SankrantiConfig,
    variation: &DashaVariationConfig,
) -> Result<DashaSnapshot, SearchError> {
    let birth_jd = utc_to_jd_utc(birth_utc);
    let query_jd = utc_to_jd_utc(query_utc);
    let moon_sid_lon = moon_sidereal_lon(engine, eop, birth_utc, aya_config)?;
    dispatch_snapshot(system, birth_jd, moon_sid_lon, query_jd, max_level, variation)
}

/// Context-sharing variant for full_kundali_for_date integration.
///
/// Takes a pre-computed Moon sidereal longitude to avoid redundant queries.
pub fn dasha_hierarchy_with_moon(
    birth_jd: f64,
    moon_sid_lon: f64,
    system: DashaSystem,
    max_level: u8,
    variation: &DashaVariationConfig,
) -> Result<DashaHierarchy, SearchError> {
    dispatch_hierarchy(system, birth_jd, moon_sid_lon, max_level, variation)
}

/// Context-sharing snapshot variant.
pub fn dasha_snapshot_with_moon(
    birth_jd: f64,
    moon_sid_lon: f64,
    query_jd: f64,
    system: DashaSystem,
    max_level: u8,
    variation: &DashaVariationConfig,
) -> Result<DashaSnapshot, SearchError> {
    dispatch_snapshot(system, birth_jd, moon_sid_lon, query_jd, max_level, variation)
}
