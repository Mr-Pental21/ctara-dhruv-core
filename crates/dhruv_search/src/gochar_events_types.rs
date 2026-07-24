//! Typed request/result data for `gochar_events`.

use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::{
    ALL_ARUDHA_PADAS, ALL_GRAHAS, ALL_SPECIAL_LAGNAS, ALL_SPHUTAS, BhavaConfig, RiseSetConfig,
};

use crate::jyotish_types::{FullKundaliConfig, FullKundaliResult};
use crate::panchang_types::MasaInfo;
use crate::sankranti_types::SankrantiConfig;

/// Return trigger basis for Tajaka charts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TajakaReturnBasis {
    /// Trigger when the tropical solar return occurs.
    TropicalSolar,
    /// Trigger when the sidereal solar return occurs.
    SiderealSolar,
}

impl TajakaReturnBasis {
    pub const fn name(self) -> &'static str {
        match self {
            Self::TropicalSolar => "Tropical Solar",
            Self::SiderealSolar => "Sidereal Solar",
        }
    }
}

/// Category for caller-supplied natal target longitudes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NatalTargetKind {
    Graha,
    Bindu,
    Sphuta,
    SpecialLagna,
    ArudhaPada,
    Custom,
}

impl NatalTargetKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Graha => "Graha",
            Self::Bindu => "Bindu",
            Self::Sphuta => "Sphuta",
            Self::SpecialLagna => "Special Lagna",
            Self::ArudhaPada => "Arudha Pada",
            Self::Custom => "Custom",
        }
    }
}

/// Caller-supplied natal target longitude used for gochara conjunction search.
#[derive(Debug, Clone, PartialEq)]
pub struct NatalTargetLongitude {
    pub kind: NatalTargetKind,
    /// Category-local index.
    ///
    /// - `Graha`: `Graha::index()`
    /// - `Sphuta`: `ALL_SPHUTAS` order
    /// - `SpecialLagna`: `ALL_SPECIAL_LAGNAS` order
    /// - `ArudhaPada`: `ALL_ARUDHA_PADAS` order
    /// - `Bindu`: core bindu order
    /// - `Custom`: caller-defined, index carried through unchanged
    pub index: u8,
    /// Caller-supplied display name carried through to output events.
    pub name: String,
    /// Sidereal longitude on the configured chart frame.
    pub longitude_deg: f64,
}

/// Gochar transit code for Ketu (true descending node).
pub use crate::transit_body::TRANSIT_CODE_KETU as GOCHAR_TRANSIT_CODE_KETU;
/// Gochar transit code for Rahu (true ascending node).
pub use crate::transit_body::TRANSIT_CODE_RAHU as GOCHAR_TRANSIT_CODE_RAHU;

/// Transit source supported by `gochar_events` (alias of [`TransitBody`],
/// which is shared with the ingress/conjunction/motion searches).
pub use crate::transit_body::TransitBody as GocharTransitBody;

impl NatalTargetLongitude {
    pub fn display_name(&self) -> &str {
        if !self.name.is_empty() {
            return self.name.as_str();
        }
        match self.kind {
            NatalTargetKind::Graha => ALL_GRAHAS
                .get(self.index as usize)
                .map(|graha| graha.name())
                .unwrap_or("Unknown Graha"),
            NatalTargetKind::Bindu => core_bindu_name(self.index),
            NatalTargetKind::Sphuta => ALL_SPHUTAS
                .get(self.index as usize)
                .map(|sphuta| sphuta.name())
                .unwrap_or("Unknown Sphuta"),
            NatalTargetKind::SpecialLagna => ALL_SPECIAL_LAGNAS
                .get(self.index as usize)
                .map(|lagna| lagna.name())
                .unwrap_or("Unknown Special Lagna"),
            NatalTargetKind::ArudhaPada => ALL_ARUDHA_PADAS
                .get(self.index as usize)
                .map(|pada| pada.name())
                .unwrap_or("Unknown Arudha Pada"),
            NatalTargetKind::Custom => "Custom",
        }
    }
}

/// Search and output settings for `gochar_events`.
#[derive(Debug, Clone)]
pub struct GocharEventsConfig {
    pub tajaka_return_basis: TajakaReturnBasis,
    pub yearly_count: usize,
    pub monthly_count: usize,
    pub transit_window_days: f64,
    pub include_return_charts: bool,
    pub solar_step_size_days: f64,
    pub lunar_step_size_days: f64,
    pub solar_convergence_days: f64,
    pub lunar_convergence_days: f64,
    pub max_iterations: u32,
}

impl Default for GocharEventsConfig {
    fn default() -> Self {
        Self {
            tajaka_return_basis: TajakaReturnBasis::SiderealSolar,
            yearly_count: 2,
            monthly_count: 12,
            transit_window_days: 365.25,
            include_return_charts: true,
            solar_step_size_days: 1.0,
            lunar_step_size_days: 0.5,
            solar_convergence_days: 1e-8,
            lunar_convergence_days: 1e-8,
            max_iterations: 50,
        }
    }
}

/// One logical `gochar_events` request.
#[derive(Debug)]
pub struct GocharEventsOperation<'a> {
    pub birth_utc: UtcTime,
    pub at_utc: UtcTime,
    pub location: dhruv_vedic_base::GeoLocation,
    pub eop: &'a EopKernel,
    pub bhava_config: BhavaConfig,
    pub riseset_config: RiseSetConfig,
    pub sankranti_config: SankrantiConfig,
    pub kundali_config: FullKundaliConfig,
    pub config: GocharEventsConfig,
    pub transit_bodies: Vec<GocharTransitBody>,
    pub natal_targets: Vec<NatalTargetLongitude>,
}

/// Reference values derived from the birth chart and used by return searches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GocharReference {
    pub natal_tropical_solar_longitude_deg: f64,
    pub natal_sidereal_solar_longitude_deg: f64,
    pub natal_elongation_deg: f64,
    pub natal_masa: MasaInfo,
}

/// A before/after window around the query time.
#[derive(Debug, Clone, PartialEq)]
pub struct EventWindow<T> {
    pub before: Vec<T>,
    pub after: Vec<T>,
}

impl<T> Default for EventWindow<T> {
    fn default() -> Self {
        Self {
            before: Vec::new(),
            after: Vec::new(),
        }
    }
}

/// One Tajaka return chart event.
#[derive(Debug, Clone)]
pub struct TajakaReturnEvent {
    pub utc: UtcTime,
    pub jd_tdb: f64,
    pub basis: TajakaReturnBasis,
    pub target_solar_longitude_deg: f64,
    pub event_solar_longitude_deg: f64,
    pub chart: Option<FullKundaliResult>,
}

/// One Tithi Pravesha return chart event.
#[derive(Debug, Clone)]
pub struct TithiPraveshaEvent {
    pub utc: UtcTime,
    pub jd_tdb: f64,
    pub target_elongation_deg: f64,
    pub event_elongation_deg: f64,
    pub masa: MasaInfo,
    pub chart: Option<FullKundaliResult>,
}

/// Exact event family for gochara-to-natal contacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitAspectKind {
    Conjunction,
    Opposition,
    Special,
}

impl TransitAspectKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Conjunction => "Conjunction",
            Self::Opposition => "Opposition",
            Self::Special => "Special Aspect",
        }
    }
}

/// Which side owns the aspect rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitAspectOwner {
    GocharBody,
    NatalTarget,
}

impl TransitAspectOwner {
    pub const fn name(self) -> &'static str {
        match self {
            Self::GocharBody => "Gochar Body",
            Self::NatalTarget => "Natal Target",
        }
    }
}

/// One exact transit event to a caller-supplied natal target.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitToNatalAspectEvent {
    pub transit_body: GocharTransitBody,
    pub target: NatalTargetLongitude,
    pub target_name: String,
    pub aspect_kind: TransitAspectKind,
    pub aspect_owner: TransitAspectOwner,
    pub aspect_angle_deg: f64,
    pub utc: UtcTime,
    pub jd_tdb: f64,
    pub transit_longitude_deg: f64,
    pub target_longitude_deg: f64,
    pub actual_separation_deg: f64,
}

/// Aggregate `gochar_events` result.
#[derive(Debug, Clone)]
pub struct GocharEventsResult {
    pub birth_utc: UtcTime,
    pub at_utc: UtcTime,
    pub reference: GocharReference,
    pub yearly_tajaka: EventWindow<TajakaReturnEvent>,
    pub yearly_tithi_pravesha: EventWindow<TithiPraveshaEvent>,
    pub monthly_tajaka: EventWindow<TajakaReturnEvent>,
    pub monthly_tithi_pravesha: EventWindow<TithiPraveshaEvent>,
    pub transit_events: Vec<TransitToNatalAspectEvent>,
}

fn core_bindu_name(index: u8) -> &'static str {
    const BINDU_NAMES: [&str; 19] = [
        "Arudha Lagna",
        "Dhana Pada",
        "Vikrama Pada",
        "Matri Pada",
        "Mantra Pada",
        "Roga Pada",
        "Dara Pada",
        "Mrityu Pada",
        "Pitri Pada",
        "Rajya Pada",
        "Labha Pada",
        "Upapada",
        "Bhrigu Bindu",
        "Pranapada Lagna",
        "Gulika",
        "Maandi",
        "Hora Lagna",
        "Ghati Lagna",
        "Sree Lagna",
    ];
    BINDU_NAMES
        .get(index as usize)
        .copied()
        .unwrap_or("Unknown Bindu")
}
