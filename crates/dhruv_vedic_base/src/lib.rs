//! Open derived Vedic calculations built on core ephemeris outputs.
//!
//! This crate provides:
//! - Ayanamsha computation for 20 sidereal reference systems
//! - Sunrise/sunset and twilight calculations
//! - Lagna (Ascendant) and MC computation
//! - Bhava (house) systems: 10 division methods
//! - Rashi (zodiac sign) and DMS conversion
//! - Nakshatra (lunar mansion) with pada, 27 and 28 schemes
//! - Tithi (lunar day), Karana (half-tithi), Yoga (luni-solar yoga)
//! - Vaar (weekday), Hora (planetary hour), Ghatika (time division)
//! - Graha (planet) enum with rashi lordship
//! - Sphuta (sensitive point) calculations (16 formulas)
//!
//! All implementations are clean-room, derived from IAU standards
//! and public astronomical formulas.

pub mod amsha;
pub mod arudha;
pub mod ashtakavarga;
pub mod avastha;
pub mod ayana_type;
pub mod ayanamsha;
mod ayanamsha_anchor;
pub mod bhava;
pub mod bhava_types;
pub mod combustion;
pub mod dasha;
pub mod drishti;
pub mod error;
pub mod ghatika;
pub mod graha;
pub mod graha_relationships;
pub mod hora;
pub mod karana;
pub mod lagna;
pub mod lunar_nodes;
pub mod masa;
pub mod nakshatra;
pub mod rashi;
pub mod riseset;
pub mod riseset_types;
pub mod samvatsara;
pub mod shadbala;
pub mod special_lagna;
pub mod sphuta;
pub mod tithi;
pub mod upagraha;
pub mod util;
pub mod vaar;
pub mod vimsopaka;
pub mod yoga;

pub use amsha::{
    amsha_from_rashi_position, amsha_longitude, amsha_longitudes, amsha_rashi_info,
    amsha_rashi_infos, rashi_element, rashi_position_to_longitude, Amsha, AmshaRequest,
    AmshaVariation, RashiElement, ALL_AMSHAS, SHODASHAVARGA,
};
pub use arudha::{all_arudha_padas, arudha_pada, ArudhaPada, ArudhaResult, ALL_ARUDHA_PADAS};
pub use ashtakavarga::{
    calculate_all_bav, calculate_ashtakavarga, calculate_bav, calculate_sav, ekadhipatya_sodhana,
    trikona_sodhana, AshtakavargaResult, BhinnaAshtakavarga, SarvaAshtakavarga, BAV_TOTALS,
    SAV_TOTAL,
};
pub use avastha::{
    all_avasthas, all_baladi_avasthas, all_deeptadi_avasthas, all_jagradadi_avasthas,
    all_lajjitadi_avasthas, all_sayanadi_avasthas, baladi_avastha, deeptadi_avastha,
    jagradadi_avastha, lajjitadi_avastha, lost_planetary_war, navamsa_number,
    sayanadi_all_sub_states, sayanadi_avastha, sayanadi_sub_state, AllGrahaAvasthas, AvasthaInputs,
    BaladiAvastha, DeeptadiAvastha, GrahaAvasthas, JagradadiAvastha, LajjitadiAvastha,
    LajjitadiInputs, NameGroup, SayanadiAvastha, SayanadiInputs, SayanadiResult, SayanadiSubState,
    ALL_NAME_GROUPS, NAME_GROUP_ANKAS,
};
pub use ayana_type::{ayana_from_sidereal_longitude, Ayana, ALL_AYANAS};
pub use ayanamsha::{
    ayanamsha_deg, ayanamsha_deg_with_model, ayanamsha_mean_deg, ayanamsha_mean_deg_with_model,
    ayanamsha_true_deg, ayanamsha_true_deg_with_model, jd_tdb_to_centuries,
    tdb_seconds_to_centuries, AyanamshaSystem,
};
pub use bhava::compute_bhavas;
pub use bhava_types::{
    Bhava, BhavaConfig, BhavaReferenceMode, BhavaResult, BhavaStartingPoint, BhavaSystem,
};
pub use combustion::{all_combustion_status, combustion_threshold, is_combust};
pub use dasha::{
    find_active_period, nakshatra_birth_balance, nakshatra_child_period, nakshatra_children,
    nakshatra_complete_level, nakshatra_hierarchy, nakshatra_level0, nakshatra_level0_entity,
    nakshatra_snapshot, snapshot_from_hierarchy, vimshottari_config, DashaEntity, DashaHierarchy,
    DashaLevel, DashaPeriod, DashaSnapshot, DashaSystem, DashaVariationConfig,
    NakshatraDashaConfig, SubPeriodMethod, YoginiScheme, ALL_DASHA_SYSTEMS, DAYS_PER_YEAR,
    DEFAULT_DASHA_LEVEL, MAX_DASHA_LEVEL, MAX_DASHA_SYSTEMS, MAX_PERIODS_PER_LEVEL,
};
pub use drishti::{
    base_virupa, graha_drishti, graha_drishti_matrix, special_virupa, DrishtiEntry,
    GrahaDrishtiMatrix,
};
pub use error::VedicError;
pub use ghatika::{ghatika_from_elapsed, GhatikaPosition, GHATIKA_COUNT, GHATIKA_MINUTES};
pub use graha::{
    nth_rashi_from, rashi_lord, rashi_lord_by_index, Graha, ALL_GRAHAS, GRAHA_KAKSHA_VALUES,
    SAPTA_GRAHAS,
};
pub use graha_relationships::{
    debilitation_degree, dignity_in_rashi, dignity_in_rashi_with_positions, exaltation_degree,
    graha_gender, hora_lord, masa_lord, moolatrikone_range, moon_benefic_nature, naisargika_maitri,
    natural_benefic_malefic, node_dignity_in_rashi, own_signs, panchadha_maitri, samvatsara_lord,
    tatkalika_maitri, vaar_lord, BeneficNature, Dignity, GrahaGender, NaisargikaMaitri,
    NodeDignityPolicy, PanchadhaMaitri, TatkalikaMaitri,
};
pub use hora::{hora_at, vaar_day_lord, Hora, CHALDEAN_SEQUENCE, HORA_COUNT};
pub use karana::{karana_from_elongation, Karana, KaranaPosition, ALL_KARANAS, KARANA_SEGMENT_DEG};
pub use lagna::{lagna_and_mc_rad, lagna_longitude_rad, mc_longitude_rad, ramc_rad};
pub use lunar_nodes::{
    lunar_node_deg, lunar_node_deg_for_epoch, lunar_node_deg_for_epoch_with_model, mean_ketu_deg,
    mean_rahu_deg, true_ketu_deg, true_rahu_deg, LunarNode, NodeMode,
};
pub use masa::{masa_from_rashi_index, Masa, ALL_MASAS};
pub use nakshatra::{
    nakshatra28_from_longitude, nakshatra28_from_tropical, nakshatra_from_longitude,
    nakshatra_from_tropical, Nakshatra, Nakshatra28, Nakshatra28Info, NakshatraInfo,
    ALL_NAKSHATRAS_27, ALL_NAKSHATRAS_28, NAKSHATRA_SPAN_27,
};
pub use rashi::{
    deg_to_dms, dms_to_deg, rashi_from_longitude, rashi_from_tropical, Dms, Rashi, RashiInfo,
    ALL_RASHIS,
};
pub use riseset::{approximate_local_noon_jd, compute_all_events, compute_rise_set};
pub use riseset_types::{GeoLocation, RiseSetConfig, RiseSetEvent, RiseSetResult, SunLimb};
pub use samvatsara::{samvatsara_from_year, Samvatsara, ALL_SAMVATSARAS, SAMVATSARA_EPOCH_YEAR};
pub use shadbala::{
    abda_bala, all_ayana_balas, all_cheshta_balas, all_dig_balas, all_drekkana_balas,
    all_drik_balas, all_hora_balas, all_kala_balas, all_kendradi_balas, all_masa_balas,
    all_naisargika_balas, all_nathonnatha_balas, all_ojhayugma_balas, all_paksha_balas,
    all_shadbalas_from_inputs, all_sthana_balas, all_tribhaga_balas, all_uchcha_balas,
    all_vara_balas, all_yuddha_balas, ayana_bala, cheshta_bala, dig_bala, drekkana_bala, drik_bala,
    hora_bala as shadbala_hora_bala, kala_bala, kendradi_bala, masa_bala as shadbala_masa_bala,
    naisargika_bala, nathonnatha_bala, ojhayugma_bala, paksha_bala, shadbala_from_inputs,
    sthana_bala, tribhaga_bala, uchcha_bala, vara_bala, yuddha_bala, KalaBalaBreakdown,
    KalaBalaInputs, ShadbalaBreakdown, ShadbalaInputs, SthanaBalaBreakdown, DIG_BALA_BHAVA,
    MAX_SPEED, NAISARGIKA_BALA, REQUIRED_STRENGTH,
};
pub use special_lagna::{
    all_special_lagnas, bhava_lagna, ghati_lagna, ghatikas_since_sunrise, hora_lagna, indu_lagna,
    pranapada_lagna, sree_lagna, varnada_lagna, vighati_lagna, AllSpecialLagnas, SpecialLagna,
    ALL_SPECIAL_LAGNAS,
};
pub use sphuta::{
    all_sphutas, avayoga_sphuta, beeja_sphuta, bhrigu_bindu, chatussphuta, deha_sphuta,
    kshetra_sphuta, kunda, mrityu_sphuta, panchasphuta, prana_sphuta, rahu_tithi_sphuta,
    sookshma_trisphuta, tithi_sphuta, trisphuta, yoga_sphuta, yoga_sphuta_normalized, Sphuta,
    SphutalInputs, ALL_SPHUTAS,
};
pub use tithi::{
    tithi_from_elongation, Paksha, Tithi, TithiPosition, ALL_TITHIS, TITHI_SEGMENT_DEG,
};
pub use upagraha::{
    day_portion_index, night_portion_index, portion_jd_range, sun_based_upagrahas,
    time_upagraha_jd, time_upagraha_planet, AllUpagrahas, SunBasedUpagrahas, Upagraha,
    ALL_UPAGRAHAS, TIME_BASED_UPAGRAHAS,
};
pub use util::normalize_360;
pub use vaar::{vaar_from_jd, Vaar, ALL_VAARS};
pub use vimsopaka::{
    all_dashavarga_vimsopaka, all_saptavarga_vimsopaka, all_shadvarga_vimsopaka,
    all_shodasavarga_vimsopaka, all_vimsopaka_balas, dashavarga_vimsopaka, saptavarga_vimsopaka,
    shadvarga_vimsopaka, shodasavarga_vimsopaka, vimsopaka_bala, vimsopaka_dignity_points,
    vimsopaka_from_entries, VargaDignityEntry, VargaWeight, VimsopakaBala, DASHAVARGA, SAPTAVARGA,
    SHADVARGA, SHODASAVARGA,
};
pub use yoga::{yoga_from_sum, Yoga, YogaPosition, ALL_YOGAS, YOGA_SEGMENT_DEG};
