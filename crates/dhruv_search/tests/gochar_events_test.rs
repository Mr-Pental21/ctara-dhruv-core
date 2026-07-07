//! Integration tests for `gochar_events`.
//!
//! Requires kernel files. Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Body, Engine, EngineConfig};
use dhruv_search::{
    GocharEventsConfig, GocharEventsOperation, GocharTransitBody, NatalTargetKind,
    NatalTargetLongitude, SankrantiConfig, TajakaReturnBasis, gochar_events,
};
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::{BhavaConfig, GeoLocation, RiseSetConfig};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";
const EOP_PATH: &str = "../../kernels/data/finals2000A.all";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping gochar_events_test: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn load_eop() -> Option<EopKernel> {
    if !Path::new(EOP_PATH).exists() {
        eprintln!("Skipping gochar_events_test: EOP file not found");
        return None;
    }
    EopKernel::load(Path::new(EOP_PATH)).ok()
}

fn new_delhi() -> GeoLocation {
    GeoLocation::new(28.6139, 77.2090, 0.0)
}

fn birth_utc() -> UtcTime {
    UtcTime::new(1990, 5, 17, 10, 30, 0.0)
}

fn at_utc() -> UtcTime {
    UtcTime::new(2024, 1, 15, 12, 0, 0.0)
}

#[test]
fn gochar_events_returns_windowed_results_and_same_masa_yearly_tithi() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };

    let mut config = GocharEventsConfig::default();
    config.tajaka_return_basis = TajakaReturnBasis::SiderealSolar;
    config.yearly_count = 1;
    config.monthly_count = 2;
    config.include_return_charts = false;

    let op = GocharEventsOperation {
        birth_utc: birth_utc(),
        at_utc: at_utc(),
        location: new_delhi(),
        eop: &eop,
        bhava_config: BhavaConfig::default(),
        riseset_config: RiseSetConfig::default(),
        sankranti_config: SankrantiConfig::default_lahiri(),
        kundali_config: Default::default(),
        config,
        transit_bodies: vec![GocharTransitBody::Body(Body::Sun)],
        natal_targets: vec![NatalTargetLongitude {
            kind: NatalTargetKind::Custom,
            index: 0,
            name: "Test Point".to_string(),
            longitude_deg: 0.0,
        }],
    };

    let result = gochar_events(&engine, &op).expect("gochar_events should succeed");

    assert_eq!(result.yearly_tajaka.before.len(), 1);
    assert_eq!(result.yearly_tajaka.after.len(), 1);
    assert_eq!(result.monthly_tajaka.before.len(), 2);
    assert_eq!(result.monthly_tajaka.after.len(), 2);
    assert_eq!(result.yearly_tithi_pravesha.before.len(), 1);
    assert_eq!(result.yearly_tithi_pravesha.after.len(), 1);
    assert_eq!(result.monthly_tithi_pravesha.before.len(), 2);
    assert_eq!(result.monthly_tithi_pravesha.after.len(), 2);
    assert!(!result.transit_events.is_empty());
    assert!(
        result
            .transit_events
            .iter()
            .all(|event| event.target_name == "Test Point"
                && event.target.display_name() == "Test Point")
    );
    assert!(
        result
            .transit_events
            .iter()
            .any(|event| event.aspect_angle_deg == 180.0)
    );

    for event in result
        .yearly_tithi_pravesha
        .before
        .iter()
        .chain(result.yearly_tithi_pravesha.after.iter())
    {
        assert_eq!(event.masa.masa, result.reference.natal_masa.masa);
        assert_eq!(event.masa.adhika, result.reference.natal_masa.adhika);
        assert!(event.chart.is_none());
    }

    for event in result
        .yearly_tajaka
        .before
        .iter()
        .chain(result.yearly_tajaka.after.iter())
    {
        assert_eq!(event.basis, TajakaReturnBasis::SiderealSolar);
        assert!(event.chart.is_none());
    }
}

#[test]
fn gochar_events_can_embed_return_charts() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };

    let mut config = GocharEventsConfig::default();
    config.yearly_count = 1;
    config.monthly_count = 1;
    config.include_return_charts = true;

    let op = GocharEventsOperation {
        birth_utc: birth_utc(),
        at_utc: at_utc(),
        location: new_delhi(),
        eop: &eop,
        bhava_config: BhavaConfig::default(),
        riseset_config: RiseSetConfig::default(),
        sankranti_config: SankrantiConfig::default_lahiri(),
        kundali_config: Default::default(),
        config,
        transit_bodies: Vec::new(),
        natal_targets: Vec::new(),
    };

    let result = gochar_events(&engine, &op).expect("gochar_events should succeed");

    assert!(result.yearly_tajaka.after[0].chart.is_some());
    assert!(result.yearly_tithi_pravesha.after[0].chart.is_some());
    assert!(result.monthly_tajaka.after[0].chart.is_some());
    assert!(result.monthly_tithi_pravesha.after[0].chart.is_some());
}

#[test]
fn gochar_events_includes_special_aspects_from_gochar_and_natal_mars() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };

    let mut config = GocharEventsConfig::default();
    config.yearly_count = 1;
    config.monthly_count = 1;
    config.include_return_charts = false;

    let op = GocharEventsOperation {
        birth_utc: birth_utc(),
        at_utc: at_utc(),
        location: new_delhi(),
        eop: &eop,
        bhava_config: BhavaConfig::default(),
        riseset_config: RiseSetConfig::default(),
        sankranti_config: SankrantiConfig::default_lahiri(),
        kundali_config: Default::default(),
        config,
        transit_bodies: vec![GocharTransitBody::Body(Body::Mars)],
        natal_targets: vec![NatalTargetLongitude {
            kind: NatalTargetKind::Graha,
            index: 2,
            name: "Natal Mars".to_string(),
            longitude_deg: 120.0,
        }],
    };

    let result = gochar_events(&engine, &op).expect("gochar_events should succeed");

    assert!(result.transit_events.iter().any(|event| {
        event.aspect_kind == dhruv_search::TransitAspectKind::Special
            && event.aspect_owner == dhruv_search::TransitAspectOwner::GocharBody
            && event.aspect_angle_deg == 90.0
    }));
    assert!(result.transit_events.iter().any(|event| {
        event.aspect_kind == dhruv_search::TransitAspectKind::Special
            && event.aspect_owner == dhruv_search::TransitAspectOwner::NatalTarget
            && event.aspect_angle_deg == 210.0
    }));
}

#[test]
fn gochar_events_accepts_rahu_ketu_and_outer_planets_as_transit_bodies() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };

    let mut config = GocharEventsConfig::default();
    config.yearly_count = 1;
    config.monthly_count = 1;
    config.include_return_charts = false;
    config.transit_window_days = 120.0;

    let op = GocharEventsOperation {
        birth_utc: birth_utc(),
        at_utc: at_utc(),
        location: new_delhi(),
        eop: &eop,
        bhava_config: BhavaConfig::default(),
        riseset_config: RiseSetConfig::default(),
        sankranti_config: SankrantiConfig::default_lahiri(),
        kundali_config: Default::default(),
        config,
        transit_bodies: vec![
            GocharTransitBody::Rahu,
            GocharTransitBody::Ketu,
            GocharTransitBody::Body(Body::Uranus),
            GocharTransitBody::Body(Body::Neptune),
            GocharTransitBody::Body(Body::Pluto),
        ],
        natal_targets: vec![NatalTargetLongitude {
            kind: NatalTargetKind::Custom,
            index: 0,
            name: "Transit Probe".to_string(),
            longitude_deg: 180.0,
        }],
    };

    let result = gochar_events(&engine, &op).expect("gochar_events should succeed");

    assert!(!result.transit_events.is_empty());
    assert!(
        result
            .transit_events
            .iter()
            .any(|event| event.transit_body == GocharTransitBody::Rahu)
    );
    assert!(
        result
            .transit_events
            .iter()
            .any(|event| event.transit_body == GocharTransitBody::Ketu)
    );
}
