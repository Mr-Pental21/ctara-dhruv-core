#![recursion_limit = "256"]

use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use dhruv_config::{ConfigResolver, DefaultsMode, load_with_discovery};
use dhruv_core::{Body, Engine, EngineConfig, Frame, Observer, Query, StateVector};
use dhruv_frames::{
    PrecessionModel, ReferencePlane, cartesian_state_to_spherical_state, cartesian_to_spherical,
    nutation_iau2000b,
};
use dhruv_search::ConjunctionConfig;
use dhruv_search::operations::{
    AyanamshaMode, AyanamshaOperation, ConjunctionOperation, ConjunctionQuery, ConjunctionResult,
    GrahanKind, GrahanOperation, GrahanQuery, GrahanResult, LunarPhaseKind, LunarPhaseOperation,
    LunarPhaseQuery, LunarPhaseResult, MotionKind, MotionOperation, MotionQuery, MotionResult,
    NodeBackend, NodeOperation, PanchangOperation, PanchangResult, SankrantiOperation,
    SankrantiQuery, SankrantiResult, SankrantiTarget, TaraOperation, TaraOutputKind, TaraResult,
};
use dhruv_search::{
    GocharEventsConfig, GocharEventsOperation, GocharEventsResult, GocharReference,
    NatalTargetKind, NatalTargetLongitude, PANCHANG_INCLUDE_ALL_CORE,
    PANCHANG_INCLUDE_LOCATION_INDEPENDENT, SankrantiConfig, StationaryConfig, TajakaReturnBasis,
    TajakaReturnEvent, TithiPraveshaEvent, TransitAspectKind, TransitAspectOwner, TransitBody,
    TransitToNatalAspectEvent, amsha_lagna_events, ayanamsha, body_ecliptic_lon_lat,
    charakaraka_events, conjunction, dasha_child_period_for_birth, dasha_child_period_with_inputs,
    dasha_children_for_birth, dasha_children_with_inputs, dasha_complete_level_for_birth,
    dasha_complete_level_with_inputs, dasha_hierarchy_for_birth, dasha_hierarchy_with_inputs,
    dasha_level0_entity_for_birth, dasha_level0_entity_with_inputs, dasha_level0_for_birth,
    dasha_level0_with_inputs, dasha_snapshot_at, dasha_snapshot_with_inputs, elongation_at,
    full_kundali_for_date, ghatika_from_sunrises, gochar_events, graha_longitudes,
    hora_from_sunrises, karana_at, lunar_node, motion, nakshatra_at, next_charakaraka_event,
    panchang, panchang_events, panchang_include_bits, prev_charakaraka_event,
    set_time_conversion_policy, sidereal_sum_at, tara as tara_op, tithi_at, vaar_from_sunrises,
    vedic_day_sunrises, yoga_at,
};
use dhruv_search::{
    GrahaLongitudeKind, GrahaLongitudesConfig, all_upagrahas_for_date,
    all_upagrahas_for_date_with_config, amsha_charts_for_date, amsha_series, arudha_padas_for_date,
    ashtakavarga_for_date, avastha_for_date, balas_for_date, bhavabala_for_date,
    charakaraka_for_date, core_bindus, drishti_for_date, graha_positions as graha_positions_fn,
    moving_osculating_apogees_for_date, shadbala_for_date, sidereal_bhavas_for_date,
    sidereal_lagna_for_date, sidereal_mc_for_date, special_lagnas_for_date, vimsopaka_for_date,
};
use dhruv_tara::apparent::{apply_aberration, apply_light_deflection};
use dhruv_tara::galactic::galactic_anticenter_icrs;
use dhruv_tara::propagation::{EquatorialPosition, propagate_position};
use dhruv_tara::{TaraAccuracy, TaraCatalog, TaraConfig, TaraId};
use dhruv_time::{
    DeltaTModel, EopKernel, FutureDeltaTTransition, SmhFutureParabolaFamily, TimeConversionOptions,
    TimeConversionPolicy, TimeDiagnostics, TimeWarning, TtUtcSource, UtcTime, jd_to_tdb_seconds,
    tdb_seconds_to_jd,
};
use dhruv_vedic_base::amsha::amsha_rashi_infos;
use dhruv_vedic_base::bhava_types::ALL_BHAVA_SYSTEMS;
use dhruv_vedic_base::bhava_types::SayanadiGhatikaRounding;
use dhruv_vedic_base::combustion::{
    all_combustion_status as all_combustion_status_fn,
    combustion_threshold as combustion_threshold_fn, is_combust as is_combust_fn,
};
use dhruv_vedic_base::dasha::yogini_name;
use dhruv_vedic_base::dasha::{
    ALL_DASHA_SYSTEMS, DashaEntity, DashaHierarchy, DashaLevel, DashaPeriod, DashaSnapshot,
    DashaSystem, DashaVariationConfig, RashiDashaInputs, SubPeriodMethod, YoginiScheme,
};
use dhruv_vedic_base::drishti::{
    DrishtiEntry, GrahaDrishtiMatrix, graha_drishti, graha_drishti_matrix,
};
use dhruv_vedic_base::ghatika::ghatika_from_elapsed;
use dhruv_vedic_base::graha_relationships::{
    BeneficNature, ChandraBeneficRule, NaisargikaMaitri, TatkalikaMaitri, debilitation_degree,
    dignity_in_rashi, dignity_in_rashi_with_positions, exaltation_degree, graha_gender, hora_lord,
    masa_lord, moolatrikone_range, moon_benefic_nature, naisargika_maitri, natural_benefic_malefic,
    node_dignity_in_rashi, panchadha_maitri, samvatsara_lord, tatkalika_maitri,
};
use dhruv_vedic_base::riseset::{approximate_local_noon_jd, compute_all_events, compute_rise_set};
use dhruv_vedic_base::riseset_types::{
    GeoLocation, RiseSetConfig, RiseSetEvent, RiseSetResult, SunLimb,
};
use dhruv_vedic_base::special_lagna::ghatikas_since_sunrise;
use dhruv_vedic_base::sphuta::{ALL_SPHUTAS, SphutalInputs, all_sphutas};
use dhruv_vedic_base::{
    ALL_GRAHAS, ALL_MASAS, ALL_NAKSHATRAS_27, ALL_NAKSHATRAS_28, ALL_RASHIS, ALL_SAMVATSARAS,
    ALL_UPAGRAHAS, ALL_VAARS, Amsha, AmshaRequest, AyanamshaSystem, BhavaConfig,
    BhavaReferenceMode, BhavaResult, BhavaStartingPoint, BhavaSystem, CharakarakaResult,
    CharakarakaScheme, Graha, GulikaMaandiPlanet, LunarNode, Nakshatra28Info, NodeDignityPolicy,
    NodeMode, RashiInfo, SunBasedUpagrahas, TimeUpagrahaConfig, TimeUpagrahaPoint, Upagraha,
    amsha_variation_catalog, amsha_variation_info, compute_bhavas, default_amsha_variation,
    is_valid_amsha_variation, lagna_longitude_rad, mc_longitude_rad, nakshatra_from_longitude,
    nakshatra_from_tropical, nakshatra28_from_longitude, nakshatra28_from_tropical, ramc_rad,
    rashi_from_longitude, rashi_from_tropical, sun_based_upagrahas, time_upagraha_jd,
};
use dhruv_vedic_base::{
    calculate_all_bav, calculate_ashtakavarga, calculate_bav, calculate_sav, ekadhipatya_sodhana,
    trikona_sodhana,
};
use rustler::{Encoder, Env, ResourceArc, Term};
use serde::Deserialize;
use serde_json::{Value, json};

mod atoms {
    rustler::atoms! {
        ok,
        error
    }
}

type JsonResult = Result<Value, Value>;

#[derive(Debug)]
struct EngineResource {
    state: RwLock<EngineState>,
}

impl rustler::Resource for EngineResource {}

#[derive(Debug)]
struct EngineState {
    engine: Option<Engine>,
    resolver: Option<ConfigResolver>,
    eop: Option<EopKernel>,
    time_policy: TimeConversionPolicy,
    tara_catalog: Arc<TaraCatalog>,
}

impl EngineState {
    fn new(engine: Engine, time_policy: TimeConversionPolicy) -> Self {
        Self {
            engine: Some(engine),
            resolver: None,
            eop: None,
            time_policy,
            tara_catalog: Arc::new(TaraCatalog::embedded().clone()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum EnumInput {
    Int(i64),
    Str(String),
}

#[derive(Debug, Clone, Deserialize)]
struct EngineConfigInput {
    spk_paths: Vec<String>,
    lsk_path: String,
    cache_capacity: Option<usize>,
    strict_validation: Option<bool>,
    time_policy: Option<TimePolicyInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct PathInput {
    path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SpkPathsInput {
    spk_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigLoadInput {
    path: Option<String>,
    defaults_mode: Option<EnumInput>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct UtcInput {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct GeoLocationInput {
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: Option<f64>,
}

/// Caller-supplied precomputed masa input. Mirrors the map emitted for masa
/// results so a previous `daily`/`events` value can be fed back verbatim.
#[derive(Debug, Clone, Deserialize)]
struct KnownMasaInput {
    masa: EnumInput,
    adhika: bool,
    start: UtcInput,
    end: UtcInput,
}

/// Caller-supplied precomputed ayana input (same shape as ayana results).
#[derive(Debug, Clone, Deserialize)]
struct KnownAyanaInput {
    ayana: EnumInput,
    start: UtcInput,
    end: UtcInput,
}

/// Caller-supplied precomputed varsha input (same shape as varsha results).
#[derive(Debug, Clone, Deserialize)]
struct KnownVarshaInput {
    samvatsara: EnumInput,
    order: u8,
    start: UtcInput,
    end: UtcInput,
}

/// Panchang include-mask input: an integer mask, an element/group name
/// (e.g. `"all"`, `"location_independent"`, `"tithi"`, `"none"`), or a list
/// of element/group names OR-ed together.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum PanchangIncludeInput {
    Int(u32),
    Str(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
struct QueryInput {
    target: EnumInput,
    observer: EnumInput,
    frame: Option<EnumInput>,
    epoch_tdb_jd: Option<f64>,
    utc: Option<UtcInput>,
    output: Option<EnumInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct BodyLonLatInput {
    body: EnumInput,
    jd_tdb: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct CartesianInput {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct TimeRunInput {
    op: String,
    utc: Option<UtcInput>,
    jd_tdb: Option<f64>,
    time_policy: Option<TimePolicyInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct TimePolicyInput {
    mode: EnumInput,
    options: Option<TimePolicyOptionsInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct TimePolicyOptionsInput {
    warn_on_fallback: Option<bool>,
    delta_t_model: Option<EnumInput>,
    freeze_future_dut1: Option<bool>,
    pre_range_dut1: Option<f64>,
    future_delta_t_transition: Option<EnumInput>,
    future_transition_years: Option<f64>,
    smh_future_family: Option<EnumInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct VedicRequest {
    op: String,
    jd_tdb: Option<f64>,
    utc: Option<UtcInput>,
    location: Option<GeoLocationInput>,
    event: Option<EnumInput>,
    system: Option<EnumInput>,
    mode: Option<EnumInput>,
    backend: Option<EnumInput>,
    config: Option<RiseSetConfigInput>,
    bhava_config: Option<BhavaConfigInput>,
    sankranti_config: Option<SankrantiConfigInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct PanchangRequest {
    op: String,
    utc: Option<UtcInput>,
    from_utc: Option<UtcInput>,
    to_utc: Option<UtcInput>,
    max_events: Option<u32>,
    jd_tdb: Option<f64>,
    query_jd: Option<f64>,
    sunrise_jd: Option<f64>,
    next_sunrise_jd: Option<f64>,
    moon_sidereal_deg: Option<f64>,
    body: Option<EnumInput>,
    location: Option<GeoLocationInput>,
    include_mask: Option<PanchangIncludeInput>,
    known_masa: Option<KnownMasaInput>,
    known_ayana: Option<KnownAyanaInput>,
    known_varsha: Option<KnownVarshaInput>,
    riseset_config: Option<RiseSetConfigInput>,
    sankranti_config: Option<SankrantiConfigInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct SearchRequest {
    op: String,
    mode: EnumInput,
    body1: Option<EnumInput>,
    body2: Option<EnumInput>,
    body: Option<EnumInput>,
    kind: Option<EnumInput>,
    target: Option<EnumInput>,
    at_jd_tdb: Option<f64>,
    start_jd_tdb: Option<f64>,
    end_jd_tdb: Option<f64>,
    at_utc: Option<UtcInput>,
    birth_utc: Option<UtcInput>,
    start_utc: Option<UtcInput>,
    end_utc: Option<UtcInput>,
    location: Option<GeoLocationInput>,
    transit_bodies: Option<Vec<EnumInput>>,
    natal_targets: Option<Vec<GocharNatalTargetInput>>,
    bhava_config: Option<BhavaConfigInput>,
    riseset_config: Option<RiseSetConfigInput>,
    config: Option<SearchConfigInput>,
    sankranti_config: Option<SankrantiConfigInput>,
    kundali_config: Option<FullKundaliConfigInput>,
    gochar_config: Option<GocharEventsConfigInput>,
    charakaraka_config: Option<CharakarakaConfigInput>,
    max_events: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct CharakarakaConfigInput {
    scheme: Option<EnumInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct GocharNatalTargetInput {
    kind: EnumInput,
    index: u8,
    name: Option<String>,
    longitude_deg: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct GocharEventsConfigInput {
    tajaka_return_basis: Option<EnumInput>,
    yearly_count: Option<u32>,
    monthly_count: Option<u32>,
    transit_window_days: Option<f64>,
    include_return_charts: Option<bool>,
    solar_step_size_days: Option<f64>,
    lunar_step_size_days: Option<f64>,
    solar_convergence_days: Option<f64>,
    lunar_convergence_days: Option<f64>,
    max_iterations: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct JyotishRequest {
    op: String,
    jd_tdb: Option<f64>,
    utc: Option<UtcInput>,
    from_utc: Option<UtcInput>,
    to_utc: Option<UtcInput>,
    step_minutes: Option<u32>,
    include_grahas: Option<bool>,
    max_segments: Option<u32>,
    location: Option<GeoLocationInput>,
    kind: Option<EnumInput>,
    system: Option<EnumInput>,
    grahas: Option<Vec<EnumInput>>,
    scheme: Option<EnumInput>,
    node_dignity_policy: Option<EnumInput>,
    upagraha_config: Option<TimeUpagrahaConfigInput>,
    graha_positions_config: Option<GrahaPositionsConfigInput>,
    bindus_config: Option<BindusConfigInput>,
    drishti_config: Option<DrishtiConfigInput>,
    bhava_config: Option<BhavaConfigInput>,
    riseset_config: Option<RiseSetConfigInput>,
    sankranti_config: Option<SankrantiConfigInput>,
    full_kundali_config: Option<FullKundaliConfigInput>,
    amsha_selection: Option<Vec<AmshaRequestInput>>,
    amsha_requests: Option<Vec<AmshaRequestInput>>,
    amsha_scope: Option<AmshaChartScopeInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct DashaRequest {
    op: String,
    birth_utc: Option<UtcInput>,
    birth_jd: Option<f64>,
    query_utc: Option<UtcInput>,
    query_jd: Option<f64>,
    location: Option<GeoLocationInput>,
    system: EnumInput,
    max_level: Option<u8>,
    bhava_config: Option<BhavaConfigInput>,
    riseset_config: Option<RiseSetConfigInput>,
    sankranti_config: Option<SankrantiConfigInput>,
    variation: Option<DashaVariationInput>,
    inputs: Option<DashaInputsInput>,
    entity: Option<DashaEntityInput>,
    parent: Option<DashaPeriodInput>,
    child_entity: Option<DashaEntityInput>,
    child_level: Option<EnumInput>,
    parent_periods: Option<Vec<DashaPeriodInput>>,
}

#[derive(Debug, Clone, Deserialize)]
struct DashaEntityInput {
    kind: EnumInput,
    index: u8,
}

#[derive(Debug, Clone, Deserialize)]
struct DashaPeriodInput {
    entity: DashaEntityInput,
    start_jd: f64,
    end_jd: f64,
    level: EnumInput,
    order: u16,
    parent_idx: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct TaraRequest {
    op: String,
    star: Option<EnumInput>,
    output: Option<EnumInput>,
    jd_tdb: Option<f64>,
    ayanamsha_deg: Option<f64>,
    config: Option<TaraConfigInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct RiseSetConfigInput {
    use_refraction: Option<bool>,
    altitude_correction: Option<bool>,
    sun_limb: Option<EnumInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct BhavaConfigInput {
    system: Option<EnumInput>,
    reference_mode: Option<EnumInput>,
    use_rashi_bhava_for_bala_avastha: Option<bool>,
    include_node_aspects_for_drik_bala: Option<bool>,
    include_special_bhavabala_rules: Option<bool>,
    divide_guru_buddh_drishti_by_4_for_drik_bala: Option<bool>,
    chandra_benefic_rule: Option<EnumInput>,
    sayanadi_ghatika_rounding: Option<EnumInput>,
    include_rashi_bhava_results: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct SankrantiConfigInput {
    ayanamsha_system: Option<EnumInput>,
    use_nutation: Option<bool>,
    precession_model: Option<EnumInput>,
    reference_plane: Option<EnumInput>,
    node_mode: Option<EnumInput>,
    step_size_days: Option<f64>,
    max_iterations: Option<u32>,
    convergence_days: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct SearchConfigInput {
    target_separation_deg: Option<f64>,
    target_separations_deg: Option<Vec<f64>>,
    node_mode: Option<EnumInput>,
    step_size_days: Option<f64>,
    max_iterations: Option<u32>,
    convergence_days: Option<f64>,
    include_penumbral: Option<bool>,
    include_peak_details: Option<bool>,
    include_path: Option<bool>,
    path_step_minutes: Option<u32>,
    boundary_step_deg: Option<u32>,
    include_local_grid: Option<bool>,
    local_grid_step_deg: Option<f64>,
    include_isolines: Option<bool>,
    duration_isoline_fractions: Option<Vec<f64>>,
    magnitude_isoline_levels: Option<Vec<f64>>,
    include_central_corridor: Option<bool>,
    include_contact_footprints: Option<bool>,
    include_umbra_footprints: Option<bool>,
    instantaneous_magnitude_levels: Option<Vec<f64>>,
    numerical_step_days: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct BasicStatesConfigInput {
    include_basic_states: Option<bool>,
    include_sensitive_point_distances: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct GrahaPositionsConfigInput {
    include_nakshatra: Option<bool>,
    include_lagna: Option<bool>,
    include_outer_planets: Option<bool>,
    include_bhava: Option<bool>,
    include_equatorial: Option<bool>,
    basic_states_config: Option<BasicStatesConfigInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct BindusConfigInput {
    include_nakshatra: Option<bool>,
    include_bhava: Option<bool>,
    upagraha_config: Option<TimeUpagrahaConfigInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct TimeUpagrahaConfigInput {
    gulika_point: Option<EnumInput>,
    maandi_point: Option<EnumInput>,
    other_point: Option<EnumInput>,
    gulika_planet: Option<EnumInput>,
    maandi_planet: Option<EnumInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct DrishtiConfigInput {
    include_bhava: Option<bool>,
    include_lagna: Option<bool>,
    include_bindus: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct FullKundaliConfigInput {
    include_bhava_cusps: Option<bool>,
    include_graha_positions: Option<bool>,
    include_bindus: Option<bool>,
    include_drishti: Option<bool>,
    include_ashtakavarga: Option<bool>,
    include_upagrahas: Option<bool>,
    include_sphutas: Option<bool>,
    include_special_lagnas: Option<bool>,
    include_amshas: Option<bool>,
    include_shadbala: Option<bool>,
    include_bhavabala: Option<bool>,
    include_vimsopaka: Option<bool>,
    include_avastha: Option<bool>,
    include_charakaraka: Option<bool>,
    panchang_include_mask: Option<PanchangIncludeInput>,
    include_dasha: Option<bool>,
    charakaraka_scheme: Option<EnumInput>,
    node_dignity_policy: Option<EnumInput>,
    upagraha_config: Option<TimeUpagrahaConfigInput>,
    graha_positions_config: Option<GrahaPositionsConfigInput>,
    bindus_config: Option<BindusConfigInput>,
    drishti_config: Option<DrishtiConfigInput>,
    amsha_scope: Option<AmshaChartScopeInput>,
    amsha_selection: Option<Vec<AmshaRequestInput>>,
    dasha_config: Option<DashaSelectionConfigInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct AmshaChartScopeInput {
    include_bhava_cusps: Option<bool>,
    include_arudha_padas: Option<bool>,
    include_upagrahas: Option<bool>,
    include_sphutas: Option<bool>,
    include_special_lagnas: Option<bool>,
    include_outer_planets: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct DashaSelectionConfigInput {
    systems: Option<Vec<EnumInput>>,
    max_level: Option<u8>,
    max_levels: Option<Vec<u8>>,
    cycles: Option<u8>,
    min_span_years: Option<f64>,
    snapshot_utc: Option<UtcInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct AmshaRequestInput {
    code: u16,
    variation: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
struct DashaVariationInput {
    level_methods: Option<Vec<EnumInput>>,
    yogini_scheme: Option<EnumInput>,
    use_abhijit: Option<bool>,
    cycles: Option<u8>,
    min_span_years: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct DashaInputsInput {
    moon_sid_lon: Option<f64>,
    graha_sidereal_lons: Option<Vec<f64>>,
    lagna_sidereal_lon: Option<f64>,
    sunrise_sunset: Option<(f64, f64)>,
    sunrise_jd: Option<f64>,
    sunset_jd: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct TaraConfigInput {
    accuracy: Option<EnumInput>,
    apply_parallax: Option<bool>,
}

const BODY_VARIANTS: [Body; 11] = [
    Body::Sun,
    Body::Mercury,
    Body::Venus,
    Body::Earth,
    Body::Moon,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
    Body::Pluto,
];
const FRAME_VARIANTS: [Frame; 2] = [Frame::IcrfJ2000, Frame::EclipticJ2000];
const LUNAR_NODE_VARIANTS: [LunarNode; 2] = [LunarNode::Rahu, LunarNode::Ketu];
const NODE_MODE_VARIANTS: [NodeMode; 2] = [NodeMode::Mean, NodeMode::True];
const NODE_BACKEND_VARIANTS: [NodeBackend; 2] = [NodeBackend::Analytic, NodeBackend::Engine];
const AYANAMSHA_MODE_VARIANTS: [AyanamshaMode; 3] = [
    AyanamshaMode::Mean,
    AyanamshaMode::True,
    AyanamshaMode::Unified,
];
const RISESET_EVENT_VARIANTS: [RiseSetEvent; 8] = [
    RiseSetEvent::Sunrise,
    RiseSetEvent::Sunset,
    RiseSetEvent::CivilDawn,
    RiseSetEvent::CivilDusk,
    RiseSetEvent::NauticalDawn,
    RiseSetEvent::NauticalDusk,
    RiseSetEvent::AstronomicalDawn,
    RiseSetEvent::AstronomicalDusk,
];
const SUN_LIMB_VARIANTS: [SunLimb; 3] = [SunLimb::UpperLimb, SunLimb::Center, SunLimb::LowerLimb];
const REFERENCE_PLANE_VARIANTS: [ReferencePlane; 2] =
    [ReferencePlane::Ecliptic, ReferencePlane::Invariable];
const BHAVA_REFERENCE_MODE_VARIANTS: [BhavaReferenceMode; 2] = [
    BhavaReferenceMode::StartOfFirst,
    BhavaReferenceMode::MiddleOfFirst,
];
const GRAHAN_KIND_VARIANTS: [GrahanKind; 2] = [GrahanKind::Chandra, GrahanKind::Surya];
const MOTION_KIND_VARIANTS: [MotionKind; 2] = [MotionKind::Stationary, MotionKind::MaxSpeed];
const LUNAR_PHASE_KIND_VARIANTS: [LunarPhaseKind; 2] =
    [LunarPhaseKind::Amavasya, LunarPhaseKind::Purnima];
const TARA_OUTPUT_VARIANTS: [TaraOutputKind; 3] = [
    TaraOutputKind::Equatorial,
    TaraOutputKind::Ecliptic,
    TaraOutputKind::Sidereal,
];
const TARA_ACCURACY_VARIANTS: [TaraAccuracy; 2] =
    [TaraAccuracy::Astrometric, TaraAccuracy::Apparent];
const CHARAKARAKA_SCHEME_VARIANTS: [CharakarakaScheme; 4] = [
    CharakarakaScheme::Eight,
    CharakarakaScheme::SevenNoPitri,
    CharakarakaScheme::SevenPkMergedMk,
    CharakarakaScheme::MixedParashara,
];
const NODE_DIGNITY_POLICY_VARIANTS: [NodeDignityPolicy; 2] = [
    NodeDignityPolicy::SignLordBased,
    NodeDignityPolicy::AlwaysSama,
];
const CHANDRA_BENEFIC_RULE_VARIANTS: [ChandraBeneficRule; 2] = [
    ChandraBeneficRule::Brightness72,
    ChandraBeneficRule::Waxing180,
];
const SAYANADI_GHATIKA_ROUNDING_VARIANTS: [SayanadiGhatikaRounding; 2] = [
    SayanadiGhatikaRounding::Floor,
    SayanadiGhatikaRounding::Ceil,
];
const TIME_UPAGRAHA_POINT_VARIANTS: [TimeUpagrahaPoint; 3] = [
    TimeUpagrahaPoint::Start,
    TimeUpagrahaPoint::Middle,
    TimeUpagrahaPoint::End,
];
const GULIKA_MAANDI_PLANET_VARIANTS: [GulikaMaandiPlanet; 2] =
    [GulikaMaandiPlanet::Rahu, GulikaMaandiPlanet::Saturn];
const YOGINI_SCHEME_VARIANTS: [YoginiScheme; 2] =
    [YoginiScheme::Default, YoginiScheme::LaDeepanshuGiri];
const SUB_PERIOD_METHOD_VARIANTS: [SubPeriodMethod; 4] = [
    SubPeriodMethod::ProportionalFromParent,
    SubPeriodMethod::EqualFromNext,
    SubPeriodMethod::EqualFromSame,
    SubPeriodMethod::ProportionalFromNext,
];

fn load(env: Env, _info: Term) -> bool {
    env.register::<EngineResource>().is_ok()
}

fn decode_term<T: for<'de> Deserialize<'de>>(term: Term<'_>) -> Result<T, rustler::Error> {
    let value: Value = rustler::serde::from_term(term).map_err(rustler::Error::from)?;
    serde_json::from_value(value).map_err(|_| rustler::Error::BadArg)
}

fn encode_json<'a>(env: Env<'a>, result: JsonResult) -> Result<Term<'a>, rustler::Error> {
    rustler::serde::to_term(env, &result).map_err(Into::into)
}

fn read_state(
    resource: &ResourceArc<EngineResource>,
    f: impl FnOnce(&EngineState) -> JsonResult,
) -> JsonResult {
    let guard = resource
        .state
        .read()
        .map_err(|_| error_payload("internal_error", "engine lock poisoned"))?;
    f(&guard)
}

fn write_state(
    resource: &ResourceArc<EngineResource>,
    f: impl FnOnce(&mut EngineState) -> JsonResult,
) -> JsonResult {
    let mut guard = resource
        .state
        .write()
        .map_err(|_| error_payload("internal_error", "engine lock poisoned"))?;
    f(&mut guard)
}

fn require_engine(state: &EngineState) -> Result<&Engine, Value> {
    state
        .engine
        .as_ref()
        .ok_or_else(|| error_payload("closed_engine", "engine is closed"))
}

fn error_payload(kind: &str, message: impl Into<String>) -> Value {
    json!({
        "kind": kind,
        "message": message.into(),
        "details": {}
    })
}

fn map_error(kind: &str, err: impl ToString) -> Value {
    error_payload(kind, err.to_string())
}

fn camel_to_snake(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for (idx, ch) in input.chars().enumerate() {
        if ch.is_uppercase() && idx > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

fn debug_name<T: Debug>(value: T) -> String {
    camel_to_snake(&format!("{value:?}"))
}

const DELTA_T_MODEL_VARIANTS: [DeltaTModel; 2] = [
    DeltaTModel::LegacyEspenakMeeus2006,
    DeltaTModel::Smh2016WithPre720Quadratic,
];

const FUTURE_DELTA_T_TRANSITION_VARIANTS: [FutureDeltaTTransition; 2] = [
    FutureDeltaTTransition::LegacyTtUtcBlend,
    FutureDeltaTTransition::BridgeFromModernEndpoint,
];

const SMH_FUTURE_FAMILY_VARIANTS: [SmhFutureParabolaFamily; 6] = [
    SmhFutureParabolaFamily::Addendum2020Piecewise,
    SmhFutureParabolaFamily::ConstantCMinus20,
    SmhFutureParabolaFamily::ConstantCMinus17p52,
    SmhFutureParabolaFamily::ConstantCMinus15p32,
    SmhFutureParabolaFamily::Stephenson1997,
    SmhFutureParabolaFamily::Stephenson2016,
];

fn parse_named<T: Copy + Debug>(value: &str, variants: &[T]) -> Option<T> {
    let normalized = value.trim().to_ascii_lowercase();
    variants
        .iter()
        .copied()
        .find(|variant| debug_name(*variant) == normalized)
}

fn raw_required_enum(raw: &Value, key: &str) -> Result<EnumInput, rustler::Error> {
    raw.get(key)
        .cloned()
        .ok_or(rustler::Error::BadArg)
        .and_then(|value| serde_json::from_value(value).map_err(|_| rustler::Error::BadArg))
}

fn raw_optional_enum(raw: &Value, key: &str) -> Result<Option<EnumInput>, rustler::Error> {
    raw.get(key)
        .cloned()
        .map(|value| serde_json::from_value(value).map_err(|_| rustler::Error::BadArg))
        .transpose()
}

fn raw_required_f64(raw: &Value, key: &str) -> Result<f64, rustler::Error> {
    raw.get(key)
        .and_then(Value::as_f64)
        .ok_or(rustler::Error::BadArg)
}

fn raw_required_bool(raw: &Value, key: &str) -> Result<bool, rustler::Error> {
    raw.get(key)
        .and_then(Value::as_bool)
        .ok_or(rustler::Error::BadArg)
}

fn raw_required_u8(raw: &Value, key: &str) -> Result<u8, rustler::Error> {
    raw.get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .ok_or(rustler::Error::BadArg)
}

fn raw_f64_array<const N: usize>(raw: &Value, key: &str) -> Result<[f64; N], rustler::Error> {
    let values = raw
        .get(key)
        .and_then(Value::as_array)
        .ok_or(rustler::Error::BadArg)?;
    if values.len() != N {
        return Err(rustler::Error::BadArg);
    }
    let mut out = [0.0; N];
    for (idx, value) in values.iter().enumerate() {
        out[idx] = value.as_f64().ok_or(rustler::Error::BadArg)?;
    }
    Ok(out)
}

fn raw_bool_array<const N: usize>(raw: &Value, key: &str) -> Result<[bool; N], rustler::Error> {
    let values = raw
        .get(key)
        .and_then(Value::as_array)
        .ok_or(rustler::Error::BadArg)?;
    if values.len() != N {
        return Err(rustler::Error::BadArg);
    }
    let mut out = [false; N];
    for (idx, value) in values.iter().enumerate() {
        out[idx] = value.as_bool().ok_or(rustler::Error::BadArg)?;
    }
    Ok(out)
}

fn raw_u8_array<const N: usize>(raw: &Value, key: &str) -> Result<[u8; N], rustler::Error> {
    let values = raw
        .get(key)
        .and_then(Value::as_array)
        .ok_or(rustler::Error::BadArg)?;
    if values.len() != N {
        return Err(rustler::Error::BadArg);
    }
    let mut out = [0u8; N];
    for (idx, value) in values.iter().enumerate() {
        out[idx] = value
            .as_u64()
            .and_then(|entry| u8::try_from(entry).ok())
            .ok_or(rustler::Error::BadArg)?;
    }
    Ok(out)
}

fn parse_delta_t_model(input: &EnumInput) -> Result<DeltaTModel, Value> {
    match input {
        EnumInput::Int(index) => DELTA_T_MODEL_VARIANTS
            .get(*index as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown delta_t_model")),
        EnumInput::Str(value) => parse_named(value, &DELTA_T_MODEL_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown delta_t_model")),
    }
}

fn parse_future_delta_t_transition(input: &EnumInput) -> Result<FutureDeltaTTransition, Value> {
    match input {
        EnumInput::Int(index) => FUTURE_DELTA_T_TRANSITION_VARIANTS
            .get(*index as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown future_delta_t_transition")),
        EnumInput::Str(value) => parse_named(value, &FUTURE_DELTA_T_TRANSITION_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown future_delta_t_transition")),
    }
}

fn parse_smh_future_family(input: &EnumInput) -> Result<SmhFutureParabolaFamily, Value> {
    match input {
        EnumInput::Int(index) => SMH_FUTURE_FAMILY_VARIANTS
            .get(*index as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown smh_future_family")),
        EnumInput::Str(value) => parse_named(value, &SMH_FUTURE_FAMILY_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown smh_future_family")),
    }
}

fn parse_time_policy_input(input: Option<TimePolicyInput>) -> Result<TimeConversionPolicy, Value> {
    let request = input.unwrap_or(TimePolicyInput {
        mode: EnumInput::Str("hybrid_delta_t".to_string()),
        options: None,
    });
    match request.mode {
        EnumInput::Str(ref value) if value == "strict_lsk" => Ok(TimeConversionPolicy::StrictLsk),
        EnumInput::Int(0) => Ok(TimeConversionPolicy::StrictLsk),
        _ => {
            let options = request.options.unwrap_or(TimePolicyOptionsInput {
                warn_on_fallback: None,
                delta_t_model: None,
                freeze_future_dut1: None,
                pre_range_dut1: None,
                future_delta_t_transition: None,
                future_transition_years: None,
                smh_future_family: None,
            });
            Ok(TimeConversionPolicy::HybridDeltaT(TimeConversionOptions {
                warn_on_fallback: options.warn_on_fallback.unwrap_or(true),
                delta_t_model: options
                    .delta_t_model
                    .as_ref()
                    .map(parse_delta_t_model)
                    .transpose()?
                    .unwrap_or_default(),
                freeze_future_dut1: options.freeze_future_dut1.unwrap_or(true),
                pre_range_dut1: options.pre_range_dut1.unwrap_or(0.0),
                future_delta_t_transition: options
                    .future_delta_t_transition
                    .as_ref()
                    .map(parse_future_delta_t_transition)
                    .transpose()?
                    .unwrap_or_default(),
                future_transition_years: options.future_transition_years.unwrap_or(100.0),
                smh_future_family: options
                    .smh_future_family
                    .as_ref()
                    .map(parse_smh_future_family)
                    .transpose()?
                    .unwrap_or_default(),
            }))
        }
    }
}

fn time_warning_json(warning: &TimeWarning) -> Value {
    match warning {
        TimeWarning::LskFutureFrozen {
            utc_seconds,
            last_entry_utc_seconds,
            used_delta_at_seconds,
        } => json!({
            "kind": "lsk_future_frozen",
            "utc_seconds": utc_seconds,
            "last_entry_utc_seconds": last_entry_utc_seconds,
            "used_delta_at_seconds": used_delta_at_seconds
        }),
        TimeWarning::LskPreRangeFallback {
            utc_seconds,
            first_entry_utc_seconds,
        } => json!({
            "kind": "lsk_pre_range_fallback",
            "utc_seconds": utc_seconds,
            "first_entry_utc_seconds": first_entry_utc_seconds
        }),
        TimeWarning::EopFutureFrozen {
            mjd,
            last_entry_mjd,
            used_dut1_seconds,
        } => json!({
            "kind": "eop_future_frozen",
            "mjd": mjd,
            "last_entry_mjd": last_entry_mjd,
            "used_dut1_seconds": used_dut1_seconds
        }),
        TimeWarning::EopPreRangeFallback {
            mjd,
            first_entry_mjd,
            used_dut1_seconds,
        } => json!({
            "kind": "eop_pre_range_fallback",
            "mjd": mjd,
            "first_entry_mjd": first_entry_mjd,
            "used_dut1_seconds": used_dut1_seconds
        }),
        TimeWarning::DeltaTModelUsed {
            model,
            segment,
            assumed_dut1_seconds,
        } => json!({
            "kind": "delta_t_model_used",
            "delta_t_model": debug_name(*model),
            "delta_t_segment": debug_name(*segment),
            "used_dut1_seconds": assumed_dut1_seconds
        }),
    }
}

fn time_diagnostics_json(diagnostics: &TimeDiagnostics) -> Value {
    json!({
        "source": match diagnostics.source {
            TtUtcSource::LskDeltaAt => "lsk_delta_at",
            TtUtcSource::DeltaTModel => "delta_t_model"
        },
        "tt_minus_utc_s": diagnostics.tt_minus_utc_s,
        "warnings": diagnostics.warnings.iter().map(time_warning_json).collect::<Vec<_>>()
    })
}

fn dms_json(dms: dhruv_vedic_base::Dms) -> Value {
    json!({
        "degrees": dms.degrees,
        "minutes": dms.minutes,
        "seconds": dms.seconds
    })
}

fn rashi_info_json(info: RashiInfo) -> Value {
    json!({
        "rashi": debug_name(info.rashi),
        "rashi_index": info.rashi_index,
        "dms": dms_json(info.dms),
        "degrees_in_rashi": info.degrees_in_rashi
    })
}

fn nakshatra_info_json(info: dhruv_vedic_base::NakshatraInfo) -> Value {
    json!({
        "nakshatra": debug_name(info.nakshatra),
        "nakshatra_index": info.nakshatra_index,
        "pada": info.pada,
        "degrees_in_nakshatra": info.degrees_in_nakshatra,
        "degrees_in_pada": info.degrees_in_pada
    })
}

fn nakshatra28_info_json(info: Nakshatra28Info) -> Value {
    json!({
        "nakshatra": debug_name(info.nakshatra),
        "nakshatra_index": info.nakshatra_index,
        "pada": info.pada,
        "degrees_in_nakshatra": info.degrees_in_nakshatra
    })
}

fn drishti_entry_json(entry: DrishtiEntry) -> Value {
    json!({
        "angular_distance": entry.angular_distance,
        "base_virupa": entry.base_virupa,
        "special_virupa": entry.special_virupa,
        "total_virupa": entry.total_virupa
    })
}

fn graha_drishti_matrix_json(matrix: GrahaDrishtiMatrix) -> Value {
    json!({
        "entries": matrix
            .entries
            .iter()
            .map(|row| row.iter().map(|entry| drishti_entry_json(*entry)).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    })
}

fn sun_based_upagrahas_value_json(result: SunBasedUpagrahas) -> Value {
    json!({
        "dhooma": result.dhooma,
        "vyatipata": result.vyatipata,
        "parivesha": result.parivesha,
        "indra_chapa": result.indra_chapa,
        "upaketu": result.upaketu
    })
}

fn benefic_nature_json(value: BeneficNature) -> Value {
    json!(debug_name(value))
}

fn equatorial_position_json(position: EquatorialPosition) -> Value {
    json!({
        "ra_deg": position.ra_deg,
        "dec_deg": position.dec_deg,
        "distance_au": position.distance_au
    })
}

fn parse_body(input: &EnumInput) -> Result<Body, Value> {
    match input {
        EnumInput::Int(code) => Body::from_code(*code as i32)
            .ok_or_else(|| error_payload("invalid_request", "unknown body code")),
        EnumInput::Str(value) => parse_named(value, &BODY_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown body name")),
    }
}

fn parse_transit_body(input: &EnumInput) -> Result<TransitBody, Value> {
    match input {
        EnumInput::Int(code) => TransitBody::from_code(*code as i32)
            .ok_or_else(|| error_payload("invalid_request", "unknown transit body code")),
        EnumInput::Str(value) => {
            let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
            match normalized.as_str() {
                "rahu" => Ok(TransitBody::Rahu),
                "ketu" => Ok(TransitBody::Ketu),
                _ => parse_named(value, &BODY_VARIANTS)
                    .map(TransitBody::Body)
                    .ok_or_else(|| error_payload("invalid_request", "unknown transit body")),
            }
        }
    }
}

fn parse_observer(input: &EnumInput) -> Result<Observer, Value> {
    match input {
        EnumInput::Int(0) => Ok(Observer::SolarSystemBarycenter),
        EnumInput::Int(code) => Body::from_code(*code as i32)
            .map(Observer::Body)
            .ok_or_else(|| error_payload("invalid_request", "unknown observer code")),
        EnumInput::Str(value) => {
            if value.eq_ignore_ascii_case("solar_system_barycenter")
                || value.eq_ignore_ascii_case("ssb")
            {
                Ok(Observer::SolarSystemBarycenter)
            } else {
                parse_named(value, &BODY_VARIANTS)
                    .map(Observer::Body)
                    .ok_or_else(|| error_payload("invalid_request", "unknown observer"))
            }
        }
    }
}

fn parse_frame(input: &EnumInput) -> Result<Frame, Value> {
    match input {
        EnumInput::Int(code) => Frame::from_code(*code as i32)
            .ok_or_else(|| error_payload("invalid_request", "unknown frame")),
        EnumInput::Str(value) => parse_named(value, &FRAME_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown frame")),
    }
}

fn parse_ayanamsha_system(input: Option<&EnumInput>) -> Result<AyanamshaSystem, Value> {
    match input {
        None => Ok(AyanamshaSystem::Lahiri),
        Some(EnumInput::Int(value)) => AyanamshaSystem::all()
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown ayanamsha system")),
        Some(EnumInput::Str(value)) => parse_named(value, AyanamshaSystem::all())
            .ok_or_else(|| error_payload("invalid_request", "unknown ayanamsha system")),
    }
}

fn parse_precession_model(input: Option<&EnumInput>) -> Result<Option<PrecessionModel>, Value> {
    match input {
        None => Ok(None),
        Some(EnumInput::Int(0)) => Ok(Some(PrecessionModel::Newcomb1895)),
        Some(EnumInput::Int(1)) => Ok(Some(PrecessionModel::Lieske1977)),
        Some(EnumInput::Int(2)) => Ok(Some(PrecessionModel::Iau2006)),
        Some(EnumInput::Int(3)) => Ok(Some(PrecessionModel::Vondrak2011)),
        Some(EnumInput::Int(_)) => {
            Err(error_payload("invalid_request", "unknown precession model"))
        }
        Some(EnumInput::Str(value)) => match value.to_ascii_lowercase().as_str() {
            "newcomb1895" | "newcomb" => Ok(Some(PrecessionModel::Newcomb1895)),
            "lieske1977" | "lieske" => Ok(Some(PrecessionModel::Lieske1977)),
            "iau2006" => Ok(Some(PrecessionModel::Iau2006)),
            "vondrak2011" | "vondrak" => Ok(Some(PrecessionModel::Vondrak2011)),
            _ => Err(error_payload("invalid_request", "unknown precession model")),
        },
    }
}

fn parse_graha_longitude_kind(input: Option<&EnumInput>) -> Result<GrahaLongitudeKind, Value> {
    match input {
        None => Ok(GrahaLongitudeKind::Sidereal),
        Some(EnumInput::Int(0)) => Ok(GrahaLongitudeKind::Sidereal),
        Some(EnumInput::Int(1)) => Ok(GrahaLongitudeKind::Tropical),
        Some(EnumInput::Int(_)) => Err(error_payload(
            "invalid_request",
            "unknown graha longitude kind",
        )),
        Some(EnumInput::Str(value)) => match value.to_ascii_lowercase().as_str() {
            "sidereal" => Ok(GrahaLongitudeKind::Sidereal),
            "tropical" => Ok(GrahaLongitudeKind::Tropical),
            _ => Err(error_payload(
                "invalid_request",
                "unknown graha longitude kind",
            )),
        },
    }
}

fn parse_charakaraka_scheme(input: Option<&EnumInput>) -> Result<CharakarakaScheme, Value> {
    match input {
        None => Ok(CharakarakaScheme::default()),
        Some(EnumInput::Int(value)) => CharakarakaScheme::from_u8(*value as u8)
            .ok_or_else(|| error_payload("invalid_request", "unknown charakaraka scheme")),
        Some(EnumInput::Str(value)) => parse_named(value, &CHARAKARAKA_SCHEME_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown charakaraka scheme")),
    }
}

fn parse_node_dignity_policy(input: Option<&EnumInput>) -> Result<NodeDignityPolicy, Value> {
    match input {
        None => Ok(NodeDignityPolicy::default()),
        Some(EnumInput::Int(value)) => NODE_DIGNITY_POLICY_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown node dignity policy")),
        Some(EnumInput::Str(value)) => parse_named(value, &NODE_DIGNITY_POLICY_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown node dignity policy")),
    }
}

fn parse_chandra_benefic_rule(input: &EnumInput) -> Result<ChandraBeneficRule, Value> {
    match input {
        EnumInput::Int(value) => CHANDRA_BENEFIC_RULE_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown chandra benefic rule")),
        EnumInput::Str(value) => {
            let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
            match normalized.as_str() {
                "brightness-72" | "brightness72" | "phase-72" | "old-72" => {
                    Ok(ChandraBeneficRule::Brightness72)
                }
                "waxing-180" | "waxing180" | "current-180" => Ok(ChandraBeneficRule::Waxing180),
                _ => parse_named(value, &CHANDRA_BENEFIC_RULE_VARIANTS).ok_or_else(|| {
                    error_payload("invalid_request", "unknown chandra benefic rule")
                }),
            }
        }
    }
}

fn parse_sayanadi_ghatika_rounding(input: &EnumInput) -> Result<SayanadiGhatikaRounding, Value> {
    match input {
        EnumInput::Int(value) => SAYANADI_GHATIKA_ROUNDING_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown sayanadi ghatika rounding")),
        EnumInput::Str(value) => {
            let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
            match normalized.as_str() {
                "floor" | "completed" | "completed-ghatika" => Ok(SayanadiGhatikaRounding::Floor),
                "ceil" | "ceiling" | "current" | "current-ghatika" => {
                    Ok(SayanadiGhatikaRounding::Ceil)
                }
                _ => parse_named(value, &SAYANADI_GHATIKA_ROUNDING_VARIANTS).ok_or_else(|| {
                    error_payload("invalid_request", "unknown sayanadi ghatika rounding")
                }),
            }
        }
    }
}

fn parse_time_upagraha_point(
    input: Option<&EnumInput>,
    default: TimeUpagrahaPoint,
) -> Result<TimeUpagrahaPoint, Value> {
    match input {
        None => Ok(default),
        Some(EnumInput::Int(value)) => TIME_UPAGRAHA_POINT_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown time upagraha point")),
        Some(EnumInput::Str(value)) => parse_named(value, &TIME_UPAGRAHA_POINT_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown time upagraha point")),
    }
}

fn parse_gulika_maandi_planet(
    input: Option<&EnumInput>,
    default: GulikaMaandiPlanet,
) -> Result<GulikaMaandiPlanet, Value> {
    match input {
        None => Ok(default),
        Some(EnumInput::Int(value)) => GULIKA_MAANDI_PLANET_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown Gulika/Maandi planet")),
        Some(EnumInput::Str(value)) => parse_named(value, &GULIKA_MAANDI_PLANET_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown Gulika/Maandi planet")),
    }
}

fn apply_time_upagraha_config(
    config: &mut TimeUpagrahaConfig,
    input: Option<&TimeUpagrahaConfigInput>,
) -> Result<(), Value> {
    let Some(input) = input else {
        return Ok(());
    };
    config.gulika_point =
        parse_time_upagraha_point(input.gulika_point.as_ref(), config.gulika_point)?;
    config.maandi_point =
        parse_time_upagraha_point(input.maandi_point.as_ref(), config.maandi_point)?;
    config.other_point = parse_time_upagraha_point(input.other_point.as_ref(), config.other_point)?;
    config.gulika_planet =
        parse_gulika_maandi_planet(input.gulika_planet.as_ref(), config.gulika_planet)?;
    config.maandi_planet =
        parse_gulika_maandi_planet(input.maandi_planet.as_ref(), config.maandi_planet)?;
    Ok(())
}

fn parse_lunar_node(input: &EnumInput) -> Result<LunarNode, Value> {
    match input {
        EnumInput::Int(0) => Ok(LunarNode::Rahu),
        EnumInput::Int(1) => Ok(LunarNode::Ketu),
        EnumInput::Int(_) => Err(error_payload("invalid_request", "unknown lunar node")),
        EnumInput::Str(value) => parse_named(value, &LUNAR_NODE_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown lunar node")),
    }
}

fn parse_graha(input: &EnumInput) -> Result<Graha, Value> {
    match input {
        EnumInput::Int(value) => ALL_GRAHAS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown graha")),
        EnumInput::Str(value) => parse_named(value, &ALL_GRAHAS)
            .ok_or_else(|| error_payload("invalid_request", "unknown graha")),
    }
}

fn parse_vaar(input: &EnumInput) -> Result<dhruv_vedic_base::Vaar, Value> {
    match input {
        EnumInput::Int(value) => ALL_VAARS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown vaar")),
        EnumInput::Str(value) => parse_named(value, &ALL_VAARS)
            .ok_or_else(|| error_payload("invalid_request", "unknown vaar")),
    }
}

fn parse_masa(input: &EnumInput) -> Result<dhruv_vedic_base::Masa, Value> {
    match input {
        EnumInput::Int(value) => ALL_MASAS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown masa")),
        EnumInput::Str(value) => parse_named(value, &ALL_MASAS)
            .ok_or_else(|| error_payload("invalid_request", "unknown masa")),
    }
}

fn parse_samvatsara(input: &EnumInput) -> Result<dhruv_vedic_base::Samvatsara, Value> {
    match input {
        EnumInput::Int(value) => ALL_SAMVATSARAS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown samvatsara")),
        EnumInput::Str(value) => parse_named(value, &ALL_SAMVATSARAS)
            .ok_or_else(|| error_payload("invalid_request", "unknown samvatsara")),
    }
}

fn parse_ayana(input: &EnumInput) -> Result<dhruv_vedic_base::ayana_type::Ayana, Value> {
    use dhruv_vedic_base::ayana_type::ALL_AYANAS;
    match input {
        EnumInput::Int(value) => ALL_AYANAS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown ayana")),
        EnumInput::Str(value) => parse_named(value, &ALL_AYANAS)
            .ok_or_else(|| error_payload("invalid_request", "unknown ayana")),
    }
}

/// Decodes the optional `known_masa`/`known_ayana`/`known_varsha` request
/// fields into the engine's caller-supplied precomputed calendar elements.
/// Unknown enum names are rejected loudly; stale validity windows are handled
/// (silently recomputed) by the engine itself.
fn to_panchang_precomputed(
    request: &PanchangRequest,
) -> Result<dhruv_search::PanchangPrecomputed, Value> {
    let masa = request
        .known_masa
        .as_ref()
        .map(|input| -> Result<dhruv_search::MasaInfo, Value> {
            Ok(dhruv_search::MasaInfo {
                masa: parse_masa(&input.masa)?,
                adhika: input.adhika,
                start: parse_utc(input.start)?,
                end: parse_utc(input.end)?,
            })
        })
        .transpose()?;
    let ayana = request
        .known_ayana
        .as_ref()
        .map(|input| -> Result<dhruv_search::AyanaInfo, Value> {
            Ok(dhruv_search::AyanaInfo {
                ayana: parse_ayana(&input.ayana)?,
                start: parse_utc(input.start)?,
                end: parse_utc(input.end)?,
            })
        })
        .transpose()?;
    let varsha = request
        .known_varsha
        .as_ref()
        .map(|input| -> Result<dhruv_search::VarshaInfo, Value> {
            if !(1..=60).contains(&input.order) {
                return Err(error_payload(
                    "invalid_request",
                    "known_varsha order must be in 1..=60",
                ));
            }
            Ok(dhruv_search::VarshaInfo {
                samvatsara: parse_samvatsara(&input.samvatsara)?,
                order: input.order,
                start: parse_utc(input.start)?,
                end: parse_utc(input.end)?,
            })
        })
        .transpose()?;
    Ok(dhruv_search::PanchangPrecomputed {
        masa,
        ayana,
        varsha,
    })
}

fn parse_upagraha(input: &EnumInput) -> Result<Upagraha, Value> {
    match input {
        EnumInput::Int(value) => ALL_UPAGRAHAS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown upagraha")),
        EnumInput::Str(value) => parse_named(value, &ALL_UPAGRAHAS)
            .ok_or_else(|| error_payload("invalid_request", "unknown upagraha")),
    }
}

fn parse_node_mode(input: Option<&EnumInput>) -> Result<NodeMode, Value> {
    match input {
        None => Ok(NodeMode::True),
        Some(EnumInput::Int(0)) => Ok(NodeMode::Mean),
        Some(EnumInput::Int(1)) => Ok(NodeMode::True),
        Some(EnumInput::Int(_)) => Err(error_payload("invalid_request", "unknown node mode")),
        Some(EnumInput::Str(value)) => parse_named(value, &NODE_MODE_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown node mode")),
    }
}

fn parse_node_backend(input: Option<&EnumInput>) -> Result<NodeBackend, Value> {
    match input {
        None => Ok(NodeBackend::Engine),
        Some(EnumInput::Int(value)) => NODE_BACKEND_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown node backend")),
        Some(EnumInput::Str(value)) => parse_named(value, &NODE_BACKEND_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown node backend")),
    }
}

fn parse_riseset_event(input: &EnumInput) -> Result<RiseSetEvent, Value> {
    match input {
        EnumInput::Int(value) => RISESET_EVENT_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown rise/set event")),
        EnumInput::Str(value) => parse_named(value, &RISESET_EVENT_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown rise/set event")),
    }
}

fn parse_reference_plane(input: Option<&EnumInput>) -> Result<Option<ReferencePlane>, Value> {
    match input {
        None => Ok(None),
        Some(EnumInput::Int(value)) => REFERENCE_PLANE_VARIANTS
            .get(*value as usize)
            .copied()
            .map(Some)
            .ok_or_else(|| error_payload("invalid_request", "unknown reference plane")),
        Some(EnumInput::Str(value)) => parse_named(value, &REFERENCE_PLANE_VARIANTS)
            .map(Some)
            .ok_or_else(|| error_payload("invalid_request", "unknown reference plane")),
    }
}

fn parse_bhava_system(input: Option<&EnumInput>) -> Result<BhavaSystem, Value> {
    match input {
        None => Ok(BhavaConfig::default().system),
        Some(EnumInput::Int(value)) => ALL_BHAVA_SYSTEMS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown bhava system")),
        Some(EnumInput::Str(value)) => parse_named(value, &ALL_BHAVA_SYSTEMS)
            .ok_or_else(|| error_payload("invalid_request", "unknown bhava system")),
    }
}

fn parse_bhava_reference_mode(input: Option<&EnumInput>) -> Result<BhavaReferenceMode, Value> {
    match input {
        None => Ok(BhavaReferenceMode::default()),
        Some(EnumInput::Int(value)) => BHAVA_REFERENCE_MODE_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown bhava reference mode")),
        Some(EnumInput::Str(value)) => parse_named(value, &BHAVA_REFERENCE_MODE_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown bhava reference mode")),
    }
}

fn parse_motion_kind(input: &EnumInput) -> Result<MotionKind, Value> {
    match input {
        EnumInput::Int(value) => MOTION_KIND_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown motion kind")),
        EnumInput::Str(value) => parse_named(value, &MOTION_KIND_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown motion kind")),
    }
}

fn parse_grahan_kind(input: &EnumInput) -> Result<GrahanKind, Value> {
    match input {
        EnumInput::Int(value) => GRAHAN_KIND_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown grahan kind")),
        EnumInput::Str(value) => parse_named(value, &GRAHAN_KIND_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown grahan kind")),
    }
}

fn parse_lunar_phase_kind(input: &EnumInput) -> Result<LunarPhaseKind, Value> {
    match input {
        EnumInput::Int(value) => LUNAR_PHASE_KIND_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown lunar phase kind")),
        EnumInput::Str(value) => parse_named(value, &LUNAR_PHASE_KIND_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown lunar phase kind")),
    }
}

fn parse_dasha_system(input: &EnumInput) -> Result<DashaSystem, Value> {
    match input {
        EnumInput::Int(value) => DashaSystem::from_u8(*value as u8)
            .ok_or_else(|| error_payload("invalid_request", "unknown dasha system")),
        EnumInput::Str(value) => parse_named(value, &ALL_DASHA_SYSTEMS)
            .ok_or_else(|| error_payload("invalid_request", "unknown dasha system")),
    }
}

fn parse_dasha_level(input: &EnumInput) -> Result<DashaLevel, Value> {
    match input {
        EnumInput::Int(value) => DashaLevel::from_u8(*value as u8)
            .ok_or_else(|| error_payload("invalid_request", "unknown dasha level")),
        EnumInput::Str(value) => match value.to_ascii_lowercase().as_str() {
            "mahadasha" => Ok(DashaLevel::Mahadasha),
            "antardasha" => Ok(DashaLevel::Antardasha),
            "pratyantardasha" => Ok(DashaLevel::Pratyantardasha),
            "sookshmadasha" => Ok(DashaLevel::Sookshmadasha),
            "pranadasha" => Ok(DashaLevel::Pranadasha),
            _ => Err(error_payload("invalid_request", "unknown dasha level")),
        },
    }
}

fn parse_dasha_entity(input: &DashaEntityInput) -> Result<DashaEntity, Value> {
    match &input.kind {
        EnumInput::Int(0) => ALL_GRAHAS
            .get(input.index as usize)
            .copied()
            .map(DashaEntity::Graha)
            .ok_or_else(|| error_payload("invalid_request", "unknown dasha graha")),
        EnumInput::Int(1) => {
            if input.index < 12 {
                Ok(DashaEntity::Rashi(input.index))
            } else {
                Err(error_payload("invalid_request", "unknown dasha rashi"))
            }
        }
        EnumInput::Int(2) => {
            if input.index < 8 {
                Ok(DashaEntity::Yogini(input.index))
            } else {
                Err(error_payload("invalid_request", "unknown dasha yogini"))
            }
        }
        EnumInput::Str(value) => match value.to_ascii_lowercase().as_str() {
            "graha" => ALL_GRAHAS
                .get(input.index as usize)
                .copied()
                .map(DashaEntity::Graha)
                .ok_or_else(|| error_payload("invalid_request", "unknown dasha graha")),
            "rashi" => {
                if input.index < 12 {
                    Ok(DashaEntity::Rashi(input.index))
                } else {
                    Err(error_payload("invalid_request", "unknown dasha rashi"))
                }
            }
            "yogini" => {
                if input.index < 8 {
                    Ok(DashaEntity::Yogini(input.index))
                } else {
                    Err(error_payload("invalid_request", "unknown dasha yogini"))
                }
            }
            _ => Err(error_payload(
                "invalid_request",
                "unknown dasha entity kind",
            )),
        },
        _ => Err(error_payload(
            "invalid_request",
            "unknown dasha entity kind",
        )),
    }
}

fn parse_dasha_period(input: &DashaPeriodInput) -> Result<DashaPeriod, Value> {
    Ok(DashaPeriod {
        entity: parse_dasha_entity(&input.entity)?,
        start_jd: input.start_jd,
        end_jd: input.end_jd,
        level: parse_dasha_level(&input.level)?,
        order: input.order,
        parent_idx: input.parent_idx,
    })
}

fn parse_tara_id(input: &EnumInput) -> Result<TaraId, Value> {
    match input {
        EnumInput::Int(_) => Err(error_payload(
            "invalid_request",
            "tara star ids must be string names",
        )),
        EnumInput::Str(value) => TaraId::from_str(value)
            .ok_or_else(|| error_payload("invalid_request", "unknown tara id")),
    }
}

fn parse_tara_output(input: Option<&EnumInput>) -> Result<TaraOutputKind, Value> {
    match input {
        None => Ok(TaraOutputKind::Ecliptic),
        Some(EnumInput::Int(value)) => TARA_OUTPUT_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown tara output")),
        Some(EnumInput::Str(value)) => parse_named(value, &TARA_OUTPUT_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown tara output")),
    }
}

fn parse_tara_accuracy(input: Option<&EnumInput>) -> Result<TaraAccuracy, Value> {
    match input {
        None => Ok(TaraAccuracy::Astrometric),
        Some(EnumInput::Int(value)) => TARA_ACCURACY_VARIANTS
            .get(*value as usize)
            .copied()
            .ok_or_else(|| error_payload("invalid_request", "unknown tara accuracy")),
        Some(EnumInput::Str(value)) => parse_named(value, &TARA_ACCURACY_VARIANTS)
            .ok_or_else(|| error_payload("invalid_request", "unknown tara accuracy")),
    }
}

fn parse_utc(input: UtcInput) -> Result<UtcTime, Value> {
    UtcTime::try_new(
        input.year,
        input.month,
        input.day,
        input.hour,
        input.minute,
        input.second,
        None,
    )
    .map_err(|err| map_error("time_error", err))
}

fn parse_location(input: GeoLocationInput) -> GeoLocation {
    GeoLocation {
        latitude_deg: input.latitude_deg,
        longitude_deg: input.longitude_deg,
        altitude_m: input.altitude_m.unwrap_or(0.0),
    }
}

fn parse_panchang_include_mask(input: &PanchangIncludeInput) -> Result<u32, Value> {
    let name_bits = |name: &str| -> Result<u32, Value> {
        if name.eq_ignore_ascii_case("none") {
            return Ok(0);
        }
        panchang_include_bits(name).ok_or_else(|| {
            error_payload(
                "invalid_request",
                format!("unknown panchang include name '{name}'"),
            )
        })
    };
    match input {
        PanchangIncludeInput::Int(mask) => Ok(*mask),
        PanchangIncludeInput::Str(name) => name_bits(name),
        PanchangIncludeInput::List(names) => {
            let mut mask = 0_u32;
            for name in names {
                mask |= name_bits(name)?;
            }
            Ok(mask)
        }
    }
}

fn to_riseset_config(
    state: &EngineState,
    input: Option<&RiseSetConfigInput>,
) -> Result<RiseSetConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_riseset(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_default();
    if let Some(input) = input {
        config.use_refraction = input.use_refraction.unwrap_or(config.use_refraction);
        config.altitude_correction = input
            .altitude_correction
            .unwrap_or(config.altitude_correction);
        if let Some(limb) = input.sun_limb.as_ref() {
            config.sun_limb = match limb {
                EnumInput::Int(value) => SUN_LIMB_VARIANTS
                    .get(*value as usize)
                    .copied()
                    .ok_or_else(|| error_payload("invalid_request", "unknown sun limb"))?,
                EnumInput::Str(value) => parse_named(value, &SUN_LIMB_VARIANTS)
                    .ok_or_else(|| error_payload("invalid_request", "unknown sun limb"))?,
            };
        }
    }
    Ok(config)
}

fn to_bhava_config(
    state: &EngineState,
    input: Option<&BhavaConfigInput>,
) -> Result<BhavaConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_bhava(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_default();
    if let Some(input) = input {
        config.system = parse_bhava_system(input.system.as_ref())?;
        config.reference_mode = parse_bhava_reference_mode(input.reference_mode.as_ref())?;
        if let Some(value) = input.use_rashi_bhava_for_bala_avastha {
            config.use_rashi_bhava_for_bala_avastha = value;
        }
        if let Some(value) = input.include_node_aspects_for_drik_bala {
            config.include_node_aspects_for_drik_bala = value;
        }
        if let Some(value) = input.include_special_bhavabala_rules {
            config.include_special_bhavabala_rules = value;
        }
        if let Some(value) = input.divide_guru_buddh_drishti_by_4_for_drik_bala {
            config.divide_guru_buddh_drishti_by_4_for_drik_bala = value;
        }
        if let Some(value) = input.chandra_benefic_rule.as_ref() {
            config.chandra_benefic_rule = parse_chandra_benefic_rule(value)?;
        }
        if let Some(value) = input.sayanadi_ghatika_rounding.as_ref() {
            config.sayanadi_ghatika_rounding = parse_sayanadi_ghatika_rounding(value)?;
        }
        if let Some(value) = input.include_rashi_bhava_results {
            config.include_rashi_bhava_results = value;
        }
        config.starting_point = BhavaStartingPoint::Lagna;
    }
    Ok(config)
}

fn to_sankranti_config(
    state: &EngineState,
    input: Option<&SankrantiConfigInput>,
) -> Result<SankrantiConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_sankranti(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_else(SankrantiConfig::default_lahiri);
    if let Some(input) = input {
        if input.ayanamsha_system.is_some() {
            config.ayanamsha_system = parse_ayanamsha_system(input.ayanamsha_system.as_ref())?;
        }
        if let Some(use_nutation) = input.use_nutation {
            config.use_nutation = use_nutation;
        }
        if let Some(plane) = parse_reference_plane(input.reference_plane.as_ref())? {
            config.reference_plane = plane;
        }
        if let Some(model) = parse_precession_model(input.precession_model.as_ref())? {
            config.precession_model = model;
        }
        if input.node_mode.is_some() {
            config.node_mode = parse_node_mode(input.node_mode.as_ref())?;
        }
        if let Some(step) = input.step_size_days {
            config.step_size_days = step;
        }
        if let Some(iterations) = input.max_iterations {
            config.max_iterations = iterations;
        }
        if let Some(convergence) = input.convergence_days {
            config.convergence_days = convergence;
        }
    }
    Ok(config)
}

fn parse_tajaka_return_basis(input: Option<&EnumInput>) -> Result<TajakaReturnBasis, Value> {
    match input {
        None => Ok(TajakaReturnBasis::SiderealSolar),
        Some(EnumInput::Int(0)) => Ok(TajakaReturnBasis::TropicalSolar),
        Some(EnumInput::Int(1)) => Ok(TajakaReturnBasis::SiderealSolar),
        Some(EnumInput::Str(value)) if value == "tropical_solar" => {
            Ok(TajakaReturnBasis::TropicalSolar)
        }
        Some(EnumInput::Str(value)) if value == "sidereal_solar" => {
            Ok(TajakaReturnBasis::SiderealSolar)
        }
        _ => Err(error_payload(
            "invalid_request",
            "unknown tajaka_return_basis",
        )),
    }
}

fn parse_natal_target_kind(input: &EnumInput) -> Result<NatalTargetKind, Value> {
    match input {
        EnumInput::Int(0) => Ok(NatalTargetKind::Graha),
        EnumInput::Int(1) => Ok(NatalTargetKind::Bindu),
        EnumInput::Int(2) => Ok(NatalTargetKind::Sphuta),
        EnumInput::Int(3) => Ok(NatalTargetKind::SpecialLagna),
        EnumInput::Int(4) => Ok(NatalTargetKind::ArudhaPada),
        EnumInput::Int(5) => Ok(NatalTargetKind::Custom),
        EnumInput::Str(value) if value == "graha" => Ok(NatalTargetKind::Graha),
        EnumInput::Str(value) if value == "bindu" => Ok(NatalTargetKind::Bindu),
        EnumInput::Str(value) if value == "sphuta" => Ok(NatalTargetKind::Sphuta),
        EnumInput::Str(value) if value == "special_lagna" => Ok(NatalTargetKind::SpecialLagna),
        EnumInput::Str(value) if value == "arudha_pada" => Ok(NatalTargetKind::ArudhaPada),
        EnumInput::Str(value) if value == "custom" => Ok(NatalTargetKind::Custom),
        _ => Err(error_payload(
            "invalid_request",
            "unknown natal target kind",
        )),
    }
}

fn to_gochar_events_config(
    input: Option<&GocharEventsConfigInput>,
) -> Result<GocharEventsConfig, Value> {
    let mut config = GocharEventsConfig::default();
    if let Some(input) = input {
        if input.tajaka_return_basis.is_some() {
            config.tajaka_return_basis =
                parse_tajaka_return_basis(input.tajaka_return_basis.as_ref())?;
        }
        if let Some(value) = input.yearly_count {
            config.yearly_count = value as usize;
        }
        if let Some(value) = input.monthly_count {
            config.monthly_count = value as usize;
        }
        if let Some(value) = input.transit_window_days {
            config.transit_window_days = value;
        }
        if let Some(value) = input.include_return_charts {
            config.include_return_charts = value;
        }
        if let Some(value) = input.solar_step_size_days {
            config.solar_step_size_days = value;
        }
        if let Some(value) = input.lunar_step_size_days {
            config.lunar_step_size_days = value;
        }
        if let Some(value) = input.solar_convergence_days {
            config.solar_convergence_days = value;
        }
        if let Some(value) = input.lunar_convergence_days {
            config.lunar_convergence_days = value;
        }
        if let Some(value) = input.max_iterations {
            config.max_iterations = value;
        }
    }
    Ok(config)
}

fn to_conjunction_config(
    state: &EngineState,
    input: Option<&SearchConfigInput>,
) -> Result<ConjunctionConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_conjunction(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_else(|| ConjunctionConfig::conjunction(0.5));
    if let Some(input) = input {
        if let Some(target) = input.target_separation_deg {
            config.target_separation_deg = target;
        }
        if input.node_mode.is_some() {
            config.node_mode = parse_node_mode(input.node_mode.as_ref())?;
        }
        if let Some(step) = input.step_size_days {
            config.step_size_days = step;
        }
        if let Some(iterations) = input.max_iterations {
            config.max_iterations = iterations;
        }
        if let Some(convergence) = input.convergence_days {
            config.convergence_days = convergence;
        }
    }
    Ok(config)
}

fn to_stationary_config(
    state: &EngineState,
    input: Option<&SearchConfigInput>,
) -> Result<StationaryConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_stationary(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_else(StationaryConfig::inner_planet);
    if let Some(input) = input {
        if input.node_mode.is_some() {
            config.node_mode = parse_node_mode(input.node_mode.as_ref())?;
        }
        if let Some(step) = input.step_size_days {
            config.step_size_days = step;
        }
        if let Some(iterations) = input.max_iterations {
            config.max_iterations = iterations;
        }
        if let Some(convergence) = input.convergence_days {
            config.convergence_days = convergence;
        }
        if let Some(numerical) = input.numerical_step_days {
            config.numerical_step_days = numerical;
        }
    }
    Ok(config)
}

fn to_grahan_config(
    state: &EngineState,
    input: Option<&SearchConfigInput>,
) -> dhruv_search::GrahanConfig {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_grahan(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_default();
    if let Some(input) = input {
        if let Some(include_penumbral) = input.include_penumbral {
            config.include_penumbral = include_penumbral;
        }
        if let Some(include_peak_details) = input.include_peak_details {
            config.include_peak_details = include_peak_details;
        }
        if let Some(include_path) = input.include_path {
            config.include_path = include_path;
        }
        if let Some(path_step_minutes) = input.path_step_minutes {
            config.path_step_minutes = path_step_minutes;
        }
        if let Some(boundary_step_deg) = input.boundary_step_deg {
            config.boundary_step_deg = boundary_step_deg;
        }
        if let Some(include_local_grid) = input.include_local_grid {
            config.include_local_grid = include_local_grid;
        }
        if let Some(local_grid_step_deg) = input.local_grid_step_deg {
            config.local_grid_step_deg = local_grid_step_deg;
        }
        if let Some(include_isolines) = input.include_isolines {
            config.include_isolines = include_isolines;
        }
        if let Some(fractions) = &input.duration_isoline_fractions {
            config.duration_isoline_fractions = fractions.clone();
        }
        if let Some(levels) = &input.magnitude_isoline_levels {
            config.magnitude_isoline_levels = levels.clone();
        }
        if let Some(include_central_corridor) = input.include_central_corridor {
            config.include_central_corridor = include_central_corridor;
        }
        if let Some(include_contact_footprints) = input.include_contact_footprints {
            config.include_contact_footprints = include_contact_footprints;
        }
        if let Some(include_umbra_footprints) = input.include_umbra_footprints {
            config.include_umbra_footprints = include_umbra_footprints;
        }
        if let Some(levels) = &input.instantaneous_magnitude_levels {
            config.instantaneous_magnitude_levels = levels.clone();
        }
    }
    config
}

fn to_graha_positions_config(
    state: &EngineState,
    input: Option<&GrahaPositionsConfigInput>,
) -> Result<dhruv_search::GrahaPositionsConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_graha_positions(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_default();
    if let Some(input) = input {
        if let Some(include_nakshatra) = input.include_nakshatra {
            config.include_nakshatra = include_nakshatra;
        }
        if let Some(include_lagna) = input.include_lagna {
            config.include_lagna = include_lagna;
        }
        if let Some(include_outer_planets) = input.include_outer_planets {
            config.include_outer_planets = include_outer_planets;
        }
        if let Some(include_bhava) = input.include_bhava {
            config.include_bhava = include_bhava;
        }
        if let Some(include_equatorial) = input.include_equatorial {
            config.include_equatorial = include_equatorial;
        }
        if let Some(basic_states_config) = input.basic_states_config.as_ref() {
            if let Some(include_basic_states) = basic_states_config.include_basic_states {
                config.basic_states_config.include_basic_states = include_basic_states;
            }
            if let Some(include_sensitive_point_distances) =
                basic_states_config.include_sensitive_point_distances
            {
                config.basic_states_config.include_sensitive_point_distances =
                    include_sensitive_point_distances;
            }
        }
    }
    Ok(config)
}

fn to_bindus_config(
    state: &EngineState,
    input: Option<&BindusConfigInput>,
) -> Result<dhruv_search::BindusConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_bindus(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_default();
    if let Some(input) = input {
        if let Some(include_nakshatra) = input.include_nakshatra {
            config.include_nakshatra = include_nakshatra;
        }
        if let Some(include_bhava) = input.include_bhava {
            config.include_bhava = include_bhava;
        }
        apply_time_upagraha_config(&mut config.upagraha_config, input.upagraha_config.as_ref())?;
    }
    Ok(config)
}

fn to_drishti_config(
    state: &EngineState,
    input: Option<&DrishtiConfigInput>,
) -> dhruv_search::DrishtiConfig {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_drishti(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_default();
    if let Some(input) = input {
        if let Some(include_bhava) = input.include_bhava {
            config.include_bhava = include_bhava;
        }
        if let Some(include_lagna) = input.include_lagna {
            config.include_lagna = include_lagna;
        }
        if let Some(include_bindus) = input.include_bindus {
            config.include_bindus = include_bindus;
        }
    }
    config
}

fn to_amsha_scope(input: Option<&AmshaChartScopeInput>) -> dhruv_search::AmshaChartScope {
    let mut scope = dhruv_search::AmshaChartScope::default();
    if let Some(input) = input {
        if let Some(value) = input.include_bhava_cusps {
            scope.include_bhava_cusps = value;
        }
        if let Some(value) = input.include_arudha_padas {
            scope.include_arudha_padas = value;
        }
        if let Some(value) = input.include_upagrahas {
            scope.include_upagrahas = value;
        }
        if let Some(value) = input.include_sphutas {
            scope.include_sphutas = value;
        }
        if let Some(value) = input.include_special_lagnas {
            scope.include_special_lagnas = value;
        }
        if let Some(value) = input.include_outer_planets {
            scope.include_outer_planets = value;
        }
    }
    scope
}

fn to_amsha_selection(
    input: Option<&[AmshaRequestInput]>,
) -> Result<dhruv_search::AmshaSelectionConfig, Value> {
    let mut selection = dhruv_search::AmshaSelectionConfig::default();
    let Some(input) = input else {
        return Ok(selection);
    };
    if input.len() > dhruv_search::MAX_AMSHA_REQUESTS {
        return Err(error_payload("invalid_request", "too many amsha requests"));
    }
    selection.count = input.len() as u8;
    for (index, request) in input.iter().enumerate() {
        let amsha = Amsha::from_code(request.code)
            .ok_or_else(|| error_payload("invalid_request", "unknown amsha code"))?;
        let variation_code = match request.variation {
            Some(code) if is_valid_amsha_variation(amsha, code) => code,
            Some(_) => {
                return Err(error_payload(
                    "invalid_request",
                    "unknown amsha variation for amsha code",
                ));
            }
            None => default_amsha_variation(amsha),
        };
        selection.codes[index] = amsha.code();
        selection.variations[index] = variation_code;
    }
    Ok(selection)
}

/// Decode `[%{code, variation}]` request maps into library `AmshaRequest`s,
/// validating codes and per-amsha variation codes.
fn to_amsha_requests(input: Option<&[AmshaRequestInput]>) -> Result<Vec<AmshaRequest>, Value> {
    input
        .unwrap_or_default()
        .iter()
        .map(|request| {
            let amsha = Amsha::from_code(request.code)
                .ok_or_else(|| error_payload("invalid_request", "unknown amsha code"))?;
            let variation = match request.variation {
                Some(code) if is_valid_amsha_variation(amsha, code) => Some(code),
                Some(_) => {
                    return Err(error_payload(
                        "invalid_request",
                        "unknown amsha variation for amsha code",
                    ));
                }
                None => None,
            };
            Ok(AmshaRequest { amsha, variation })
        })
        .collect()
}

fn to_full_kundali_config(
    state: &EngineState,
    input: Option<&FullKundaliConfigInput>,
) -> Result<dhruv_search::FullKundaliConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_full_kundali(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_default();
    if let Some(input) = input {
        if let Some(value) = input.include_bhava_cusps {
            config.include_bhava_cusps = value;
        }
        if let Some(value) = input.include_graha_positions {
            config.include_graha_positions = value;
        }
        if let Some(value) = input.include_bindus {
            config.include_bindus = value;
        }
        if let Some(value) = input.include_drishti {
            config.include_drishti = value;
        }
        if let Some(value) = input.include_ashtakavarga {
            config.include_ashtakavarga = value;
        }
        if let Some(value) = input.include_upagrahas {
            config.include_upagrahas = value;
        }
        if let Some(value) = input.include_sphutas {
            config.include_sphutas = value;
        }
        if let Some(value) = input.include_special_lagnas {
            config.include_special_lagnas = value;
        }
        if let Some(value) = input.include_amshas {
            config.include_amshas = value;
        }
        if let Some(value) = input.include_shadbala {
            config.include_shadbala = value;
        }
        if let Some(value) = input.include_bhavabala {
            config.include_bhavabala = value;
        }
        if let Some(value) = input.include_vimsopaka {
            config.include_vimsopaka = value;
        }
        if let Some(value) = input.include_avastha {
            config.include_avastha = value;
        }
        if let Some(value) = input.include_charakaraka {
            config.include_charakaraka = value;
        }
        if let Some(value) = input.panchang_include_mask.as_ref() {
            config.panchang_include_mask = parse_panchang_include_mask(value)?;
        }
        if let Some(value) = input.include_dasha {
            config.include_dasha = value;
        }
        if input.charakaraka_scheme.is_some() {
            config.charakaraka_scheme =
                parse_charakaraka_scheme(input.charakaraka_scheme.as_ref())?;
        }
        if input.node_dignity_policy.is_some() {
            config.node_dignity_policy =
                parse_node_dignity_policy(input.node_dignity_policy.as_ref())?;
        }
        if let Some(graha_positions_config) = input.graha_positions_config.as_ref() {
            config.graha_positions_config =
                to_graha_positions_config(state, Some(graha_positions_config))?;
        }
        if let Some(bindus_config) = input.bindus_config.as_ref() {
            config.bindus_config = to_bindus_config(state, Some(bindus_config))?;
        }
        // Apply upagraha_config AFTER bindus_config so the copy to
        // bindus_config.upagraha_config is not overwritten.
        if let Some(upagraha_config) = input.upagraha_config.as_ref() {
            apply_time_upagraha_config(&mut config.upagraha_config, Some(upagraha_config))?;
            config.bindus_config.upagraha_config = config.upagraha_config;
        }
        if let Some(drishti_config) = input.drishti_config.as_ref() {
            config.drishti_config = to_drishti_config(state, Some(drishti_config));
        }
        let amsha_scope = to_amsha_scope(input.amsha_scope.as_ref());
        let amsha_selection = to_amsha_selection(input.amsha_selection.as_deref())?;
        if input.amsha_scope.is_some() {
            config.amsha_scope = amsha_scope;
        }
        if input.amsha_selection.is_some() {
            config.amsha_selection = amsha_selection;
            config.include_amshas = true;
        }
        if config.include_amshas {
            config.graha_positions_config.include_lagna = true;
            if config.amsha_scope.include_bhava_cusps {
                config.include_bhava_cusps = true;
            }
            if config.amsha_scope.include_arudha_padas {
                config.include_bindus = true;
            }
            if config.amsha_scope.include_upagrahas {
                config.include_upagrahas = true;
            }
            if config.amsha_scope.include_sphutas {
                config.include_sphutas = true;
            }
            if config.amsha_scope.include_special_lagnas {
                config.include_special_lagnas = true;
            }
        }
        if let Some(dasha_config) = input.dasha_config.as_ref() {
            if let Some(snapshot_utc) = dasha_config.snapshot_utc.clone() {
                config.dasha_config.snapshot_time = Some(dhruv_search::DashaSnapshotTime::Utc(
                    parse_utc(snapshot_utc)?,
                ));
            }
            if let Some(max_level) = dasha_config.max_level {
                config.dasha_config.max_level = max_level;
            }
            if let Some(cycles) = dasha_config.cycles {
                config.dasha_config.cycles = cycles;
            }
            if let Some(min_span_years) = dasha_config.min_span_years {
                config.dasha_config.min_span_years = min_span_years;
            }
            if let Some(max_levels) = dasha_config.max_levels.as_ref() {
                if max_levels.len() > config.dasha_config.max_levels.len() {
                    return Err(error_payload(
                        "invalid_request",
                        format!(
                            "max_levels may contain at most {} entries",
                            config.dasha_config.max_levels.len()
                        ),
                    ));
                }
                for (index, max_level) in max_levels
                    .iter()
                    .copied()
                    .enumerate()
                    .take(config.dasha_config.max_levels.len())
                {
                    config.dasha_config.max_levels[index] = max_level;
                }
            }
            if let Some(systems) = dasha_config.systems.as_ref() {
                if systems.len() > config.dasha_config.systems.len() {
                    return Err(error_payload(
                        "invalid_request",
                        format!(
                            "systems may contain at most {} entries",
                            config.dasha_config.systems.len()
                        ),
                    ));
                }
                config.dasha_config.count = systems.len() as u8;
                for (index, system) in systems.iter().enumerate() {
                    config.dasha_config.systems[index] = parse_dasha_system(system)? as u8;
                }
            }
        }
    }
    Ok(config)
}

fn to_dasha_variation(input: Option<&DashaVariationInput>) -> Result<DashaVariationConfig, Value> {
    let mut config = DashaVariationConfig::default();
    if let Some(input) = input {
        if let Some(use_abhijit) = input.use_abhijit {
            config.use_abhijit = use_abhijit;
        }
        if let Some(cycles) = input.cycles {
            if cycles == 0 {
                return Err(error_payload("invalid_request", "cycles must be >= 1"));
            }
            config.cycles = Some(cycles);
        }
        if let Some(min_span_years) = input.min_span_years {
            if !min_span_years.is_finite() || min_span_years <= 0.0 {
                return Err(error_payload(
                    "invalid_request",
                    "min_span_years must be a positive number",
                ));
            }
            config.min_span_years = Some(min_span_years);
        }
        if let Some(scheme) = input.yogini_scheme.as_ref() {
            config.yogini_scheme = match scheme {
                EnumInput::Int(value) => YOGINI_SCHEME_VARIANTS
                    .get(*value as usize)
                    .copied()
                    .ok_or_else(|| error_payload("invalid_request", "unknown yogini scheme"))?,
                EnumInput::Str(value) => parse_named(value, &YOGINI_SCHEME_VARIANTS)
                    .ok_or_else(|| error_payload("invalid_request", "unknown yogini scheme"))?,
            };
        }
        if let Some(level_methods) = input.level_methods.as_ref() {
            for (index, method) in level_methods
                .iter()
                .enumerate()
                .take(config.level_methods.len())
            {
                config.level_methods[index] = Some(match method {
                    EnumInput::Int(value) => SUB_PERIOD_METHOD_VARIANTS
                        .get(*value as usize)
                        .copied()
                        .ok_or_else(|| {
                            error_payload("invalid_request", "unknown sub-period method")
                        })?,
                    EnumInput::Str(value) => parse_named(value, &SUB_PERIOD_METHOD_VARIANTS)
                        .ok_or_else(|| {
                            error_payload("invalid_request", "unknown sub-period method")
                        })?,
                });
            }
        }
    }
    Ok(config)
}

#[derive(Debug, Clone, Default)]
struct OwnedDashaInputs {
    moon_sid_lon: Option<f64>,
    rashi_inputs: Option<RashiDashaInputs>,
    sunrise_sunset: Option<(f64, f64)>,
}

impl OwnedDashaInputs {
    fn borrowed(&self) -> dhruv_search::DashaInputs<'_> {
        dhruv_search::DashaInputs {
            moon_sid_lon: self.moon_sid_lon,
            rashi_inputs: self.rashi_inputs.as_ref(),
            sunrise_sunset: self.sunrise_sunset,
        }
    }
}

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

fn utc_to_jd_tdb(engine: &Engine, utc: &UtcTime) -> f64 {
    dhruv_time::Epoch::from_utc(
        utc.year,
        utc.month,
        utc.day,
        utc.hour,
        utc.minute,
        utc.second,
        engine.lsk(),
    )
    .as_jd_tdb()
}

fn search_at_jd_tdb(engine: &Engine, request: &SearchRequest) -> Result<f64, Value> {
    match (&request.at_jd_tdb, &request.at_utc) {
        (Some(jd), None) => Ok(*jd),
        (None, Some(utc)) => Ok(utc_to_jd_tdb(engine, &parse_utc(utc.clone())?)),
        (Some(_), Some(_)) => Err(error_payload(
            "invalid_request",
            "provide only one of at_jd_tdb or at_utc",
        )),
        (None, None) => Err(error_payload(
            "invalid_request",
            "at_jd_tdb or at_utc is required",
        )),
    }
}

fn search_range_jd_tdb(engine: &Engine, request: &SearchRequest) -> Result<(f64, f64), Value> {
    match (
        &request.start_jd_tdb,
        &request.end_jd_tdb,
        &request.start_utc,
        &request.end_utc,
    ) {
        (Some(start), Some(end), None, None) => Ok((*start, *end)),
        (None, None, Some(start), Some(end)) => Ok((
            utc_to_jd_tdb(engine, &parse_utc(start.clone())?),
            utc_to_jd_tdb(engine, &parse_utc(end.clone())?),
        )),
        (Some(_), Some(_), Some(_), Some(_)) => Err(error_payload(
            "invalid_request",
            "provide either start/end_jd_tdb or start/end_utc, not both",
        )),
        _ => Err(error_payload(
            "invalid_request",
            "start/end_jd_tdb or start/end_utc are required",
        )),
    }
}

fn search_at_utc(engine: &Engine, request: &SearchRequest) -> Result<UtcTime, Value> {
    match (&request.at_jd_tdb, &request.at_utc) {
        (Some(jd), None) => Ok(UtcTime::from_jd_tdb(*jd, engine.lsk())),
        (None, Some(utc)) => parse_utc(utc.clone()),
        (Some(_), Some(_)) => Err(error_payload(
            "invalid_request",
            "provide only one of at_jd_tdb or at_utc",
        )),
        (None, None) => Err(error_payload(
            "invalid_request",
            "at_jd_tdb or at_utc is required",
        )),
    }
}

fn search_range_utc(engine: &Engine, request: &SearchRequest) -> Result<(UtcTime, UtcTime), Value> {
    match (
        &request.start_jd_tdb,
        &request.end_jd_tdb,
        &request.start_utc,
        &request.end_utc,
    ) {
        (Some(start), Some(end), None, None) => Ok((
            UtcTime::from_jd_tdb(*start, engine.lsk()),
            UtcTime::from_jd_tdb(*end, engine.lsk()),
        )),
        (None, None, Some(start), Some(end)) => {
            Ok((parse_utc(start.clone())?, parse_utc(end.clone())?))
        }
        (Some(_), Some(_), Some(_), Some(_)) => Err(error_payload(
            "invalid_request",
            "provide either start/end_jd_tdb or start/end_utc, not both",
        )),
        _ => Err(error_payload(
            "invalid_request",
            "start/end_jd_tdb or start/end_utc are required",
        )),
    }
}

fn parse_defaults_mode(input: Option<&EnumInput>) -> Result<DefaultsMode, Value> {
    match input {
        None => Ok(DefaultsMode::Recommended),
        Some(EnumInput::Int(0)) => Ok(DefaultsMode::Recommended),
        Some(EnumInput::Int(1)) => Ok(DefaultsMode::None),
        Some(EnumInput::Str(value)) if value == "recommended" => Ok(DefaultsMode::Recommended),
        Some(EnumInput::Str(value)) if value == "none" => Ok(DefaultsMode::None),
        _ => Err(error_payload("invalid_request", "unknown defaults mode")),
    }
}

fn parse_dasha_inputs(input: Option<&DashaInputsInput>) -> Result<Option<OwnedDashaInputs>, Value> {
    let Some(input) = input else {
        return Ok(None);
    };
    let rashi_inputs = match (input.graha_sidereal_lons.as_ref(), input.lagna_sidereal_lon) {
        (Some(lons), Some(lagna)) => {
            if lons.len() != 9 {
                return Err(error_payload(
                    "invalid_request",
                    "graha_sidereal_lons must contain exactly 9 values",
                ));
            }
            let mut arr = [0.0; 9];
            arr.copy_from_slice(lons);
            Some(RashiDashaInputs::new(arr, lagna))
        }
        (None, None) => None,
        _ => {
            return Err(error_payload(
                "invalid_request",
                "rashi inputs require graha_sidereal_lons and lagna_sidereal_lon",
            ));
        }
    };
    let sunrise_sunset = match (input.sunrise_sunset, input.sunrise_jd, input.sunset_jd) {
        (Some(pair), None, None) => Some(pair),
        (None, Some(sunrise), Some(sunset)) => Some((sunrise, sunset)),
        (None, None, None) => None,
        _ => {
            return Err(error_payload(
                "invalid_request",
                "sunrise_jd and sunset_jd must be provided together",
            ));
        }
    };
    Ok(Some(OwnedDashaInputs {
        moon_sid_lon: input.moon_sid_lon,
        rashi_inputs,
        sunrise_sunset,
    }))
}

fn to_tara_config(
    state: &EngineState,
    input: Option<&TaraConfigInput>,
) -> Result<TaraConfig, Value> {
    let mut config = state
        .resolver
        .as_ref()
        .and_then(|resolver| {
            resolver
                .resolve_tara(None)
                .ok()
                .map(|effective| effective.value)
        })
        .unwrap_or_default();
    if let Some(input) = input {
        config.apply_parallax = input.apply_parallax.unwrap_or(config.apply_parallax);
        if input.accuracy.is_some() {
            config.accuracy = parse_tara_accuracy(input.accuracy.as_ref())?;
        }
    }
    Ok(config)
}

fn apply_time_policy(state: &EngineState) {
    set_time_conversion_policy(state.time_policy);
}

fn utc_json(utc: UtcTime) -> Value {
    json!({
        "year": utc.year,
        "month": utc.month,
        "day": utc.day,
        "hour": utc.hour,
        "minute": utc.minute,
        "second": utc.second
    })
}

fn state_vector_json(state: StateVector) -> Value {
    json!({
        "position_km": state.position_km,
        "velocity_km_s": state.velocity_km_s
    })
}

fn spherical_json(coords: dhruv_frames::SphericalCoords) -> Value {
    json!({
        "lon_deg": coords.lon_deg,
        "lat_deg": coords.lat_deg,
        "distance_km": coords.distance_km
    })
}

fn spherical_state_json(state: dhruv_frames::SphericalState) -> Value {
    json!({
        "lon_deg": state.lon_deg,
        "lat_deg": state.lat_deg,
        "distance_km": state.distance_km,
        "lon_speed": state.lon_speed,
        "lat_speed": state.lat_speed,
        "distance_speed": state.distance_speed
    })
}

fn parse_query_output(input: Option<&EnumInput>) -> Result<i32, Value> {
    match input {
        None => Ok(0),
        Some(EnumInput::Int(0)) => Ok(0),
        Some(EnumInput::Int(1)) => Ok(1),
        Some(EnumInput::Int(2)) => Ok(2),
        Some(EnumInput::Str(value)) => match value.as_str() {
            "cartesian" => Ok(0),
            "spherical" => Ok(1),
            "both" => Ok(2),
            _ => Err(error_payload("invalid_request", "unknown query output")),
        },
        _ => Err(error_payload("invalid_request", "unknown query output")),
    }
}

fn query_epoch_tdb_jd(state: &EngineState, request: &QueryInput) -> Result<f64, Value> {
    match (request.epoch_tdb_jd, request.utc.as_ref()) {
        (Some(epoch_tdb_jd), None) => Ok(epoch_tdb_jd),
        (None, Some(utc_input)) => {
            let utc = parse_utc(utc_input.clone())?;
            let jd_utc = dhruv_time::calendar_to_jd(
                utc.year,
                utc.month,
                utc.day as f64
                    + utc.hour as f64 / 24.0
                    + utc.minute as f64 / 1440.0
                    + utc.second / 86_400.0,
            );
            let utc_seconds = jd_to_tdb_seconds(jd_utc);
            let tdb_seconds = state
                .engine
                .as_ref()
                .ok_or_else(|| error_payload("engine_error", "engine not initialized"))?
                .lsk()
                .utc_to_tdb_with_policy_and_eop(utc_seconds, state.eop.as_ref(), state.time_policy)
                .tdb_seconds;
            Ok(tdb_seconds_to_jd(tdb_seconds))
        }
        _ => Err(error_payload(
            "invalid_request",
            "provide exactly one of epoch_tdb_jd or utc",
        )),
    }
}

fn ephemeris_query_result_json(state_vector: StateVector, output_mode: i32) -> Value {
    let spherical_state =
        cartesian_state_to_spherical_state(&state_vector.position_km, &state_vector.velocity_km_s);
    let output_name = match output_mode {
        1 => "spherical",
        2 => "both",
        _ => "cartesian",
    };
    let state_value = if output_mode == 1 {
        Value::Null
    } else {
        state_vector_json(state_vector)
    };
    let spherical_value = if output_mode == 0 {
        Value::Null
    } else {
        spherical_state_json(spherical_state)
    };
    json!({
        "state": state_value,
        "spherical_state": spherical_value,
        "output": output_name
    })
}

fn rise_set_result_json(result: RiseSetResult) -> Value {
    match result {
        RiseSetResult::Event { jd_tdb, event } => json!({
            "status": "event",
            "jd_tdb": jd_tdb,
            "event": debug_name(event)
        }),
        RiseSetResult::NeverRises => json!({ "status": "never_rises" }),
        RiseSetResult::NeverSets => json!({ "status": "never_sets" }),
    }
}

fn bhava_result_json(result: BhavaResult) -> Value {
    json!({
        "lagna_deg": result.lagna_deg,
        "mc_deg": result.mc_deg,
        "bhavas": result.bhavas.iter().map(|bhava| json!({
            "number": bhava.number,
            "cusp_deg": bhava.cusp_deg,
            "start_deg": bhava.start_deg,
            "end_deg": bhava.end_deg
        })).collect::<Vec<_>>()
    })
}

fn rashi_bhava_result_from_lagna(lagna_deg: f64) -> BhavaResult {
    let lagna = lagna_deg.rem_euclid(360.0);
    let lagna_rashi = (lagna / 30.0).floor() as u8;
    let degree_in_rashi = lagna % 30.0;
    let mut bhavas = [dhruv_vedic_base::Bhava {
        number: 0,
        cusp_deg: 0.0,
        start_deg: 0.0,
        end_deg: 0.0,
    }; 12];
    for i in 0..12 {
        let rashi = (lagna_rashi + i as u8) % 12;
        let start = f64::from(rashi) * 30.0;
        bhavas[i] = dhruv_vedic_base::Bhava {
            number: (i + 1) as u8,
            cusp_deg: (start + degree_in_rashi).rem_euclid(360.0),
            start_deg: start,
            end_deg: (start + 30.0).rem_euclid(360.0),
        };
    }
    BhavaResult {
        bhavas,
        lagna_deg: lagna,
        mc_deg: bhavas[9].cusp_deg,
    }
}

fn bhava_result_json_with_rashi(result: BhavaResult, include_rashi_bhava: bool) -> Value {
    let mut value = bhava_result_json(result);
    if include_rashi_bhava {
        value["rashi_bhava"] = bhava_result_json(rashi_bhava_result_from_lagna(result.lagna_deg));
    }
    value
}

fn panchang_value_json(result: &PanchangResult) -> Value {
    json!({
        "tithi": result.tithi.map(tithi_json),
        "karana": result.karana.map(karana_json),
        "yoga": result.yoga.map(yoga_json),
        "vaar": result.vaar.map(vaar_json),
        "hora": result.hora.map(hora_json),
        "ghatika": result.ghatika.map(ghatika_json),
        "nakshatra": result.nakshatra.map(nakshatra_json),
        "masa": result.masa.map(masa_json),
        "ayana": result.ayana.map(ayana_json),
        "varsha": result.varsha.map(varsha_json)
    })
}

fn tithi_json(info: dhruv_search::TithiInfo) -> Value {
    json!({
        "tithi_index": info.tithi_index,
        "paksha": debug_name(info.paksha),
        "tithi_in_paksha": info.tithi_in_paksha,
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn karana_json(info: dhruv_search::KaranaInfo) -> Value {
    json!({
        "karana_index": info.karana_index,
        "karana": debug_name(info.karana),
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn yoga_json(info: dhruv_search::YogaInfo) -> Value {
    json!({
        "yoga_index": info.yoga_index,
        "yoga": debug_name(info.yoga),
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn vaar_json(info: dhruv_search::VaarInfo) -> Value {
    json!({
        "vaar": debug_name(info.vaar),
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn hora_json(info: dhruv_search::HoraInfo) -> Value {
    json!({
        "hora_index": info.hora_index,
        "hora": debug_name(info.hora),
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn ghatika_json(info: dhruv_search::GhatikaInfo) -> Value {
    json!({
        "value": info.value,
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn nakshatra_json(info: dhruv_search::PanchangNakshatraInfo) -> Value {
    json!({
        "nakshatra_index": info.nakshatra_index,
        "nakshatra": debug_name(info.nakshatra),
        "pada": info.pada,
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn masa_json(info: dhruv_search::MasaInfo) -> Value {
    json!({
        "masa": debug_name(info.masa),
        "adhika": info.adhika,
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn ayana_json(info: dhruv_search::AyanaInfo) -> Value {
    json!({
        "ayana": debug_name(info.ayana),
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn varsha_json(info: dhruv_search::VarshaInfo) -> Value {
    json!({
        "samvatsara": debug_name(info.samvatsara),
        "order": info.order,
        "start": utc_json(info.start),
        "end": utc_json(info.end)
    })
}

fn conjunction_result_json(result: ConjunctionResult) -> Value {
    match result {
        ConjunctionResult::Single(event) => json!({ "events": event.map(conjunction_event_json) }),
        ConjunctionResult::Many(events) => {
            json!({ "events": events.into_iter().map(conjunction_event_json).collect::<Vec<_>>() })
        }
    }
}

fn conjunction_event_json(event: dhruv_search::ConjunctionEvent) -> Value {
    json!({
        "utc": utc_json(event.utc),
        "jd_tdb": event.jd_tdb,
        "actual_separation_deg": event.actual_separation_deg,
        "target_separation_deg": event.target_separation_deg,
        "body1_longitude_deg": event.body1_longitude_deg,
        "body2_longitude_deg": event.body2_longitude_deg,
        "body1_latitude_deg": event.body1_latitude_deg,
        "body2_latitude_deg": event.body2_latitude_deg,
        "body1_sidereal_longitude_deg": event.body1_sidereal_longitude_deg,
        "body2_sidereal_longitude_deg": event.body2_sidereal_longitude_deg,
        "body1_rashi_index": event.body1_rashi_index,
        "body2_rashi_index": event.body2_rashi_index,
        "body1": debug_name(event.body1),
        "body2": debug_name(event.body2)
    })
}

fn grahan_result_json(result: GrahanResult, config: &dhruv_search::GrahanConfig) -> Value {
    let effective_config = grahan_effective_config_json(config);
    match result {
        GrahanResult::ChandraSingle(event) => {
            json!({ "kind": "chandra", "events": event.map(|value| chandra_grahan_json(*value)), "effective_config": effective_config })
        }
        GrahanResult::ChandraMany(events) => {
            json!({ "kind": "chandra", "events": events.into_iter().map(chandra_grahan_json).collect::<Vec<_>>(), "effective_config": effective_config })
        }
        GrahanResult::SuryaSingle(event) => {
            json!({ "kind": "surya", "events": event.map(|value| surya_grahan_json(*value)), "effective_config": effective_config })
        }
        GrahanResult::SuryaMany(events) => {
            json!({ "kind": "surya", "events": events.into_iter().map(surya_grahan_json).collect::<Vec<_>>(), "effective_config": effective_config })
        }
    }
}

/// The configuration actually applied after clamping/sanitizing; consumers
/// should build cache keys against this echo rather than the raw request.
fn grahan_effective_config_json(config: &dhruv_search::GrahanConfig) -> Value {
    let effective = config.effective();
    json!({
        "include_penumbral": effective.include_penumbral,
        "include_peak_details": effective.include_peak_details,
        "include_path": effective.include_path,
        "path_step_minutes": effective.path_step_minutes,
        "boundary_step_deg": effective.boundary_step_deg,
        "include_local_grid": effective.include_local_grid,
        "local_grid_step_deg": effective.local_grid_step_deg,
        "include_isolines": effective.include_isolines,
        "duration_isoline_fractions": effective.duration_isoline_fractions,
        "magnitude_isoline_levels": effective.magnitude_isoline_levels,
        "include_central_corridor": effective.include_central_corridor,
        "include_contact_footprints": effective.include_contact_footprints,
        "include_umbra_footprints": effective.include_umbra_footprints,
        "instantaneous_magnitude_levels": effective.instantaneous_magnitude_levels
    })
}

fn magnitude_rings_json(rings: Vec<dhruv_search::SuryaMagnitudeRing>) -> Value {
    Value::Array(
        rings
            .into_iter()
            .map(|ring| {
                json!({
                    "level": ring.level,
                    "contains_pole": pole_side_json(ring.contains_pole),
                    "boundary": ring.boundary.into_iter().map(|p| json!({
                        "latitude_deg": p.latitude_deg, "longitude_deg": p.longitude_deg
                    })).collect::<Vec<_>>()
                })
            })
            .collect(),
    )
}

fn pole_side_json(side: Option<dhruv_search::PoleSide>) -> Value {
    match side {
        None => Value::Null,
        Some(dhruv_search::PoleSide::North) => Value::String("north".to_string()),
        Some(dhruv_search::PoleSide::South) => Value::String("south".to_string()),
    }
}

fn isoline_rings_json(rings: Vec<dhruv_search::SuryaIsolineRing>) -> Value {
    Value::Array(
        rings
            .into_iter()
            .map(|ring| {
                json!({
                    "contains_pole": pole_side_json(ring.contains_pole),
                    "boundary": ring.boundary.into_iter().map(|p| json!({
                        "latitude_deg": p.latitude_deg, "longitude_deg": p.longitude_deg
                    })).collect::<Vec<_>>()
                })
            })
            .collect(),
    )
}

fn chandra_grahan_json(event: dhruv_search::ChandraGrahan) -> Value {
    json!({
        "grahan_type": debug_name(event.grahan_type),
        "magnitude": event.magnitude,
        "penumbral_magnitude": event.penumbral_magnitude,
        "greatest_grahan_utc": utc_json(event.greatest_grahan_utc),
        "greatest_grahan_jd": event.greatest_grahan_jd,
        "p1_utc": utc_json(event.p1_utc),
        "p1_jd": event.p1_jd,
        "u1_utc": event.u1_utc.map(utc_json),
        "u1_jd": event.u1_jd,
        "u2_utc": event.u2_utc.map(utc_json),
        "u2_jd": event.u2_jd,
        "u3_utc": event.u3_utc.map(utc_json),
        "u3_jd": event.u3_jd,
        "u4_utc": event.u4_utc.map(utc_json),
        "u4_jd": event.u4_jd,
        "p4_utc": utc_json(event.p4_utc),
        "p4_jd": event.p4_jd,
        "moon_ecliptic_lat_deg": event.moon_ecliptic_lat_deg,
        "angular_separation_deg": event.angular_separation_deg,
        "moon_right_ascension_deg": event.moon_right_ascension_deg,
        "moon_declination_deg": event.moon_declination_deg
    })
}

fn surya_grahan_json(event: dhruv_search::SuryaGrahan) -> Value {
    json!({
        "grahan_type": debug_name(event.grahan_type),
        "magnitude": event.magnitude,
        "obscuration": event.obscuration,
        "apparent_diameter_ratio": event.apparent_diameter_ratio,
        "gamma": event.gamma,
        "greatest_grahan_utc": utc_json(event.greatest_grahan_utc),
        "greatest_grahan_jd": event.greatest_grahan_jd,
        "c1_utc": event.c1_utc.map(utc_json),
        "c1_jd": event.c1_jd,
        "c2_utc": event.c2_utc.map(utc_json),
        "c2_jd": event.c2_jd,
        "c3_utc": event.c3_utc.map(utc_json),
        "c3_jd": event.c3_jd,
        "c4_utc": event.c4_utc.map(utc_json),
        "c4_jd": event.c4_jd,
        "moon_ecliptic_lat_deg": event.moon_ecliptic_lat_deg,
        "angular_separation_deg": event.angular_separation_deg,
        "sun_right_ascension_deg": event.sun_right_ascension_deg,
        "sun_declination_deg": event.sun_declination_deg,
        "greatest_location": event.greatest_location.map(|p| json!({
            "latitude_deg": p.latitude_deg, "longitude_deg": p.longitude_deg
        })),
        "besselian": {
            "utc": utc_json(event.besselian.utc),
            "jd_tdb": event.besselian.jd_tdb,
            "x": event.besselian.x, "y": event.besselian.y,
            "d_deg": event.besselian.d_deg, "mu_deg": event.besselian.mu_deg,
            "l1": event.besselian.l1, "l2": event.besselian.l2,
            "tan_f1": event.besselian.tan_f1, "tan_f2": event.besselian.tan_f2
        },
        "path": event.path.into_iter().map(|p| json!({
            "utc": utc_json(p.utc), "jd_tdb": p.jd_tdb,
            "center": {"latitude_deg": p.center.latitude_deg, "longitude_deg": p.center.longitude_deg},
            "northern_limit": p.northern_limit.map(|v| json!({"latitude_deg": v.latitude_deg, "longitude_deg": v.longitude_deg})),
            "southern_limit": p.southern_limit.map(|v| json!({"latitude_deg": v.latitude_deg, "longitude_deg": v.longitude_deg})),
            "width_km": p.width_km, "central_duration_seconds": p.central_duration_seconds,
            "sun_altitude_deg": p.sun_altitude_deg, "sun_azimuth_deg": p.sun_azimuth_deg,
            "grahan_type": debug_name(p.grahan_type)
        })).collect::<Vec<_>>(),
        "footprints": event.footprints.into_iter().map(|f| json!({
            "utc": utc_json(f.utc), "jd_tdb": f.jd_tdb,
            "boundary": f.boundary.into_iter().map(|p| json!({"latitude_deg": p.latitude_deg, "longitude_deg": p.longitude_deg})).collect::<Vec<_>>(),
            "contains_pole": pole_side_json(f.contains_pole),
            "magnitude_rings": magnitude_rings_json(f.magnitude_rings)
        })).collect::<Vec<_>>(),
        "contact_footprints": event.contact_footprints.into_iter().map(|f| json!({
            "contact": match f.contact {
                dhruv_search::SuryaContactKind::C1 => "c1",
                dhruv_search::SuryaContactKind::C2 => "c2",
                dhruv_search::SuryaContactKind::Greatest => "greatest",
                dhruv_search::SuryaContactKind::C3 => "c3",
                dhruv_search::SuryaContactKind::C4 => "c4",
            },
            "utc": utc_json(f.utc), "jd_tdb": f.jd_tdb,
            "boundary": f.boundary.into_iter().map(|p| json!({"latitude_deg": p.latitude_deg, "longitude_deg": p.longitude_deg})).collect::<Vec<_>>(),
            "contains_pole": pole_side_json(f.contains_pole),
            "magnitude_rings": magnitude_rings_json(f.magnitude_rings)
        })).collect::<Vec<_>>(),
        "umbra_footprints": event.umbra_footprints.into_iter().map(|f| json!({
            "utc": utc_json(f.utc), "jd_tdb": f.jd_tdb,
            "grahan_type": debug_name(f.grahan_type),
            "boundary": f.boundary.into_iter().map(|p| json!({"latitude_deg": p.latitude_deg, "longitude_deg": p.longitude_deg})).collect::<Vec<_>>(),
            "contains_pole": pole_side_json(f.contains_pole)
        })).collect::<Vec<_>>(),
        "local": event.local.map(|l| json!({
            "location": {"latitude_deg": l.location.latitude_deg, "longitude_deg": l.location.longitude_deg, "altitude_m": l.location.altitude_m},
            "visible": l.visible, "grahan_type": l.grahan_type.map(debug_name),
            "maximum_utc": l.maximum_utc.map(utc_json), "maximum_jd": l.maximum_jd,
            "c1_utc": l.c1_utc.map(utc_json), "c1_jd": l.c1_jd,
            "c2_utc": l.c2_utc.map(utc_json), "c2_jd": l.c2_jd,
            "c3_utc": l.c3_utc.map(utc_json), "c3_jd": l.c3_jd,
            "c4_utc": l.c4_utc.map(utc_json), "c4_jd": l.c4_jd,
            "magnitude": l.magnitude, "obscuration": l.obscuration,
            "sun_altitude_deg": l.sun_altitude_deg, "sun_azimuth_deg": l.sun_azimuth_deg,
            "central_duration_seconds": l.central_duration_seconds
        })),
        "centrality": match event.centrality {
            dhruv_search::SuryaCentrality::Full => "full",
            dhruv_search::SuryaCentrality::Partial => "partial",
            dhruv_search::SuryaCentrality::None => "none",
        },
        "local_grid": event.local_grid.into_iter().map(|s| json!({
            "latitude_deg": s.latitude_deg, "longitude_deg": s.longitude_deg,
            "magnitude": s.magnitude, "obscuration": s.obscuration,
            "maximum_utc": utc_json(s.maximum_utc), "maximum_jd": s.maximum_jd,
            "first_contact_utc": utc_json(s.first_contact_utc), "first_contact_jd": s.first_contact_jd,
            "last_contact_utc": utc_json(s.last_contact_utc), "last_contact_jd": s.last_contact_jd,
            "visible_duration_seconds": s.visible_duration_seconds
        })).collect::<Vec<_>>(),
        "isolines": event.isolines.map(|iso| json!({
            "visibility_boundary": isoline_rings_json(iso.visibility_boundary),
            "duration_isolines": iso.duration_isolines.into_iter().map(|entry| json!({
                "fraction": entry.fraction, "rings": isoline_rings_json(entry.rings)
            })).collect::<Vec<_>>(),
            "magnitude_isolines": iso.magnitude_isolines.into_iter().map(|entry| json!({
                "level": entry.level, "rings": isoline_rings_json(entry.rings)
            })).collect::<Vec<_>>()
        })),
        "central_corridor": event.central_corridor.map(|corridor| json!({
            "segments": corridor.segments.into_iter().map(|segment| json!({
                "grahan_type": debug_name(segment.grahan_type),
                "rings": isoline_rings_json(segment.rings)
            })).collect::<Vec<_>>()
        }))
    })
}

fn lunar_phase_result_json(result: LunarPhaseResult) -> Value {
    match result {
        LunarPhaseResult::Single(event) => json!({ "events": event.map(lunar_phase_event_json) }),
        LunarPhaseResult::Many(events) => {
            json!({ "events": events.into_iter().map(lunar_phase_event_json).collect::<Vec<_>>() })
        }
    }
}

fn lunar_phase_event_json(event: dhruv_search::LunarPhaseEvent) -> Value {
    json!({
        "utc": utc_json(event.utc),
        "phase": debug_name(event.phase),
        "moon_longitude_deg": event.moon_longitude_deg,
        "sun_longitude_deg": event.sun_longitude_deg,
        "moon_sidereal_longitude_deg": event.moon_sidereal_longitude_deg,
        "sun_sidereal_longitude_deg": event.sun_sidereal_longitude_deg,
        "moon_rashi_index": event.moon_rashi_index,
        "sun_rashi_index": event.sun_rashi_index
    })
}

fn sankranti_result_json(result: SankrantiResult) -> Value {
    match result {
        SankrantiResult::Single(event) => json!({ "events": event.map(sankranti_event_json) }),
        SankrantiResult::Many(events) => {
            json!({ "events": events.into_iter().map(sankranti_event_json).collect::<Vec<_>>() })
        }
    }
}

fn sankranti_event_json(event: dhruv_search::SankrantiEvent) -> Value {
    let mut value = json!({
        "utc": utc_json(event.utc),
        "body": debug_name(event.body),
        "rashi": debug_name(event.rashi),
        "rashi_index": event.rashi_index,
        "sidereal_longitude_deg": event.sidereal_longitude_deg,
        "tropical_longitude_deg": event.tropical_longitude_deg,
        "is_retrograde": event.is_retrograde
    });
    if event.body == TransitBody::Body(Body::Sun) {
        // Legacy keys kept for wire compatibility with Sun-only callers.
        value["sun_sidereal_longitude_deg"] = json!(event.sidereal_longitude_deg);
        value["sun_tropical_longitude_deg"] = json!(event.tropical_longitude_deg);
    }
    value
}

fn motion_result_json(result: MotionResult) -> Value {
    match result {
        MotionResult::StationarySingle(event) => {
            json!({ "kind": "stationary", "events": event.map(stationary_event_json) })
        }
        MotionResult::StationaryMany(events) => {
            json!({ "kind": "stationary", "events": events.into_iter().map(stationary_event_json).collect::<Vec<_>>() })
        }
        MotionResult::MaxSpeedSingle(event) => {
            json!({ "kind": "max_speed", "events": event.map(max_speed_event_json) })
        }
        MotionResult::MaxSpeedMany(events) => {
            json!({ "kind": "max_speed", "events": events.into_iter().map(max_speed_event_json).collect::<Vec<_>>() })
        }
    }
}

fn stationary_event_json(event: dhruv_search::StationaryEvent) -> Value {
    json!({
        "utc": utc_json(event.utc),
        "jd_tdb": event.jd_tdb,
        "body": debug_name(event.body),
        "longitude_deg": event.longitude_deg,
        "latitude_deg": event.latitude_deg,
        "sidereal_longitude_deg": event.sidereal_longitude_deg,
        "rashi_index": event.rashi_index,
        "station_type": debug_name(event.station_type)
    })
}

fn max_speed_event_json(event: dhruv_search::MaxSpeedEvent) -> Value {
    json!({
        "utc": utc_json(event.utc),
        "jd_tdb": event.jd_tdb,
        "body": debug_name(event.body),
        "longitude_deg": event.longitude_deg,
        "latitude_deg": event.latitude_deg,
        "speed_deg_per_day": event.speed_deg_per_day,
        "sidereal_longitude_deg": event.sidereal_longitude_deg,
        "rashi_index": event.rashi_index,
        "speed_type": debug_name(event.speed_type)
    })
}

fn graha_entry_json(entry: dhruv_search::GrahaEntry, graha: Option<Graha>) -> Value {
    json!({
        "graha": graha.map(debug_name),
        "sidereal_longitude": entry.sidereal_longitude,
        "rashi": debug_name(entry.rashi),
        "rashi_index": entry.rashi_index,
        "nakshatra": debug_name(entry.nakshatra),
        "nakshatra_index": entry.nakshatra_index,
        "pada": entry.pada,
        "bhava_number": entry.bhava_number,
        "rashi_bhava_number": entry.rashi_bhava_number,
        "equatorial_valid": entry.equatorial_valid,
        "right_ascension_deg": entry.right_ascension_deg,
        "declination_deg": entry.declination_deg,
        "ecliptic_latitude_deg": entry.ecliptic_latitude_deg,
        "basic_states": entry.basic_states_valid.then_some(json!({
            "exalted": entry.basic_states.exalted,
            "debilitated": entry.basic_states.debilitated,
            "combust": entry.basic_states.combust,
            "retrograde": entry.basic_states.retrograde,
            "moolatrikone": entry.basic_states.moolatrikone,
            "marankarak_sthana": entry.basic_states.marankarak_sthana,
            "mrityubhaga": entry.basic_states.mrityubhaga,
            "pushkaramsha": entry.basic_states.pushkaramsha,
            "pushkarbhaga": entry.basic_states.pushkarbhaga
        })),
        "sensitive_point_distances": entry.sensitive_point_distances_valid.then_some(json!({
            "mrityubhaga": entry.sensitive_point_distances.mrityubhaga,
            "pushkarbhaga": entry.sensitive_point_distances.pushkarbhaga
        }))
    })
}

fn graha_longitudes_json(result: dhruv_search::GrahaLongitudes) -> Value {
    let grahas = ALL_GRAHAS
        .iter()
        .enumerate()
        .map(|(idx, graha)| {
            json!({
                "graha": debug_name(*graha),
                "longitude": result.longitudes[idx]
            })
        })
        .collect::<Vec<_>>();
    json!({
        "grahas": grahas,
        "longitudes": result.longitudes,
        "outer_planets": result.outer_planets.map(|values| json!([
            { "graha": "Uranus", "longitude": values[0] },
            { "graha": "Neptune", "longitude": values[1] },
            { "graha": "Pluto", "longitude": values[2] }
        ]))
    })
}

fn moving_osculating_apogees_json(result: dhruv_search::MovingOsculatingApogees) -> Value {
    json!({
        "entries": result.entries.into_iter().map(|entry| {
            json!({
                "graha": debug_name(entry.graha),
                "graha_index": entry.graha.index(),
                "sidereal_longitude": entry.sidereal_longitude,
                "ayanamsha_deg": entry.ayanamsha_deg,
                "reference_plane_longitude": entry.reference_plane_longitude
            })
        }).collect::<Vec<_>>()
    })
}

fn graha_positions_json(result: dhruv_search::GrahaPositions) -> Value {
    json!({
        "grahas": ALL_GRAHAS.iter().enumerate().map(|(idx, graha)| graha_entry_json(result.grahas[idx], Some(*graha))).collect::<Vec<_>>(),
        "lagna": graha_entry_json(result.lagna, None),
        "outer_planets": [
            graha_entry_json(result.outer_planets[0], None),
            graha_entry_json(result.outer_planets[1], None),
            graha_entry_json(result.outer_planets[2], None)
        ],
        "earth_orientation_valid": result.earth_orientation_valid,
        "gmst_deg": result.gmst_deg,
        "gast_deg": result.gast_deg
    })
}

fn special_lagnas_json(result: dhruv_vedic_base::AllSpecialLagnas) -> Value {
    json!({
        "bhava_lagna": result.bhava_lagna,
        "hora_lagna": result.hora_lagna,
        "ghati_lagna": result.ghati_lagna,
        "vighati_lagna": result.vighati_lagna,
        "varnada_lagna": result.varnada_lagna,
        "sree_lagna": result.sree_lagna,
        "pranapada_lagna": result.pranapada_lagna,
        "indu_lagna": result.indu_lagna
    })
}

fn arudha_json(result: [dhruv_vedic_base::ArudhaResult; 12]) -> Value {
    json!({
        "entries": result.into_iter().enumerate().map(|(idx, entry)| json!({
            "index": idx + 1,
            "longitude_deg": entry.longitude_deg,
            "rashi_index": entry.rashi_index
        })).collect::<Vec<_>>()
    })
}

fn upagrahas_json(result: dhruv_vedic_base::AllUpagrahas) -> Value {
    json!({
        "gulika": result.gulika,
        "maandi": result.maandi,
        "kaala": result.kaala,
        "mrityu": result.mrityu,
        "artha_prahara": result.artha_prahara,
        "yama_ghantaka": result.yama_ghantaka,
        "dhooma": result.dhooma,
        "vyatipata": result.vyatipata,
        "parivesha": result.parivesha,
        "indra_chapa": result.indra_chapa,
        "upaketu": result.upaketu
    })
}

fn ashtakavarga_json(result: dhruv_vedic_base::AshtakavargaResult) -> Value {
    json!({
        "bhinna": result.bavs.iter().map(|bav| json!({
            "graha_index": bav.graha_index,
            "points": bav.points,
            "contributors": bav.contributors
        })).collect::<Vec<_>>(),
        "sarva": {
            "total_points": result.sav.total_points,
            "after_trikona": result.sav.after_trikona,
            "after_ekadhipatya": result.sav.after_ekadhipatya
        }
    })
}

fn drishti_json(result: dhruv_search::DrishtiResult) -> Value {
    let graha_to_graha = result
        .graha_to_graha
        .entries
        .iter()
        .map(|row| {
            row.iter()
                .map(|entry| entry.total_virupa)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    json!({
        "graha_to_graha": graha_to_graha,
        "graha_to_bhava": result.graha_to_bhava.iter().map(|row| {
            row.iter().map(|entry| entry.total_virupa).collect::<Vec<_>>()
        }).collect::<Vec<_>>(),
        "graha_to_rashi_bhava": result.graha_to_rashi_bhava.iter().map(|row| {
            row.iter().map(|entry| entry.total_virupa).collect::<Vec<_>>()
        }).collect::<Vec<_>>(),
        "graha_to_lagna": result.graha_to_lagna.iter().map(|entry| entry.total_virupa).collect::<Vec<_>>()
    })
}

fn charakaraka_json(result: CharakarakaResult) -> Value {
    json!({
        "scheme": debug_name(result.scheme),
        "used_eight_karakas": result.used_eight_karakas,
        "entries": result.entries.into_iter().map(|entry| json!({
            "role": debug_name(entry.role),
            "graha": debug_name(entry.graha),
            "rank": entry.rank,
            "longitude_deg": entry.longitude_deg,
            "degrees_in_rashi": entry.degrees_in_rashi,
            "effective_degrees_in_rashi": entry.effective_degrees_in_rashi
        })).collect::<Vec<_>>()
    })
}

fn charakaraka_change_event_json(event: dhruv_search::CharakarakaChangeEvent) -> Value {
    let ranking_before = event
        .before
        .entries
        .iter()
        .map(|entry| debug_name(entry.graha))
        .collect::<Vec<_>>();
    let ranking_after = event
        .after
        .entries
        .iter()
        .map(|entry| debug_name(entry.graha))
        .collect::<Vec<_>>();
    json!({
        "at": utc_json(event.utc),
        "jd_tdb": event.jd_tdb,
        "trigger": debug_name(event.trigger),
        "changed_roles": event
            .changed_roles
            .into_iter()
            .map(debug_name)
            .collect::<Vec<_>>(),
        "ranking_before": ranking_before,
        "ranking_after": ranking_after,
        "used_eight_karakas_before": event.before.used_eight_karakas,
        "used_eight_karakas_after": event.after.used_eight_karakas,
        "before": charakaraka_json(event.before),
        "after": charakaraka_json(event.after)
    })
}

fn charakaraka_events_json(result: dhruv_search::CharakarakaEventsResult) -> Value {
    json!({
        "events": result
            .events
            .into_iter()
            .map(charakaraka_change_event_json)
            .collect::<Vec<_>>(),
        "truncated": result.truncated,
        "next_from_utc": result.next_from_utc.map(utc_json),
    })
}

fn shadbala_json(result: dhruv_search::ShadbalaResult) -> Value {
    json!({
        "entries": result.entries.into_iter().map(|entry| json!({
            "graha": debug_name(entry.graha),
            "sthana": {
                "uchcha": entry.sthana.uchcha,
                "saptavargaja": entry.sthana.saptavargaja,
                "ojhayugma": entry.sthana.ojhayugma,
                "kendradi": entry.sthana.kendradi,
                "drekkana": entry.sthana.drekkana,
                "total": entry.sthana.total
            },
            "dig": entry.dig,
            "kala": {
                "nathonnatha": entry.kala.nathonnatha,
                "paksha": entry.kala.paksha,
                "tribhaga": entry.kala.tribhaga,
                "abda": entry.kala.abda,
                "masa": entry.kala.masa,
                "vara": entry.kala.vara,
                "hora": entry.kala.hora,
                "ayana": entry.kala.ayana,
                "yuddha": entry.kala.yuddha,
                "total": entry.kala.total
            },
            "cheshta": entry.cheshta,
            "naisargika": entry.naisargika,
            "drik": entry.drik,
            "total_shashtiamsas": entry.total_shashtiamsas,
            "total_rupas": entry.total_rupas,
            "required_strength": entry.required_strength,
            "is_strong": entry.is_strong
        })).collect::<Vec<_>>()
    })
}

fn bhavabala_json(result: dhruv_vedic_base::BhavaBalaResult) -> Value {
    json!({
        "entries": result.entries.into_iter().map(|entry| json!({
            "bhava_number": entry.bhava_number,
            "cusp_sidereal_lon": entry.cusp_sidereal_lon,
            "rashi_index": entry.rashi_index,
            "rashi": debug_name(dhruv_vedic_base::ALL_RASHIS[entry.rashi_index as usize]),
            "lord": debug_name(entry.lord),
            "bhavadhipati": entry.bhavadhipati,
            "dig": entry.dig,
            "drishti": entry.drishti,
            "occupation_bonus": entry.occupation_bonus,
            "rising_bonus": entry.rising_bonus,
            "total_virupas": entry.total_virupas,
            "total_rupas": entry.total_rupas
        })).collect::<Vec<_>>()
    })
}

fn vimsopaka_json(result: dhruv_search::VimsopakaResult) -> Value {
    json!({
        "entries": result.entries.into_iter().map(|entry| json!({
            "graha": debug_name(entry.graha),
            "shadvarga": entry.shadvarga,
            "saptavarga": entry.saptavarga,
            "dashavarga": entry.dashavarga,
            "shodasavarga": entry.shodasavarga
        })).collect::<Vec<_>>()
    })
}

fn avastha_json(result: dhruv_vedic_base::AllGrahaAvasthas) -> Value {
    json!({
        "entries": result.entries.into_iter().enumerate().map(|(idx, entry)| {
            let deeptadi_states = {
                let names = entry.deeptadi_states.as_names();
                names[..entry.deeptadi_states.count()].to_vec()
            };
            let lajjitadi_states = {
                let names = entry.lajjitadi_states.as_names();
                names[..entry.lajjitadi_states.count()].to_vec()
            };
            json!({
                "graha": debug_name(ALL_GRAHAS[idx]),
                "baladi": debug_name(entry.baladi),
                "jagradadi": debug_name(entry.jagradadi),
                "deeptadi": debug_name(entry.deeptadi),
                "deeptadi_states": deeptadi_states,
                "deeptadi_mask": entry.deeptadi_states.mask(),
                "lajjitadi": entry.lajjitadi.map(debug_name),
                "lajjitadi_states": lajjitadi_states,
                "lajjitadi_mask": entry.lajjitadi_states.mask(),
                "sayanadi": {
                    "avastha": debug_name(entry.sayanadi.avastha),
                    "sub_states": entry.sayanadi.sub_states.iter().map(|sub_state| debug_name(*sub_state)).collect::<Vec<_>>()
                }
            })
        }).collect::<Vec<_>>()
    })
}

fn amsha_result_json(result: dhruv_search::AmshaResult) -> Value {
    json!({
        "charts": result.charts.into_iter().map(|chart| json!({
            "amsha": debug_name(chart.amsha),
            "sanskrit_name": chart.amsha.sanskrit_name(),
            "variation": amsha_variation_info(chart.amsha, chart.variation_code)
                .map(|info| info.name)
                .unwrap_or("default"),
            "grahas": chart.grahas.into_iter().map(amsha_entry_json).collect::<Vec<_>>(),
            "lagna": amsha_entry_json(chart.lagna),
            "bhava_cusps": chart
                .bhava_cusps
                .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>()),
            "rashi_bhava_cusps": chart
                .rashi_bhava_cusps
                .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>()),
            "arudha_padas": chart
                .arudha_padas
                .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>()),
            "rashi_bhava_arudha_padas": chart
                .rashi_bhava_arudha_padas
                .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>()),
            "upagrahas": chart
                .upagrahas
                .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>()),
            "sphutas": chart
                .sphutas
                .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>()),
            "special_lagnas": chart
                .special_lagnas
                .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>()),
            "outer_planets": chart
                .outer_planets
                .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>())
        })).collect::<Vec<_>>()
    })
}

fn amsha_series_chart_json(chart: dhruv_search::AmshaSeriesChart) -> Value {
    json!({
        "amsha": debug_name(chart.amsha),
        "sanskrit_name": chart.amsha.sanskrit_name(),
        "variation_code": chart.variation_code,
        "lagna": amsha_entry_json(chart.lagna),
        "grahas": chart
            .grahas
            .map(|entries| entries.into_iter().map(amsha_entry_json).collect::<Vec<_>>())
    })
}

fn amsha_series_json(series: dhruv_search::AmshaSeries) -> Value {
    json!({
        "points": series
            .points
            .into_iter()
            .map(|point| json!({
                "utc": utc_json(point.utc),
                "jd_utc": point.jd_utc,
                "charts": point
                    .charts
                    .into_iter()
                    .map(amsha_series_chart_json)
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>()
    })
}

fn amsha_lagna_segment_json(segment: dhruv_search::AmshaLagnaSegment) -> Value {
    json!({
        "rashi": debug_name(segment.rashi),
        "rashi_index": segment.rashi_index,
        "start": utc_json(segment.start),
        "end": utc_json(segment.end),
    })
}

fn amsha_lagna_events_json(result: dhruv_search::AmshaLagnaEventsResult) -> Value {
    json!({
        "entries": result
            .entries
            .into_iter()
            .map(|entry| json!({
                "amsha": debug_name(entry.amsha),
                "variation_code": entry.variation_code,
                "segments": entry
                    .segments
                    .into_iter()
                    .map(amsha_lagna_segment_json)
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
        "truncated": result.truncated,
        "next_from_utc": result.next_from_utc.map(utc_json),
    })
}

fn panchang_events_json(result: dhruv_search::PanchangEventsResult) -> Value {
    json!({
        "tithi": result.tithi.into_iter().map(tithi_json).collect::<Vec<_>>(),
        "karana": result.karana.into_iter().map(karana_json).collect::<Vec<_>>(),
        "yoga": result.yoga.into_iter().map(yoga_json).collect::<Vec<_>>(),
        "vaar": result.vaar.into_iter().map(vaar_json).collect::<Vec<_>>(),
        "hora": result.hora.into_iter().map(hora_json).collect::<Vec<_>>(),
        "ghatika": result
            .ghatika
            .into_iter()
            .map(ghatika_json)
            .collect::<Vec<_>>(),
        "nakshatra": result
            .nakshatra
            .into_iter()
            .map(nakshatra_json)
            .collect::<Vec<_>>(),
        "masa": result.masa.into_iter().map(masa_json).collect::<Vec<_>>(),
        "ayana": result.ayana.into_iter().map(ayana_json).collect::<Vec<_>>(),
        "varsha": result.varsha.into_iter().map(varsha_json).collect::<Vec<_>>(),
        "truncated": result.truncated,
        "next_from_utc": result.next_from_utc.map(utc_json),
    })
}

fn amsha_variation_catalog_json(amsha: Amsha) -> Value {
    let catalog = amsha_variation_catalog(amsha);
    json!({
        "amsha_code": catalog.amsha.code(),
        "default_variation_code": catalog.default_variation_code,
        "variations": catalog.variations.iter().map(|info| json!({
            "amsha_code": catalog.amsha.code(),
            "variation_code": info.variation_code,
            "name": info.name,
            "label": info.label,
            "is_default": info.is_default,
            "description": info.description
        })).collect::<Vec<_>>()
    })
}

fn amsha_entry_json(entry: dhruv_search::AmshaEntry) -> Value {
    json!({
        // Point identity. `name` uses the same snake_case vocabulary as
        // `graha_positions.grahas[].graha`; `family` + `point_index` make an
        // entry self-describing once detached from its array.
        "name": entry.point.key(),
        "display_name": entry.point.name(),
        "family": entry.point.family.key(),
        "point_index": entry.point.index,
        "sidereal_longitude": entry.sidereal_longitude,
        "rashi": debug_name(entry.rashi),
        "rashi_index": entry.rashi_index,
        "degrees_in_rashi": entry.degrees_in_rashi,
        "dms": {
            "degrees": entry.dms.degrees,
            "minutes": entry.dms.minutes,
            "seconds": entry.dms.seconds
        },
        "nakshatra": debug_name(entry.nakshatra),
        "nakshatra_index": entry.nakshatra_index,
        "pada": entry.pada,
        "rashi_bhava_number": entry.rashi_bhava_number
    })
}

fn bindus_json(bindus: dhruv_search::BindusResult) -> Value {
    json!({
        "arudha_padas": bindus.arudha_padas.into_iter().map(|entry| graha_entry_json(entry, None)).collect::<Vec<_>>(),
        "rashi_bhava_arudha_padas": bindus.rashi_bhava_arudha_padas.map(|entries| entries.into_iter().map(|entry| graha_entry_json(entry, None)).collect::<Vec<_>>()),
        "bhrigu_bindu": graha_entry_json(bindus.bhrigu_bindu, None),
        "pranapada_lagna": graha_entry_json(bindus.pranapada_lagna, None),
        "gulika": graha_entry_json(bindus.gulika, None),
        "maandi": graha_entry_json(bindus.maandi, None),
        "hora_lagna": graha_entry_json(bindus.hora_lagna, None),
        "ghati_lagna": graha_entry_json(bindus.ghati_lagna, None),
        "sree_lagna": graha_entry_json(bindus.sree_lagna, None)
    })
}

fn full_kundali_json(result: dhruv_search::FullKundaliResult) -> Value {
    json!({
        "ayanamsha_deg": result.ayanamsha_deg,
        "bhava_cusps": result.bhava_cusps.map(bhava_result_json),
        "rashi_bhava_cusps": result.rashi_bhava_cusps.map(bhava_result_json),
        "bhava_cusp_sensitive_point_distances": result
            .bhava_cusp_sensitive_point_distances
            .map(|items| items.into_iter().map(|item| json!({
                "mrityubhaga": item.mrityubhaga,
                "pushkarbhaga": item.pushkarbhaga
            })).collect::<Vec<_>>()),
        "rashi_bhava_cusp_sensitive_point_distances": result
            .rashi_bhava_cusp_sensitive_point_distances
            .map(|items| items.into_iter().map(|item| json!({
                "mrityubhaga": item.mrityubhaga,
                "pushkarbhaga": item.pushkarbhaga
            })).collect::<Vec<_>>()),
        "graha_positions": result.graha_positions.map(graha_positions_json),
        "bindus": result.bindus.map(bindus_json),
        "drishti": result.drishti.map(drishti_json),
        "ashtakavarga": result.ashtakavarga.map(ashtakavarga_json),
        "upagrahas": result.upagrahas.map(upagrahas_json),
        "sphutas": result.sphutas.map(|sphutas| json!({ "longitudes": sphutas.longitudes })),
        "special_lagnas": result.special_lagnas.map(special_lagnas_json),
        "amshas": result.amshas.map(amsha_result_json),
        "shadbala": result.shadbala.map(shadbala_json),
        "bhavabala": result.bhavabala.map(bhavabala_json),
        "vimsopaka": result.vimsopaka.map(vimsopaka_json),
        "avastha": result.avastha.map(avastha_json),
        "charakaraka": result.charakaraka.map(charakaraka_json),
        "panchang": result.panchang.as_ref().map(panchang_value_json),
        "dasha": result.dasha.map(|items| items.into_iter().map(dasha_hierarchy_json).collect::<Vec<_>>()),
        "dasha_snapshots": result.dasha_snapshots.map(|items| items.into_iter().map(dasha_snapshot_json).collect::<Vec<_>>())
    })
}

fn gochar_reference_json(reference: GocharReference) -> Value {
    json!({
        "natal_tropical_solar_longitude_deg": reference.natal_tropical_solar_longitude_deg,
        "natal_sidereal_solar_longitude_deg": reference.natal_sidereal_solar_longitude_deg,
        "natal_elongation_deg": reference.natal_elongation_deg,
        "natal_masa": masa_json(reference.natal_masa)
    })
}

fn tajaka_return_event_json(event: TajakaReturnEvent) -> Value {
    json!({
        "utc": utc_json(event.utc),
        "jd_tdb": event.jd_tdb,
        "basis": match event.basis {
            TajakaReturnBasis::TropicalSolar => "tropical_solar",
            TajakaReturnBasis::SiderealSolar => "sidereal_solar"
        },
        "target_solar_longitude_deg": event.target_solar_longitude_deg,
        "event_solar_longitude_deg": event.event_solar_longitude_deg,
        "chart": event.chart.map(full_kundali_json)
    })
}

fn tithi_pravesha_event_json(event: TithiPraveshaEvent) -> Value {
    json!({
        "utc": utc_json(event.utc),
        "jd_tdb": event.jd_tdb,
        "target_elongation_deg": event.target_elongation_deg,
        "event_elongation_deg": event.event_elongation_deg,
        "masa": masa_json(event.masa),
        "chart": event.chart.map(full_kundali_json)
    })
}

fn transit_to_natal_aspect_event_json(event: TransitToNatalAspectEvent) -> Value {
    json!({
        "transit_body": event.transit_body.name(),
        "transit_body_code": event.transit_body.code(),
        "target_name": event.target_name,
        "target": {
            "kind": match event.target.kind {
                NatalTargetKind::Graha => "graha",
                NatalTargetKind::Bindu => "bindu",
                NatalTargetKind::Sphuta => "sphuta",
                NatalTargetKind::SpecialLagna => "special_lagna",
                NatalTargetKind::ArudhaPada => "arudha_pada",
                NatalTargetKind::Custom => "custom"
            },
            "index": event.target.index,
            "name": event.target.name,
            "longitude_deg": event.target.longitude_deg
        },
        "aspect_kind": match event.aspect_kind {
            TransitAspectKind::Conjunction => "conjunction",
            TransitAspectKind::Opposition => "opposition",
            TransitAspectKind::Special => "special"
        },
        "aspect_owner": match event.aspect_owner {
            TransitAspectOwner::GocharBody => "gochar_body",
            TransitAspectOwner::NatalTarget => "natal_target"
        },
        "aspect_angle_deg": event.aspect_angle_deg,
        "utc": utc_json(event.utc),
        "jd_tdb": event.jd_tdb,
        "transit_longitude_deg": event.transit_longitude_deg,
        "target_longitude_deg": event.target_longitude_deg,
        "actual_separation_deg": event.actual_separation_deg
    })
}

fn gochar_events_json(result: GocharEventsResult) -> Value {
    json!({
        "birth_utc": utc_json(result.birth_utc),
        "at_utc": utc_json(result.at_utc),
        "reference": gochar_reference_json(result.reference),
        "yearly_tajaka": {
            "before": result.yearly_tajaka.before.into_iter().map(tajaka_return_event_json).collect::<Vec<_>>(),
            "after": result.yearly_tajaka.after.into_iter().map(tajaka_return_event_json).collect::<Vec<_>>()
        },
        "yearly_tithi_pravesha": {
            "before": result.yearly_tithi_pravesha.before.into_iter().map(tithi_pravesha_event_json).collect::<Vec<_>>(),
            "after": result.yearly_tithi_pravesha.after.into_iter().map(tithi_pravesha_event_json).collect::<Vec<_>>()
        },
        "monthly_tajaka": {
            "before": result.monthly_tajaka.before.into_iter().map(tajaka_return_event_json).collect::<Vec<_>>(),
            "after": result.monthly_tajaka.after.into_iter().map(tajaka_return_event_json).collect::<Vec<_>>()
        },
        "monthly_tithi_pravesha": {
            "before": result.monthly_tithi_pravesha.before.into_iter().map(tithi_pravesha_event_json).collect::<Vec<_>>(),
            "after": result.monthly_tithi_pravesha.after.into_iter().map(tithi_pravesha_event_json).collect::<Vec<_>>()
        },
        "transit_events": result.transit_events.into_iter().map(transit_to_natal_aspect_event_json).collect::<Vec<_>>()
    })
}

fn dasha_hierarchy_json(result: DashaHierarchy) -> Value {
    json!({
        "system": debug_name(result.system),
        "birth_jd": result.birth_jd,
        "levels": result.levels.into_iter().enumerate().map(|(idx, level)| json!({
            "level": idx,
            "name": debug_name(DashaLevel::from_u8(idx as u8).unwrap_or(DashaLevel::Mahadasha)),
            "periods": level.into_iter().map(dasha_period_json).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

fn dasha_snapshot_json(result: DashaSnapshot) -> Value {
    json!({
        "system": debug_name(result.system),
        "query_utc": utc_json_from_jd_utc(result.query_jd),
        "query_jd": result.query_jd,
        "periods": result.periods.into_iter().map(dasha_period_json).collect::<Vec<_>>()
    })
}

fn dasha_period_json(period: DashaPeriod) -> Value {
    json!({
        "entity": dasha_entity_json(period.entity),
        "start_utc": utc_json_from_jd_utc(period.start_jd),
        "start_jd": period.start_jd,
        "end_utc": utc_json_from_jd_utc(period.end_jd),
        "end_jd": period.end_jd,
        "level": debug_name(period.level),
        "order": period.order,
        "parent_idx": period.parent_idx
    })
}

fn utc_json_from_jd_utc(jd_utc: f64) -> Value {
    let (year, month, day_frac) = dhruv_time::jd_to_calendar(jd_utc);
    let day = day_frac.floor() as u32;
    let frac = day_frac.fract();
    let total_seconds = frac * 86_400.0;
    let hour = (total_seconds / 3600.0).floor() as u32;
    let minute = ((total_seconds % 3600.0) / 60.0).floor() as u32;
    let second = total_seconds % 60.0;
    json!({
        "year": year,
        "month": month,
        "day": day,
        "hour": hour,
        "minute": minute,
        "second": second
    })
}

fn dasha_entity_json(entity: DashaEntity) -> Value {
    match entity {
        DashaEntity::Graha(graha) => {
            json!({ "kind": "graha", "index": graha.index(), "name": graha.name() })
        }
        DashaEntity::Rashi(index) => {
            let name = dhruv_vedic_base::ALL_RASHIS
                .get(index as usize)
                .map(|rashi| rashi.name())
                .unwrap_or("Unknown");
            json!({ "kind": "rashi", "index": index, "name": name })
        }
        DashaEntity::Yogini(index) => {
            let name = DashaEntity::Yogini(index).name();
            json!({ "kind": "yogini", "index": index, "name": name })
        }
    }
}

fn tara_result_json(result: TaraResult) -> Value {
    match result {
        TaraResult::Equatorial(position) => json!({
            "output": "equatorial",
            "value": {
                "ra_deg": position.ra_deg,
                "dec_deg": position.dec_deg,
                "distance_au": position.distance_au
            }
        }),
        TaraResult::Ecliptic(coords) => json!({
            "output": "ecliptic",
            "value": {
                "lon_deg": coords.lon_deg,
                "lat_deg": coords.lat_deg,
                "distance_km": coords.distance_km
            }
        }),
        TaraResult::Sidereal(longitude) => json!({
            "output": "sidereal",
            "value": {
                "longitude_deg": longitude
            }
        }),
    }
}

fn handle_ephemeris(resource: &ResourceArc<EngineResource>, request: QueryInput) -> JsonResult {
    read_state(resource, |state| {
        let engine = require_engine(state)?;
        let output_mode = parse_query_output(request.output.as_ref())?;
        let query = Query {
            target: parse_body(&request.target)?,
            observer: parse_observer(&request.observer)?,
            frame: parse_frame(request.frame.as_ref().unwrap_or(&EnumInput::Int(0)))?,
            epoch_tdb_jd: query_epoch_tdb_jd(state, &request)?,
        };
        engine
            .query(query)
            .map(|state_vector| ephemeris_query_result_json(state_vector, output_mode))
            .map_err(|err| map_error("engine_error", err))
    })
}

fn handle_body_ecliptic_lon_lat(
    resource: &ResourceArc<EngineResource>,
    request: BodyLonLatInput,
) -> JsonResult {
    read_state(resource, |state| {
        let engine = require_engine(state)?;
        dhruv_search::body_ecliptic_lon_lat(engine, parse_body(&request.body)?, request.jd_tdb)
            .map(|(lon_deg, lat_deg)| json!({ "lon_deg": lon_deg, "lat_deg": lat_deg }))
            .map_err(|err| map_error("search_error", err))
    })
}

fn handle_time(resource: &ResourceArc<EngineResource>, request: TimeRunInput) -> JsonResult {
    read_state(resource, |state| {
        let engine = require_engine(state)?;
        match request.op.as_str() {
            "utc_to_jd_tdb" => {
                let utc = parse_utc(
                    request
                        .utc
                        .ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                )?;
                let policy = parse_time_policy_input(request.time_policy)?;
                let utc_seconds = jd_to_tdb_seconds(dhruv_time::calendar_to_jd(
                    utc.year,
                    utc.month,
                    utc.day as f64
                        + utc.hour as f64 / 24.0
                        + utc.minute as f64 / 1440.0
                        + utc.second / 86_400.0,
                ));
                let result = engine.lsk().utc_to_tdb_with_policy_and_eop(
                    utc_seconds,
                    state.eop.as_ref(),
                    policy,
                );
                Ok(json!({
                    "jd_tdb": tdb_seconds_to_jd(result.tdb_seconds),
                    "diagnostics": time_diagnostics_json(&result.diagnostics)
                }))
            }
            "jd_tdb_to_utc" => {
                let jd_tdb = request
                    .jd_tdb
                    .ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?;
                Ok(utc_json(UtcTime::from_jd_tdb(jd_tdb, engine.lsk())))
            }
            "nutation_utc" => {
                let utc = parse_utc(
                    request
                        .utc
                        .ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                )?;
                let jd_tdb = utc.to_jd_tdb(engine.lsk());
                let t = dhruv_vedic_base::jd_tdb_to_centuries(jd_tdb);
                let (dpsi_arcsec, deps_arcsec) = nutation_iau2000b(t);
                Ok(json!({ "dpsi_arcsec": dpsi_arcsec, "deps_arcsec": deps_arcsec }))
            }
            _ => Err(error_payload("invalid_request", "unknown time operation")),
        }
    })
}

fn handle_vedic(resource: &ResourceArc<EngineResource>, request: VedicRequest) -> JsonResult {
    read_state(resource, |state| {
        let engine = require_engine(state)?;
        apply_time_policy(state);
        match request.op.as_str() {
            "ayanamsha" => {
                let op = AyanamshaOperation {
                    system: parse_ayanamsha_system(request.system.as_ref())?,
                    mode: request
                        .mode
                        .as_ref()
                        .map(|input| match input {
                            EnumInput::Int(value) => AYANAMSHA_MODE_VARIANTS
                                .get(*value as usize)
                                .copied()
                                .ok_or_else(|| {
                                    error_payload("invalid_request", "unknown ayanamsha mode")
                                }),
                            EnumInput::Str(value) => parse_named(value, &AYANAMSHA_MODE_VARIANTS)
                                .ok_or_else(|| {
                                    error_payload("invalid_request", "unknown ayanamsha mode")
                                }),
                        })
                        .transpose()?
                        .unwrap_or(AyanamshaMode::Unified),
                    at_jd_tdb: request
                        .jd_tdb
                        .ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                    use_nutation: request
                        .sankranti_config
                        .as_ref()
                        .and_then(|config| config.use_nutation)
                        .unwrap_or(false),
                    delta_psi_arcsec: 0.0,
                };
                ayanamsha(&op)
                    .map(|value| json!({ "ayanamsha_deg": value }))
                    .map_err(|err| map_error("search_error", err))
            }
            "lunar_node" => {
                let op =
                    NodeOperation {
                        node: parse_lunar_node(request.system.as_ref().ok_or_else(|| {
                            error_payload("invalid_request", "node is required")
                        })?)?,
                        mode: parse_node_mode(request.mode.as_ref())?,
                        backend: parse_node_backend(request.backend.as_ref())?,
                        at_jd_tdb: request.jd_tdb.ok_or_else(|| {
                            error_payload("invalid_request", "jd_tdb is required")
                        })?,
                    };
                lunar_node(engine, &op)
                    .map(|value| json!({ "longitude_deg": value }))
                    .map_err(|err| map_error("search_error", err))
            }
            "rise_set" => {
                let eop = state.eop.as_ref().ok_or_else(|| {
                    error_payload("missing_eop", "rise/set requires loaded EOP data")
                })?;
                let utc = parse_utc(
                    request
                        .utc
                        .ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                )?;
                let location = parse_location(
                    request
                        .location
                        .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                );
                let config = to_riseset_config(state, request.config.as_ref())?;
                let event = parse_riseset_event(
                    request
                        .event
                        .as_ref()
                        .ok_or_else(|| error_payload("invalid_request", "event is required"))?,
                )?;
                let jd_utc_noon = approximate_local_noon_jd(
                    dhruv_time::calendar_to_jd(utc.year, utc.month, utc.day as f64),
                    location.longitude_deg,
                );
                compute_rise_set(
                    engine,
                    engine.lsk(),
                    eop,
                    &location,
                    event,
                    jd_utc_noon,
                    &config,
                )
                .map(rise_set_result_json)
                .map_err(|err| map_error("vedic_error", err))
            }
            "all_events" => {
                let eop = state.eop.as_ref().ok_or_else(|| {
                    error_payload("missing_eop", "rise/set requires loaded EOP data")
                })?;
                let utc = parse_utc(
                    request
                        .utc
                        .ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                )?;
                let location = parse_location(
                    request
                        .location
                        .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                );
                let config = to_riseset_config(state, request.config.as_ref())?;
                let jd_utc_noon = approximate_local_noon_jd(
                    dhruv_time::calendar_to_jd(utc.year, utc.month, utc.day as f64),
                    location.longitude_deg,
                );
                compute_all_events(engine, engine.lsk(), eop, &location, jd_utc_noon, &config)
                    .map(|events| json!({ "events": events.into_iter().map(rise_set_result_json).collect::<Vec<_>>() }))
                    .map_err(|err| map_error("vedic_error", err))
            }
            "lagna" | "mc" | "ramc" | "bhavas" => {
                let eop = state.eop.as_ref().ok_or_else(|| {
                    error_payload("missing_eop", "operation requires loaded EOP data")
                })?;
                let utc = parse_utc(
                    request
                        .utc
                        .ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                )?;
                let location = parse_location(
                    request
                        .location
                        .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                );
                let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
                let sidereal_output = request.sankranti_config.is_some();
                let sankranti_config = if sidereal_output {
                    Some(to_sankranti_config(
                        state,
                        request.sankranti_config.as_ref(),
                    )?)
                } else {
                    None
                };
                let jd_utc = dhruv_time::calendar_to_jd(
                    utc.year,
                    utc.month,
                    utc.day as f64
                        + utc.hour as f64 / 24.0
                        + utc.minute as f64 / 1440.0
                        + utc.second / 86_400.0,
                );
                match request.op.as_str() {
                    "lagna" if sidereal_output => sidereal_lagna_for_date(
                        engine,
                        eop,
                        &utc,
                        &location,
                        sankranti_config.as_ref().expect("sidereal config set"),
                    )
                    .map(|value| json!({ "longitude_deg": value }))
                    .map_err(|err| map_error("vedic_error", err)),
                    "lagna" => lagna_longitude_rad(engine.lsk(), eop, &location, jd_utc)
                        .map(|value| json!({ "longitude_deg": value.to_degrees() }))
                        .map_err(|err| map_error("vedic_error", err)),
                    "mc" if sidereal_output => sidereal_mc_for_date(
                        engine,
                        eop,
                        &utc,
                        &location,
                        &bhava_config,
                        sankranti_config.as_ref().expect("sidereal config set"),
                    )
                    .map(|value| json!({ "longitude_deg": value }))
                    .map_err(|err| map_error("vedic_error", err)),
                    "mc" => mc_longitude_rad(engine.lsk(), eop, &location, jd_utc)
                        .map(|value| json!({ "longitude_deg": value.to_degrees() }))
                        .map_err(|err| map_error("vedic_error", err)),
                    "ramc" => ramc_rad(engine.lsk(), eop, &location, jd_utc)
                        .map(|value| json!({ "longitude_deg": value.to_degrees() }))
                        .map_err(|err| map_error("vedic_error", err)),
                    _ if sidereal_output => sidereal_bhavas_for_date(
                        engine,
                        eop,
                        &utc,
                        &location,
                        &bhava_config,
                        sankranti_config.as_ref().expect("sidereal config set"),
                    )
                    .map(|result| {
                        bhava_result_json_with_rashi(
                            result,
                            bhava_config.include_rashi_bhava_results,
                        )
                    })
                    .map_err(|err| map_error("vedic_error", err)),
                    _ => {
                        compute_bhavas(engine, engine.lsk(), eop, &location, jd_utc, &bhava_config)
                            .map(|result| {
                                bhava_result_json_with_rashi(
                                    result,
                                    bhava_config.include_rashi_bhava_results,
                                )
                            })
                            .map_err(|err| map_error("vedic_error", err))
                    }
                }
            }
            _ => Err(error_payload("invalid_request", "unknown vedic operation")),
        }
    })
}

fn handle_panchang(resource: &ResourceArc<EngineResource>, request: PanchangRequest) -> JsonResult {
    read_state(resource, |state| {
        let engine = require_engine(state)?;
        let eop = state
            .eop
            .as_ref()
            .ok_or_else(|| error_payload("missing_eop", "panchang requires loaded EOP data"))?;
        apply_time_policy(state);
        let utc = request.utc.clone().map(parse_utc).transpose()?;
        let location = request.location.map(parse_location);
        let location_or_default = location.unwrap_or(GeoLocation {
            latitude_deg: 0.0,
            longitude_deg: 0.0,
            altitude_m: 0.0,
        });
        let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
        let sankranti_config = to_sankranti_config(state, request.sankranti_config.as_ref())?;
        let result = match request.op.as_str() {
            "tithi" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "tithi": tithi_json(dhruv_search::tithi_for_date(engine, utc).map_err(|err| map_error("search_error", err))?) })
            }
            "karana" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "karana": karana_json(dhruv_search::karana_for_date(engine, utc).map_err(|err| map_error("search_error", err))?) })
            }
            "yoga" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "yoga": yoga_json(dhruv_search::yoga_for_date(engine, utc, &sankranti_config).map_err(|err| map_error("search_error", err))?) })
            }
            "nakshatra" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "nakshatra": nakshatra_json(dhruv_search::nakshatra_for_date(engine, utc, &sankranti_config).map_err(|err| map_error("search_error", err))?) })
            }
            "vaar" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "vaar": vaar_json(dhruv_search::vaar_for_date(engine, eop, utc, &location_or_default, &riseset_config).map_err(|err| map_error("search_error", err))?) })
            }
            "hora" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "hora": hora_json(dhruv_search::hora_for_date(engine, eop, utc, &location_or_default, &riseset_config).map_err(|err| map_error("search_error", err))?) })
            }
            "ghatika" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "ghatika": ghatika_json(dhruv_search::ghatika_for_date(engine, eop, utc, &location_or_default, &riseset_config).map_err(|err| map_error("search_error", err))?) })
            }
            "masa" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "masa": masa_json(dhruv_search::masa_for_date(engine, utc, &sankranti_config).map_err(|err| map_error("search_error", err))?) })
            }
            "ayana" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "ayana": ayana_json(dhruv_search::ayana_for_date(engine, utc, &sankranti_config).map_err(|err| map_error("search_error", err))?) })
            }
            "varsha" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                json!({ "varsha": varsha_json(dhruv_search::varsha_for_date(engine, utc, &sankranti_config).map_err(|err| map_error("search_error", err))?) })
            }
            "daily" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                // Default preserves the historical "daily" payload and cost:
                // core elements only, calendar elements opt-in via the mask.
                let include_mask = match request.include_mask.as_ref() {
                    Some(input) => parse_panchang_include_mask(input)?,
                    None => PANCHANG_INCLUDE_ALL_CORE,
                };
                let op = PanchangOperation {
                    at_utc: *utc,
                    location,
                    riseset_config,
                    sankranti_config,
                    include_mask,
                    known: to_panchang_precomputed(&request)?,
                };
                let result =
                    panchang(engine, eop, &op).map_err(|err| map_error("search_error", err))?;
                panchang_value_json(&result)
            }
            "events" => handle_panchang_events(
                engine,
                eop,
                &request,
                location.as_ref(),
                &riseset_config,
                &sankranti_config,
            )?,
            "elongation_at" => json!({
                "value": elongation_at(
                    engine,
                    request.jd_tdb.ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                )
                .map_err(|err| map_error("search_error", err))?
            }),
            "sidereal_sum_at" => json!({
                "value": sidereal_sum_at(
                    engine,
                    request.jd_tdb.ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                    &sankranti_config,
                )
                .map_err(|err| map_error("search_error", err))?
            }),
            "vedic_day_sunrises" => {
                let utc = utc
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                let (sunrise_jd, next_sunrise_jd) =
                    vedic_day_sunrises(engine, eop, utc, &location_or_default, &riseset_config)
                        .map_err(|err| map_error("search_error", err))?;
                json!({
                    "sunrise_jd": sunrise_jd,
                    "next_sunrise_jd": next_sunrise_jd
                })
            }
            "body_ecliptic_lon_lat" => {
                let body = parse_body(
                    request
                        .body
                        .as_ref()
                        .ok_or_else(|| error_payload("invalid_request", "body is required"))?,
                )?;
                let (lon_deg, lat_deg) = body_ecliptic_lon_lat(
                    engine,
                    body,
                    request
                        .jd_tdb
                        .ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                )
                .map_err(|err| map_error("search_error", err))?;
                json!({ "lon_deg": lon_deg, "lat_deg": lat_deg })
            }
            "tithi_at" => json!({
                "tithi": tithi_json(tithi_at(
                    engine,
                    request.jd_tdb.ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                    request.sunrise_jd.ok_or_else(|| error_payload("invalid_request", "sunrise_jd is required"))?,
                )
                .map_err(|err| map_error("search_error", err))?)
            }),
            "karana_at" => json!({
                "karana": karana_json(karana_at(
                    engine,
                    request.jd_tdb.ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                    request.sunrise_jd.ok_or_else(|| error_payload("invalid_request", "sunrise_jd is required"))?,
                )
                .map_err(|err| map_error("search_error", err))?)
            }),
            "yoga_at" => json!({
                "yoga": yoga_json(yoga_at(
                    engine,
                    request.jd_tdb.ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                    request.sunrise_jd.ok_or_else(|| error_payload("invalid_request", "sunrise_jd is required"))?,
                    &sankranti_config,
                )
                .map_err(|err| map_error("search_error", err))?)
            }),
            "nakshatra_at" => json!({
                "nakshatra": nakshatra_json(nakshatra_at(
                    engine,
                    request.jd_tdb.ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                    request.moon_sidereal_deg.ok_or_else(|| error_payload("invalid_request", "moon_sidereal_deg is required"))?,
                    &sankranti_config,
                )
                .map_err(|err| map_error("search_error", err))?)
            }),
            "vaar_from_sunrises" => json!({
                "vaar": vaar_json(vaar_from_sunrises(
                    request.sunrise_jd.ok_or_else(|| error_payload("invalid_request", "sunrise_jd is required"))?,
                    request.next_sunrise_jd.ok_or_else(|| error_payload("invalid_request", "next_sunrise_jd is required"))?,
                    engine.lsk(),
                ))
            }),
            "hora_from_sunrises" => json!({
                "hora": hora_json(hora_from_sunrises(
                    request.query_jd.ok_or_else(|| error_payload("invalid_request", "query_jd is required"))?,
                    request.sunrise_jd.ok_or_else(|| error_payload("invalid_request", "sunrise_jd is required"))?,
                    request.next_sunrise_jd.ok_or_else(|| error_payload("invalid_request", "next_sunrise_jd is required"))?,
                    engine.lsk(),
                ))
            }),
            "ghatika_from_sunrises" => json!({
                "ghatika": ghatika_json(ghatika_from_sunrises(
                    request.query_jd.ok_or_else(|| error_payload("invalid_request", "query_jd is required"))?,
                    request.sunrise_jd.ok_or_else(|| error_payload("invalid_request", "sunrise_jd is required"))?,
                    request.next_sunrise_jd.ok_or_else(|| error_payload("invalid_request", "next_sunrise_jd is required"))?,
                    engine.lsk(),
                ))
            }),
            _ => {
                return Err(error_payload(
                    "invalid_request",
                    "unknown panchang operation",
                ));
            }
        };
        Ok(result)
    })
}

fn handle_search(resource: &ResourceArc<EngineResource>, request: SearchRequest) -> JsonResult {
    read_state(resource, |state| {
        let engine = require_engine(state)?;
        apply_time_policy(state);
        match request.op.as_str() {
            "conjunction" => {
                let query = match request.mode {
                    EnumInput::Str(ref value) if value == "range" => {
                        let (start_jd_tdb, end_jd_tdb) = search_range_jd_tdb(engine, &request)?;
                        ConjunctionQuery::Range {
                            start_jd_tdb,
                            end_jd_tdb,
                        }
                    }
                    EnumInput::Str(ref value) if value == "prev" => ConjunctionQuery::Prev {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                    EnumInput::Int(1) => ConjunctionQuery::Prev {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                    EnumInput::Int(2) => {
                        let (start_jd_tdb, end_jd_tdb) = search_range_jd_tdb(engine, &request)?;
                        ConjunctionQuery::Range {
                            start_jd_tdb,
                            end_jd_tdb,
                        }
                    }
                    _ => ConjunctionQuery::Next {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                };
                let op =
                    ConjunctionOperation {
                        body1: parse_transit_body(request.body1.as_ref().ok_or_else(|| {
                            error_payload("invalid_request", "body1 is required")
                        })?)?,
                        body2: parse_transit_body(request.body2.as_ref().ok_or_else(|| {
                            error_payload("invalid_request", "body2 is required")
                        })?)?,
                        config: to_conjunction_config(state, request.config.as_ref())?,
                        target_separations_deg: request
                            .config
                            .as_ref()
                            .and_then(|config| config.target_separations_deg.clone())
                            .unwrap_or_default(),
                        sankranti_config: request
                            .sankranti_config
                            .as_ref()
                            .map(|input| to_sankranti_config(state, Some(input)))
                            .transpose()?,
                        query,
                    };
                conjunction(engine, &op)
                    .map(conjunction_result_json)
                    .map_err(|err| map_error("search_error", err))
            }
            "grahan" => {
                let query = match request.mode {
                    EnumInput::Str(ref value) if value == "range" => {
                        let (start_jd_tdb, end_jd_tdb) = search_range_jd_tdb(engine, &request)?;
                        GrahanQuery::Range {
                            start_jd_tdb,
                            end_jd_tdb,
                        }
                    }
                    EnumInput::Str(ref value) if value == "prev" => GrahanQuery::Prev {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                    _ => GrahanQuery::Next {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                };
                let op =
                    GrahanOperation {
                        kind: parse_grahan_kind(request.kind.as_ref().ok_or_else(|| {
                            error_payload("invalid_request", "kind is required")
                        })?)?,
                        config: to_grahan_config(state, request.config.as_ref()),
                        location: request.location.map(|location| {
                            dhruv_search::GeoLocation::new(
                                location.latitude_deg,
                                location.longitude_deg,
                                location.altitude_m.unwrap_or(0.0),
                            )
                        }),
                        query,
                    };
                dhruv_search::grahan(engine, state.eop.as_ref(), &op)
                    .map(|result| grahan_result_json(result, &op.config))
                    .map_err(|err| map_error("search_error", err))
            }
            "lunar_phase" => {
                let query = match request.mode {
                    EnumInput::Str(ref value) if value == "range" => {
                        let (start_jd_tdb, end_jd_tdb) = search_range_jd_tdb(engine, &request)?;
                        LunarPhaseQuery::Range {
                            start_jd_tdb,
                            end_jd_tdb,
                        }
                    }
                    EnumInput::Str(ref value) if value == "prev" => LunarPhaseQuery::Prev {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                    _ => LunarPhaseQuery::Next {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                };
                let op =
                    LunarPhaseOperation {
                        kind: parse_lunar_phase_kind(request.kind.as_ref().ok_or_else(|| {
                            error_payload("invalid_request", "kind is required")
                        })?)?,
                        sankranti_config: request
                            .sankranti_config
                            .as_ref()
                            .map(|input| to_sankranti_config(state, Some(input)))
                            .transpose()?,
                        query,
                    };
                dhruv_search::lunar_phase(engine, &op)
                    .map(lunar_phase_result_json)
                    .map_err(|err| map_error("search_error", err))
            }
            "sankranti" => {
                let query = match request.mode {
                    EnumInput::Str(ref value) if value == "range" => {
                        let (start_jd_tdb, end_jd_tdb) = search_range_jd_tdb(engine, &request)?;
                        SankrantiQuery::Range {
                            start_jd_tdb,
                            end_jd_tdb,
                        }
                    }
                    EnumInput::Str(ref value) if value == "prev" => SankrantiQuery::Prev {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                    _ => SankrantiQuery::Next {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                };
                let target = match request.target.as_ref() {
                    None => SankrantiTarget::Any,
                    Some(EnumInput::Int(value)) => SankrantiTarget::SpecificRashi(
                        dhruv_vedic_base::ALL_RASHIS
                            .get(*value as usize)
                            .copied()
                            .ok_or_else(|| {
                                error_payload("invalid_request", "unknown rashi target")
                            })?,
                    ),
                    Some(EnumInput::Str(value)) => {
                        let rashi =
                            parse_named(value, &dhruv_vedic_base::ALL_RASHIS).ok_or_else(|| {
                                error_payload("invalid_request", "unknown rashi target")
                            })?;
                        SankrantiTarget::SpecificRashi(rashi)
                    }
                };
                let body = match request.body.as_ref() {
                    Some(input) => parse_transit_body(input)?,
                    None => TransitBody::Body(Body::Sun),
                };
                let mut config = to_sankranti_config(state, request.sankranti_config.as_ref())?;
                let step_given = request
                    .sankranti_config
                    .as_ref()
                    .and_then(|input| input.step_size_days)
                    .is_some();
                if !step_given && body != TransitBody::Body(Body::Sun) {
                    config.step_size_days = body.default_ingress_step_days();
                }
                let op = SankrantiOperation {
                    body,
                    target,
                    config,
                    query,
                };
                dhruv_search::sankranti(engine, &op)
                    .map(sankranti_result_json)
                    .map_err(|err| map_error("search_error", err))
            }
            "motion" => {
                let query = match request.mode {
                    EnumInput::Str(ref value) if value == "range" => {
                        let (start_jd_tdb, end_jd_tdb) = search_range_jd_tdb(engine, &request)?;
                        MotionQuery::Range {
                            start_jd_tdb,
                            end_jd_tdb,
                        }
                    }
                    EnumInput::Str(ref value) if value == "prev" => MotionQuery::Prev {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                    _ => MotionQuery::Next {
                        at_jd_tdb: search_at_jd_tdb(engine, &request)?,
                    },
                };
                let op =
                    MotionOperation {
                        body: parse_transit_body(request.body.as_ref().ok_or_else(|| {
                            error_payload("invalid_request", "body is required")
                        })?)?,
                        kind: parse_motion_kind(request.kind.as_ref().ok_or_else(|| {
                            error_payload("invalid_request", "kind is required")
                        })?)?,
                        config: to_stationary_config(state, request.config.as_ref())?,
                        sankranti_config: request
                            .sankranti_config
                            .as_ref()
                            .map(|input| to_sankranti_config(state, Some(input)))
                            .transpose()?,
                        query,
                    };
                motion(engine, &op)
                    .map(motion_result_json)
                    .map_err(|err| map_error("search_error", err))
            }
            "gochar_events" => {
                let birth_utc =
                    parse_utc(request.birth_utc.clone().ok_or_else(|| {
                        error_payload("invalid_request", "birth_utc is required")
                    })?)?;
                let at_utc = parse_utc(
                    request
                        .at_utc
                        .clone()
                        .ok_or_else(|| error_payload("invalid_request", "at_utc is required"))?,
                )?;
                let location = parse_location(
                    request
                        .location
                        .clone()
                        .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                );
                let eop = state.eop.as_ref().ok_or_else(|| {
                    error_payload("missing_eop", "gochar_events requires loaded EOP data")
                })?;
                let transit_bodies = request
                    .transit_bodies
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|input| parse_transit_body(&input))
                    .collect::<Result<Vec<_>, _>>()?;
                let natal_targets = request
                    .natal_targets
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|target| {
                        Ok(NatalTargetLongitude {
                            kind: parse_natal_target_kind(&target.kind)?,
                            index: target.index,
                            name: target.name.unwrap_or_default(),
                            longitude_deg: target.longitude_deg,
                        })
                    })
                    .collect::<Result<Vec<_>, Value>>()?;
                let op = GocharEventsOperation {
                    birth_utc,
                    at_utc,
                    location,
                    eop,
                    bhava_config: to_bhava_config(state, request.bhava_config.as_ref())?,
                    riseset_config: to_riseset_config(state, request.riseset_config.as_ref())?,
                    sankranti_config: to_sankranti_config(
                        state,
                        request.sankranti_config.as_ref(),
                    )?,
                    kundali_config: to_full_kundali_config(state, request.kundali_config.as_ref())?,
                    config: to_gochar_events_config(request.gochar_config.as_ref())?,
                    transit_bodies,
                    natal_targets,
                };
                gochar_events(engine, &op)
                    .map(gochar_events_json)
                    .map_err(|err| map_error("search_error", err))
            }
            "charakaraka_events" => {
                let eop = state.eop.as_ref().ok_or_else(|| {
                    error_payload("missing_eop", "charakaraka_events requires loaded EOP data")
                })?;
                let scheme = parse_charakaraka_scheme(
                    request
                        .charakaraka_config
                        .as_ref()
                        .and_then(|config| config.scheme.as_ref()),
                )?;
                let sankranti_config =
                    to_sankranti_config(state, request.sankranti_config.as_ref())?;
                match request.mode {
                    EnumInput::Str(ref value) if value == "range" => {
                        let (from_utc, to_utc) = search_range_utc(engine, &request)?;
                        charakaraka_events(
                            engine,
                            eop,
                            &from_utc,
                            &to_utc,
                            &sankranti_config,
                            scheme,
                            request.max_events.unwrap_or(0),
                        )
                        .map(charakaraka_events_json)
                        .map_err(|err| map_error("search_error", err))
                    }
                    EnumInput::Str(ref value) if value == "prev" => {
                        let at_utc = search_at_utc(engine, &request)?;
                        prev_charakaraka_event(engine, eop, &at_utc, &sankranti_config, scheme)
                            .map(|event| {
                                json!({ "event": event.map(charakaraka_change_event_json) })
                            })
                            .map_err(|err| map_error("search_error", err))
                    }
                    _ => {
                        let at_utc = search_at_utc(engine, &request)?;
                        next_charakaraka_event(engine, eop, &at_utc, &sankranti_config, scheme)
                            .map(|event| {
                                json!({ "event": event.map(charakaraka_change_event_json) })
                            })
                            .map_err(|err| map_error("search_error", err))
                    }
                }
            }
            _ => Err(error_payload("invalid_request", "unknown search operation")),
        }
    })
}

/// Decode a required `[from_utc, to_utc]` range from a jyotish request.
fn jyotish_range_utc(request: &JyotishRequest) -> Result<(UtcTime, UtcTime), Value> {
    let from_utc = parse_utc(
        request
            .from_utc
            .ok_or_else(|| error_payload("invalid_request", "from_utc is required"))?,
    )?;
    let to_utc = parse_utc(
        request
            .to_utc
            .ok_or_else(|| error_payload("invalid_request", "to_utc is required"))?,
    )?;
    Ok((from_utc, to_utc))
}

fn handle_amsha_series(
    engine: &Engine,
    eop: &EopKernel,
    request: &JyotishRequest,
    location: Option<GeoLocation>,
    sankranti_config: &SankrantiConfig,
) -> JsonResult {
    let (from_utc, to_utc) = jyotish_range_utc(request)?;
    let step_minutes = request
        .step_minutes
        .ok_or_else(|| error_payload("invalid_request", "step_minutes is required"))?;
    let requests = to_amsha_requests(request.amsha_requests.as_deref())?;
    let series = amsha_series(
        engine,
        eop,
        &from_utc,
        &to_utc,
        step_minutes,
        &location.ok_or_else(|| error_payload("invalid_request", "location is required"))?,
        sankranti_config,
        &requests,
        request.include_grahas.unwrap_or(false),
    )
    .map_err(|err| map_error("search_error", err))?;
    Ok(amsha_series_json(series))
}

fn handle_amsha_lagna_events(
    engine: &Engine,
    eop: &EopKernel,
    request: &JyotishRequest,
    location: Option<GeoLocation>,
    sankranti_config: &SankrantiConfig,
) -> JsonResult {
    let (from_utc, to_utc) = jyotish_range_utc(request)?;
    let requests = to_amsha_requests(request.amsha_requests.as_deref())?;
    let result = amsha_lagna_events(
        engine,
        eop,
        &from_utc,
        &to_utc,
        &location.ok_or_else(|| error_payload("invalid_request", "location is required"))?,
        sankranti_config,
        &requests,
        request.max_segments.unwrap_or(0),
    )
    .map_err(|err| map_error("search_error", err))?;
    Ok(amsha_lagna_events_json(result))
}

fn handle_panchang_events(
    engine: &Engine,
    eop: &EopKernel,
    request: &PanchangRequest,
    location: Option<&GeoLocation>,
    riseset_config: &RiseSetConfig,
    sankranti_config: &SankrantiConfig,
) -> JsonResult {
    let from_utc = parse_utc(
        request
            .from_utc
            .ok_or_else(|| error_payload("invalid_request", "from_utc is required"))?,
    )?;
    let to_utc = parse_utc(
        request
            .to_utc
            .ok_or_else(|| error_payload("invalid_request", "to_utc is required"))?,
    )?;
    // Default preserves the historical payload: location-independent elements
    // only. Location-dependent elements (vaar/hora/ghatika) are opt-in via the
    // mask and validated by the engine (location required when selected).
    let include_mask = match request.include_mask.as_ref() {
        Some(input) => parse_panchang_include_mask(input)?,
        None => PANCHANG_INCLUDE_LOCATION_INDEPENDENT,
    };
    let result = panchang_events(
        engine,
        eop,
        &from_utc,
        &to_utc,
        include_mask,
        location,
        riseset_config,
        sankranti_config,
        request.max_events.unwrap_or(0),
    )
    .map_err(|err| map_error("search_error", err))?;
    Ok(panchang_events_json(result))
}

fn handle_jyotish(resource: &ResourceArc<EngineResource>, request: JyotishRequest) -> JsonResult {
    read_state(resource, |state| {
        let engine = require_engine(state)?;
        let eop = state.eop.as_ref().ok_or_else(|| {
            error_payload("missing_eop", "jyotish operations require loaded EOP data")
        })?;
        apply_time_policy(state);
        let utc = request.utc.map(parse_utc).transpose()?;
        let location = request.location.map(parse_location);
        let sankranti_config = to_sankranti_config(state, request.sankranti_config.as_ref())?;
        let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
        match request.op.as_str() {
            "graha_longitudes" => {
                let kind = parse_graha_longitude_kind(request.kind.as_ref())?;
                let system = request
                    .system
                    .as_ref()
                    .map(|_| parse_ayanamsha_system(request.system.as_ref()))
                    .transpose()?
                    .unwrap_or(sankranti_config.ayanamsha_system);
                let config = match kind {
                    GrahaLongitudeKind::Sidereal => GrahaLongitudesConfig::sidereal_with_model(
                        system,
                        sankranti_config.use_nutation,
                        sankranti_config.precession_model,
                        sankranti_config.reference_plane,
                    ),
                    GrahaLongitudeKind::Tropical => GrahaLongitudesConfig::tropical_with_model(
                        sankranti_config.use_nutation,
                        sankranti_config.precession_model,
                        sankranti_config.reference_plane,
                    ),
                };
                let jd_tdb = request
                    .jd_tdb
                    .ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?;
                let navagrahas = graha_longitudes(engine, jd_tdb, &config)
                    .map_err(|err| map_error("search_error", err))?;
                Ok(graha_longitudes_json(navagrahas))
            }
            "moving_osculating_apogees" => {
                let system = request
                    .system
                    .as_ref()
                    .map(|_| parse_ayanamsha_system(request.system.as_ref()))
                    .transpose()?
                    .unwrap_or(sankranti_config.ayanamsha_system);
                let config = GrahaLongitudesConfig::sidereal_with_model(
                    system,
                    sankranti_config.use_nutation,
                    sankranti_config.precession_model,
                    sankranti_config.reference_plane,
                );
                let grahas = request
                    .grahas
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "grahas is required"))?
                    .iter()
                    .map(parse_graha)
                    .collect::<Result<Vec<_>, _>>()?;
                moving_osculating_apogees_for_date(
                    engine,
                    Some(eop),
                    &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                    &config,
                    &grahas,
                )
                .map(moving_osculating_apogees_json)
                .map_err(|err| map_error("search_error", err))
            }
            "graha_positions" => {
                let positions = graha_positions_fn(
                    engine,
                    eop,
                    &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                    &location
                        .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                    &bhava_config,
                    &sankranti_config,
                    &to_graha_positions_config(state, request.graha_positions_config.as_ref())?,
                )
                .map_err(|err| map_error("search_error", err))?;
                Ok(graha_positions_json(positions))
            }
            "graha_positions_series" => {
                let from_utc =
                    parse_utc(request.from_utc.clone().ok_or_else(|| {
                        error_payload("invalid_request", "from_utc is required")
                    })?)?;
                let to_utc = parse_utc(
                    request
                        .to_utc
                        .clone()
                        .ok_or_else(|| error_payload("invalid_request", "to_utc is required"))?,
                )?;
                let step_minutes = request
                    .step_minutes
                    .ok_or_else(|| error_payload("invalid_request", "step_minutes is required"))?;
                let series = dhruv_search::graha_positions_series(
                    engine,
                    eop,
                    &from_utc,
                    &to_utc,
                    step_minutes,
                    &location
                        .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                    &bhava_config,
                    &sankranti_config,
                    &to_graha_positions_config(state, request.graha_positions_config.as_ref())?,
                )
                .map_err(|err| map_error("search_error", err))?;
                Ok(json!({
                    "points": series
                        .points
                        .into_iter()
                        .map(|point| {
                            json!({
                                "utc": utc_json(point.utc),
                                "jd_utc": point.jd_utc,
                                "positions": graha_positions_json(point.positions),
                            })
                        })
                        .collect::<Vec<_>>()
                }))
            }
            "special_lagnas" => special_lagnas_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &to_riseset_config(state, request.riseset_config.as_ref())?,
                &sankranti_config,
            )
            .map(special_lagnas_json)
            .map_err(|err| map_error("search_error", err)),
            "arudha" => arudha_padas_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &bhava_config,
                &sankranti_config,
            )
            .map(arudha_json)
            .map_err(|err| map_error("search_error", err)),
            "upagrahas" => {
                let utc = utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?;
                let location = location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?;
                let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
                let result = if let Some(input) = request.upagraha_config.as_ref() {
                    let mut upagraha_config = TimeUpagrahaConfig::default();
                    apply_time_upagraha_config(&mut upagraha_config, Some(input))?;
                    all_upagrahas_for_date_with_config(
                        engine,
                        eop,
                        &utc,
                        &location,
                        &riseset_config,
                        &sankranti_config,
                        &upagraha_config,
                    )
                } else {
                    all_upagrahas_for_date(
                        engine,
                        eop,
                        &utc,
                        &location,
                        &riseset_config,
                        &sankranti_config,
                    )
                };
                result
                    .map(upagrahas_json)
                    .map_err(|err| map_error("search_error", err))
            }
            "bindus" => core_bindus(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &bhava_config,
                &to_riseset_config(state, request.riseset_config.as_ref())?,
                &sankranti_config,
                &to_bindus_config(state, request.bindus_config.as_ref())?,
            )
            .map(bindus_json)
            .map_err(|err| map_error("search_error", err)),
            "ashtakavarga" => ashtakavarga_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &sankranti_config,
            )
            .map(ashtakavarga_json)
            .map_err(|err| map_error("search_error", err)),
            "drishti" => drishti_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &bhava_config,
                &to_riseset_config(state, request.riseset_config.as_ref())?,
                &sankranti_config,
                &to_drishti_config(state, request.drishti_config.as_ref()),
            )
            .map(drishti_json)
            .map_err(|err| map_error("search_error", err)),
            "charakaraka" => charakaraka_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &sankranti_config,
                parse_charakaraka_scheme(request.scheme.as_ref())?,
            )
            .map(charakaraka_json)
            .map_err(|err| map_error("search_error", err)),
            "shadbala" => shadbala_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &bhava_config,
                &to_riseset_config(state, request.riseset_config.as_ref())?,
                &sankranti_config,
                &to_amsha_selection(request.amsha_selection.as_deref())?,
            )
            .map(shadbala_json)
            .map_err(|err| map_error("search_error", err)),
            "bhavabala" => bhavabala_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &bhava_config,
                &to_riseset_config(state, request.riseset_config.as_ref())?,
                &sankranti_config,
            )
            .map(bhavabala_json)
            .map_err(|err| map_error("search_error", err)),
            "vimsopaka" => vimsopaka_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &sankranti_config,
                parse_node_dignity_policy(request.node_dignity_policy.as_ref())?,
                &to_amsha_selection(request.amsha_selection.as_deref())?,
            )
            .map(vimsopaka_json)
            .map_err(|err| map_error("search_error", err)),
            "balas" => balas_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &bhava_config,
                &to_riseset_config(state, request.riseset_config.as_ref())?,
                &sankranti_config,
                parse_node_dignity_policy(request.node_dignity_policy.as_ref())?,
                &to_amsha_selection(request.amsha_selection.as_deref())?,
            )
            .map(|result| {
                json!({
                    "shadbala": shadbala_json(result.shadbala),
                    "vimsopaka": vimsopaka_json(result.vimsopaka),
                    "ashtakavarga": ashtakavarga_json(result.ashtakavarga),
                    "bhavabala": bhavabala_json(result.bhavabala)
                })
            })
            .map_err(|err| map_error("search_error", err)),
            "avastha" => avastha_for_date(
                engine,
                eop,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &bhava_config,
                &to_riseset_config(state, request.riseset_config.as_ref())?,
                &sankranti_config,
                parse_node_dignity_policy(request.node_dignity_policy.as_ref())?,
                &to_amsha_selection(request.amsha_selection.as_deref())?,
            )
            .map(avastha_json)
            .map_err(|err| map_error("search_error", err)),
            "full_kundali" => full_kundali_for_date(
                engine,
                eop,
                &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                &location
                    .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                &bhava_config,
                &to_riseset_config(state, request.riseset_config.as_ref())?,
                &sankranti_config,
                &to_full_kundali_config(state, request.full_kundali_config.as_ref())?,
            )
            .map(full_kundali_json)
            .map_err(|err| map_error("search_error", err)),
            "amsha_series" => {
                handle_amsha_series(engine, eop, &request, location, &sankranti_config)
            }
            "amsha_lagna_events" => {
                handle_amsha_lagna_events(engine, eop, &request, location, &sankranti_config)
            }
            "amsha" => {
                let scope = to_amsha_scope(request.amsha_scope.as_ref());
                let requests = to_amsha_requests(request.amsha_requests.as_deref())?;
                amsha_charts_for_date(
                    engine,
                    eop,
                    &utc.ok_or_else(|| error_payload("invalid_request", "utc is required"))?,
                    &location
                        .ok_or_else(|| error_payload("invalid_request", "location is required"))?,
                    &bhava_config,
                    &to_riseset_config(state, request.riseset_config.as_ref())?,
                    &sankranti_config,
                    &requests,
                    &scope,
                )
                .map(amsha_result_json)
                .map_err(|err| map_error("search_error", err))
            }
            _ => Err(error_payload(
                "invalid_request",
                "unknown jyotish operation",
            )),
        }
    })
}

fn handle_dasha(resource: &ResourceArc<EngineResource>, request: DashaRequest) -> JsonResult {
    read_state(resource, |state| {
        let engine = require_engine(state)?;
        let eop = state.eop.as_ref().ok_or_else(|| {
            error_payload("missing_eop", "dasha operations require loaded EOP data")
        })?;
        apply_time_policy(state);
        let system = parse_dasha_system(&request.system)?;
        let variation = to_dasha_variation(request.variation.as_ref())?;
        let raw_inputs = parse_dasha_inputs(request.inputs.as_ref())?;
        let birth_jd = match (request.birth_jd, request.birth_utc.as_ref()) {
            (Some(jd), _) => jd,
            (None, Some(utc)) if raw_inputs.is_some() => utc_to_jd_utc(&parse_utc(utc.clone())?),
            (None, Some(_)) => 0.0,
            (None, None) => {
                return Err(error_payload(
                    "invalid_request",
                    "birth_utc or birth_jd is required",
                ));
            }
        };
        match request.op.as_str() {
            "hierarchy" => {
                if let Some(inputs) = raw_inputs.as_ref() {
                    let inputs = inputs.borrowed();
                    dasha_hierarchy_with_inputs(
                        birth_jd,
                        system,
                        request.max_level.unwrap_or(2),
                        &variation,
                        &inputs,
                    )
                    .map(dasha_hierarchy_json)
                    .map_err(|err| map_error("search_error", err))
                } else {
                    let birth_utc = parse_utc(request.birth_utc.ok_or_else(|| {
                        error_payload("invalid_request", "birth_utc is required")
                    })?)?;
                    let location =
                        parse_location(request.location.ok_or_else(|| {
                            error_payload("invalid_request", "location is required")
                        })?);
                    let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
                    let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
                    let sankranti_config =
                        to_sankranti_config(state, request.sankranti_config.as_ref())?;
                    dasha_hierarchy_for_birth(
                        engine,
                        eop,
                        &birth_utc,
                        &location,
                        system,
                        request.max_level.unwrap_or(2),
                        &bhava_config,
                        &riseset_config,
                        &sankranti_config,
                        &variation,
                    )
                    .map(dasha_hierarchy_json)
                    .map_err(|err| map_error("search_error", err))
                }
            }
            "snapshot" => {
                if let Some(inputs) = raw_inputs.as_ref() {
                    let query_jd = match (request.query_jd, request.query_utc.as_ref()) {
                        (Some(jd), _) => jd,
                        (None, Some(utc)) => utc_to_jd_utc(&parse_utc(utc.clone())?),
                        (None, None) => {
                            return Err(error_payload(
                                "invalid_request",
                                "query_utc or query_jd is required",
                            ));
                        }
                    };
                    let inputs = inputs.borrowed();
                    dasha_snapshot_with_inputs(
                        birth_jd,
                        query_jd,
                        system,
                        request.max_level.unwrap_or(2),
                        &variation,
                        &inputs,
                    )
                    .map(dasha_snapshot_json)
                    .map_err(|err| map_error("search_error", err))
                } else {
                    let birth_utc = parse_utc(request.birth_utc.ok_or_else(|| {
                        error_payload("invalid_request", "birth_utc is required")
                    })?)?;
                    let location =
                        parse_location(request.location.ok_or_else(|| {
                            error_payload("invalid_request", "location is required")
                        })?);
                    let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
                    let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
                    let sankranti_config =
                        to_sankranti_config(state, request.sankranti_config.as_ref())?;
                    dasha_snapshot_at(
                        engine,
                        eop,
                        &birth_utc,
                        &parse_utc(request.query_utc.ok_or_else(|| {
                            error_payload("invalid_request", "query_utc is required")
                        })?)?,
                        &location,
                        system,
                        request.max_level.unwrap_or(2),
                        &bhava_config,
                        &riseset_config,
                        &sankranti_config,
                        &variation,
                    )
                    .map(dasha_snapshot_json)
                    .map_err(|err| map_error("search_error", err))
                }
            }
            "level0" => {
                if let Some(inputs) = raw_inputs.as_ref() {
                    let inputs = inputs.borrowed();
                    dasha_level0_with_inputs(birth_jd, system, &variation, &inputs)
                        .map(|periods| {
                            json!(
                                periods
                                    .into_iter()
                                    .map(dasha_period_json)
                                    .collect::<Vec<_>>()
                            )
                        })
                        .map_err(|err| map_error("search_error", err))
                } else {
                    let birth_utc = parse_utc(request.birth_utc.ok_or_else(|| {
                        error_payload("invalid_request", "birth_utc is required")
                    })?)?;
                    let location =
                        parse_location(request.location.ok_or_else(|| {
                            error_payload("invalid_request", "location is required")
                        })?);
                    let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
                    let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
                    let sankranti_config =
                        to_sankranti_config(state, request.sankranti_config.as_ref())?;
                    dasha_level0_for_birth(
                        engine,
                        eop,
                        &birth_utc,
                        &location,
                        system,
                        &bhava_config,
                        &riseset_config,
                        &sankranti_config,
                        &variation,
                    )
                    .map(|periods| {
                        json!(
                            periods
                                .into_iter()
                                .map(dasha_period_json)
                                .collect::<Vec<_>>()
                        )
                    })
                    .map_err(|err| map_error("search_error", err))
                }
            }
            "level0_entity" => {
                let entity = parse_dasha_entity(
                    request
                        .entity
                        .as_ref()
                        .ok_or_else(|| error_payload("invalid_request", "entity is required"))?,
                )?;
                if let Some(inputs) = raw_inputs.as_ref() {
                    let inputs = inputs.borrowed();
                    dasha_level0_entity_with_inputs(birth_jd, system, entity, &variation, &inputs)
                        .map(|period| period.map(dasha_period_json).unwrap_or(Value::Null))
                        .map_err(|err| map_error("search_error", err))
                } else {
                    let birth_utc = parse_utc(request.birth_utc.ok_or_else(|| {
                        error_payload("invalid_request", "birth_utc is required")
                    })?)?;
                    let location =
                        parse_location(request.location.ok_or_else(|| {
                            error_payload("invalid_request", "location is required")
                        })?);
                    let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
                    let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
                    let sankranti_config =
                        to_sankranti_config(state, request.sankranti_config.as_ref())?;
                    dasha_level0_entity_for_birth(
                        engine,
                        eop,
                        &birth_utc,
                        &location,
                        system,
                        entity,
                        &bhava_config,
                        &riseset_config,
                        &sankranti_config,
                        &variation,
                    )
                    .map(|period| period.map(dasha_period_json).unwrap_or(Value::Null))
                    .map_err(|err| map_error("search_error", err))
                }
            }
            "children" => {
                let parent = parse_dasha_period(
                    request
                        .parent
                        .as_ref()
                        .ok_or_else(|| error_payload("invalid_request", "parent is required"))?,
                )?;
                if let Some(inputs) = raw_inputs.as_ref() {
                    let inputs = inputs.borrowed();
                    dasha_children_with_inputs(system, &parent, &variation, &inputs)
                        .map(|periods| {
                            json!(
                                periods
                                    .into_iter()
                                    .map(dasha_period_json)
                                    .collect::<Vec<_>>()
                            )
                        })
                        .map_err(|err| map_error("search_error", err))
                } else {
                    let birth_utc = parse_utc(request.birth_utc.ok_or_else(|| {
                        error_payload("invalid_request", "birth_utc is required")
                    })?)?;
                    let location =
                        parse_location(request.location.ok_or_else(|| {
                            error_payload("invalid_request", "location is required")
                        })?);
                    let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
                    let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
                    let sankranti_config =
                        to_sankranti_config(state, request.sankranti_config.as_ref())?;
                    dasha_children_for_birth(
                        engine,
                        eop,
                        &birth_utc,
                        &location,
                        system,
                        &parent,
                        &bhava_config,
                        &riseset_config,
                        &sankranti_config,
                        &variation,
                    )
                    .map(|periods| {
                        json!(
                            periods
                                .into_iter()
                                .map(dasha_period_json)
                                .collect::<Vec<_>>()
                        )
                    })
                    .map_err(|err| map_error("search_error", err))
                }
            }
            "child_period" => {
                let parent = parse_dasha_period(
                    request
                        .parent
                        .as_ref()
                        .ok_or_else(|| error_payload("invalid_request", "parent is required"))?,
                )?;
                let child_entity =
                    parse_dasha_entity(request.child_entity.as_ref().ok_or_else(|| {
                        error_payload("invalid_request", "child_entity is required")
                    })?)?;
                if let Some(inputs) = raw_inputs.as_ref() {
                    let inputs = inputs.borrowed();
                    dasha_child_period_with_inputs(
                        system,
                        &parent,
                        child_entity,
                        &variation,
                        &inputs,
                    )
                    .map(|period| period.map(dasha_period_json).unwrap_or(Value::Null))
                    .map_err(|err| map_error("search_error", err))
                } else {
                    let birth_utc = parse_utc(request.birth_utc.ok_or_else(|| {
                        error_payload("invalid_request", "birth_utc is required")
                    })?)?;
                    let location =
                        parse_location(request.location.ok_or_else(|| {
                            error_payload("invalid_request", "location is required")
                        })?);
                    let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
                    let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
                    let sankranti_config =
                        to_sankranti_config(state, request.sankranti_config.as_ref())?;
                    dasha_child_period_for_birth(
                        engine,
                        eop,
                        &birth_utc,
                        &location,
                        system,
                        &parent,
                        child_entity,
                        &bhava_config,
                        &riseset_config,
                        &sankranti_config,
                        &variation,
                    )
                    .map(|period| period.map(dasha_period_json).unwrap_or(Value::Null))
                    .map_err(|err| map_error("search_error", err))
                }
            }
            "complete_level" => {
                let child_level =
                    parse_dasha_level(request.child_level.as_ref().ok_or_else(|| {
                        error_payload("invalid_request", "child_level is required")
                    })?)?;
                let parent_periods = request
                    .parent_periods
                    .as_ref()
                    .ok_or_else(|| error_payload("invalid_request", "parent_periods is required"))?
                    .iter()
                    .map(parse_dasha_period)
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(inputs) = raw_inputs.as_ref() {
                    let inputs = inputs.borrowed();
                    dasha_complete_level_with_inputs(
                        system,
                        &parent_periods,
                        child_level,
                        &variation,
                        &inputs,
                    )
                    .map(|periods| {
                        json!(
                            periods
                                .into_iter()
                                .map(dasha_period_json)
                                .collect::<Vec<_>>()
                        )
                    })
                    .map_err(|err| map_error("search_error", err))
                } else {
                    let birth_utc = parse_utc(request.birth_utc.ok_or_else(|| {
                        error_payload("invalid_request", "birth_utc is required")
                    })?)?;
                    let location =
                        parse_location(request.location.ok_or_else(|| {
                            error_payload("invalid_request", "location is required")
                        })?);
                    let bhava_config = to_bhava_config(state, request.bhava_config.as_ref())?;
                    let riseset_config = to_riseset_config(state, request.riseset_config.as_ref())?;
                    let sankranti_config =
                        to_sankranti_config(state, request.sankranti_config.as_ref())?;
                    dasha_complete_level_for_birth(
                        engine,
                        eop,
                        &birth_utc,
                        &location,
                        system,
                        &parent_periods,
                        child_level,
                        &bhava_config,
                        &riseset_config,
                        &sankranti_config,
                        &variation,
                    )
                    .map(|periods| {
                        json!(
                            periods
                                .into_iter()
                                .map(dasha_period_json)
                                .collect::<Vec<_>>()
                        )
                    })
                    .map_err(|err| map_error("search_error", err))
                }
            }
            _ => Err(error_payload("invalid_request", "unknown dasha operation")),
        }
    })
}

fn handle_tara(resource: &ResourceArc<EngineResource>, request: TaraRequest) -> JsonResult {
    read_state(resource, |state| match request.op.as_str() {
        "catalog_info" => Ok(json!({
            "source": state.tara_catalog.source,
            "reference_epoch_jy": state.tara_catalog.reference_epoch_jy,
            "count": state.tara_catalog.len()
        })),
        "compute" => {
            let op = TaraOperation {
                star: parse_tara_id(
                    request
                        .star
                        .as_ref()
                        .ok_or_else(|| error_payload("invalid_request", "star is required"))?,
                )?,
                output: parse_tara_output(request.output.as_ref())?,
                at_jd_tdb: request
                    .jd_tdb
                    .ok_or_else(|| error_payload("invalid_request", "jd_tdb is required"))?,
                ayanamsha_deg: request.ayanamsha_deg.unwrap_or(0.0),
                config: to_tara_config(state, request.config.as_ref())?,
                earth_state: None,
            };
            tara_op(state.tara_catalog.as_ref(), &op)
                .map(tara_result_json)
                .map_err(|err| map_error("tara_error", err))
        }
        _ => Err(error_payload("invalid_request", "unknown tara operation")),
    })
}

#[rustler::nif(schedule = "DirtyCpu")]
fn engine_new<'a>(env: Env<'a>, config: Term<'a>) -> Result<Term<'a>, rustler::Error> {
    let config = decode_term::<EngineConfigInput>(config)?;
    let time_policy =
        parse_time_policy_input(config.time_policy).map_err(|_| rustler::Error::BadArg)?;
    let engine_config = EngineConfig {
        spk_paths: config.spk_paths.into_iter().map(PathBuf::from).collect(),
        lsk_path: PathBuf::from(config.lsk_path),
        cache_capacity: config.cache_capacity.unwrap_or(256),
        strict_validation: config.strict_validation.unwrap_or(true),
    };
    let engine = match Engine::new(engine_config) {
        Ok(engine) => engine,
        Err(err) => {
            let error_term =
                rustler::serde::to_term(env, &error_payload("engine_error", err.to_string()))?;
            return Ok((atoms::error(), error_term).encode(env));
        }
    };
    let resource = ResourceArc::new(EngineResource {
        state: RwLock::new(EngineState::new(engine, time_policy)),
    });
    Ok((atoms::ok(), resource).encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn engine_close<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
) -> Result<Term<'a>, rustler::Error> {
    let response = write_state(&resource, |state| {
        state.engine = None;
        Ok(json!({ "closed": true }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyIo")]
fn engine_load_config<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<ConfigLoadInput>(request)?;
    let response = write_state(&resource, |state| {
        let defaults_mode = parse_defaults_mode(request.defaults_mode.as_ref())?;
        let explicit_path = request.path.as_ref().map(PathBuf::from);
        let loaded = load_with_discovery(explicit_path.as_deref(), false)
            .map_err(|err| map_error("config_error", err))?;
        let loaded = loaded.ok_or_else(|| {
            error_payload("config_error", "no config file found for discovery request")
        })?;
        state.resolver = Some(ConfigResolver::new(loaded.file, defaults_mode));
        Ok(json!({ "loaded": true, "path": loaded.path }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn engine_clear_config<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
) -> Result<Term<'a>, rustler::Error> {
    let response = write_state(&resource, |state| {
        state.resolver = None;
        Ok(json!({ "cleared": true }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyIo")]
fn engine_replace_spks<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<SpkPathsInput>(request)?;
    let response = write_state(&resource, |state| {
        let engine = require_engine(state)?;
        let report = engine
            .replace_spk_paths(request.spk_paths.into_iter().map(PathBuf::from).collect())
            .map_err(|err| map_error("engine_error", err))?;
        Ok(json!({
            "generation": report.generation,
            "active_count": report.active_count,
            "loaded_count": report.loaded_count,
            "reused_count": report.reused_count
        }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn engine_list_spks<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
) -> Result<Term<'a>, rustler::Error> {
    let response = read_state(&resource, |state| {
        let engine = require_engine(state)?;
        let spks: Vec<Value> = engine
            .spk_infos()
            .into_iter()
            .map(|info| {
                json!({
                    "path": info.path,
                    "segment_count": info.segment_count,
                    "generation": info.generation
                })
            })
            .collect();
        Ok(json!({ "spks": spks }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyIo")]
fn engine_load_eop<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<PathInput>(request)?;
    let response = write_state(&resource, |state| {
        state.eop = Some(
            EopKernel::load(PathBuf::from(&request.path).as_path())
                .map_err(|err| map_error("time_error", err))?,
        );
        Ok(json!({ "loaded": true, "path": request.path }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn engine_clear_eop<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
) -> Result<Term<'a>, rustler::Error> {
    let response = write_state(&resource, |state| {
        state.eop = None;
        Ok(json!({ "cleared": true }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyIo")]
fn engine_load_tara_catalog<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<PathInput>(request)?;
    let response = write_state(&resource, |state| {
        state.tara_catalog = Arc::new(
            TaraCatalog::load(PathBuf::from(&request.path).as_path())
                .map_err(|err| map_error("tara_error", err))?,
        );
        Ok(json!({ "loaded": true, "path": request.path }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn engine_reset_tara_catalog<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
) -> Result<Term<'a>, rustler::Error> {
    let response = write_state(&resource, |state| {
        state.tara_catalog = Arc::new(TaraCatalog::embedded().clone());
        Ok(json!({ "reset": true }))
    });
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn ephemeris_run<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let raw = decode_term::<Value>(request)?;
    let op = raw
        .get("op")
        .and_then(Value::as_str)
        .ok_or(rustler::Error::BadArg)?;
    let response = match op {
        "query" => handle_ephemeris(
            &resource,
            serde_json::from_value(raw).map_err(|_| rustler::Error::BadArg)?,
        ),
        "body_ecliptic_lon_lat" => handle_body_ecliptic_lon_lat(
            &resource,
            serde_json::from_value(raw).map_err(|_| rustler::Error::BadArg)?,
        ),
        _ => Err(error_payload(
            "invalid_request",
            "unknown ephemeris operation",
        )),
    };
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn time_run<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<TimeRunInput>(request)?;
    encode_json(env, handle_time(&resource, request))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn util_run<'a>(env: Env<'a>, request: Term<'a>) -> Result<Term<'a>, rustler::Error> {
    let raw = decode_term::<Value>(request)?;
    let response = match raw.get("op").and_then(Value::as_str).unwrap_or_default() {
        "build_info" => Ok(json!({
            "version": dhruv_build_info::version(),
            "git_hash": dhruv_build_info::git_hash()
        })),
        "cartesian_to_spherical" => {
            let input: CartesianInput =
                serde_json::from_value(raw).map_err(|_| rustler::Error::BadArg)?;
            Ok(spherical_json(cartesian_to_spherical(&[
                input.x, input.y, input.z,
            ])))
        }
        "nutation" => {
            let jd_tdb = raw
                .get("jd_tdb")
                .and_then(Value::as_f64)
                .ok_or(rustler::Error::BadArg)?;
            let t = dhruv_vedic_base::jd_tdb_to_centuries(jd_tdb);
            let (dpsi_arcsec, deps_arcsec) = nutation_iau2000b(t);
            Ok(json!({ "dpsi_arcsec": dpsi_arcsec, "deps_arcsec": deps_arcsec }))
        }
        "approximate_local_noon" => {
            let jd_ut_midnight = raw
                .get("jd_ut_midnight")
                .and_then(Value::as_f64)
                .ok_or(rustler::Error::BadArg)?;
            let longitude_deg = raw
                .get("longitude_deg")
                .and_then(Value::as_f64)
                .ok_or(rustler::Error::BadArg)?;
            Ok(json!({
                "jd_approx_local_noon": approximate_local_noon_jd(jd_ut_midnight, longitude_deg)
            }))
        }
        "ayanamsha_system_count" => Ok(json!({
            "count": AyanamshaSystem::all().len()
        })),
        "reference_plane_default" => {
            let system = raw
                .get("system")
                .ok_or(rustler::Error::BadArg)
                .and_then(|value| {
                    serde_json::from_value::<EnumInput>(value.clone())
                        .map_err(|_| rustler::Error::BadArg)
                })
                .and_then(|value| {
                    parse_ayanamsha_system(Some(&value)).map_err(|_| rustler::Error::BadArg)
                })?;
            Ok(json!({
                "reference_plane": match system.default_reference_plane() {
                    dhruv_frames::ReferencePlane::Ecliptic => "ecliptic",
                    dhruv_frames::ReferencePlane::Invariable => "invariable",
                }
            }))
        }
        "rashi_from_longitude" => Ok(rashi_info_json(rashi_from_longitude(raw_required_f64(
            &raw,
            "sidereal_lon_deg",
        )?))),
        "nakshatra_from_longitude" => Ok(nakshatra_info_json(nakshatra_from_longitude(
            raw_required_f64(&raw, "sidereal_lon_deg")?,
        ))),
        "nakshatra28_from_longitude" => Ok(nakshatra28_info_json(nakshatra28_from_longitude(
            raw_required_f64(&raw, "sidereal_lon_deg")?,
        ))),
        "rashi_from_tropical" => Ok(rashi_info_json(rashi_from_tropical(
            raw_required_f64(&raw, "tropical_lon_deg")?,
            parse_ayanamsha_system(raw_optional_enum(&raw, "system")?.as_ref())
                .map_err(|_| rustler::Error::BadArg)?,
            raw_required_f64(&raw, "jd_tdb")?,
            raw.get("use_nutation")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ))),
        "nakshatra_from_tropical" => Ok(nakshatra_info_json(nakshatra_from_tropical(
            raw_required_f64(&raw, "tropical_lon_deg")?,
            parse_ayanamsha_system(raw_optional_enum(&raw, "system")?.as_ref())
                .map_err(|_| rustler::Error::BadArg)?,
            raw_required_f64(&raw, "jd_tdb")?,
            raw.get("use_nutation")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ))),
        "nakshatra28_from_tropical" => Ok(nakshatra28_info_json(nakshatra28_from_tropical(
            raw_required_f64(&raw, "tropical_lon_deg")?,
            parse_ayanamsha_system(raw_optional_enum(&raw, "system")?.as_ref())
                .map_err(|_| rustler::Error::BadArg)?,
            raw_required_f64(&raw, "jd_tdb")?,
            raw.get("use_nutation")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ))),
        "graha_name" => {
            let graha_input = raw
                .get("graha")
                .or_else(|| raw.get("index"))
                .cloned()
                .ok_or(rustler::Error::BadArg)
                .and_then(|value| {
                    serde_json::from_value(value).map_err(|_| rustler::Error::BadArg)
                })?;
            let graha = parse_graha(&graha_input).map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({ "name": graha.name() }))
        }
        "yogini_name" => Ok(json!({ "name": yogini_name(raw_required_u8(&raw, "index")?) })),
        "rashi_name" => {
            let idx = raw_required_u8(&raw, "index")?;
            let rashi = ALL_RASHIS.get(idx as usize).ok_or(rustler::Error::BadArg)?;
            Ok(json!({ "name": rashi.name() }))
        }
        "nakshatra_name" => {
            let idx = raw_required_u8(&raw, "index")?;
            let nakshatra = ALL_NAKSHATRAS_27
                .get(idx as usize)
                .ok_or(rustler::Error::BadArg)?;
            Ok(json!({ "name": nakshatra.name() }))
        }
        "nakshatra28_name" => {
            let idx = raw_required_u8(&raw, "index")?;
            let nakshatra = ALL_NAKSHATRAS_28
                .get(idx as usize)
                .ok_or(rustler::Error::BadArg)?;
            Ok(json!({ "name": nakshatra.name() }))
        }
        "sphuta_name" => {
            let idx = raw_required_u8(&raw, "index")?;
            let sphuta = ALL_SPHUTAS
                .get(idx as usize)
                .ok_or(rustler::Error::BadArg)?;
            Ok(json!({ "name": sphuta.name() }))
        }
        "upagraha_name" => {
            let idx = raw_required_u8(&raw, "index")?;
            let upagraha = ALL_UPAGRAHAS
                .get(idx as usize)
                .ok_or(rustler::Error::BadArg)?;
            Ok(json!({ "name": upagraha.name() }))
        }
        "amsha_variations" => {
            let amsha = Amsha::from_code(
                raw.get("amsha_code")
                    .and_then(Value::as_u64)
                    .and_then(|entry| u16::try_from(entry).ok())
                    .ok_or(rustler::Error::BadArg)?,
            )
            .ok_or(rustler::Error::BadArg)?;
            Ok(amsha_variation_catalog_json(amsha))
        }
        "amsha_rashi_infos" => {
            let longitudes = raw
                .get("longitudes")
                .and_then(Value::as_array)
                .ok_or(rustler::Error::BadArg)?
                .iter()
                .map(|value| value.as_f64().ok_or(rustler::Error::BadArg))
                .collect::<Result<Vec<_>, rustler::Error>>()?;
            let inputs: Vec<AmshaRequestInput> = raw
                .get("amsha_requests")
                .cloned()
                .ok_or(rustler::Error::BadArg)
                .and_then(|value| {
                    serde_json::from_value(value).map_err(|_| rustler::Error::BadArg)
                })?;
            to_amsha_requests(Some(&inputs)).map(|requests| {
                json!({
                    "entries": longitudes
                        .into_iter()
                        .map(|sidereal_lon| {
                            amsha_rashi_infos(sidereal_lon, &requests)
                                .into_iter()
                                .map(|info| {
                                    let mut value = rashi_info_json(info);
                                    value["amsha_longitude"] = json!(
                                        f64::from(info.rashi_index) * 30.0
                                            + info.degrees_in_rashi
                                    );
                                    value
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>()
                })
            })
        }
        "amsha_variations_many" => {
            let amsha_codes = raw
                .get("amsha_codes")
                .and_then(Value::as_array)
                .ok_or(rustler::Error::BadArg)?;
            let catalogs = amsha_codes
                .iter()
                .map(|value| {
                    let code = value
                        .as_u64()
                        .and_then(|entry| u16::try_from(entry).ok())
                        .ok_or(rustler::Error::BadArg)?;
                    let amsha = Amsha::from_code(code).ok_or(rustler::Error::BadArg)?;
                    Ok(amsha_variation_catalog_json(amsha))
                })
                .collect::<Result<Vec<_>, rustler::Error>>()?;
            Ok(json!({ "catalogs": catalogs }))
        }
        "hora_lord" => {
            let vaar = parse_vaar(&raw_required_enum(&raw, "vaar")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(
                json!({ "graha": debug_name(hora_lord(vaar, raw_required_u8(&raw, "hora_index")?)) }),
            )
        }
        "masa_lord" => {
            let masa = parse_masa(&raw_required_enum(&raw, "masa")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({ "graha": debug_name(masa_lord(masa)) }))
        }
        "samvatsara_lord" => {
            let samvatsara = parse_samvatsara(&raw_required_enum(&raw, "samvatsara")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({ "graha": debug_name(samvatsara_lord(samvatsara)) }))
        }
        "exaltation_degree" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({ "degree": exaltation_degree(graha) }))
        }
        "debilitation_degree" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({ "degree": debilitation_degree(graha) }))
        }
        "moolatrikone_range" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(match moolatrikone_range(graha) {
                Some((rashi_index, start_deg, end_deg)) => json!({
                    "range": {
                        "rashi_index": rashi_index,
                        "rashi": debug_name(ALL_RASHIS[rashi_index as usize]),
                        "start_deg": start_deg,
                        "end_deg": end_deg
                    }
                }),
                None => json!({ "range": Value::Null }),
            })
        }
        "combustion_threshold" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({
                "threshold_deg": combustion_threshold_fn(
                    graha,
                    raw.get("is_retrograde").and_then(Value::as_bool).unwrap_or(false),
                )
            }))
        }
        "is_combust" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({
                "combust": is_combust_fn(
                    graha,
                    raw_required_f64(&raw, "graha_sid_lon")?,
                    raw_required_f64(&raw, "sun_sid_lon")?,
                    raw.get("is_retrograde").and_then(Value::as_bool).unwrap_or(false),
                )
            }))
        }
        "all_combustion_status" => Ok(json!({
            "statuses": all_combustion_status_fn(
                &raw_f64_array::<9>(&raw, "sidereal_lons_9")?,
                &raw_bool_array::<9>(&raw, "retrograde_flags_9")?,
            )
        })),
        "naisargika_maitri" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            let other = parse_graha(&raw_required_enum(&raw, "other")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({ "relationship": debug_name(naisargika_maitri(graha, other)) }))
        }
        "tatkalika_maitri" => Ok(json!({
            "relationship": debug_name(tatkalika_maitri(
                raw_required_u8(&raw, "graha_rashi_index")?,
                raw_required_u8(&raw, "other_rashi_index")?,
            ))
        })),
        "panchadha_maitri" => {
            let naisargika = match raw_required_enum(&raw, "naisargika")? {
                EnumInput::Int(0) => NaisargikaMaitri::Friend,
                EnumInput::Int(1) => NaisargikaMaitri::Enemy,
                EnumInput::Int(2) => NaisargikaMaitri::Neutral,
                EnumInput::Int(_) => return Err(rustler::Error::BadArg),
                EnumInput::Str(value) => match value.to_ascii_lowercase().as_str() {
                    "friend" => NaisargikaMaitri::Friend,
                    "enemy" => NaisargikaMaitri::Enemy,
                    "neutral" => NaisargikaMaitri::Neutral,
                    _ => return Err(rustler::Error::BadArg),
                },
            };
            let tatkalika = match raw_required_enum(&raw, "tatkalika")? {
                EnumInput::Int(0) => TatkalikaMaitri::Friend,
                EnumInput::Int(1) => TatkalikaMaitri::Enemy,
                EnumInput::Int(_) => return Err(rustler::Error::BadArg),
                EnumInput::Str(value) => match value.to_ascii_lowercase().as_str() {
                    "friend" => TatkalikaMaitri::Friend,
                    "enemy" => TatkalikaMaitri::Enemy,
                    _ => return Err(rustler::Error::BadArg),
                },
            };
            Ok(json!({ "relationship": debug_name(panchadha_maitri(naisargika, tatkalika)) }))
        }
        "dignity_in_rashi" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({
                "dignity": debug_name(dignity_in_rashi(
                    graha,
                    raw_required_f64(&raw, "sidereal_lon")?,
                    raw_required_u8(&raw, "rashi_index")?,
                ))
            }))
        }
        "dignity_in_rashi_with_positions" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({
                "dignity": debug_name(dignity_in_rashi_with_positions(
                    graha,
                    raw_required_f64(&raw, "sidereal_lon")?,
                    raw_required_u8(&raw, "rashi_index")?,
                    &raw_u8_array::<7>(&raw, "all_rashi_indices")?,
                ))
            }))
        }
        "node_dignity_in_rashi" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            let policy = parse_node_dignity_policy(raw_optional_enum(&raw, "policy")?.as_ref())
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({
                "dignity": debug_name(node_dignity_in_rashi(
                    graha,
                    raw_required_u8(&raw, "rashi_index")?,
                    &raw_u8_array::<9>(&raw, "all_rashi_indices_9")?,
                    policy,
                ))
            }))
        }
        "natural_benefic_malefic" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({ "nature": benefic_nature_json(natural_benefic_malefic(graha)) }))
        }
        "moon_benefic_nature" => Ok(json!({
            "nature": benefic_nature_json(moon_benefic_nature(raw_required_f64(
                &raw,
                "moon_sun_elongation",
            )?))
        })),
        "graha_gender" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({ "gender": debug_name(graha_gender(graha)) }))
        }
        "graha_drishti" => {
            let graha = parse_graha(&raw_required_enum(&raw, "graha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(drishti_entry_json(graha_drishti(
                graha,
                raw_required_f64(&raw, "source_lon")?,
                raw_required_f64(&raw, "target_lon")?,
            )))
        }
        "graha_drishti_matrix" => Ok(graha_drishti_matrix_json(graha_drishti_matrix(
            &raw_f64_array::<9>(&raw, "longitudes")?,
        ))),
        "sun_based_upagrahas" => Ok(sun_based_upagrahas_value_json(sun_based_upagrahas(
            raw_required_f64(&raw, "sun_sid_lon")?,
        ))),
        "time_upagraha_jd" => {
            let upagraha = parse_upagraha(&raw_required_enum(&raw, "upagraha")?)
                .map_err(|_| rustler::Error::BadArg)?;
            Ok(json!({
                "jd": time_upagraha_jd(
                    upagraha,
                    raw_required_u8(&raw, "weekday")?,
                    raw_required_bool(&raw, "is_day")?,
                    raw_required_f64(&raw, "day_start_jd")?,
                    raw_required_f64(&raw, "day_end_jd")?,
                    raw_required_f64(&raw, "night_end_jd")?,
                )
            }))
        }
        "all_sphutas" => {
            let inputs = SphutalInputs {
                sun: raw_required_f64(&raw, "sun")?,
                moon: raw_required_f64(&raw, "moon")?,
                mars: raw_required_f64(&raw, "mars")?,
                jupiter: raw_required_f64(&raw, "jupiter")?,
                venus: raw_required_f64(&raw, "venus")?,
                rahu: raw_required_f64(&raw, "rahu")?,
                lagna: raw_required_f64(&raw, "lagna")?,
                eighth_lord: raw_required_f64(&raw, "eighth_lord")?,
                gulika: raw_required_f64(&raw, "gulika")?,
            };
            Ok(json!({
                "entries": all_sphutas(&inputs)
                    .into_iter()
                    .map(|(sphuta, longitude_deg)| json!({
                        "sphuta": debug_name(sphuta),
                        "longitude_deg": longitude_deg
                    }))
                    .collect::<Vec<_>>()
            }))
        }
        "calculate_bav" => {
            let bav = calculate_bav(
                raw_required_u8(&raw, "graha_index")?,
                &raw_u8_array::<7>(&raw, "graha_rashis")?,
                raw_required_u8(&raw, "lagna_rashi")?,
            );
            Ok(json!({
                "bav": {
                    "graha_index": bav.graha_index,
                    "points": bav.points,
                    "contributors": bav.contributors
                }
            }))
        }
        "calculate_all_bav" => Ok(json!({
            "bavs": calculate_all_bav(
                &raw_u8_array::<7>(&raw, "graha_rashis")?,
                raw_required_u8(&raw, "lagna_rashi")?,
            )
            .into_iter()
            .map(|bav| json!({
                "graha_index": bav.graha_index,
                "points": bav.points,
                "contributors": bav.contributors
            }))
            .collect::<Vec<_>>()
        })),
        "calculate_sav" => {
            let bavs = calculate_all_bav(
                &raw_u8_array::<7>(&raw, "graha_rashis")?,
                raw_required_u8(&raw, "lagna_rashi")?,
            );
            let sav = calculate_sav(&bavs);
            Ok(json!({
                "sav": {
                    "total_points": sav.total_points,
                    "after_trikona": sav.after_trikona,
                    "after_ekadhipatya": sav.after_ekadhipatya
                }
            }))
        }
        "calculate_ashtakavarga" => Ok(ashtakavarga_json(calculate_ashtakavarga(
            &raw_u8_array::<7>(&raw, "graha_rashis")?,
            raw_required_u8(&raw, "lagna_rashi")?,
        ))),
        "trikona_sodhana" => Ok(json!({
            "totals": trikona_sodhana(&raw_u8_array::<12>(&raw, "totals")?)
        })),
        "ekadhipatya_sodhana" => Ok(json!({
            "totals": ekadhipatya_sodhana(&raw_u8_array::<12>(&raw, "totals")?)
        })),
        "ghatika_from_elapsed" => {
            let position = ghatika_from_elapsed(
                raw_required_f64(&raw, "seconds_since_sunrise")?,
                raw_required_f64(&raw, "vedic_day_duration_seconds")?,
            );
            Ok(json!({ "value": position.value, "index": position.index }))
        }
        "ghatikas_since_sunrise" => Ok(json!({
            "ghatikas": ghatikas_since_sunrise(
                raw_required_f64(&raw, "jd_moment")?,
                raw_required_f64(&raw, "jd_sunrise")?,
                raw_required_f64(&raw, "jd_next_sunrise")?,
            )
        })),
        "tara_propagate_position" => Ok(equatorial_position_json(propagate_position(
            raw_required_f64(&raw, "ra_deg")?,
            raw_required_f64(&raw, "dec_deg")?,
            raw_required_f64(&raw, "parallax_mas")?,
            raw_required_f64(&raw, "pm_ra_mas_yr")?,
            raw_required_f64(&raw, "pm_dec_mas_yr")?,
            raw_required_f64(&raw, "rv_km_s")?,
            raw_required_f64(&raw, "dt_years")?,
        ))),
        "tara_apply_aberration" => Ok(json!({
            "direction": apply_aberration(
                &raw_f64_array::<3>(&raw, "direction")?,
                &raw_f64_array::<3>(&raw, "earth_vel_au_day")?,
            )
        })),
        "tara_apply_light_deflection" => Ok(json!({
            "direction": apply_light_deflection(
                &raw_f64_array::<3>(&raw, "direction")?,
                &raw_f64_array::<3>(&raw, "sun_to_observer")?,
                raw_required_f64(&raw, "sun_observer_distance_au")?,
            )
        })),
        "tara_galactic_anticenter_icrs" => Ok(json!({
            "direction": galactic_anticenter_icrs()
        })),
        _ => Err(error_payload(
            "invalid_request",
            "unknown utility operation",
        )),
    };
    encode_json(env, response)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn vedic_run<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<VedicRequest>(request)?;
    encode_json(env, handle_vedic(&resource, request))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn panchang_run<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<PanchangRequest>(request)?;
    encode_json(env, handle_panchang(&resource, request))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn search_run<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<SearchRequest>(request)?;
    encode_json(env, handle_search(&resource, request))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn jyotish_run<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<JyotishRequest>(request)?;
    encode_json(env, handle_jyotish(&resource, request))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn dasha_run<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<DashaRequest>(request)?;
    encode_json(env, handle_dasha(&resource, request))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn tara_run<'a>(
    env: Env<'a>,
    resource: ResourceArc<EngineResource>,
    request: Term<'a>,
) -> Result<Term<'a>, rustler::Error> {
    let request = decode_term::<TaraRequest>(request)?;
    encode_json(env, handle_tara(&resource, request))
}

rustler::init!("Elixir.CtaraDhruv.Native", load = load);

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_state() -> EngineState {
        EngineState {
            engine: None,
            resolver: None,
            eop: None,
            time_policy: TimeConversionPolicy::StrictLsk,
            tara_catalog: Arc::new(TaraCatalog::embedded().clone()),
        }
    }

    #[test]
    fn camel_to_snake_handles_mixed_case() {
        assert_eq!(camel_to_snake("MixedParashara"), "mixed_parashara");
        assert_eq!(camel_to_snake("StrictLsk"), "strict_lsk");
    }

    #[test]
    fn parse_named_matches_debug_form() {
        assert_eq!(
            parse_named("lahiri", AyanamshaSystem::all()),
            Some(AyanamshaSystem::Lahiri)
        );
        assert_eq!(
            parse_named("mixed_parashara", &CHARAKARAKA_SCHEME_VARIANTS),
            Some(CharakarakaScheme::MixedParashara)
        );
    }

    #[test]
    fn resource_mutation_updates_state() {
        let dummy = dummy_state();
        assert!(matches!(dummy.time_policy, TimeConversionPolicy::StrictLsk));
        assert!(dummy.tara_catalog.len() > 0);
    }

    #[test]
    fn bhava_config_parses_special_bhavabala_rules() {
        let state = dummy_state();
        let default_config = to_bhava_config(&state, None).unwrap();
        assert!(default_config.include_special_bhavabala_rules);

        let input = BhavaConfigInput {
            system: None,
            reference_mode: None,
            use_rashi_bhava_for_bala_avastha: None,
            include_node_aspects_for_drik_bala: None,
            include_special_bhavabala_rules: Some(false),
            divide_guru_buddh_drishti_by_4_for_drik_bala: None,
            chandra_benefic_rule: None,
            sayanadi_ghatika_rounding: None,
            include_rashi_bhava_results: None,
        };
        let config = to_bhava_config(&state, Some(&input)).unwrap();
        assert!(!config.include_special_bhavabala_rules);
    }

    #[test]
    fn error_payload_shape_is_stable() {
        let payload = error_payload("search_error", "boom");
        assert_eq!(payload["kind"], "search_error");
        assert_eq!(payload["message"], "boom");
    }

    #[test]
    fn amsha_selection_conversion_preserves_codes_and_variations() {
        let selection = to_amsha_selection(Some(&[
            AmshaRequestInput {
                code: 9,
                variation: None,
            },
            AmshaRequestInput {
                code: 2,
                variation: Some(1),
            },
        ]))
        .expect("amsha selection should parse");
        assert_eq!(selection.count, 2);
        assert_eq!(selection.codes[0], 9);
        assert_eq!(selection.variations[0], 0);
        assert_eq!(selection.codes[1], 2);
        assert_eq!(selection.variations[1], 1);
    }

    #[test]
    fn amsha_requests_conversion_preserves_codes_and_variations() {
        let requests = to_amsha_requests(Some(&[
            AmshaRequestInput {
                code: 9,
                variation: None,
            },
            AmshaRequestInput {
                code: 2,
                variation: Some(1),
            },
        ]))
        .expect("amsha requests should parse");
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].amsha, Amsha::D9);
        assert_eq!(requests[0].variation, None);
        assert_eq!(requests[1].amsha, Amsha::D2);
        assert_eq!(requests[1].variation, Some(1));
        assert!(
            to_amsha_requests(None)
                .expect("no input is empty")
                .is_empty()
        );
    }

    #[test]
    fn amsha_requests_conversion_rejects_invalid_input() {
        let err = to_amsha_requests(Some(&[AmshaRequestInput {
            code: 13,
            variation: None,
        }]))
        .expect_err("unknown amsha code should be rejected");
        assert_eq!(err["kind"], "invalid_request");

        let err = to_amsha_requests(Some(&[AmshaRequestInput {
            code: 9,
            variation: Some(200),
        }]))
        .expect_err("unknown variation should be rejected");
        assert_eq!(err["kind"], "invalid_request");
    }

    fn known_request(patch: Value) -> PanchangRequest {
        let mut base = json!({ "op": "daily" });
        base.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        serde_json::from_value(base).expect("panchang request should decode")
    }

    fn utc_map(year: i32, month: u32, day: u32) -> Value {
        json!({
            "year": year, "month": month, "day": day,
            "hour": 0, "minute": 0, "second": 0.0
        })
    }

    #[test]
    fn panchang_known_inputs_round_trip_names_and_indices() {
        let request = known_request(json!({
            "known_masa": {
                "masa": "chaitra",
                "adhika": true,
                "start": utc_map(2015, 3, 21),
                "end": utc_map(2015, 4, 18)
            },
            "known_ayana": {
                "ayana": "uttarayana",
                "start": utc_map(2015, 1, 14),
                "end": utc_map(2015, 7, 16)
            },
            "known_varsha": {
                "samvatsara": "vishvavasu",
                "order": 39,
                "start": utc_map(2015, 3, 21),
                "end": utc_map(2016, 4, 8)
            }
        }));
        let known = to_panchang_precomputed(&request).expect("known inputs should decode");
        let masa = known.masa.expect("masa should be present");
        assert_eq!(masa.masa, dhruv_vedic_base::Masa::Chaitra);
        assert!(masa.adhika);
        assert_eq!(masa.start.month, 3);
        assert_eq!(masa.end.day, 18);
        assert_eq!(
            known.ayana.unwrap().ayana,
            dhruv_vedic_base::ayana_type::Ayana::Uttarayana
        );
        let varsha = known.varsha.expect("varsha should be present");
        assert_eq!(varsha.samvatsara, dhruv_vedic_base::Samvatsara::Vishvavasu);
        assert_eq!(varsha.order, 39);

        // Integer indices follow the same index-ordered arrays as the names.
        let request = known_request(json!({
            "known_masa": {
                "masa": 1,
                "adhika": false,
                "start": utc_map(2015, 4, 18),
                "end": utc_map(2015, 5, 18)
            },
            "known_ayana": {
                "ayana": 1,
                "start": utc_map(2015, 7, 16),
                "end": utc_map(2016, 1, 15)
            }
        }));
        let known = to_panchang_precomputed(&request).expect("indices should decode");
        assert_eq!(known.masa.unwrap().masa, dhruv_vedic_base::Masa::Vaishakha);
        assert_eq!(
            known.ayana.unwrap().ayana,
            dhruv_vedic_base::ayana_type::Ayana::Dakshinayana
        );
        assert!(known.varsha.is_none());

        // Nothing supplied decodes to the all-None default.
        let request = known_request(json!({}));
        let known = to_panchang_precomputed(&request).expect("empty known should decode");
        assert_eq!(known, dhruv_search::PanchangPrecomputed::default());
    }

    #[test]
    fn panchang_known_inputs_reject_invalid_values() {
        let request = known_request(json!({
            "known_masa": {
                "masa": "not_a_masa",
                "adhika": false,
                "start": utc_map(2015, 3, 21),
                "end": utc_map(2015, 4, 18)
            }
        }));
        let err = to_panchang_precomputed(&request).expect_err("unknown masa should be rejected");
        assert_eq!(err["kind"], "invalid_request");

        let request = known_request(json!({
            "known_ayana": {
                "ayana": "sideways",
                "start": utc_map(2015, 1, 14),
                "end": utc_map(2015, 7, 16)
            }
        }));
        let err = to_panchang_precomputed(&request).expect_err("unknown ayana should be rejected");
        assert_eq!(err["kind"], "invalid_request");

        let request = known_request(json!({
            "known_varsha": {
                "samvatsara": "vishvavasu",
                "order": 61,
                "start": utc_map(2015, 3, 21),
                "end": utc_map(2016, 4, 8)
            }
        }));
        let err =
            to_panchang_precomputed(&request).expect_err("out-of-range order should be rejected");
        assert_eq!(err["kind"], "invalid_request");
    }

    #[test]
    fn panchang_include_mask_parses_events_selections() {
        let mask = parse_panchang_include_mask(&PanchangIncludeInput::List(vec![
            "tithi".to_string(),
            "masa".to_string(),
        ]))
        .expect("name list should parse");
        assert_eq!(
            mask,
            panchang_include_bits("tithi").unwrap() | panchang_include_bits("masa").unwrap()
        );

        let independent = parse_panchang_include_mask(&PanchangIncludeInput::Str(
            "location_independent".to_string(),
        ))
        .expect("group name should parse");
        assert_eq!(independent, PANCHANG_INCLUDE_LOCATION_INDEPENDENT);
        assert_eq!(independent & !PANCHANG_INCLUDE_LOCATION_INDEPENDENT, 0);
    }

    #[test]
    fn amsha_variation_catalog_json_uses_per_amsha_catalog() {
        let d2 = amsha_variation_catalog_json(Amsha::D2);
        let d9 = amsha_variation_catalog_json(Amsha::D9);

        assert_eq!(d2["default_variation_code"], 0);
        assert_eq!(d2["variations"].as_array().unwrap().len(), 4);
        assert_eq!(d2["variations"][1]["name"], "cancer-leo-only");
        assert_eq!(d2["variations"][2]["name"], "lunar-hora");
        assert_eq!(d2["variations"][3]["name"], "kashinath-hora");
        assert_eq!(d9["variations"].as_array().unwrap().len(), 1);
        assert_eq!(d9["variations"][0]["variation_code"], 0);
    }

    #[test]
    fn full_kundali_config_promotes_amsha_scope_dependencies() {
        let state = dummy_state();
        let config = to_full_kundali_config(
            &state,
            Some(&FullKundaliConfigInput {
                include_bhava_cusps: None,
                include_graha_positions: None,
                include_bindus: None,
                include_drishti: None,
                include_ashtakavarga: None,
                include_upagrahas: None,
                include_sphutas: None,
                include_special_lagnas: None,
                include_amshas: Some(true),
                include_shadbala: None,
                include_bhavabala: None,
                include_vimsopaka: None,
                include_avastha: None,
                include_charakaraka: None,
                panchang_include_mask: None,
                include_dasha: None,
                charakaraka_scheme: None,
                node_dignity_policy: None,
                upagraha_config: None,
                graha_positions_config: None,
                bindus_config: None,
                drishti_config: None,
                amsha_scope: Some(AmshaChartScopeInput {
                    include_bhava_cusps: Some(true),
                    include_arudha_padas: Some(true),
                    include_upagrahas: Some(true),
                    include_sphutas: Some(true),
                    include_special_lagnas: Some(true),
                    include_outer_planets: Some(true),
                }),
                amsha_selection: Some(vec![AmshaRequestInput {
                    code: 9,
                    variation: None,
                }]),
                dasha_config: None,
            }),
        )
        .expect("full kundali config should parse");
        assert!(config.include_amshas);
        assert!(config.graha_positions_config.include_lagna);
        assert!(config.include_bhava_cusps);
        assert!(config.include_bindus);
        assert!(config.include_upagrahas);
        assert!(config.include_sphutas);
        assert!(config.include_special_lagnas);
        assert_eq!(config.amsha_selection.count, 1);
        assert_eq!(config.amsha_selection.codes[0], 9);
        assert!(config.amsha_scope.include_bhava_cusps);
        assert!(config.amsha_scope.include_arudha_padas);
        assert!(config.amsha_scope.include_upagrahas);
        assert!(config.amsha_scope.include_sphutas);
        assert!(config.amsha_scope.include_special_lagnas);
    }

    #[test]
    fn full_kundali_config_rejects_too_many_dasha_systems() {
        let state = dummy_state();
        let too_many = (0..(dhruv_vedic_base::dasha::MAX_DASHA_SYSTEMS + 1))
            .map(|_| EnumInput::Str("vimshottari".to_string()))
            .collect::<Vec<_>>();
        let err = to_full_kundali_config(
            &state,
            Some(&FullKundaliConfigInput {
                include_bhava_cusps: None,
                include_graha_positions: None,
                include_bindus: None,
                include_drishti: None,
                include_ashtakavarga: None,
                include_upagrahas: None,
                include_sphutas: None,
                include_special_lagnas: None,
                include_amshas: None,
                include_shadbala: None,
                include_bhavabala: None,
                include_vimsopaka: None,
                include_avastha: None,
                include_charakaraka: None,
                panchang_include_mask: None,
                include_dasha: Some(true),
                charakaraka_scheme: None,
                node_dignity_policy: None,
                upagraha_config: None,
                graha_positions_config: None,
                bindus_config: None,
                drishti_config: None,
                amsha_scope: None,
                amsha_selection: None,
                dasha_config: Some(DashaSelectionConfigInput {
                    systems: Some(too_many),
                    max_level: None,
                    max_levels: None,
                    cycles: None,
                    min_span_years: None,
                    snapshot_utc: None,
                }),
            }),
        )
        .expect_err("oversized systems list should be rejected");
        assert_eq!(err["kind"], "invalid_request");
        assert!(
            err["message"]
                .as_str()
                .expect("error message")
                .contains("systems may contain at most"),
            "unexpected error payload: {err:?}"
        );
    }

    #[test]
    fn full_kundali_config_rejects_too_many_dasha_max_levels() {
        let state = dummy_state();
        let too_many = vec![2; dhruv_vedic_base::dasha::MAX_DASHA_SYSTEMS + 1];
        let err = to_full_kundali_config(
            &state,
            Some(&FullKundaliConfigInput {
                include_bhava_cusps: None,
                include_graha_positions: None,
                include_bindus: None,
                include_drishti: None,
                include_ashtakavarga: None,
                include_upagrahas: None,
                include_sphutas: None,
                include_special_lagnas: None,
                include_amshas: None,
                include_shadbala: None,
                include_bhavabala: None,
                include_vimsopaka: None,
                include_avastha: None,
                include_charakaraka: None,
                panchang_include_mask: None,
                include_dasha: Some(true),
                charakaraka_scheme: None,
                node_dignity_policy: None,
                upagraha_config: None,
                graha_positions_config: None,
                bindus_config: None,
                drishti_config: None,
                amsha_scope: None,
                amsha_selection: None,
                dasha_config: Some(DashaSelectionConfigInput {
                    systems: None,
                    max_level: None,
                    max_levels: Some(too_many),
                    cycles: None,
                    min_span_years: None,
                    snapshot_utc: None,
                }),
            }),
        )
        .expect_err("oversized max_levels list should be rejected");
        assert_eq!(err["kind"], "invalid_request");
        assert!(
            err["message"]
                .as_str()
                .expect("error message")
                .contains("max_levels may contain at most"),
            "unexpected error payload: {err:?}"
        );
    }
}
