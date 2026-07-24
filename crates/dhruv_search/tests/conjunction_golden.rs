//! Golden-value integration tests for conjunction/opposition search.
//!
//! Validates against JPL Horizons new/full moon dates and planetary conjunctions.
//! Requires kernel files (de442s.bsp, naif0012.tls). Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Body, Engine, EngineConfig};
use dhruv_search::{ConjunctionConfig, next_conjunction, prev_conjunction, search_conjunctions};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping conjunction_golden: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn jd_from_date(year: i32, month: u32, day: f64) -> f64 {
    dhruv_time::calendar_to_jd(year, month, day)
}

/// New moon: Sun-Moon conjunction (0 deg).
/// 2024-Jan-11 ~11:57 UTC → JD TDB ~2460320.0
/// Horizons: 2024-Jan-11 11:57 UTC
#[test]
fn new_moon_jan_2024() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 1.0);
    let config = ConjunctionConfig::conjunction(0.5);
    let result = next_conjunction(
        &engine,
        Body::Sun.into(),
        Body::Moon.into(),
        jd_start,
        &config,
    )
    .expect("search should succeed");
    let event = result.expect("should find a new moon");

    // New moon ~2024-Jan-11 11:57 UTC ≈ JD 2460320.998
    let expected_jd = jd_from_date(2024, 1, 11.498); // ~11:57 UTC
    let diff_hours = (event.jd_tdb - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 2.0,
        "new moon off by {diff_hours:.1}h, got JD {}, expected ~JD {}",
        event.jd_tdb,
        expected_jd
    );
    // Separation should be near 0
    assert!(
        event.actual_separation_deg < 1.0,
        "separation = {} deg",
        event.actual_separation_deg
    );
}

/// Full moon: Sun-Moon opposition (180 deg).
/// 2024-Jan-25 ~17:54 UTC
#[test]
fn full_moon_jan_2024() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 20.0);
    let config = ConjunctionConfig::opposition(0.5);
    let result = next_conjunction(
        &engine,
        Body::Sun.into(),
        Body::Moon.into(),
        jd_start,
        &config,
    )
    .expect("search should succeed");
    let event = result.expect("should find a full moon");

    let expected_jd = jd_from_date(2024, 1, 25.746); // ~17:54 UTC
    let diff_hours = (event.jd_tdb - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 2.0,
        "full moon off by {diff_hours:.1}h, got JD {}, expected ~JD {}",
        event.jd_tdb,
        expected_jd
    );
    assert!(
        (event.actual_separation_deg - 180.0).abs() < 1.0,
        "separation = {} deg",
        event.actual_separation_deg
    );
}

/// Jupiter-Saturn conjunction: 2020-Dec-21 ~18:22 UTC.
/// The "Great Conjunction" — closest in centuries.
#[test]
fn jupiter_saturn_conjunction_2020() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2020, 11, 1.0);
    let config = ConjunctionConfig::conjunction(2.0);
    let result = next_conjunction(
        &engine,
        Body::Jupiter.into(),
        Body::Saturn.into(),
        jd_start,
        &config,
    )
    .expect("search should succeed");
    let event = result.expect("should find Jupiter-Saturn conjunction");

    let expected_jd = jd_from_date(2020, 12, 21.765); // ~18:22 UTC
    let diff_days = (event.jd_tdb - expected_jd).abs();
    assert!(
        diff_days < 1.0,
        "great conjunction off by {diff_days:.2} days, got JD {}, expected ~JD {}",
        event.jd_tdb,
        expected_jd
    );
    // Separation should be very small (<1 deg)
    assert!(
        event.actual_separation_deg < 1.0,
        "separation = {} deg",
        event.actual_separation_deg
    );
}

/// Sun-Moon aspect: first quarter moon.
/// With body1=Sun, body2=Moon, first quarter (Moon 90° ahead) means
/// lon_Sun - lon_Moon = -90° = 270° in [0, 360).
#[test]
fn first_quarter_moon_jan_2024() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 15.0);
    let config = ConjunctionConfig::aspect(270.0, 0.5);
    let result = next_conjunction(
        &engine,
        Body::Sun.into(),
        Body::Moon.into(),
        jd_start,
        &config,
    )
    .expect("search should succeed");
    let event = result.expect("should find first quarter");

    // First quarter ~2024-Jan-18
    let expected_jd = jd_from_date(2024, 1, 18.0);
    let diff_days = (event.jd_tdb - expected_jd).abs();
    assert!(diff_days < 2.0, "first quarter off by {diff_days:.1} days");
    assert!(
        (event.actual_separation_deg - 270.0).abs() < 2.0,
        "separation = {} deg, expected ~270",
        event.actual_separation_deg
    );
}

/// Search for multiple new moons in a 3-month window.
#[test]
fn multiple_new_moons_q1_2024() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 1.0);
    let jd_end = jd_from_date(2024, 4, 1.0);
    let config = ConjunctionConfig::conjunction(0.5);
    let events = search_conjunctions(
        &engine,
        Body::Sun.into(),
        Body::Moon.into(),
        jd_start,
        jd_end,
        &config,
    )
    .expect("search should succeed");

    // ~3 new moons in 3 months
    assert!(
        events.len() >= 2 && events.len() <= 4,
        "found {} new moons, expected 2-4",
        events.len()
    );

    // Check they're ~29.5 days apart
    for window in events.windows(2) {
        let gap = window[1].jd_tdb - window[0].jd_tdb;
        assert!(
            (gap - 29.5).abs() < 2.0,
            "gap between new moons = {gap:.1} days, expected ~29.5"
        );
    }
}

/// Backward search for previous new moon.
#[test]
fn prev_new_moon() {
    let Some(engine) = load_engine() else { return };
    let jd = jd_from_date(2024, 2, 1.0);
    let config = ConjunctionConfig::conjunction(0.5);
    let result = prev_conjunction(&engine, Body::Sun.into(), Body::Moon.into(), jd, &config)
        .expect("search should succeed");
    let event = result.expect("should find previous new moon");

    // Previous new moon ~2024-Jan-11
    assert!(
        event.jd_tdb < jd,
        "previous event should be before search date"
    );
    let expected_jd = jd_from_date(2024, 1, 11.5);
    let diff_days = (event.jd_tdb - expected_jd).abs();
    assert!(diff_days < 2.0, "prev new moon off by {diff_days:.1} days");
}

/// Sun-Rahu (true node) conjunction: solar eclipses occur when the Sun is
/// near a lunar node. The total solar eclipse of 2024-Apr-08 was at the
/// ascending node, so the Sun-Rahu conjunction must land within a few days
/// of it.
#[test]
fn sun_rahu_conjunction_near_april_2024_eclipse() {
    let Some(engine) = load_engine() else { return };
    use dhruv_search::TransitBody;
    let jd_start = jd_from_date(2024, 3, 1.0);
    let config = ConjunctionConfig::conjunction(1.0);
    let event = next_conjunction(
        &engine,
        Body::Sun.into(),
        TransitBody::Rahu,
        jd_start,
        &config,
    )
    .expect("search should succeed")
    .expect("should find Sun-Rahu conjunction");

    let eclipse_jd = jd_from_date(2024, 4, 8.75);
    let diff_days = (event.jd_tdb - eclipse_jd).abs();
    assert!(
        diff_days < 5.0,
        "Sun-Rahu conjunction off by {diff_days:.1} d from the 2024-04-08 eclipse"
    );
    assert_eq!(event.body2, TransitBody::Rahu);
    assert!(event.actual_separation_deg.abs() < 1.0);
    assert!(
        event.body2_latitude_deg.abs() < 1e-9,
        "node latitude must be 0"
    );
}

/// Sun-Ketu conjunction near the 2024-Oct-02 annular eclipse (descending node).
#[test]
fn sun_ketu_conjunction_near_october_2024_eclipse() {
    let Some(engine) = load_engine() else { return };
    use dhruv_search::TransitBody;
    let jd_start = jd_from_date(2024, 9, 1.0);
    let config = ConjunctionConfig::conjunction(1.0);
    let event = next_conjunction(
        &engine,
        Body::Sun.into(),
        TransitBody::Ketu,
        jd_start,
        &config,
    )
    .expect("search should succeed")
    .expect("should find Sun-Ketu conjunction");

    let eclipse_jd = jd_from_date(2024, 10, 2.79);
    let diff_days = (event.jd_tdb - eclipse_jd).abs();
    assert!(
        diff_days < 5.0,
        "Sun-Ketu conjunction off by {diff_days:.1} d from the 2024-10-02 eclipse"
    );
}

/// Jupiter-Saturn: the pair-aware scan cap must find the next great
/// conjunction from 2015 even though it is ~5.5 years out (beyond the old
/// 800-day scan window).
#[test]
fn jupiter_saturn_next_found_beyond_800_days() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2015, 1, 1.0);
    let config = ConjunctionConfig::conjunction(2.0);
    let event = next_conjunction(
        &engine,
        Body::Jupiter.into(),
        Body::Saturn.into(),
        jd_start,
        &config,
    )
    .expect("search should succeed")
    .expect("should find the 2020 great conjunction");
    let expected_jd = jd_from_date(2020, 12, 21.5);
    assert!(
        (event.jd_tdb - expected_jd).abs() < 2.0,
        "got JD {}, expected ~{}",
        event.jd_tdb,
        expected_jd
    );
}

/// Multi-angle sweep must equal the union of the single-angle searches, with
/// every event tagged by its matched angle.
#[test]
fn multi_angle_matches_single_angle_union() {
    let Some(engine) = load_engine() else { return };
    use dhruv_search::operations::{ConjunctionOperation, ConjunctionQuery, ConjunctionResult};

    let start = jd_from_date(2024, 1, 1.0);
    let end = jd_from_date(2024, 2, 1.0);
    let base = ConjunctionConfig::conjunction(0.5);

    let mut single_union = Vec::new();
    for angle in [0.0, 90.0, 180.0, 270.0] {
        let config = ConjunctionConfig::aspect(angle, 0.5);
        single_union.extend(
            search_conjunctions(
                &engine,
                Body::Sun.into(),
                Body::Moon.into(),
                start,
                end,
                &config,
            )
            .unwrap(),
        );
    }
    single_union.sort_by(|a, b| a.jd_tdb.total_cmp(&b.jd_tdb));

    let op = ConjunctionOperation {
        body1: Body::Sun.into(),
        body2: Body::Moon.into(),
        config: base,
        target_separations_deg: vec![0.0, 90.0, 180.0, 270.0],
        sankranti_config: None,
        query: ConjunctionQuery::Range {
            start_jd_tdb: start,
            end_jd_tdb: end,
        },
    };
    let ConjunctionResult::Many(multi) = dhruv_search::conjunction(&engine, &op).unwrap() else {
        panic!("expected Many");
    };

    assert_eq!(multi.len(), single_union.len());
    // ~4 lunar quarters in one month
    assert!(
        (4..=5).contains(&multi.len()),
        "expected 4-5 quarter events, got {}",
        multi.len()
    );
    for (m, s) in multi.iter().zip(&single_union) {
        assert!((m.jd_tdb - s.jd_tdb).abs() < 1e-6);
        assert!([0.0, 90.0, 180.0, 270.0].contains(&m.target_separation_deg));
    }
}

/// Sidereal echo: with a sankranti config on the operation, events must
/// carry sidereal longitudes equal to tropical minus ayanamsha.
#[test]
fn sidereal_echo_consistent_with_ayanamsha() {
    let Some(engine) = load_engine() else { return };
    use dhruv_search::SankrantiConfig;
    use dhruv_search::operations::{ConjunctionOperation, ConjunctionQuery, ConjunctionResult};
    use dhruv_vedic_base::jd_tdb_to_centuries;

    let sc = SankrantiConfig::default_lahiri();
    let op = ConjunctionOperation {
        body1: Body::Sun.into(),
        body2: Body::Moon.into(),
        config: ConjunctionConfig::conjunction(0.5),
        target_separations_deg: Vec::new(),
        sankranti_config: Some(sc),
        query: ConjunctionQuery::Next {
            at_jd_tdb: jd_from_date(2024, 1, 1.0),
        },
    };
    let ConjunctionResult::Single(Some(event)) = dhruv_search::conjunction(&engine, &op).unwrap()
    else {
        panic!("expected event");
    };

    let sid1 = event.body1_sidereal_longitude_deg.expect("echo present");
    let aya = sc.ayanamsha_deg_at_centuries(jd_tdb_to_centuries(event.jd_tdb));
    let expected = (event.body1_longitude_deg - aya).rem_euclid(360.0);
    // The event longitudes are sampled at the final bisection midpoint while
    // the echo is recomputed at the converged time, so allow the sub-µdeg
    // difference that time delta implies.
    assert!(
        (sid1 - expected).abs() < 1e-6,
        "sidereal echo {sid1} != tropical-ayanamsha {expected}"
    );
    let rashi = event.body1_rashi_index.expect("rashi echo present");
    assert_eq!(rashi, (sid1 / 30.0) as u8 % 12);
}
