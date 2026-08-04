//! Canonical operation-style APIs shared across wrappers and frontends.
//!
//! This module is the migration layer from split function surfaces
//! (`next_*`, `prev_*`, `search_*`) to config-driven operation requests.

use crate::transit_body::TransitBody;
use dhruv_core::{Body, Engine};
use dhruv_frames::SphericalCoords;
use dhruv_tara::{
    EarthState, EquatorialPosition, TaraCatalog, TaraConfig, TaraError, TaraId,
    position_ecliptic_with_config, position_equatorial_with_config, sidereal_longitude_with_config,
};
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::{
    AyanamshaSystem, GeoLocation, LunarNode, NodeMode, Rashi, RiseSetConfig, ayanamsha_deg,
    ayanamsha_mean_deg, ayanamsha_true_deg, jd_tdb_to_centuries, lunar_node_deg,
    lunar_node_deg_for_epoch,
};

use crate::conjunction_types::{ConjunctionConfig, ConjunctionEvent};
use crate::error::SearchError;
use crate::grahan_types::{
    ChandraGrahan, GeoLocation as GrahanGeoLocation, GrahanConfig, SuryaGrahan,
};
use crate::lunar_phase_types::LunarPhaseEvent;
use crate::sankranti_types::{SankrantiConfig, SankrantiEvent};
use crate::stationary_types::{MaxSpeedEvent, StationaryConfig, StationaryEvent};
use crate::{
    next_amavasya, next_chandra_grahan, next_conjunction, next_fixed_longitude, next_ingress,
    next_max_speed, next_purnima, next_specific_ingress, next_stationary, next_surya_grahan,
    panchang_for_date, prev_amavasya, prev_chandra_grahan, prev_conjunction, prev_fixed_longitude,
    prev_ingress, prev_max_speed, prev_purnima, prev_specific_ingress, prev_stationary,
    prev_surya_grahan, search_amavasyas, search_chandra_grahan, search_conjunctions,
    search_fixed_longitudes, search_ingresses, search_max_speed, search_purnimas,
    search_stationary, search_surya_grahan,
};

/// High-level query mode used by operation requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryMode {
    /// Find the first event after a timestamp.
    Next,
    /// Find the first event before a timestamp.
    Prev,
    /// Find all events in an interval.
    Range,
}

/// Conjunction search query variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConjunctionQuery {
    /// Find next event after `at_jd_tdb`.
    Next { at_jd_tdb: f64 },
    /// Find previous event before `at_jd_tdb`.
    Prev { at_jd_tdb: f64 },
    /// Find all events in `[start_jd_tdb, end_jd_tdb]`.
    Range { start_jd_tdb: f64, end_jd_tdb: f64 },
}

impl ConjunctionQuery {
    /// Returns the mode represented by this query.
    pub fn mode(self) -> QueryMode {
        match self {
            Self::Next { .. } => QueryMode::Next,
            Self::Prev { .. } => QueryMode::Prev,
            Self::Range { .. } => QueryMode::Range,
        }
    }
}

/// Canonical conjunction operation request.
#[derive(Debug, Clone, PartialEq)]
pub struct ConjunctionOperation {
    /// First body (plain body or Rahu/Ketu).
    pub body1: TransitBody,
    /// Second body (plain body or Rahu/Ketu).
    pub body2: TransitBody,
    /// Numerical search configuration.
    pub config: ConjunctionConfig,
    /// Additional target separation angles for a multi-angle sweep.
    /// Empty = use `config.target_separation_deg` only. Each returned event
    /// carries the angle it matched in `target_separation_deg`.
    pub target_separations_deg: Vec<f64>,
    /// When set, events also carry sidereal longitudes and rashi indices
    /// computed with this configuration.
    pub sankranti_config: Option<SankrantiConfig>,
    /// Query selector and time bounds.
    pub query: ConjunctionQuery,
}

/// Canonical conjunction operation response.
#[derive(Debug, Clone, PartialEq)]
pub enum ConjunctionResult {
    /// Result for next/prev requests.
    Single(Option<ConjunctionEvent>),
    /// Result for range requests.
    Many(Vec<ConjunctionEvent>),
}

/// Sidereal longitude + rashi index of a transit body at an event time.
fn sidereal_echo_at(
    engine: &Engine,
    body: TransitBody,
    jd_tdb: f64,
    sankranti_config: &SankrantiConfig,
) -> Result<(f64, u8), SearchError> {
    let sid = crate::sankranti::transit_sidereal_longitude(engine, body, jd_tdb, sankranti_config)?;
    Ok((sid, (sid.rem_euclid(360.0) / 30.0) as u8 % 12))
}

fn enrich_conjunction_event(
    engine: &Engine,
    event: &mut ConjunctionEvent,
    sankranti_config: &SankrantiConfig,
) -> Result<(), SearchError> {
    let (s1, r1) = sidereal_echo_at(engine, event.body1, event.jd_tdb, sankranti_config)?;
    let (s2, r2) = sidereal_echo_at(engine, event.body2, event.jd_tdb, sankranti_config)?;
    event.body1_sidereal_longitude_deg = Some(s1);
    event.body2_sidereal_longitude_deg = Some(s2);
    event.body1_rashi_index = Some(r1);
    event.body2_rashi_index = Some(r2);
    Ok(())
}

/// Sidereal config used for echoes: the node model must match the one the
/// search itself used, so it is taken from the search config.
fn echo_config(
    sankranti_config: &SankrantiConfig,
    node_mode: dhruv_vedic_base::NodeMode,
) -> SankrantiConfig {
    SankrantiConfig {
        node_mode,
        ..*sankranti_config
    }
}

/// Execute a conjunction operation request.
pub fn conjunction(
    engine: &Engine,
    op: &ConjunctionOperation,
) -> Result<ConjunctionResult, SearchError> {
    let single_angle = [op.config.target_separation_deg];
    let angles: &[f64] = if op.target_separations_deg.is_empty() {
        &single_angle
    } else {
        &op.target_separations_deg
    };
    let config_for = |angle: f64| ConjunctionConfig {
        target_separation_deg: angle,
        ..op.config
    };

    let mut result = match op.query {
        ConjunctionQuery::Next { at_jd_tdb } => {
            let mut best: Option<ConjunctionEvent> = None;
            for &angle in angles {
                let found =
                    next_conjunction(engine, op.body1, op.body2, at_jd_tdb, &config_for(angle))?;
                if let Some(ev) = found {
                    if best.is_none_or(|b| ev.jd_tdb < b.jd_tdb) {
                        best = Some(ev);
                    }
                }
            }
            ConjunctionResult::Single(best)
        }
        ConjunctionQuery::Prev { at_jd_tdb } => {
            let mut best: Option<ConjunctionEvent> = None;
            for &angle in angles {
                let found =
                    prev_conjunction(engine, op.body1, op.body2, at_jd_tdb, &config_for(angle))?;
                if let Some(ev) = found {
                    if best.is_none_or(|b| ev.jd_tdb > b.jd_tdb) {
                        best = Some(ev);
                    }
                }
            }
            ConjunctionResult::Single(best)
        }
        ConjunctionQuery::Range {
            start_jd_tdb,
            end_jd_tdb,
        } => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            let mut events = Vec::new();
            for &angle in angles {
                events.extend(search_conjunctions(
                    engine,
                    op.body1,
                    op.body2,
                    start_jd_tdb,
                    end_jd_tdb,
                    &config_for(angle),
                )?);
            }
            events.sort_by(|a, b| {
                a.jd_tdb
                    .total_cmp(&b.jd_tdb)
                    .then_with(|| a.target_separation_deg.total_cmp(&b.target_separation_deg))
            });
            ConjunctionResult::Many(events)
        }
    };

    if let Some(sc) = op.sankranti_config.as_ref() {
        let sc = echo_config(sc, op.config.node_mode);
        match &mut result {
            ConjunctionResult::Single(Some(ev)) => enrich_conjunction_event(engine, ev, &sc)?,
            ConjunctionResult::Many(events) => {
                for ev in events.iter_mut() {
                    enrich_conjunction_event(engine, ev, &sc)?;
                }
            }
            ConjunctionResult::Single(None) => {}
        }
    }

    Ok(result)
}

/// Grahan kind selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrahanKind {
    /// Lunar eclipse.
    Chandra,
    /// Solar eclipse.
    Surya,
}

/// Grahan search query variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GrahanQuery {
    /// Find next event after `at_jd_tdb`.
    Next { at_jd_tdb: f64 },
    /// Find previous event before `at_jd_tdb`.
    Prev { at_jd_tdb: f64 },
    /// Find all events in `[start_jd_tdb, end_jd_tdb]`.
    Range { start_jd_tdb: f64, end_jd_tdb: f64 },
}

impl GrahanQuery {
    /// Returns the mode represented by this query.
    pub fn mode(self) -> QueryMode {
        match self {
            Self::Next { .. } => QueryMode::Next,
            Self::Prev { .. } => QueryMode::Prev,
            Self::Range { .. } => QueryMode::Range,
        }
    }
}

/// Canonical grahan operation request.
#[derive(Debug, Clone, PartialEq)]
pub struct GrahanOperation {
    /// Which grahan family to query.
    pub kind: GrahanKind,
    /// Search configuration.
    pub config: GrahanConfig,
    /// Optional observer location for local solar-eclipse circumstances.
    /// Ignored for lunar eclipses.
    pub location: Option<GrahanGeoLocation>,
    /// Query selector and time bounds.
    pub query: GrahanQuery,
}

/// Canonical grahan operation response.
#[derive(Debug, Clone, PartialEq)]
pub enum GrahanResult {
    /// Single chandra result (next/prev).
    ChandraSingle(Option<Box<ChandraGrahan>>),
    /// Chandra range result.
    ChandraMany(Vec<ChandraGrahan>),
    /// Single surya result (next/prev).
    SuryaSingle(Option<Box<SuryaGrahan>>),
    /// Surya range result.
    SuryaMany(Vec<SuryaGrahan>),
}

/// Execute a grahan operation request.
pub fn grahan(
    engine: &Engine,
    eop: Option<&EopKernel>,
    op: &GrahanOperation,
) -> Result<GrahanResult, SearchError> {
    match (op.kind, op.query) {
        (GrahanKind::Chandra, GrahanQuery::Next { at_jd_tdb }) => Ok(GrahanResult::ChandraSingle(
            next_chandra_grahan(engine, at_jd_tdb, &op.config)?.map(Box::new),
        )),
        (GrahanKind::Chandra, GrahanQuery::Prev { at_jd_tdb }) => Ok(GrahanResult::ChandraSingle(
            prev_chandra_grahan(engine, at_jd_tdb, &op.config)?.map(Box::new),
        )),
        (
            GrahanKind::Chandra,
            GrahanQuery::Range {
                start_jd_tdb,
                end_jd_tdb,
            },
        ) => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            Ok(GrahanResult::ChandraMany(search_chandra_grahan(
                engine,
                start_jd_tdb,
                end_jd_tdb,
                &op.config,
            )?))
        }
        (GrahanKind::Surya, GrahanQuery::Next { at_jd_tdb }) => Ok(GrahanResult::SuryaSingle(
            next_surya_grahan(engine, eop, at_jd_tdb, op.location, &op.config)?.map(Box::new),
        )),
        (GrahanKind::Surya, GrahanQuery::Prev { at_jd_tdb }) => Ok(GrahanResult::SuryaSingle(
            prev_surya_grahan(engine, eop, at_jd_tdb, op.location, &op.config)?.map(Box::new),
        )),
        (
            GrahanKind::Surya,
            GrahanQuery::Range {
                start_jd_tdb,
                end_jd_tdb,
            },
        ) => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            Ok(GrahanResult::SuryaMany(search_surya_grahan(
                engine,
                eop,
                start_jd_tdb,
                end_jd_tdb,
                op.location,
                &op.config,
            )?))
        }
    }
}

/// Motion event kind selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MotionKind {
    /// Stationary events (retrograde/direct station points).
    Stationary,
    /// Maximum-speed events (direct/retrograde extrema).
    MaxSpeed,
}

/// Motion search query variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionQuery {
    /// Find next event after `at_jd_tdb`.
    Next { at_jd_tdb: f64 },
    /// Find previous event before `at_jd_tdb`.
    Prev { at_jd_tdb: f64 },
    /// Find all events in `[start_jd_tdb, end_jd_tdb]`.
    Range { start_jd_tdb: f64, end_jd_tdb: f64 },
}

impl MotionQuery {
    /// Returns the mode represented by this query.
    pub fn mode(self) -> QueryMode {
        match self {
            Self::Next { .. } => QueryMode::Next,
            Self::Prev { .. } => QueryMode::Prev,
            Self::Range { .. } => QueryMode::Range,
        }
    }
}

/// Canonical motion operation request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionOperation {
    /// Body to search for (plain body or Rahu/Ketu; stationary search of the
    /// nodes requires the true node model).
    pub body: TransitBody,
    /// Motion event family.
    pub kind: MotionKind,
    /// Search configuration.
    pub config: StationaryConfig,
    /// When set, events also carry sidereal longitudes and rashi indices
    /// computed with this configuration.
    pub sankranti_config: Option<SankrantiConfig>,
    /// Query selector and time bounds.
    pub query: MotionQuery,
}

/// Canonical motion operation response.
#[derive(Debug, Clone, PartialEq)]
pub enum MotionResult {
    /// Single stationary event for next/prev requests.
    StationarySingle(Option<StationaryEvent>),
    /// Stationary range results.
    StationaryMany(Vec<StationaryEvent>),
    /// Single max-speed event for next/prev requests.
    MaxSpeedSingle(Option<MaxSpeedEvent>),
    /// Max-speed range results.
    MaxSpeedMany(Vec<MaxSpeedEvent>),
}

fn enrich_stationary_event(
    engine: &Engine,
    event: &mut StationaryEvent,
    sankranti_config: &SankrantiConfig,
) -> Result<(), SearchError> {
    let (sid, rashi) = sidereal_echo_at(engine, event.body, event.jd_tdb, sankranti_config)?;
    event.sidereal_longitude_deg = Some(sid);
    event.rashi_index = Some(rashi);
    Ok(())
}

fn enrich_max_speed_event(
    engine: &Engine,
    event: &mut MaxSpeedEvent,
    sankranti_config: &SankrantiConfig,
) -> Result<(), SearchError> {
    let (sid, rashi) = sidereal_echo_at(engine, event.body, event.jd_tdb, sankranti_config)?;
    event.sidereal_longitude_deg = Some(sid);
    event.rashi_index = Some(rashi);
    Ok(())
}

/// Execute a motion operation request.
pub fn motion(engine: &Engine, op: &MotionOperation) -> Result<MotionResult, SearchError> {
    let mut result = motion_query(engine, op)?;
    if let Some(sc) = op.sankranti_config.as_ref() {
        let sc = echo_config(sc, op.config.node_mode);
        match &mut result {
            MotionResult::StationarySingle(Some(ev)) => enrich_stationary_event(engine, ev, &sc)?,
            MotionResult::StationaryMany(events) => {
                for ev in events.iter_mut() {
                    enrich_stationary_event(engine, ev, &sc)?;
                }
            }
            MotionResult::MaxSpeedSingle(Some(ev)) => enrich_max_speed_event(engine, ev, &sc)?,
            MotionResult::MaxSpeedMany(events) => {
                for ev in events.iter_mut() {
                    enrich_max_speed_event(engine, ev, &sc)?;
                }
            }
            MotionResult::StationarySingle(None) | MotionResult::MaxSpeedSingle(None) => {}
        }
    }
    Ok(result)
}

fn motion_query(engine: &Engine, op: &MotionOperation) -> Result<MotionResult, SearchError> {
    match (op.kind, op.query) {
        (MotionKind::Stationary, MotionQuery::Next { at_jd_tdb }) => {
            Ok(MotionResult::StationarySingle(next_stationary(
                engine, op.body, at_jd_tdb, &op.config,
            )?))
        }
        (MotionKind::Stationary, MotionQuery::Prev { at_jd_tdb }) => {
            Ok(MotionResult::StationarySingle(prev_stationary(
                engine, op.body, at_jd_tdb, &op.config,
            )?))
        }
        (
            MotionKind::Stationary,
            MotionQuery::Range {
                start_jd_tdb,
                end_jd_tdb,
            },
        ) => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            Ok(MotionResult::StationaryMany(search_stationary(
                engine,
                op.body,
                start_jd_tdb,
                end_jd_tdb,
                &op.config,
            )?))
        }
        (MotionKind::MaxSpeed, MotionQuery::Next { at_jd_tdb }) => Ok(
            MotionResult::MaxSpeedSingle(next_max_speed(engine, op.body, at_jd_tdb, &op.config)?),
        ),
        (MotionKind::MaxSpeed, MotionQuery::Prev { at_jd_tdb }) => Ok(
            MotionResult::MaxSpeedSingle(prev_max_speed(engine, op.body, at_jd_tdb, &op.config)?),
        ),
        (
            MotionKind::MaxSpeed,
            MotionQuery::Range {
                start_jd_tdb,
                end_jd_tdb,
            },
        ) => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            Ok(MotionResult::MaxSpeedMany(search_max_speed(
                engine,
                op.body,
                start_jd_tdb,
                end_jd_tdb,
                &op.config,
            )?))
        }
    }
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

/// Lunar phase kind selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LunarPhaseKind {
    /// Amavasya / new moon events.
    Amavasya,
    /// Purnima / full moon events.
    Purnima,
}

/// Lunar phase search query variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LunarPhaseQuery {
    /// Find next event after `at_jd_tdb`.
    Next { at_jd_tdb: f64 },
    /// Find previous event before `at_jd_tdb`.
    Prev { at_jd_tdb: f64 },
    /// Find all events in `[start_jd_tdb, end_jd_tdb]`.
    Range { start_jd_tdb: f64, end_jd_tdb: f64 },
}

impl LunarPhaseQuery {
    /// Returns the mode represented by this query.
    pub fn mode(self) -> QueryMode {
        match self {
            Self::Next { .. } => QueryMode::Next,
            Self::Prev { .. } => QueryMode::Prev,
            Self::Range { .. } => QueryMode::Range,
        }
    }
}

/// Canonical lunar-phase operation request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LunarPhaseOperation {
    /// Which lunar phase family to query.
    pub kind: LunarPhaseKind,
    /// When set, events also carry sidereal longitudes and rashi indices
    /// computed with this configuration.
    pub sankranti_config: Option<SankrantiConfig>,
    /// Query selector and time bounds.
    pub query: LunarPhaseQuery,
}

/// Canonical lunar-phase operation response.
#[derive(Debug, Clone, PartialEq)]
pub enum LunarPhaseResult {
    /// Result for next/prev requests.
    Single(Option<LunarPhaseEvent>),
    /// Result for range requests.
    Many(Vec<LunarPhaseEvent>),
}

fn jd_tdb_to_utc(engine: &Engine, jd_tdb: f64) -> UtcTime {
    UtcTime::from_jd_tdb(jd_tdb, engine.lsk())
}

fn enrich_lunar_phase_event(
    engine: &Engine,
    event: &mut LunarPhaseEvent,
    sankranti_config: &SankrantiConfig,
) -> Result<(), SearchError> {
    let jd_tdb = crate::search_util::utc_to_jd_tdb(engine, &event.utc);
    let (sun_sid, sun_rashi) = sidereal_echo_at(
        engine,
        TransitBody::Body(Body::Sun),
        jd_tdb,
        sankranti_config,
    )?;
    let (moon_sid, moon_rashi) = sidereal_echo_at(
        engine,
        TransitBody::Body(Body::Moon),
        jd_tdb,
        sankranti_config,
    )?;
    event.sun_sidereal_longitude_deg = Some(sun_sid);
    event.moon_sidereal_longitude_deg = Some(moon_sid);
    event.sun_rashi_index = Some(sun_rashi);
    event.moon_rashi_index = Some(moon_rashi);
    Ok(())
}

/// Execute a lunar-phase operation request.
pub fn lunar_phase(
    engine: &Engine,
    op: &LunarPhaseOperation,
) -> Result<LunarPhaseResult, SearchError> {
    let mut result = lunar_phase_query(engine, op)?;
    if let Some(sc) = op.sankranti_config.as_ref() {
        match &mut result {
            LunarPhaseResult::Single(Some(ev)) => enrich_lunar_phase_event(engine, ev, sc)?,
            LunarPhaseResult::Many(events) => {
                for ev in events.iter_mut() {
                    enrich_lunar_phase_event(engine, ev, sc)?;
                }
            }
            LunarPhaseResult::Single(None) => {}
        }
    }
    Ok(result)
}

fn lunar_phase_query(
    engine: &Engine,
    op: &LunarPhaseOperation,
) -> Result<LunarPhaseResult, SearchError> {
    match (op.kind, op.query) {
        (LunarPhaseKind::Amavasya, LunarPhaseQuery::Next { at_jd_tdb }) => Ok(
            LunarPhaseResult::Single(next_amavasya(engine, &jd_tdb_to_utc(engine, at_jd_tdb))?),
        ),
        (LunarPhaseKind::Amavasya, LunarPhaseQuery::Prev { at_jd_tdb }) => Ok(
            LunarPhaseResult::Single(prev_amavasya(engine, &jd_tdb_to_utc(engine, at_jd_tdb))?),
        ),
        (
            LunarPhaseKind::Amavasya,
            LunarPhaseQuery::Range {
                start_jd_tdb,
                end_jd_tdb,
            },
        ) => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            Ok(LunarPhaseResult::Many(search_amavasyas(
                engine,
                &jd_tdb_to_utc(engine, start_jd_tdb),
                &jd_tdb_to_utc(engine, end_jd_tdb),
            )?))
        }
        (LunarPhaseKind::Purnima, LunarPhaseQuery::Next { at_jd_tdb }) => Ok(
            LunarPhaseResult::Single(next_purnima(engine, &jd_tdb_to_utc(engine, at_jd_tdb))?),
        ),
        (LunarPhaseKind::Purnima, LunarPhaseQuery::Prev { at_jd_tdb }) => Ok(
            LunarPhaseResult::Single(prev_purnima(engine, &jd_tdb_to_utc(engine, at_jd_tdb))?),
        ),
        (
            LunarPhaseKind::Purnima,
            LunarPhaseQuery::Range {
                start_jd_tdb,
                end_jd_tdb,
            },
        ) => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            Ok(LunarPhaseResult::Many(search_purnimas(
                engine,
                &jd_tdb_to_utc(engine, start_jd_tdb),
                &jd_tdb_to_utc(engine, end_jd_tdb),
            )?))
        }
    }
}

/// Sankranti target selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SankrantiTarget {
    /// Any rashi entry.
    Any,
    /// Specific rashi entry.
    SpecificRashi(Rashi),
}

/// Sankranti search query variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SankrantiQuery {
    /// Find next event after `at_jd_tdb`.
    Next { at_jd_tdb: f64 },
    /// Find previous event before `at_jd_tdb`.
    Prev { at_jd_tdb: f64 },
    /// Find all events in `[start_jd_tdb, end_jd_tdb]`.
    Range { start_jd_tdb: f64, end_jd_tdb: f64 },
}

impl SankrantiQuery {
    /// Returns the mode represented by this query.
    pub fn mode(self) -> QueryMode {
        match self {
            Self::Next { .. } => QueryMode::Next,
            Self::Prev { .. } => QueryMode::Prev,
            Self::Range { .. } => QueryMode::Range,
        }
    }
}

/// Canonical sankranti / ingress operation request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SankrantiOperation {
    /// Body whose rashi ingresses are searched. Defaults to the Sun
    /// (classical sankranti); accepts any plain body or Rahu/Ketu.
    pub body: TransitBody,
    /// Which sankranti target family to query.
    pub target: SankrantiTarget,
    /// Search configuration.
    pub config: SankrantiConfig,
    /// Query selector and time bounds.
    pub query: SankrantiQuery,
}

/// Canonical sankranti operation response.
#[derive(Debug, Clone, PartialEq)]
pub enum SankrantiResult {
    /// Result for next/prev requests.
    Single(Option<SankrantiEvent>),
    /// Result for range requests.
    Many(Vec<SankrantiEvent>),
}

/// Execute a sankranti / ingress operation request.
pub fn sankranti(engine: &Engine, op: &SankrantiOperation) -> Result<SankrantiResult, SearchError> {
    match (op.target, op.query) {
        (SankrantiTarget::Any, SankrantiQuery::Next { at_jd_tdb }) => {
            Ok(SankrantiResult::Single(next_ingress(
                engine,
                op.body,
                &jd_tdb_to_utc(engine, at_jd_tdb),
                &op.config,
            )?))
        }
        (SankrantiTarget::Any, SankrantiQuery::Prev { at_jd_tdb }) => {
            Ok(SankrantiResult::Single(prev_ingress(
                engine,
                op.body,
                &jd_tdb_to_utc(engine, at_jd_tdb),
                &op.config,
            )?))
        }
        (
            SankrantiTarget::Any,
            SankrantiQuery::Range {
                start_jd_tdb,
                end_jd_tdb,
            },
        ) => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            Ok(SankrantiResult::Many(search_ingresses(
                engine,
                op.body,
                &jd_tdb_to_utc(engine, start_jd_tdb),
                &jd_tdb_to_utc(engine, end_jd_tdb),
                &op.config,
            )?))
        }
        (SankrantiTarget::SpecificRashi(rashi), SankrantiQuery::Next { at_jd_tdb }) => {
            Ok(SankrantiResult::Single(next_specific_ingress(
                engine,
                op.body,
                &jd_tdb_to_utc(engine, at_jd_tdb),
                rashi,
                &op.config,
            )?))
        }
        (SankrantiTarget::SpecificRashi(rashi), SankrantiQuery::Prev { at_jd_tdb }) => {
            Ok(SankrantiResult::Single(prev_specific_ingress(
                engine,
                op.body,
                &jd_tdb_to_utc(engine, at_jd_tdb),
                rashi,
                &op.config,
            )?))
        }
        (
            SankrantiTarget::SpecificRashi(rashi),
            SankrantiQuery::Range {
                start_jd_tdb,
                end_jd_tdb,
            },
        ) => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            let all = search_ingresses(
                engine,
                op.body,
                &jd_tdb_to_utc(engine, start_jd_tdb),
                &jd_tdb_to_utc(engine, end_jd_tdb),
                &op.config,
            )?;
            let filtered = all.into_iter().filter(|ev| ev.rashi == rashi).collect();
            Ok(SankrantiResult::Many(filtered))
        }
    }
}

/// Fixed-longitude search query variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FixedLongitudeQuery {
    /// Find next event after `at_jd_tdb`.
    Next { at_jd_tdb: f64 },
    /// Find previous event before `at_jd_tdb`.
    Prev { at_jd_tdb: f64 },
    /// Find all events in `[start_jd_tdb, end_jd_tdb]`.
    Range { start_jd_tdb: f64, end_jd_tdb: f64 },
}

impl FixedLongitudeQuery {
    /// Returns the mode represented by this query.
    pub fn mode(self) -> QueryMode {
        match self {
            Self::Next { .. } => QueryMode::Next,
            Self::Prev { .. } => QueryMode::Prev,
            Self::Range { .. } => QueryMode::Range,
        }
    }
}

/// Canonical fixed-longitude operation request: when does a moving body
/// reach a fixed sidereal longitude (plus an optional angle set).
#[derive(Debug, Clone, PartialEq)]
pub struct FixedLongitudeOperation {
    /// The moving body (plain body or Rahu/Ketu).
    pub body: TransitBody,
    /// Fixed target sidereal longitude on the configured frame, degrees.
    pub target_longitude_deg: f64,
    /// Angle offsets added to the target (mod 360); an event fires when
    /// the body reaches each offset longitude. Empty = `[0.0]`
    /// (conjunction only). Offsets are normalized to [0, 360).
    pub target_angles_deg: Vec<f64>,
    /// Additionally search the body's classical special-aspect angles
    /// (Mars 90/210, Jupiter 120/240, Saturn 60/270) applied so the
    /// MOVING body casts that aspect onto the target — i.e. offsets of
    /// `360 − angle`. No-op for bodies without special aspects.
    pub include_special_angles: bool,
    /// Longitude model (frame, ayanamsha, node mode) and numerical
    /// parameters (step / convergence / max iterations).
    pub config: SankrantiConfig,
    /// Query selector and time bounds.
    pub query: FixedLongitudeQuery,
}

/// Canonical fixed-longitude operation response.
#[derive(Debug, Clone, PartialEq)]
pub enum FixedLongitudeResult {
    /// Result for next/prev requests.
    Single(Option<crate::fixed_longitude::FixedLongitudeEvent>),
    /// Result for range requests.
    Many(Vec<crate::fixed_longitude::FixedLongitudeEvent>),
}

/// Execute a fixed-longitude operation request.
pub fn fixed_longitude(
    engine: &Engine,
    op: &FixedLongitudeOperation,
) -> Result<FixedLongitudeResult, SearchError> {
    let mut angles: Vec<f64> = if op.target_angles_deg.is_empty() {
        vec![0.0]
    } else {
        op.target_angles_deg.clone()
    };
    if op.include_special_angles
        && let Some(body) = op.body.body()
    {
        for &special in crate::gochar_events::special_angles_for_body(body) {
            angles.push((360.0 - special).rem_euclid(360.0));
        }
    }
    match op.query {
        FixedLongitudeQuery::Next { at_jd_tdb } => {
            Ok(FixedLongitudeResult::Single(next_fixed_longitude(
                engine,
                op.body,
                at_jd_tdb,
                op.target_longitude_deg,
                &angles,
                &op.config,
            )?))
        }
        FixedLongitudeQuery::Prev { at_jd_tdb } => {
            Ok(FixedLongitudeResult::Single(prev_fixed_longitude(
                engine,
                op.body,
                at_jd_tdb,
                op.target_longitude_deg,
                &angles,
                &op.config,
            )?))
        }
        FixedLongitudeQuery::Range {
            start_jd_tdb,
            end_jd_tdb,
        } => Ok(FixedLongitudeResult::Many(search_fixed_longitudes(
            engine,
            op.body,
            start_jd_tdb,
            end_jd_tdb,
            op.target_longitude_deg,
            &angles,
            &op.config,
        )?)),
    }
}

/// Include bit for Tithi in panchang operations.
pub const PANCHANG_INCLUDE_TITHI: u32 = 1 << 0;
/// Include bit for Karana in panchang operations.
pub const PANCHANG_INCLUDE_KARANA: u32 = 1 << 1;
/// Include bit for Yoga in panchang operations.
pub const PANCHANG_INCLUDE_YOGA: u32 = 1 << 2;
/// Include bit for Vaar in panchang operations.
pub const PANCHANG_INCLUDE_VAAR: u32 = 1 << 3;
/// Include bit for Hora in panchang operations.
pub const PANCHANG_INCLUDE_HORA: u32 = 1 << 4;
/// Include bit for Ghatika in panchang operations.
pub const PANCHANG_INCLUDE_GHATIKA: u32 = 1 << 5;
/// Include bit for Nakshatra in panchang operations.
pub const PANCHANG_INCLUDE_NAKSHATRA: u32 = 1 << 6;
/// Include bit for Masa in panchang operations.
pub const PANCHANG_INCLUDE_MASA: u32 = 1 << 7;
/// Include bit for Ayana in panchang operations.
pub const PANCHANG_INCLUDE_AYANA: u32 = 1 << 8;
/// Include bit for Varsha in panchang operations.
pub const PANCHANG_INCLUDE_VARSHA: u32 = 1 << 9;

/// Include mask containing all core daily panchang elements.
pub const PANCHANG_INCLUDE_ALL_CORE: u32 = PANCHANG_INCLUDE_TITHI
    | PANCHANG_INCLUDE_KARANA
    | PANCHANG_INCLUDE_YOGA
    | PANCHANG_INCLUDE_VAAR
    | PANCHANG_INCLUDE_HORA
    | PANCHANG_INCLUDE_GHATIKA
    | PANCHANG_INCLUDE_NAKSHATRA;

/// Include mask containing all calendar elements.
pub const PANCHANG_INCLUDE_ALL_CALENDAR: u32 =
    PANCHANG_INCLUDE_MASA | PANCHANG_INCLUDE_AYANA | PANCHANG_INCLUDE_VARSHA;

/// Include mask containing all panchang elements.
pub const PANCHANG_INCLUDE_ALL: u32 = PANCHANG_INCLUDE_ALL_CORE | PANCHANG_INCLUDE_ALL_CALENDAR;

/// All location-independent elements (no observer location required):
/// tithi, karana, yoga, nakshatra, masa, ayana, varsha.
pub const PANCHANG_INCLUDE_LOCATION_INDEPENDENT: u32 = PANCHANG_INCLUDE_TITHI
    | PANCHANG_INCLUDE_KARANA
    | PANCHANG_INCLUDE_YOGA
    | PANCHANG_INCLUDE_NAKSHATRA
    | PANCHANG_INCLUDE_MASA
    | PANCHANG_INCLUDE_AYANA
    | PANCHANG_INCLUDE_VARSHA;

/// All location-dependent (sunrise-anchored) elements: vaar, hora, ghatika.
pub const PANCHANG_INCLUDE_LOCATION_DEPENDENT: u32 =
    PANCHANG_INCLUDE_VAAR | PANCHANG_INCLUDE_HORA | PANCHANG_INCLUDE_GHATIKA;

/// Resolve a panchang element or group name to its include-mask bits.
///
/// Accepted names (case-insensitive): the ten element names (`tithi`,
/// `karana`, `yoga`, `nakshatra`, `vaar`, `hora`, `ghatika`, `masa`,
/// `ayana`, `varsha`) and the group names `all`, `all_core`, `all_calendar`,
/// `location_independent`, `location_dependent`. Returns `None` for unknown
/// names.
pub fn panchang_include_bits(name: &str) -> Option<u32> {
    match name.to_ascii_lowercase().as_str() {
        "tithi" => Some(PANCHANG_INCLUDE_TITHI),
        "karana" => Some(PANCHANG_INCLUDE_KARANA),
        "yoga" => Some(PANCHANG_INCLUDE_YOGA),
        "vaar" => Some(PANCHANG_INCLUDE_VAAR),
        "hora" => Some(PANCHANG_INCLUDE_HORA),
        "ghatika" => Some(PANCHANG_INCLUDE_GHATIKA),
        "nakshatra" => Some(PANCHANG_INCLUDE_NAKSHATRA),
        "masa" => Some(PANCHANG_INCLUDE_MASA),
        "ayana" => Some(PANCHANG_INCLUDE_AYANA),
        "varsha" => Some(PANCHANG_INCLUDE_VARSHA),
        "all" => Some(PANCHANG_INCLUDE_ALL),
        "all_core" => Some(PANCHANG_INCLUDE_ALL_CORE),
        "all_calendar" => Some(PANCHANG_INCLUDE_ALL_CALENDAR),
        "location_independent" => Some(PANCHANG_INCLUDE_LOCATION_INDEPENDENT),
        "location_dependent" => Some(PANCHANG_INCLUDE_LOCATION_DEPENDENT),
        _ => None,
    }
}

/// Canonical panchang operation request.
#[derive(Debug, Clone, PartialEq)]
pub struct PanchangOperation {
    /// Input timestamp in UTC.
    pub at_utc: UtcTime,
    /// Observer location. Required only when a location-dependent element
    /// (vaar, hora, ghatika) is selected in `include_mask`.
    pub location: Option<GeoLocation>,
    /// Sunrise/sunset model configuration.
    pub riseset_config: RiseSetConfig,
    /// Ayanamsha/search configuration.
    pub sankranti_config: SankrantiConfig,
    /// Include mask with `PANCHANG_INCLUDE_*` bits.
    pub include_mask: u32,
    /// Caller-supplied precomputed calendar elements, reused when their
    /// validity window contains `at_utc` (see [`PanchangPrecomputed`]).
    pub known: PanchangPrecomputed,
}

pub use crate::panchang_types::{PanchangPrecomputed, PanchangResult};

/// Execute a panchang operation request.
///
/// Only elements selected in `include_mask` are computed; see
/// [`panchang_for_date`] for the sharing/skipping semantics.
pub fn panchang(
    engine: &Engine,
    eop: &EopKernel,
    op: &PanchangOperation,
) -> Result<PanchangResult, SearchError> {
    panchang_for_date(
        engine,
        eop,
        &op.at_utc,
        op.location.as_ref(),
        &op.riseset_config,
        &op.sankranti_config,
        op.include_mask,
        &op.known,
    )
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
    fn conjunction_query_mode_is_stable() {
        assert_eq!(
            ConjunctionQuery::Next { at_jd_tdb: 0.0 }.mode(),
            QueryMode::Next
        );
        assert_eq!(
            ConjunctionQuery::Prev { at_jd_tdb: 0.0 }.mode(),
            QueryMode::Prev
        );
        assert_eq!(
            ConjunctionQuery::Range {
                start_jd_tdb: 0.0,
                end_jd_tdb: 1.0
            }
            .mode(),
            QueryMode::Range
        );
    }

    #[test]
    fn grahan_query_mode_is_stable() {
        assert_eq!(GrahanQuery::Next { at_jd_tdb: 0.0 }.mode(), QueryMode::Next);
        assert_eq!(GrahanQuery::Prev { at_jd_tdb: 0.0 }.mode(), QueryMode::Prev);
        assert_eq!(
            GrahanQuery::Range {
                start_jd_tdb: 0.0,
                end_jd_tdb: 1.0
            }
            .mode(),
            QueryMode::Range
        );
    }

    #[test]
    fn motion_query_mode_is_stable() {
        assert_eq!(MotionQuery::Next { at_jd_tdb: 0.0 }.mode(), QueryMode::Next);
        assert_eq!(MotionQuery::Prev { at_jd_tdb: 0.0 }.mode(), QueryMode::Prev);
        assert_eq!(
            MotionQuery::Range {
                start_jd_tdb: 0.0,
                end_jd_tdb: 1.0
            }
            .mode(),
            QueryMode::Range
        );
    }

    #[test]
    fn lunar_phase_query_mode_is_stable() {
        assert_eq!(
            LunarPhaseQuery::Next { at_jd_tdb: 0.0 }.mode(),
            QueryMode::Next
        );
        assert_eq!(
            LunarPhaseQuery::Prev { at_jd_tdb: 0.0 }.mode(),
            QueryMode::Prev
        );
        assert_eq!(
            LunarPhaseQuery::Range {
                start_jd_tdb: 0.0,
                end_jd_tdb: 1.0
            }
            .mode(),
            QueryMode::Range
        );
    }

    #[test]
    fn sankranti_query_mode_is_stable() {
        assert_eq!(
            SankrantiQuery::Next { at_jd_tdb: 0.0 }.mode(),
            QueryMode::Next
        );
        assert_eq!(
            SankrantiQuery::Prev { at_jd_tdb: 0.0 }.mode(),
            QueryMode::Prev
        );
        assert_eq!(
            SankrantiQuery::Range {
                start_jd_tdb: 0.0,
                end_jd_tdb: 1.0
            }
            .mode(),
            QueryMode::Range
        );
    }

    #[test]
    fn fixed_longitude_query_mode_is_stable() {
        assert_eq!(
            FixedLongitudeQuery::Next { at_jd_tdb: 0.0 }.mode(),
            QueryMode::Next
        );
        assert_eq!(
            FixedLongitudeQuery::Prev { at_jd_tdb: 0.0 }.mode(),
            QueryMode::Prev
        );
        assert_eq!(
            FixedLongitudeQuery::Range {
                start_jd_tdb: 0.0,
                end_jd_tdb: 1.0
            }
            .mode(),
            QueryMode::Range
        );
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

    #[test]
    fn tara_output_kind_is_stable() {
        assert_eq!(TaraOutputKind::Equatorial, TaraOutputKind::Equatorial);
        assert_eq!(TaraOutputKind::Ecliptic, TaraOutputKind::Ecliptic);
        assert_eq!(TaraOutputKind::Sidereal, TaraOutputKind::Sidereal);
    }
}
