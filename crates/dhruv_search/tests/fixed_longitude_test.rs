//! Integration tests for the fixed-longitude transit search op.
//!
//! Requires kernel files. Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Body, Engine, EngineConfig};
use dhruv_search::{
    FixedLongitudeOperation, FixedLongitudeQuery, FixedLongitudeResult, SankrantiConfig,
    SearchError, TransitBody, fixed_longitude, next_fixed_longitude, next_specific_ingress,
    prev_fixed_longitude, search_fixed_longitudes,
};
use dhruv_time::{UtcTime, calendar_to_jd};
use dhruv_vedic_base::{AyanamshaSystem, NodeMode, Rashi};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping fixed_longitude_test: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn config_for(body: TransitBody) -> SankrantiConfig {
    SankrantiConfig::for_body(AyanamshaSystem::Lahiri, false, body)
}

/// Approximate JD of a UTC timestamp; used only as a search anchor and for
/// comparing two UTC event stamps on the same scale.
fn approx_jd(utc: &UtcTime) -> f64 {
    let day_frac = f64::from(utc.day)
        + f64::from(utc.hour) / 24.0
        + f64::from(utc.minute) / 1440.0
        + utc.second / 86_400.0;
    calendar_to_jd(utc.year, utc.month, day_frac)
}

fn pm180(deg: f64) -> f64 {
    let mut d = deg % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d <= -180.0 {
        d += 360.0;
    }
    d
}

/// Reaching a rashi-cusp longitude is the same physical event as the
/// specific-rashi ingress of that cusp.
#[test]
fn sun_cusp_reach_matches_specific_ingress() {
    let Some(engine) = load_engine() else { return };
    let config = SankrantiConfig::default_lahiri();
    let utc = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let at_jd = approx_jd(&utc);

    let via_ingress = next_specific_ingress(
        &engine,
        TransitBody::Body(Body::Sun),
        &utc,
        Rashi::Vrishabha,
        &config,
    )
    .unwrap()
    .expect("ingress event");
    let via_fixed = next_fixed_longitude(
        &engine,
        TransitBody::Body(Body::Sun),
        at_jd,
        30.0,
        &[],
        &config,
    )
    .unwrap()
    .expect("fixed-longitude event");

    let delta_days = (approx_jd(&via_fixed.utc) - approx_jd(&via_ingress.utc)).abs();
    assert!(
        delta_days < 2e-5,
        "cusp reach vs ingress differ by {delta_days} days"
    );
    assert!(via_fixed.actual_separation_deg < 1e-3);
    assert!((via_fixed.matched_longitude_deg - 30.0).abs() < 1e-12);
    assert!((via_fixed.sidereal_longitude_deg - 30.0).abs() < 1e-3);
}

#[test]
fn next_prev_round_trip() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Sun);
    let config = config_for(body);
    let at_jd = approx_jd(&UtcTime::new(2024, 3, 1, 0, 0, 0.0));

    let next = next_fixed_longitude(&engine, body, at_jd, 123.0, &[], &config)
        .unwrap()
        .expect("next event");
    assert!(next.jd_tdb > at_jd);
    let prev = prev_fixed_longitude(&engine, body, next.jd_tdb + 0.5, 123.0, &[], &config)
        .unwrap()
        .expect("prev event");
    assert!(
        (prev.jd_tdb - next.jd_tdb).abs() < 1e-6,
        "round trip drifted {} days",
        (prev.jd_tdb - next.jd_tdb).abs()
    );
}

/// Two years of Sun vs two angles: one event per angle per lap, sorted by
/// time, each on its matched longitude.
#[test]
fn sun_range_multi_angle() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Sun);
    let config = config_for(body);
    let start = approx_jd(&UtcTime::new(2024, 1, 1, 0, 0, 0.0));
    let end = approx_jd(&UtcTime::new(2026, 1, 1, 0, 0, 0.0));

    let events =
        search_fixed_longitudes(&engine, body, start, end, 100.0, &[0.0, 180.0], &config).unwrap();
    assert_eq!(events.len(), 4, "2 laps x 2 angles, got {}", events.len());
    for pair in events.windows(2) {
        assert!(pair[0].jd_tdb < pair[1].jd_tdb, "events must be sorted");
    }
    for ev in &events {
        assert!(ev.actual_separation_deg < 1e-3);
        let expected = (100.0 + ev.angle_deg).rem_euclid(360.0);
        assert!((ev.matched_longitude_deg - expected).abs() < 1e-12);
        assert!(pm180(ev.sidereal_longitude_deg - ev.matched_longitude_deg).abs() < 1e-3);
    }
}

/// Mars special-aspect flag: offsets 360-90, 360-210 are appended so the
/// moving body casts its 4th/8th drishti onto the target.
#[test]
fn mars_special_angles_flag() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Mars);
    let target = 50.0;
    let op = FixedLongitudeOperation {
        body,
        target_longitude_deg: target,
        target_angles_deg: Vec::new(),
        include_special_angles: true,
        config: config_for(body),
        query: FixedLongitudeQuery::Range {
            start_jd_tdb: approx_jd(&UtcTime::new(2024, 1, 1, 0, 0, 0.0)),
            end_jd_tdb: approx_jd(&UtcTime::new(2026, 6, 1, 0, 0, 0.0)),
        },
    };
    let FixedLongitudeResult::Many(events) = fixed_longitude(&engine, &op).unwrap() else {
        panic!("range query must return Many");
    };
    for angle in [0.0, 150.0, 270.0] {
        assert!(
            events.iter().any(|ev| (ev.angle_deg - angle).abs() < 1e-9),
            "expected at least one event for angle {angle}"
        );
    }
    for ev in &events {
        // angle 270 = special 90 (4th), angle 150 = special 210 (8th):
        // body longitude + special ≡ target.
        let special = (360.0 - ev.angle_deg).rem_euclid(360.0);
        let cast = pm180(ev.sidereal_longitude_deg + special - target).abs();
        assert!(
            cast < 1e-3,
            "special aspect must land on the target, off by {cast} deg"
        );
    }
}

/// Rahu/Ketu are searchable; mean vs true node models give distinct roots.
#[test]
fn rahu_reaches_target_in_both_node_modes() {
    let Some(engine) = load_engine() else { return };
    let at_jd = approx_jd(&UtcTime::new(2024, 1, 1, 0, 0, 0.0));
    let mut config = config_for(TransitBody::Rahu);

    let true_event = next_fixed_longitude(&engine, TransitBody::Rahu, at_jd, 200.0, &[], &config)
        .unwrap()
        .expect("true-node event");
    assert!(true_event.actual_separation_deg < 1e-3);

    config.node_mode = NodeMode::Mean;
    let mean_event = next_fixed_longitude(&engine, TransitBody::Rahu, at_jd, 200.0, &[], &config)
        .unwrap()
        .expect("mean-node event");
    assert!(mean_event.actual_separation_deg < 1e-3);
    assert!(
        (mean_event.jd_tdb - true_event.jd_tdb).abs() > 1e-4,
        "mean and true node roots should differ"
    );
}

/// A range reaching past the loaded ephemeris coverage returns the events
/// found up to the edge instead of erroring (de442s ends early 2150).
#[test]
fn range_past_coverage_edge_returns_partial_results() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Sun);
    let config = config_for(body);
    let start = approx_jd(&UtcTime::new(2149, 6, 1, 0, 0, 0.0));
    let end = approx_jd(&UtcTime::new(2152, 1, 1, 0, 0, 0.0));

    let events = search_fixed_longitudes(&engine, body, start, end, 100.0, &[], &config)
        .expect("coverage edge must not fail the range");
    assert_eq!(
        events.len(),
        1,
        "only the in-coverage lap should be found, got {}",
        events.len()
    );
}

#[test]
fn invalid_inputs_are_rejected() {
    let Some(engine) = load_engine() else { return };
    let config = SankrantiConfig::default_lahiri();
    let at_jd = approx_jd(&UtcTime::new(2024, 1, 1, 0, 0, 0.0));

    let earth = next_fixed_longitude(
        &engine,
        TransitBody::Body(Body::Earth),
        at_jd,
        0.0,
        &[],
        &config,
    );
    assert!(matches!(earth, Err(SearchError::InvalidConfig(_))));

    let nan_target = next_fixed_longitude(
        &engine,
        TransitBody::Body(Body::Sun),
        at_jd,
        f64::NAN,
        &[],
        &config,
    );
    assert!(matches!(nan_target, Err(SearchError::InvalidConfig(_))));

    let nan_angle = next_fixed_longitude(
        &engine,
        TransitBody::Body(Body::Sun),
        at_jd,
        0.0,
        &[f64::INFINITY],
        &config,
    );
    assert!(matches!(nan_angle, Err(SearchError::InvalidConfig(_))));

    let bad_range = search_fixed_longitudes(
        &engine,
        TransitBody::Body(Body::Sun),
        at_jd,
        at_jd,
        0.0,
        &[],
        &config,
    );
    assert!(matches!(bad_range, Err(SearchError::InvalidConfig(_))));
}
