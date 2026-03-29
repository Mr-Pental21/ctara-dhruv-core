//! Regression tests for time-based upagraha day anchoring.
//!
//! Verifies that post-noon UTC inputs still use the correct civil day's
//! sunset when deriving time-based upagrahas.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::sankranti_types::SankrantiConfig;
use dhruv_search::{all_upagrahas_for_date, vedic_day_sunrises};
use dhruv_time::{EopKernel, UtcTime, calendar_to_jd, jd_to_tdb_seconds, tdb_seconds_to_jd};
use dhruv_vedic_base::{
    GeoLocation, GulikaMaandiPlanet, RiseSetConfig, RiseSetEvent, RiseSetResult,
    TimeUpagrahaConfig, TimeUpagrahaPoint, Upagraha, approximate_local_noon_jd, compute_rise_set,
    jd_tdb_to_centuries, lagna_longitude_rad, time_upagraha_jd, time_upagraha_jd_with_config,
    utc_day_start_jd, vaar_from_jd,
};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";
const EOP_PATH: &str = "../../kernels/data/finals2000A.all";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping upagraha_regression: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

#[test]
fn time_based_upagrahas_honor_custom_period_selection() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };

    let utc = UtcTime::new(2026, 3, 17, 15, 6, 19.0);
    let location = bengaluru();
    let riseset_config = RiseSetConfig::default();
    let aya_config = SankrantiConfig::default_lahiri();
    let upagraha_config = TimeUpagrahaConfig {
        gulika_point: TimeUpagrahaPoint::Middle,
        gulika_planet: GulikaMaandiPlanet::Saturn,
        maandi_point: TimeUpagrahaPoint::Start,
        maandi_planet: GulikaMaandiPlanet::Saturn,
        other_point: TimeUpagrahaPoint::End,
    };

    let computed = dhruv_search::all_upagrahas_for_date_with_config(
        &engine,
        &eop,
        &utc,
        &location,
        &riseset_config,
        &aya_config,
        &upagraha_config,
    )
    .expect("all_upagrahas_for_date_with_config should succeed");

    let (sunrise_jd, next_sunrise_jd) =
        vedic_day_sunrises(&engine, &eop, &utc, &location, &riseset_config)
            .expect("sunrise pair should succeed");
    let noon_jd = approximate_local_noon_jd(utc_day_start_jd(jd_utc(&utc)), location.longitude_deg);
    let sunset_jd = match compute_rise_set(
        &engine,
        engine.lsk(),
        &eop,
        &location,
        RiseSetEvent::Sunset,
        noon_jd,
        &riseset_config,
    )
    .expect("sunset should succeed")
    {
        RiseSetResult::Event { jd_tdb, .. } => jd_tdb,
        other => panic!("expected sunset event, got {other:?}"),
    };

    let weekday = vaar_from_jd(sunrise_jd).index();

    let manual_lon = |upagraha: Upagraha| {
        let target_jd = time_upagraha_jd_with_config(
            upagraha,
            weekday,
            false,
            sunrise_jd,
            sunset_jd,
            next_sunrise_jd,
            &upagraha_config,
        );
        let lagna_deg = lagna_longitude_rad(
            engine.lsk(),
            &eop,
            &location,
            jd_tdb_to_utc_jd(&engine, target_jd),
        )
        .expect("lagna at configured upagraha JD")
        .to_degrees();
        let aya = aya_config.ayanamsha_deg_at_centuries(jd_tdb_to_centuries(target_jd));
        (lagna_deg - aya).rem_euclid(360.0)
    };

    for (upagraha, actual_lon) in [
        (Upagraha::Gulika, computed.gulika),
        (Upagraha::Maandi, computed.maandi),
        (Upagraha::Kaala, computed.kaala),
        (Upagraha::Mrityu, computed.mrityu),
        (Upagraha::ArthaPrahara, computed.artha_prahara),
        (Upagraha::YamaGhantaka, computed.yama_ghantaka),
    ] {
        let manual = manual_lon(upagraha);
        assert!(
            (manual - actual_lon).abs() < 2e-5,
            "{upagraha:?}: manual={manual}, actual={actual_lon}"
        );
    }
}

fn load_eop() -> Option<EopKernel> {
    if !Path::new(EOP_PATH).exists() {
        eprintln!("Skipping upagraha_regression: EOP file not found");
        return None;
    }
    EopKernel::load(Path::new(EOP_PATH)).ok()
}

fn jd_utc(utc: &UtcTime) -> f64 {
    let day_frac = utc.day as f64
        + utc.hour as f64 / 24.0
        + utc.minute as f64 / 1440.0
        + utc.second / 86_400.0;
    calendar_to_jd(utc.year, utc.month, day_frac)
}

fn jd_tdb_to_utc_jd(engine: &Engine, jd_tdb: f64) -> f64 {
    let tdb_s = jd_to_tdb_seconds(jd_tdb);
    let utc_s = engine.lsk().tdb_to_utc(tdb_s);
    tdb_seconds_to_jd(utc_s)
}

fn bengaluru() -> GeoLocation {
    GeoLocation::new(12.9716, 77.5946, 0.0)
}

#[test]
fn time_based_upagrahas_match_manual_night_portions_after_noon_utc() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };

    let utc = UtcTime::new(2026, 3, 17, 15, 6, 19.0);
    let location = bengaluru();
    let riseset_config = RiseSetConfig::default();
    let aya_config = SankrantiConfig::default_lahiri();

    let computed =
        all_upagrahas_for_date(&engine, &eop, &utc, &location, &riseset_config, &aya_config)
            .expect("all_upagrahas_for_date should succeed");

    let (sunrise_jd, next_sunrise_jd) =
        vedic_day_sunrises(&engine, &eop, &utc, &location, &riseset_config)
            .expect("sunrise pair should succeed");
    let noon_jd = approximate_local_noon_jd(utc_day_start_jd(jd_utc(&utc)), location.longitude_deg);
    let sunset_jd = match compute_rise_set(
        &engine,
        engine.lsk(),
        &eop,
        &location,
        RiseSetEvent::Sunset,
        noon_jd,
        &riseset_config,
    )
    .expect("sunset should succeed")
    {
        RiseSetResult::Event { jd_tdb, .. } => jd_tdb,
        other => panic!("expected sunset event, got {other:?}"),
    };

    let query_jd_tdb = utc.to_jd_tdb(engine.lsk());
    assert!(
        query_jd_tdb >= sunset_jd,
        "query should be after same-day sunset for this regression case"
    );

    let weekday = vaar_from_jd(sunrise_jd).index();

    let manual_lon = |upagraha: Upagraha| {
        let target_jd = time_upagraha_jd(
            upagraha,
            weekday,
            false,
            sunrise_jd,
            sunset_jd,
            next_sunrise_jd,
        );
        assert!(
            target_jd >= sunset_jd && target_jd <= next_sunrise_jd,
            "{upagraha:?} JD should lie in the night interval"
        );
        let lagna_deg = lagna_longitude_rad(
            engine.lsk(),
            &eop,
            &location,
            jd_tdb_to_utc_jd(&engine, target_jd),
        )
        .expect("lagna at upagraha JD")
        .to_degrees();
        let aya = aya_config.ayanamsha_deg_at_centuries(jd_tdb_to_centuries(target_jd));
        (lagna_deg - aya).rem_euclid(360.0)
    };

    let expected = [
        (Upagraha::Gulika, computed.gulika),
        (Upagraha::Maandi, computed.maandi),
        (Upagraha::Kaala, computed.kaala),
        (Upagraha::Mrityu, computed.mrityu),
        (Upagraha::ArthaPrahara, computed.artha_prahara),
        (Upagraha::YamaGhantaka, computed.yama_ghantaka),
    ];

    for (upagraha, actual_lon) in expected {
        let manual = manual_lon(upagraha);
        assert!(
            (manual - actual_lon).abs() < 2e-5,
            "{upagraha:?}: manual={manual}, actual={actual_lon}"
        );
    }
}
