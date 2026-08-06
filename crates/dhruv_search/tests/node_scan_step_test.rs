//! Scan-step regression tests for the lunar nodes.
//!
//! The true node stations about weekly. Each direct excursion re-crosses
//! longitudes it just passed, so a contact near a station is a *pair* of
//! crossings often less than two days apart. A coarse-scan step that spans
//! the pair sees no sign change and drops both — silently, since a missing
//! root looks the same as no event. These tests pin the node step to a
//! value that resolves those pairs, and pin every scan path to the same
//! value so they cannot drift apart again.
//!
//! Requires kernel files. Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::{
    GocharEventsConfig, GocharEventsOperation, GocharTransitBody, NatalTargetKind,
    NatalTargetLongitude, SankrantiConfig, TransitBody, gochar_events, search_fixed_longitudes,
};
use dhruv_time::{EopKernel, UtcTime, calendar_to_jd};
use dhruv_vedic_base::{BhavaConfig, GeoLocation, RiseSetConfig};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";
const EOP_PATH: &str = "../../kernels/data/finals2000A.all";

/// A sidereal longitude Rahu reaches only via a station excursion in
/// 2020-2024: it is crossed twice within 1.7 days (2020-01-09 and
/// 2020-01-10) and never again in the window, because the node's -19.3
/// deg/yr mean motion does not bring it back for ~18.6 years. A 2-day scan
/// step reports *no* contact at all for this longitude.
const VULNERABLE_TARGET_DEG: f64 = 74.306_269;
const WINDOW_START_JD: f64 = 2_458_849.5;
const WINDOW_YEARS: f64 = 4.0;
/// Fine reference step: 32x finer than the node's production step.
const REFERENCE_STEP_DAYS: f64 = 0.062_5;

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping node_scan_step_test: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 4096, false);
    Engine::new(config).ok()
}

fn approx_jd(utc: &UtcTime) -> f64 {
    let day_frac = f64::from(utc.day)
        + f64::from(utc.hour) / 24.0
        + f64::from(utc.minute) / 1440.0
        + utc.second / 86_400.0;
    calendar_to_jd(utc.year, utc.month, day_frac)
}

/// The node's default step resolves a station-excursion contact pair that a
/// coarser step drops entirely.
#[test]
fn node_default_step_resolves_station_excursion_contacts() {
    let Some(engine) = load_engine() else { return };
    let end_jd = WINDOW_START_JD + 365.25 * WINDOW_YEARS;

    let default_step = TransitBody::Rahu.default_ingress_step_days();
    let mut production = SankrantiConfig::for_body(
        dhruv_vedic_base::AyanamshaSystem::Lahiri,
        false,
        TransitBody::Rahu,
    );
    production.step_size_days = default_step;

    let mut reference = production;
    reference.step_size_days = REFERENCE_STEP_DAYS;

    let found = search_fixed_longitudes(
        &engine,
        TransitBody::Rahu,
        WINDOW_START_JD,
        end_jd,
        VULNERABLE_TARGET_DEG,
        &[],
        &production,
    )
    .expect("production scan");
    let truth = search_fixed_longitudes(
        &engine,
        TransitBody::Rahu,
        WINDOW_START_JD,
        end_jd,
        VULNERABLE_TARGET_DEG,
        &[],
        &reference,
    )
    .expect("reference scan");

    assert!(
        truth.len() >= 2,
        "fixture no longer exercises a station excursion: reference found {} contacts",
        truth.len()
    );
    assert_eq!(
        found.len(),
        truth.len(),
        "node step {default_step} d missed contacts near a station: found {} of {}",
        found.len(),
        truth.len()
    );
    for (a, b) in found.iter().zip(truth.iter()) {
        assert!(
            (a.jd_tdb - b.jd_tdb).abs() < 1e-6,
            "contact instant disagrees with the fine-step reference: {} vs {}",
            a.jd_tdb,
            b.jd_tdb
        );
    }
}

/// `gochar_events` must resolve node contacts as well as a fine-step
/// reference does. The two paths ran off separate step tables once, and the
/// gochar one was coarse enough to drop station-excursion pairs.
///
/// Whether a coarse step drops a pair depends on where its scan grid
/// happens to land, so this sweeps the query instant across several phases
/// of a 2-day grid. A step that cannot resolve the pair fails at some of
/// those phases even though it looks correct at others.
#[test]
fn gochar_resolves_node_contacts_at_every_grid_phase() {
    let Some(engine) = load_engine() else { return };
    if !Path::new(EOP_PATH).exists() {
        eprintln!("Skipping node_scan_step_test: EOP file not found");
        return;
    }
    let Ok(eop) = EopKernel::load(Path::new(EOP_PATH)) else {
        return;
    };

    let window_days = 20.0;
    let sankranti_config = SankrantiConfig::default_lahiri();
    let mut reference_config = sankranti_config;
    reference_config.step_size_days = REFERENCE_STEP_DAYS;

    let mut checked = 0usize;
    for hour_offset in [0, 5, 10, 15, 20, 25, 30, 35, 40, 45] {
        // Walk the query instant across two days in 5-hour steps, so the
        // scan grid falls at many different phases relative to the contacts.
        let at_utc = UtcTime::new(2020, 1, 9 + hour_offset / 24, hour_offset % 24, 0, 0.0);
        let at_jd = approx_jd(&at_utc);

        let mut config = GocharEventsConfig::default();
        config.include_return_charts = false;
        config.yearly_count = 1;
        config.monthly_count = 1;
        config.transit_window_days = window_days;

        let op = GocharEventsOperation {
            birth_utc: UtcTime::new(1990, 5, 17, 10, 30, 0.0),
            at_utc,
            location: GeoLocation::new(28.6139, 77.2090, 0.0),
            eop: &eop,
            bhava_config: BhavaConfig::default(),
            riseset_config: RiseSetConfig::default(),
            sankranti_config,
            kundali_config: Default::default(),
            config,
            transit_bodies: vec![GocharTransitBody::Rahu],
            natal_targets: vec![NatalTargetLongitude {
                kind: NatalTargetKind::Custom,
                index: 0,
                name: "Vulnerable Point".to_string(),
                longitude_deg: VULNERABLE_TARGET_DEG,
            }],
        };

        let result = gochar_events(&engine, &op).expect("gochar_events");
        let gochar_conjunctions: Vec<f64> = result
            .transit_events
            .iter()
            .filter(|e| e.aspect_angle_deg == 0.0)
            .map(|e| e.jd_tdb)
            .collect();

        // Fine-step truth over a window inset by a day, so the UTC->TDB
        // offset between the two anchors cannot move a contact across an
        // edge and cause a spurious mismatch.
        let truth = search_fixed_longitudes(
            &engine,
            TransitBody::Rahu,
            at_jd - (window_days - 1.0),
            at_jd + (window_days - 1.0),
            VULNERABLE_TARGET_DEG,
            &[0.0],
            &reference_config,
        )
        .expect("reference scan");

        for event in &truth {
            checked += 1;
            let matched = gochar_conjunctions
                .iter()
                .any(|&jd| (jd - event.jd_tdb).abs() < 1e-6);
            assert!(
                matched,
                "at +{hour_offset}h, gochar_events missed the Rahu contact at jd {} that a \
                 {REFERENCE_STEP_DAYS}-day reference scan finds; the gochar node scan step is \
                 too coarse to resolve a station excursion",
                event.jd_tdb
            );
        }
    }

    assert!(
        checked >= 10,
        "fixture no longer exercises node contacts: only {checked} reference contacts across \
         all phases"
    );
}
