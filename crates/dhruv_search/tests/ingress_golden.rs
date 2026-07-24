//! Golden-value integration tests for general rashi-ingress search.
//!
//! Validates non-Sun bodies (Moon, Jupiter, Saturn, Mercury, Rahu/Ketu)
//! entering rashis against published almanac dates and internal-consistency
//! invariants. Requires kernel files. Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Body, Engine, EngineConfig};
use dhruv_search::operations::{
    SankrantiOperation, SankrantiQuery, SankrantiResult, SankrantiTarget,
};
use dhruv_search::sankranti_types::SankrantiConfig;
use dhruv_search::{
    TransitBody, next_ingress, next_sankranti, next_specific_ingress, search_ingresses,
};
use dhruv_time::UtcTime;
use dhruv_vedic_base::{AyanamshaSystem, NodeMode, Rashi};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping ingress_golden: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn config_for(body: TransitBody) -> SankrantiConfig {
    SankrantiConfig::for_body(AyanamshaSystem::Lahiri, false, body)
}

/// The Sun path through the general ingress engine must reproduce the
/// classical sankranti search exactly.
#[test]
fn sun_ingress_matches_sankranti() {
    let Some(engine) = load_engine() else { return };
    let config = SankrantiConfig::default_lahiri();
    let utc = UtcTime::new(2024, 1, 1, 0, 0, 0.0);

    let via_sankranti = next_sankranti(&engine, &utc, &config)
        .unwrap()
        .expect("sankranti");
    let via_ingress = next_ingress(&engine, TransitBody::Body(Body::Sun), &utc, &config)
        .unwrap()
        .expect("ingress");

    assert_eq!(via_sankranti, via_ingress);
    assert_eq!(via_ingress.body, TransitBody::Body(Body::Sun));
    assert!(!via_ingress.is_retrograde);
}

/// The Moon crosses a rashi boundary roughly every 2.3 days: January 2024
/// must contain 13-14 ingresses, strictly ascending rashi order, all direct,
/// each exactly on a 30-degree cusp.
#[test]
fn moon_ingresses_january_2024() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Moon);
    let config = config_for(body);
    let start = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let end = UtcTime::new(2024, 2, 1, 0, 0, 0.0);

    let events = search_ingresses(&engine, body, &start, &end, &config).unwrap();
    assert!(
        (13..=14).contains(&events.len()),
        "expected 13-14 moon ingresses, got {}",
        events.len()
    );
    for pair in events.windows(2) {
        let expected_next = (pair[0].rashi_index + 1) % 12;
        assert_eq!(
            pair[1].rashi_index, expected_next,
            "moon rashi sequence must ascend consecutively"
        );
    }
    for ev in &events {
        assert_eq!(ev.body, body);
        assert!(!ev.is_retrograde, "the Moon never ingresses retrograde");
        let boundary = f64::from(ev.rashi_index) * 30.0;
        assert!(
            (ev.sidereal_longitude_deg - boundary).abs() < 1e-3,
            "cusp longitude {:.6} not at boundary {boundary}",
            ev.sidereal_longitude_deg
        );
    }
}

/// Published gochar date: Guru (Jupiter) entered sidereal Vrishabha on
/// 2024-05-01 (Lahiri).
#[test]
fn jupiter_enters_vrishabha_2024() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Jupiter);
    let config = config_for(body);
    let utc = UtcTime::new(2024, 1, 1, 0, 0, 0.0);

    let event = next_specific_ingress(&engine, body, &utc, Rashi::Vrishabha, &config)
        .unwrap()
        .expect("should find Jupiter's Vrishabha ingress");
    assert_eq!(event.utc.year, 2024);
    let ok = (event.utc.month == 4 && event.utc.day >= 29)
        || (event.utc.month == 5 && event.utc.day <= 3);
    assert!(
        ok,
        "expected ~2024-05-01, got {:04}-{:02}-{:02}",
        event.utc.year, event.utc.month, event.utc.day
    );
    assert!(!event.is_retrograde);
}

/// Published gochar date: Shani (Saturn) entered sidereal Meena on
/// 2025-03-29 (Lahiri).
#[test]
fn saturn_enters_meena_2025() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Saturn);
    let config = config_for(body);
    let utc = UtcTime::new(2025, 1, 1, 0, 0, 0.0);

    let event = next_specific_ingress(&engine, body, &utc, Rashi::Meena, &config)
        .unwrap()
        .expect("should find Saturn's Meena ingress");
    assert_eq!(event.utc.year, 2025);
    assert_eq!(event.utc.month, 3);
    assert!(
        (27..=31).contains(&event.utc.day),
        "expected ~2025-03-29, got day {}",
        event.utc.day
    );
}

/// Mercury retrogrades three times a year, producing retrograde re-ingresses:
/// a full year must contain more than 12 ingresses and at least one
/// retrograde crossing, and every crossing sits on a cusp.
#[test]
fn mercury_retrograde_reingresses_2024() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Mercury);
    let config = config_for(body);
    let start = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let end = UtcTime::new(2025, 1, 1, 0, 0, 0.0);

    let events = search_ingresses(&engine, body, &start, &end, &config).unwrap();
    assert!(
        events.len() > 12,
        "expected more than 12 Mercury ingresses (retro re-entries), got {}",
        events.len()
    );
    let retro = events.iter().filter(|e| e.is_retrograde).count();
    assert!(retro >= 1, "expected at least one retrograde re-ingress");
    for ev in &events {
        let cusp = ev.sidereal_longitude_deg.rem_euclid(30.0);
        let dist = cusp.min(30.0 - cusp);
        assert!(
            dist < 1e-3,
            "not on a cusp: {:.6}",
            ev.sidereal_longitude_deg
        );
        if ev.is_retrograde {
            // A retrograde crossing of cusp B enters the rashi below B.
            let boundary_rashi =
                ((ev.sidereal_longitude_deg / 30.0).round() as u8).rem_euclid(12) % 12;
            assert_eq!((ev.rashi_index + 1) % 12, boundary_rashi % 12);
        }
    }
}

/// Mean-node Rahu ingress: the mean node regresses steadily, so every
/// ingress is retrograde and rashi order descends. Rahu's Meena entry
/// (published mean-node gochar: mid 2025) must appear.
#[test]
fn mean_rahu_enters_kumbha_2025() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Rahu;
    let mut config = config_for(body);
    config.node_mode = NodeMode::Mean;
    let start = UtcTime::new(2025, 1, 1, 0, 0, 0.0);
    let end = UtcTime::new(2026, 1, 1, 0, 0, 0.0);

    let events = search_ingresses(&engine, body, &start, &end, &config).unwrap();
    assert!(!events.is_empty(), "expected mean-node ingresses in 2025");
    for ev in &events {
        assert!(ev.is_retrograde, "mean node only moves retrograde");
        assert_eq!(ev.body, TransitBody::Rahu);
    }
    let kumbha: Vec<_> = events.iter().filter(|e| e.rashi == Rashi::Kumbha).collect();
    assert_eq!(
        kumbha.len(),
        1,
        "exactly one mean-node Kumbha entry in 2025"
    );
    let ev = kumbha[0];
    assert!(
        (4..=7).contains(&ev.utc.month),
        "expected Apr-Jul 2025, got month {}",
        ev.utc.month
    );
    // Retrograde entry into Kumbha happens across the Meena cusp (330 deg).
    let cusp = ev.sidereal_longitude_deg.rem_euclid(30.0);
    assert!(cusp.min(30.0 - cusp) < 1e-3);
}

/// True-node (osculating) Rahu ingresses: the true node oscillates around
/// its mean regression, so cusps can be crossed several times. Every
/// crossing must sit on a cusp, and Ketu's events must mirror Rahu's
/// (180 deg apart means the same cusp times shifted by 6 rashis).
#[test]
fn true_rahu_ingresses_2025_kumbha() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Rahu;
    let config = config_for(body); // node_mode defaults to True
    let start = UtcTime::new(2025, 1, 1, 0, 0, 0.0);
    let end = UtcTime::new(2026, 1, 1, 0, 0, 0.0);

    let events = search_ingresses(&engine, body, &start, &end, &config).unwrap();
    assert!(
        !events.is_empty(),
        "expected true-node ingresses during 2025"
    );
    let kumbha_entries = events.iter().filter(|e| e.rashi == Rashi::Kumbha).count();
    assert!(
        kumbha_entries >= 1,
        "true Rahu must enter Kumbha during 2025 (published gochar ~May 2025)"
    );
    for ev in &events {
        let cusp = ev.sidereal_longitude_deg.rem_euclid(30.0);
        assert!(cusp.min(30.0 - cusp) < 1e-3);
    }

    // Ketu events at the same times, 6 rashis apart.
    let ketu_events = search_ingresses(&engine, TransitBody::Ketu, &start, &end, &config).unwrap();
    assert_eq!(events.len(), ketu_events.len());
    for (r, k) in events.iter().zip(&ketu_events) {
        assert_eq!((r.rashi_index + 6) % 12, k.rashi_index);
        assert_eq!(r.is_retrograde, k.is_retrograde);
    }
}

/// The operation-layer entry point must honor the body field.
#[test]
fn sankranti_operation_with_moon_body() {
    let Some(engine) = load_engine() else { return };
    let body = TransitBody::Body(Body::Moon);
    let start = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let end = UtcTime::new(2024, 2, 1, 0, 0, 0.0);
    let start_jd = start.to_jd_tdb(engine.lsk());
    let end_jd = end.to_jd_tdb(engine.lsk());

    let op = SankrantiOperation {
        body,
        target: SankrantiTarget::Any,
        config: config_for(body),
        query: SankrantiQuery::Range {
            start_jd_tdb: start_jd,
            end_jd_tdb: end_jd,
        },
    };
    match dhruv_search::sankranti(&engine, &op).unwrap() {
        SankrantiResult::Many(events) => {
            assert!((13..=14).contains(&events.len()));
            assert!(events.iter().all(|e| e.body == body));
        }
        other => panic!("expected Many, got {other:?}"),
    }
}
