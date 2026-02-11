//! Golden-value integration tests for eclipse computation.
//!
//! Validates against NASA Five Millennium Eclipse Catalog data.
//! Requires kernel files (de442s.bsp, naif0012.tls). Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::{
    EclipseConfig, LunarEclipseType,
    next_lunar_eclipse, prev_lunar_eclipse, search_lunar_eclipses,
    next_solar_eclipse, prev_solar_eclipse, search_solar_eclipses,
};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping eclipse_golden: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(
        SPK_PATH.into(),
        LSK_PATH.into(),
        1024,
        false,
    );
    Engine::new(config).ok()
}

fn jd_from_date(year: i32, month: u32, day: f64) -> f64 {
    dhruv_time::calendar_to_jd(year, month, day)
}

// ---------------------------------------------------------------------------
// Lunar eclipses
// ---------------------------------------------------------------------------

/// 2024-Mar-25: Penumbral lunar eclipse
/// NASA catalog: Greatest eclipse 07:13 UTC
#[test]
fn lunar_eclipse_2024_mar_penumbral() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 3, 1.0);
    let config = EclipseConfig::default();
    let result = next_lunar_eclipse(&engine, jd_start, &config)
        .expect("search should succeed");
    let eclipse = result.expect("should find a lunar eclipse");

    // Should be in March 2024
    let expected_jd = jd_from_date(2024, 3, 25.3); // ~07:13 UTC
    let diff_hours = (eclipse.greatest_eclipse_jd - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 12.0,
        "lunar eclipse off by {diff_hours:.1}h, got JD {}, expected ~JD {}",
        eclipse.greatest_eclipse_jd, expected_jd
    );
    assert_eq!(eclipse.eclipse_type, LunarEclipseType::Penumbral);
}

/// 2025-Mar-14: Total lunar eclipse
/// NASA catalog: Greatest eclipse ~06:59 UTC, magnitude 1.178
#[test]
fn lunar_eclipse_2025_mar_total() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2025, 3, 1.0);
    let config = EclipseConfig::default();
    let result = next_lunar_eclipse(&engine, jd_start, &config)
        .expect("search should succeed");
    let eclipse = result.expect("should find a lunar eclipse");

    let expected_jd = jd_from_date(2025, 3, 14.29); // ~06:59 UTC
    let diff_hours = (eclipse.greatest_eclipse_jd - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 12.0,
        "lunar eclipse off by {diff_hours:.1}h, got JD {}",
        eclipse.greatest_eclipse_jd
    );
    assert_eq!(eclipse.eclipse_type, LunarEclipseType::Total);
    // Magnitude should be > 1 for total
    assert!(
        eclipse.magnitude > 1.0,
        "total lunar magnitude = {}, expected > 1",
        eclipse.magnitude
    );
}

/// Search for lunar eclipses in 2024 — should find 2 (Mar and Sep).
#[test]
fn lunar_eclipses_2024_count() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 1.0);
    let jd_end = jd_from_date(2025, 1, 1.0);
    let config = EclipseConfig::default();
    let eclipses = search_lunar_eclipses(&engine, jd_start, jd_end, &config)
        .expect("search should succeed");

    // 2024 has 2 lunar eclipses: Mar 25 (penumbral) and Sep 18 (partial)
    assert!(
        eclipses.len() >= 2,
        "found {} lunar eclipses in 2024, expected at least 2",
        eclipses.len()
    );
}

/// Penumbral-only filter: exclude penumbral eclipses.
#[test]
fn penumbral_filter() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 1.0);
    let jd_end = jd_from_date(2025, 1, 1.0);
    let config = EclipseConfig {
        include_penumbral: false,
        ..Default::default()
    };
    let eclipses = search_lunar_eclipses(&engine, jd_start, jd_end, &config)
        .expect("search should succeed");

    // With penumbral excluded, should have fewer eclipses
    for e in &eclipses {
        assert_ne!(
            e.eclipse_type,
            LunarEclipseType::Penumbral,
            "penumbral eclipse should be filtered"
        );
    }
}

/// Backward search for previous lunar eclipse.
#[test]
fn prev_lunar_eclipse_from_2024() {
    let Some(engine) = load_engine() else { return };
    let jd = jd_from_date(2024, 3, 1.0);
    let config = EclipseConfig::default();
    let result = prev_lunar_eclipse(&engine, jd, &config)
        .expect("search should succeed");
    let eclipse = result.expect("should find previous lunar eclipse");

    // Previous lunar eclipse should be before our search date
    assert!(eclipse.greatest_eclipse_jd < jd);
    // Contact times should be ordered: P1 < greatest < P4
    assert!(eclipse.p1_jd < eclipse.greatest_eclipse_jd);
    assert!(eclipse.greatest_eclipse_jd < eclipse.p4_jd);
}

// ---------------------------------------------------------------------------
// Solar eclipses
// ---------------------------------------------------------------------------

/// 2024-Apr-08: Solar eclipse (Total for surface observers).
/// NASA catalog: Greatest eclipse 18:18 UTC.
/// Geocentric classification may differ from surface classification.
#[test]
fn solar_eclipse_2024_apr() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 3, 1.0);
    let config = EclipseConfig::default();
    let result = next_solar_eclipse(&engine, jd_start, &config)
        .expect("search should succeed");
    let eclipse = result.expect("should find a solar eclipse");

    let expected_jd = jd_from_date(2024, 4, 8.763); // ~18:18 UTC
    let diff_hours = (eclipse.greatest_eclipse_jd - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 12.0,
        "solar eclipse off by {diff_hours:.1}h, got JD {}, expected ~JD {}",
        eclipse.greatest_eclipse_jd, expected_jd
    );
    // Geocentric: could be Partial or Total depending on exact geometry
    // Magnitude should be close to 1.0 (Moon is close to Sun's size)
    assert!(
        eclipse.magnitude > 0.90,
        "solar eclipse magnitude = {}, expected > 0.90",
        eclipse.magnitude
    );
}

/// 2024-Oct-02: Solar eclipse (Annular for surface observers).
/// NASA catalog: Greatest eclipse ~18:45 UTC.
/// Geocentric classification may differ from surface classification.
#[test]
fn solar_eclipse_2024_oct() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 9, 1.0);
    let config = EclipseConfig::default();
    let result = next_solar_eclipse(&engine, jd_start, &config)
        .expect("search should succeed");
    let eclipse = result.expect("should find a solar eclipse");

    let expected_jd = jd_from_date(2024, 10, 2.78); // ~18:45 UTC
    let diff_hours = (eclipse.greatest_eclipse_jd - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 12.0,
        "solar eclipse off by {diff_hours:.1}h, got JD {}",
        eclipse.greatest_eclipse_jd
    );
    // Geocentric: could be Partial or Annular
    assert!(
        eclipse.magnitude > 0.90,
        "solar eclipse magnitude = {}, expected > 0.90",
        eclipse.magnitude
    );
}

/// Search for solar eclipses in 2024 — should find 2 (Apr total, Oct annular).
#[test]
fn solar_eclipses_2024_count() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 1.0);
    let jd_end = jd_from_date(2025, 1, 1.0);
    let config = EclipseConfig::default();
    let eclipses = search_solar_eclipses(&engine, jd_start, jd_end, &config)
        .expect("search should succeed");

    assert!(
        eclipses.len() >= 2,
        "found {} solar eclipses in 2024, expected at least 2",
        eclipses.len()
    );
}

/// Backward search for previous solar eclipse.
#[test]
fn prev_solar_eclipse_from_2024() {
    let Some(engine) = load_engine() else { return };
    let jd = jd_from_date(2024, 3, 1.0);
    let config = EclipseConfig::default();
    let result = prev_solar_eclipse(&engine, jd, &config)
        .expect("search should succeed");
    let eclipse = result.expect("should find previous solar eclipse");

    assert!(eclipse.greatest_eclipse_jd < jd);
    // Contact times C1 < greatest < C4 (if present)
    if let Some(c1) = eclipse.c1_jd {
        assert!(c1 < eclipse.greatest_eclipse_jd);
    }
    if let Some(c4) = eclipse.c4_jd {
        assert!(eclipse.greatest_eclipse_jd < c4);
    }
}
