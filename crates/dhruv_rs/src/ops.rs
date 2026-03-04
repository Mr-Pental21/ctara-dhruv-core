//! Canonical operation-style APIs for `dhruv_rs`.
//!
//! This layer provides unified request/response models and maps them to
//! `dhruv_search::operations`.

use dhruv_core::Body;
use dhruv_search::{
    AyanamshaMode, AyanamshaOperation, ConjunctionConfig, ConjunctionOperation, ConjunctionQuery,
    ConjunctionResult, GrahanConfig, GrahanKind, GrahanOperation, GrahanQuery, GrahanResult,
    LunarPhaseKind, LunarPhaseOperation, LunarPhaseQuery, LunarPhaseResult, MotionKind,
    MotionOperation, MotionQuery, MotionResult, NodeBackend, NodeOperation, PanchangOperation,
    PanchangResult, SankrantiConfig, SankrantiOperation, SankrantiQuery, SankrantiResult,
    SankrantiTarget, StationaryConfig, TaraOperation, TaraOutputKind, TaraResult,
};
use dhruv_tara::{EarthState, TaraCatalog, TaraConfig, TaraId};
use dhruv_time::{EopKernel, UtcTime, calendar_to_jd, jd_to_tdb_seconds, tdb_seconds_to_jd};
use dhruv_vedic_base::{AyanamshaSystem, GeoLocation, LunarNode, NodeMode, RiseSetConfig};

use crate::date::UtcDate;
use crate::error::DhruvError;
use crate::global::{engine, time_conversion_policy};

/// Time input accepted by operation requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeInput {
    /// UTC calendar timestamp.
    Utc(UtcDate),
    /// Julian Date in TDB.
    JdTdb(f64),
}

/// Query selector for conjunction operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConjunctionRequestQuery {
    /// Find next event after `at`.
    Next { at: TimeInput },
    /// Find previous event before `at`.
    Prev { at: TimeInput },
    /// Find all events in `[start, end]`.
    Range { start: TimeInput, end: TimeInput },
}

/// Unified conjunction request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConjunctionRequest {
    pub body1: Body,
    pub body2: Body,
    pub config: ConjunctionConfig,
    pub query: ConjunctionRequestQuery,
}

fn utc_to_jd_tdb_for_engine(eng: &dhruv_core::Engine, date: UtcDate) -> f64 {
    let day_frac =
        date.day as f64 + date.hour as f64 / 24.0 + date.min as f64 / 1440.0 + date.sec / 86_400.0;
    let jd_utc = calendar_to_jd(date.year, date.month, day_frac);
    let utc_seconds = jd_to_tdb_seconds(jd_utc);
    let out = eng
        .lsk()
        .utc_to_tdb_with_policy_and_eop(utc_seconds, None, time_conversion_policy());
    tdb_seconds_to_jd(out.tdb_seconds)
}

fn time_input_to_jd_tdb(eng: &dhruv_core::Engine, input: TimeInput) -> f64 {
    match input {
        TimeInput::Utc(date) => utc_to_jd_tdb_for_engine(eng, date),
        TimeInput::JdTdb(jd) => jd,
    }
}

fn time_input_to_utc_for_engine(eng: &dhruv_core::Engine, input: TimeInput) -> UtcTime {
    match input {
        TimeInput::Utc(date) => date.into(),
        TimeInput::JdTdb(jd) => UtcTime::from_jd_tdb(jd, eng.lsk()),
    }
}

/// Execute a unified conjunction operation.
pub fn conjunction(request: &ConjunctionRequest) -> Result<ConjunctionResult, DhruvError> {
    let eng = engine()?;
    let query = match request.query {
        ConjunctionRequestQuery::Next { at } => ConjunctionQuery::Next {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        ConjunctionRequestQuery::Prev { at } => ConjunctionQuery::Prev {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        ConjunctionRequestQuery::Range { start, end } => ConjunctionQuery::Range {
            start_jd_tdb: time_input_to_jd_tdb(eng, start),
            end_jd_tdb: time_input_to_jd_tdb(eng, end),
        },
    };
    let op = ConjunctionOperation {
        body1: request.body1,
        body2: request.body2,
        config: request.config,
        query,
    };
    Ok(dhruv_search::conjunction(eng, &op)?)
}

/// Query selector for grahan operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GrahanRequestQuery {
    /// Find next event after `at`.
    Next { at: TimeInput },
    /// Find previous event before `at`.
    Prev { at: TimeInput },
    /// Find all events in `[start, end]`.
    Range { start: TimeInput, end: TimeInput },
}

/// Unified grahan request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrahanRequest {
    pub kind: GrahanKind,
    pub config: GrahanConfig,
    pub query: GrahanRequestQuery,
}

/// Execute a unified grahan operation.
pub fn grahan(request: &GrahanRequest) -> Result<GrahanResult, DhruvError> {
    let eng = engine()?;
    let query = match request.query {
        GrahanRequestQuery::Next { at } => GrahanQuery::Next {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        GrahanRequestQuery::Prev { at } => GrahanQuery::Prev {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        GrahanRequestQuery::Range { start, end } => GrahanQuery::Range {
            start_jd_tdb: time_input_to_jd_tdb(eng, start),
            end_jd_tdb: time_input_to_jd_tdb(eng, end),
        },
    };
    let op = GrahanOperation {
        kind: request.kind,
        config: request.config,
        query,
    };
    Ok(dhruv_search::grahan(eng, &op)?)
}

/// Query selector for motion operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionRequestQuery {
    /// Find next event after `at`.
    Next { at: TimeInput },
    /// Find previous event before `at`.
    Prev { at: TimeInput },
    /// Find all events in `[start, end]`.
    Range { start: TimeInput, end: TimeInput },
}

/// Unified motion request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionRequest {
    pub body: Body,
    pub kind: MotionKind,
    pub config: StationaryConfig,
    pub query: MotionRequestQuery,
}

/// Execute a unified motion operation.
pub fn motion(request: &MotionRequest) -> Result<MotionResult, DhruvError> {
    let eng = engine()?;
    let query = match request.query {
        MotionRequestQuery::Next { at } => MotionQuery::Next {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        MotionRequestQuery::Prev { at } => MotionQuery::Prev {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        MotionRequestQuery::Range { start, end } => MotionQuery::Range {
            start_jd_tdb: time_input_to_jd_tdb(eng, start),
            end_jd_tdb: time_input_to_jd_tdb(eng, end),
        },
    };
    let op = MotionOperation {
        body: request.body,
        kind: request.kind,
        config: request.config,
        query,
    };
    Ok(dhruv_search::motion(eng, &op)?)
}

/// Query selector for lunar-phase operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LunarPhaseRequestQuery {
    /// Find next event after `at`.
    Next { at: TimeInput },
    /// Find previous event before `at`.
    Prev { at: TimeInput },
    /// Find all events in `[start, end]`.
    Range { start: TimeInput, end: TimeInput },
}

/// Unified lunar-phase request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LunarPhaseRequest {
    pub kind: LunarPhaseKind,
    pub query: LunarPhaseRequestQuery,
}

/// Execute a unified lunar-phase operation.
pub fn lunar_phase(request: &LunarPhaseRequest) -> Result<LunarPhaseResult, DhruvError> {
    let eng = engine()?;
    let query = match request.query {
        LunarPhaseRequestQuery::Next { at } => LunarPhaseQuery::Next {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        LunarPhaseRequestQuery::Prev { at } => LunarPhaseQuery::Prev {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        LunarPhaseRequestQuery::Range { start, end } => LunarPhaseQuery::Range {
            start_jd_tdb: time_input_to_jd_tdb(eng, start),
            end_jd_tdb: time_input_to_jd_tdb(eng, end),
        },
    };
    let op = LunarPhaseOperation {
        kind: request.kind,
        query,
    };
    Ok(dhruv_search::lunar_phase(eng, &op)?)
}

/// Query selector for sankranti operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SankrantiRequestQuery {
    /// Find next event after `at`.
    Next { at: TimeInput },
    /// Find previous event before `at`.
    Prev { at: TimeInput },
    /// Find all events in `[start, end]`.
    Range { start: TimeInput, end: TimeInput },
}

/// Unified sankranti request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SankrantiRequest {
    pub target: SankrantiTarget,
    pub config: SankrantiConfig,
    pub query: SankrantiRequestQuery,
}

/// Execute a unified sankranti operation.
pub fn sankranti(request: &SankrantiRequest) -> Result<SankrantiResult, DhruvError> {
    let eng = engine()?;
    let query = match request.query {
        SankrantiRequestQuery::Next { at } => SankrantiQuery::Next {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        SankrantiRequestQuery::Prev { at } => SankrantiQuery::Prev {
            at_jd_tdb: time_input_to_jd_tdb(eng, at),
        },
        SankrantiRequestQuery::Range { start, end } => SankrantiQuery::Range {
            start_jd_tdb: time_input_to_jd_tdb(eng, start),
            end_jd_tdb: time_input_to_jd_tdb(eng, end),
        },
    };
    let op = SankrantiOperation {
        target: request.target,
        config: request.config,
        query,
    };
    Ok(dhruv_search::sankranti(eng, &op)?)
}

/// Ayanamsha request mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AyanamshaRequestMode {
    /// Mean ayanamsha.
    Mean,
    /// True ayanamsha from explicit delta-psi arcseconds.
    True { delta_psi_arcsec: f64 },
    /// Unified ayanamsha with `use_nutation`.
    Unified { use_nutation: bool },
}

/// Unified ayanamsha request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AyanamshaRequest {
    pub system: AyanamshaSystem,
    pub at: TimeInput,
    pub mode: AyanamshaRequestMode,
}

/// Execute a unified ayanamsha operation.
pub fn ayanamsha_op(request: &AyanamshaRequest) -> Result<f64, DhruvError> {
    let eng = engine()?;
    let (mode, use_nutation, delta_psi_arcsec) = match request.mode {
        AyanamshaRequestMode::Mean => (AyanamshaMode::Mean, false, 0.0),
        AyanamshaRequestMode::True { delta_psi_arcsec } => {
            (AyanamshaMode::True, false, delta_psi_arcsec)
        }
        AyanamshaRequestMode::Unified { use_nutation } => {
            (AyanamshaMode::Unified, use_nutation, 0.0)
        }
    };
    let op = AyanamshaOperation {
        system: request.system,
        mode,
        at_jd_tdb: time_input_to_jd_tdb(eng, request.at),
        use_nutation,
        delta_psi_arcsec,
    };
    Ok(dhruv_search::ayanamsha(&op)?)
}

/// Unified lunar-node request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeRequest {
    pub node: LunarNode,
    pub mode: NodeMode,
    pub backend: NodeBackend,
    pub at: TimeInput,
}

/// Execute a unified lunar-node operation.
pub fn lunar_node_op(request: &NodeRequest) -> Result<f64, DhruvError> {
    let eng = engine()?;
    let op = NodeOperation {
        node: request.node,
        mode: request.mode,
        backend: request.backend,
        at_jd_tdb: time_input_to_jd_tdb(eng, request.at),
    };
    Ok(dhruv_search::lunar_node(eng, &op)?)
}

/// Unified panchang request.
#[derive(Debug, Clone, PartialEq)]
pub struct PanchangRequest {
    pub at: TimeInput,
    pub location: GeoLocation,
    pub riseset_config: RiseSetConfig,
    pub sankranti_config: SankrantiConfig,
    pub include_mask: u32,
}

/// Execute a unified panchang operation.
pub fn panchang_op(
    request: &PanchangRequest,
    eop: &EopKernel,
) -> Result<PanchangResult, DhruvError> {
    let eng = engine()?;
    let op = PanchangOperation {
        at_utc: time_input_to_utc_for_engine(eng, request.at),
        location: request.location,
        riseset_config: request.riseset_config,
        sankranti_config: request.sankranti_config,
        include_mask: request.include_mask,
    };
    Ok(dhruv_search::panchang(eng, eop, &op)?)
}

/// Unified tara request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaraRequest {
    pub star: TaraId,
    pub output: TaraOutputKind,
    pub at: TimeInput,
    pub ayanamsha_deg: f64,
    pub config: TaraConfig,
    pub earth_state: Option<EarthState>,
}

/// Execute a unified tara operation.
pub fn tara_op(catalog: &TaraCatalog, request: &TaraRequest) -> Result<TaraResult, DhruvError> {
    let eng = engine()?;
    let op = TaraOperation {
        star: request.star,
        output: request.output,
        at_jd_tdb: time_input_to_jd_tdb(eng, request.at),
        ayanamsha_deg: request.ayanamsha_deg,
        config: request.config,
        earth_state: request.earth_state,
    };
    Ok(dhruv_search::tara(catalog, &op)?)
}
