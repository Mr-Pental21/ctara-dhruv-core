//! Canonical operation-style APIs shared across wrappers and frontends.
//!
//! This module is the migration layer from split function surfaces
//! (`next_*`, `prev_*`, `search_*`) to config-driven operation requests.

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
use crate::grahan_types::{ChandraGrahan, GrahanConfig, SuryaGrahan};
use crate::lunar_phase_types::LunarPhaseEvent;
use crate::panchang_types::{
    AyanaInfo, GhatikaInfo, HoraInfo, KaranaInfo, MasaInfo, PanchangNakshatraInfo, TithiInfo,
    VaarInfo, VarshaInfo, YogaInfo,
};
use crate::sankranti_types::{SankrantiConfig, SankrantiEvent};
use crate::stationary_types::{MaxSpeedEvent, StationaryConfig, StationaryEvent};
use crate::{
    ayana_for_date, ghatika_for_date, hora_for_date, karana_for_date, masa_for_date,
    nakshatra_for_date, next_amavasya, next_chandra_grahan, next_conjunction, next_max_speed,
    next_purnima, next_sankranti, next_specific_sankranti, next_stationary, next_surya_grahan,
    panchang_for_date, prev_amavasya, prev_chandra_grahan, prev_conjunction, prev_max_speed,
    prev_purnima, prev_sankranti, prev_specific_sankranti, prev_stationary, prev_surya_grahan,
    search_amavasyas, search_chandra_grahan, search_conjunctions, search_max_speed,
    search_purnimas, search_sankrantis, search_stationary, search_surya_grahan, tithi_for_date,
    vaar_for_date, varsha_for_date, yoga_for_date,
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConjunctionOperation {
    /// First body.
    pub body1: Body,
    /// Second body.
    pub body2: Body,
    /// Numerical search configuration.
    pub config: ConjunctionConfig,
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

/// Execute a conjunction operation request.
pub fn conjunction(
    engine: &Engine,
    op: &ConjunctionOperation,
) -> Result<ConjunctionResult, SearchError> {
    match op.query {
        ConjunctionQuery::Next { at_jd_tdb } => Ok(ConjunctionResult::Single(next_conjunction(
            engine, op.body1, op.body2, at_jd_tdb, &op.config,
        )?)),
        ConjunctionQuery::Prev { at_jd_tdb } => Ok(ConjunctionResult::Single(prev_conjunction(
            engine, op.body1, op.body2, at_jd_tdb, &op.config,
        )?)),
        ConjunctionQuery::Range {
            start_jd_tdb,
            end_jd_tdb,
        } => {
            if end_jd_tdb <= start_jd_tdb {
                return Err(SearchError::InvalidConfig(
                    "end_jd_tdb must be greater than start_jd_tdb",
                ));
            }
            Ok(ConjunctionResult::Many(search_conjunctions(
                engine,
                op.body1,
                op.body2,
                start_jd_tdb,
                end_jd_tdb,
                &op.config,
            )?))
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrahanOperation {
    /// Which grahan family to query.
    pub kind: GrahanKind,
    /// Search configuration.
    pub config: GrahanConfig,
    /// Query selector and time bounds.
    pub query: GrahanQuery,
}

/// Canonical grahan operation response.
#[derive(Debug, Clone, PartialEq)]
pub enum GrahanResult {
    /// Single chandra result (next/prev).
    ChandraSingle(Option<ChandraGrahan>),
    /// Chandra range result.
    ChandraMany(Vec<ChandraGrahan>),
    /// Single surya result (next/prev).
    SuryaSingle(Option<SuryaGrahan>),
    /// Surya range result.
    SuryaMany(Vec<SuryaGrahan>),
}

/// Execute a grahan operation request.
pub fn grahan(engine: &Engine, op: &GrahanOperation) -> Result<GrahanResult, SearchError> {
    match (op.kind, op.query) {
        (GrahanKind::Chandra, GrahanQuery::Next { at_jd_tdb }) => Ok(GrahanResult::ChandraSingle(
            next_chandra_grahan(engine, at_jd_tdb, &op.config)?,
        )),
        (GrahanKind::Chandra, GrahanQuery::Prev { at_jd_tdb }) => Ok(GrahanResult::ChandraSingle(
            prev_chandra_grahan(engine, at_jd_tdb, &op.config)?,
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
            next_surya_grahan(engine, at_jd_tdb, &op.config)?,
        )),
        (GrahanKind::Surya, GrahanQuery::Prev { at_jd_tdb }) => Ok(GrahanResult::SuryaSingle(
            prev_surya_grahan(engine, at_jd_tdb, &op.config)?,
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
                start_jd_tdb,
                end_jd_tdb,
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
    /// Body to search for.
    pub body: Body,
    /// Motion event family.
    pub kind: MotionKind,
    /// Search configuration.
    pub config: StationaryConfig,
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

/// Execute a motion operation request.
pub fn motion(engine: &Engine, op: &MotionOperation) -> Result<MotionResult, SearchError> {
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

/// Execute a lunar-phase operation request.
pub fn lunar_phase(
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

/// Canonical sankranti operation request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SankrantiOperation {
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

/// Execute a sankranti operation request.
pub fn sankranti(engine: &Engine, op: &SankrantiOperation) -> Result<SankrantiResult, SearchError> {
    match (op.target, op.query) {
        (SankrantiTarget::Any, SankrantiQuery::Next { at_jd_tdb }) => Ok(SankrantiResult::Single(
            next_sankranti(engine, &jd_tdb_to_utc(engine, at_jd_tdb), &op.config)?,
        )),
        (SankrantiTarget::Any, SankrantiQuery::Prev { at_jd_tdb }) => Ok(SankrantiResult::Single(
            prev_sankranti(engine, &jd_tdb_to_utc(engine, at_jd_tdb), &op.config)?,
        )),
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
            Ok(SankrantiResult::Many(search_sankrantis(
                engine,
                &jd_tdb_to_utc(engine, start_jd_tdb),
                &jd_tdb_to_utc(engine, end_jd_tdb),
                &op.config,
            )?))
        }
        (SankrantiTarget::SpecificRashi(rashi), SankrantiQuery::Next { at_jd_tdb }) => {
            Ok(SankrantiResult::Single(next_specific_sankranti(
                engine,
                &jd_tdb_to_utc(engine, at_jd_tdb),
                rashi,
                &op.config,
            )?))
        }
        (SankrantiTarget::SpecificRashi(rashi), SankrantiQuery::Prev { at_jd_tdb }) => {
            Ok(SankrantiResult::Single(prev_specific_sankranti(
                engine,
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
            let all = search_sankrantis(
                engine,
                &jd_tdb_to_utc(engine, start_jd_tdb),
                &jd_tdb_to_utc(engine, end_jd_tdb),
                &op.config,
            )?;
            let filtered = all.into_iter().filter(|ev| ev.rashi == rashi).collect();
            Ok(SankrantiResult::Many(filtered))
        }
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

/// Canonical panchang operation request.
#[derive(Debug, Clone, PartialEq)]
pub struct PanchangOperation {
    /// Input timestamp in UTC.
    pub at_utc: UtcTime,
    /// Observer location.
    pub location: GeoLocation,
    /// Sunrise/sunset model configuration.
    pub riseset_config: RiseSetConfig,
    /// Ayanamsha/search configuration.
    pub sankranti_config: SankrantiConfig,
    /// Include mask with `PANCHANG_INCLUDE_*` bits.
    pub include_mask: u32,
}

/// Canonical panchang operation response.
#[derive(Debug, Clone, PartialEq)]
pub struct PanchangResult {
    pub tithi: Option<TithiInfo>,
    pub karana: Option<KaranaInfo>,
    pub yoga: Option<YogaInfo>,
    pub vaar: Option<VaarInfo>,
    pub hora: Option<HoraInfo>,
    pub ghatika: Option<GhatikaInfo>,
    pub nakshatra: Option<PanchangNakshatraInfo>,
    pub masa: Option<MasaInfo>,
    pub ayana: Option<AyanaInfo>,
    pub varsha: Option<VarshaInfo>,
}

fn include(mask: u32, bit: u32) -> bool {
    (mask & bit) != 0
}

/// Execute a panchang operation request.
pub fn panchang(
    engine: &Engine,
    eop: &EopKernel,
    op: &PanchangOperation,
) -> Result<PanchangResult, SearchError> {
    if op.include_mask == 0 {
        return Err(SearchError::InvalidConfig("include_mask must be non-zero"));
    }

    let mut result = PanchangResult {
        tithi: None,
        karana: None,
        yoga: None,
        vaar: None,
        hora: None,
        ghatika: None,
        nakshatra: None,
        masa: None,
        ayana: None,
        varsha: None,
    };

    let any_core = (op.include_mask & PANCHANG_INCLUDE_ALL_CORE) != 0;
    let any_calendar = (op.include_mask & PANCHANG_INCLUDE_ALL_CALENDAR) != 0;

    if any_core {
        let full = panchang_for_date(
            engine,
            eop,
            &op.at_utc,
            &op.location,
            &op.riseset_config,
            &op.sankranti_config,
            any_calendar,
        )?;
        if include(op.include_mask, PANCHANG_INCLUDE_TITHI) {
            result.tithi = Some(full.tithi);
        }
        if include(op.include_mask, PANCHANG_INCLUDE_KARANA) {
            result.karana = Some(full.karana);
        }
        if include(op.include_mask, PANCHANG_INCLUDE_YOGA) {
            result.yoga = Some(full.yoga);
        }
        if include(op.include_mask, PANCHANG_INCLUDE_VAAR) {
            result.vaar = Some(full.vaar);
        }
        if include(op.include_mask, PANCHANG_INCLUDE_HORA) {
            result.hora = Some(full.hora);
        }
        if include(op.include_mask, PANCHANG_INCLUDE_GHATIKA) {
            result.ghatika = Some(full.ghatika);
        }
        if include(op.include_mask, PANCHANG_INCLUDE_NAKSHATRA) {
            result.nakshatra = Some(full.nakshatra);
        }
        if include(op.include_mask, PANCHANG_INCLUDE_MASA) {
            result.masa = full.masa;
        }
        if include(op.include_mask, PANCHANG_INCLUDE_AYANA) {
            result.ayana = full.ayana;
        }
        if include(op.include_mask, PANCHANG_INCLUDE_VARSHA) {
            result.varsha = full.varsha;
        }
        return Ok(result);
    }

    if include(op.include_mask, PANCHANG_INCLUDE_MASA) {
        result.masa = Some(masa_for_date(engine, &op.at_utc, &op.sankranti_config)?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_AYANA) {
        result.ayana = Some(ayana_for_date(engine, &op.at_utc, &op.sankranti_config)?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_VARSHA) {
        result.varsha = Some(varsha_for_date(engine, &op.at_utc, &op.sankranti_config)?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_TITHI) {
        result.tithi = Some(tithi_for_date(engine, &op.at_utc)?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_KARANA) {
        result.karana = Some(karana_for_date(engine, &op.at_utc)?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_YOGA) {
        result.yoga = Some(yoga_for_date(engine, &op.at_utc, &op.sankranti_config)?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_NAKSHATRA) {
        result.nakshatra = Some(nakshatra_for_date(
            engine,
            &op.at_utc,
            &op.sankranti_config,
        )?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_VAAR) {
        result.vaar = Some(vaar_for_date(
            engine,
            eop,
            &op.at_utc,
            &op.location,
            &op.riseset_config,
        )?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_HORA) {
        result.hora = Some(hora_for_date(
            engine,
            eop,
            &op.at_utc,
            &op.location,
            &op.riseset_config,
        )?);
    }
    if include(op.include_mask, PANCHANG_INCLUDE_GHATIKA) {
        result.ghatika = Some(ghatika_for_date(
            engine,
            eop,
            &op.at_utc,
            &op.location,
            &op.riseset_config,
        )?);
    }
    Ok(result)
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
