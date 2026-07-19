//! Integration tests for the range operations: amsha_series,
//! panchang_events, and amsha_lagna_events.
//!
//! Requires kernel files. Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::sankranti_types::SankrantiConfig;
use dhruv_search::{
    AmshaChartScope, PANCHANG_INCLUDE_ALL_CALENDAR, PANCHANG_INCLUDE_GHATIKA,
    PANCHANG_INCLUDE_HORA, PANCHANG_INCLUDE_TITHI, PANCHANG_INCLUDE_VAAR, SearchError,
    amsha_charts_for_date, amsha_lagna_events, amsha_series, hora_for_date, masa_for_date,
    panchang_events, sidereal_lagna_for_date, tithi_for_date, vaar_for_date,
};
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::amsha::amsha_rashi_info;
use dhruv_vedic_base::riseset_types::{GeoLocation, RiseSetConfig};
use dhruv_vedic_base::{Amsha, AmshaRequest, BhavaConfig};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";
const EOP_PATH: &str = "../../kernels/data/finals2000A.all";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping range_events: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn load_eop() -> Option<EopKernel> {
    if !Path::new(EOP_PATH).exists() {
        return None;
    }
    EopKernel::load(Path::new(EOP_PATH)).ok()
}

fn new_delhi() -> GeoLocation {
    GeoLocation::new(28.6139, 77.2090, 0.0)
}

fn aya() -> SankrantiConfig {
    SankrantiConfig::default_lahiri()
}

fn jd(engine: &Engine, utc: &UtcTime) -> f64 {
    utc.to_jd_tdb(engine.lsk())
}

// ---------------------------------------------------------------------------
// amsha_series
// ---------------------------------------------------------------------------

#[test]
fn amsha_series_matches_single_epoch() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 15, 6, 0, 0.0);
    let to = UtcTime::new(2024, 1, 15, 7, 0, 0.0);
    let loc = new_delhi();
    let requests = [
        AmshaRequest::new(Amsha::D1),
        AmshaRequest::new(Amsha::D9),
        AmshaRequest::new(Amsha::D60),
    ];

    let series = amsha_series(&engine, &eop, &from, &to, 30, &loc, &aya(), &requests, true)
        .expect("series should compute");
    assert_eq!(series.points.len(), 3, "1h at 30min step inclusive");
    assert_eq!(series.points[0].charts.len(), 3);

    // First point must match the single-epoch amsha charts op.
    let charts = amsha_charts_for_date(
        &engine,
        &eop,
        &from,
        &loc,
        &BhavaConfig::default(),
        &RiseSetConfig::default(),
        &aya(),
        &requests,
        &AmshaChartScope::default(),
    )
    .expect("single-epoch charts should compute");

    for (series_chart, chart) in series.points[0].charts.iter().zip(charts.charts.iter()) {
        assert_eq!(series_chart.amsha, chart.amsha);
        assert_eq!(series_chart.lagna.rashi_index, chart.lagna.rashi_index);
        assert!(
            (series_chart.lagna.sidereal_longitude - chart.lagna.sidereal_longitude).abs() < 1e-9,
            "lagna longitude mismatch for {:?}",
            chart.amsha
        );
        let grahas = series_chart.grahas.expect("grahas requested");
        for g in 0..9 {
            assert_eq!(grahas[g].rashi_index, chart.grahas[g].rashi_index);
            assert!(
                (grahas[g].sidereal_longitude - chart.grahas[g].sidereal_longitude).abs() < 1e-9
            );
        }
    }
}

#[test]
fn amsha_series_validation() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 15, 6, 0, 0.0);
    let to = UtcTime::new(2024, 1, 16, 6, 0, 0.0);
    let loc = new_delhi();
    let requests = [AmshaRequest::new(Amsha::D9)];

    let err =
        amsha_series(&engine, &eop, &from, &to, 0, &loc, &aya(), &requests, false).unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));

    let err = amsha_series(&engine, &eop, &from, &to, 60, &loc, &aya(), &[], false).unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));

    let err = amsha_series(
        &engine,
        &eop,
        &to,
        &from,
        60,
        &loc,
        &aya(),
        &requests,
        false,
    )
    .unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));

    // 1-minute cadence over ~70 days = >100k cells for one request.
    let far = UtcTime::new(2024, 3, 26, 6, 0, 0.0);
    let err = amsha_series(
        &engine,
        &eop,
        &from,
        &far,
        1,
        &loc,
        &aya(),
        &requests,
        false,
    )
    .unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));
}

// ---------------------------------------------------------------------------
// panchang_events
// ---------------------------------------------------------------------------

#[test]
fn panchang_events_tithi_chain_matches_per_moment() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 2, 5, 0, 0, 0.0);

    let result = panchang_events(
        &engine,
        &eop,
        &from,
        &to,
        PANCHANG_INCLUDE_TITHI,
        None,
        &RiseSetConfig::default(),
        &aya(),
        0,
    )
    .expect("events should compute");
    assert!(!result.truncated);
    // 35 days at ~0.98 d/tithi: expect around 35-37 segments.
    assert!(
        result.tithi.len() >= 33 && result.tithi.len() <= 40,
        "unexpected tithi count {}",
        result.tithi.len()
    );

    // Segments chain exactly and cover the range.
    let from_jd = jd(&engine, &from);
    let to_jd = jd(&engine, &to);
    assert!(jd(&engine, &result.tithi[0].start) <= from_jd);
    assert!(jd(&engine, result.tithi.last().map(|t| &t.end).unwrap()) >= to_jd);
    for pair in result.tithi.windows(2) {
        assert_eq!(pair[0].end, pair[1].start, "tithi segments must chain");
        assert_eq!(
            (pair[0].tithi_index + 1) % 30,
            pair[1].tithi_index,
            "tithi indices must be consecutive"
        );
    }

    // Boundary times agree with the per-moment API at segment midpoints.
    for info in result.tithi.iter().take(5) {
        let mid_jd = 0.5 * (jd(&engine, &info.start) + jd(&engine, &info.end));
        let mid = UtcTime::from_jd_tdb(mid_jd, engine.lsk());
        let direct = tithi_for_date(&engine, &mid).expect("per-moment tithi");
        assert_eq!(direct.tithi, info.tithi);
        assert_eq!(direct.tithi_index, info.tithi_index);
        assert!(
            (jd(&engine, &direct.start) - jd(&engine, &info.start)).abs() < 2e-6,
            "start mismatch"
        );
        assert!(
            (jd(&engine, &direct.end) - jd(&engine, &info.end)).abs() < 2e-6,
            "end mismatch"
        );
    }
}

#[test]
fn panchang_events_calendar_kinds() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2023, 6, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 7, 1, 0, 0, 0.0);

    let result = panchang_events(
        &engine,
        &eop,
        &from,
        &to,
        PANCHANG_INCLUDE_ALL_CALENDAR,
        None,
        &RiseSetConfig::default(),
        &aya(),
        0,
    )
    .expect("calendar events should compute");
    assert!(!result.truncated);

    // ~13 months, 2-3 ayana transitions, 1-2 varshas.
    assert!(
        result.masa.len() >= 13 && result.masa.len() <= 15,
        "masa count {}",
        result.masa.len()
    );
    assert!(result.ayana.len() >= 2 && result.ayana.len() <= 4);
    assert!(!result.varsha.is_empty() && result.varsha.len() <= 3);

    for pair in result.masa.windows(2) {
        assert_eq!(pair[0].end, pair[1].start, "masa segments must chain");
    }

    let direct = masa_for_date(&engine, &from, &aya()).expect("per-moment masa");
    assert_eq!(result.masa[0].masa, direct.masa);
    assert_eq!(result.masa[0].adhika, direct.adhika);
}

#[test]
fn panchang_events_validation_and_truncation() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 1, 31, 0, 0, 0.0);

    let rs = RiseSetConfig::default();
    // A location-dependent element without a location is invalid.
    let err = panchang_events(
        &engine,
        &eop,
        &from,
        &to,
        PANCHANG_INCLUDE_VAAR,
        None,
        &rs,
        &aya(),
        0,
    )
    .unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));
    let err = panchang_events(&engine, &eop, &from, &to, 0, None, &rs, &aya(), 0).unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));
    let err = panchang_events(
        &engine,
        &eop,
        &to,
        &from,
        PANCHANG_INCLUDE_TITHI,
        None,
        &rs,
        &aya(),
        0,
    )
    .unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));

    // Truncation with resume covers the same events as one big call.
    let full = panchang_events(
        &engine,
        &eop,
        &from,
        &to,
        PANCHANG_INCLUDE_TITHI,
        None,
        &rs,
        &aya(),
        0,
    )
    .expect("full sweep");
    let first = panchang_events(
        &engine,
        &eop,
        &from,
        &to,
        PANCHANG_INCLUDE_TITHI,
        None,
        &rs,
        &aya(),
        5,
    )
    .expect("capped sweep");
    assert!(first.truncated);
    assert_eq!(first.tithi.len(), 5);
    let resume_at = first.next_from_utc.expect("resume point");
    let rest = panchang_events(
        &engine,
        &eop,
        &resume_at,
        &to,
        PANCHANG_INCLUDE_TITHI,
        None,
        &RiseSetConfig::default(),
        &aya(),
        0,
    )
    .expect("resumed sweep");

    // Stitch with dedup on start time.
    let mut all = first.tithi.clone();
    for info in &rest.tithi {
        if !all
            .iter()
            .any(|seen| jd(&engine, &seen.start) == jd(&engine, &info.start))
        {
            all.push(*info);
        }
    }
    assert_eq!(
        all.len(),
        full.tithi.len(),
        "stitched sweep must match full"
    );
}

#[test]
fn panchang_events_sunrise_anchored_kinds() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 1, 8, 0, 0, 0.0);
    let loc = new_delhi();
    let rs = RiseSetConfig::default();

    let result = panchang_events(
        &engine,
        &eop,
        &from,
        &to,
        PANCHANG_INCLUDE_VAAR | PANCHANG_INCLUDE_HORA | PANCHANG_INCLUDE_GHATIKA,
        Some(&loc),
        &rs,
        &aya(),
        0,
    )
    .expect("sunrise-anchored events should compute");
    assert!(!result.truncated);

    // 7 days: 7-8 vedic days, 24 horas and 60 ghatikas per day.
    assert!(
        result.vaar.len() >= 7 && result.vaar.len() <= 9,
        "vaar count {}",
        result.vaar.len()
    );
    assert!(
        result.hora.len() >= 7 * 24 && result.hora.len() <= 9 * 24,
        "hora count {}",
        result.hora.len()
    );
    assert!(
        result.ghatika.len() >= 7 * 60 && result.ghatika.len() <= 9 * 60,
        "ghatika count {}",
        result.ghatika.len()
    );

    // Segments chain exactly, including across Vedic-day rolls.
    for pair in result.vaar.windows(2) {
        assert_eq!(pair[0].end, pair[1].start, "vaar segments must chain");
        assert_ne!(pair[0].vaar, pair[1].vaar, "consecutive vaars must differ");
    }
    for pair in result.hora.windows(2) {
        assert_eq!(pair[0].end, pair[1].start, "hora segments must chain");
    }
    for pair in result.ghatika.windows(2) {
        assert_eq!(pair[0].end, pair[1].start, "ghatika segments must chain");
    }

    // Hora indices cycle 0..23; ghatika values cycle 1..60.
    for pair in result.hora.windows(2) {
        let expected = (pair[0].hora_index + 1) % 24;
        assert_eq!(pair[1].hora_index, expected, "hora indices must cycle");
    }
    for pair in result.ghatika.windows(2) {
        let expected = pair[0].value % 60 + 1;
        assert_eq!(pair[1].value, expected, "ghatika values must cycle");
    }

    // Cross-check against the per-moment API at segment midpoints.
    for info in result.vaar.iter().take(3) {
        let mid_jd = 0.5 * (jd(&engine, &info.start) + jd(&engine, &info.end));
        let mid = UtcTime::from_jd_tdb(mid_jd, engine.lsk());
        let direct = vaar_for_date(&engine, &eop, &mid, &loc, &rs).expect("per-moment vaar");
        assert_eq!(direct.vaar, info.vaar);
        assert!((jd(&engine, &direct.start) - jd(&engine, &info.start)).abs() < 2e-6);
        assert!((jd(&engine, &direct.end) - jd(&engine, &info.end)).abs() < 2e-6);
    }
    for info in result.hora.iter().take(30).skip(20) {
        let mid_jd = 0.5 * (jd(&engine, &info.start) + jd(&engine, &info.end));
        let mid = UtcTime::from_jd_tdb(mid_jd, engine.lsk());
        let direct = hora_for_date(&engine, &eop, &mid, &loc, &rs).expect("per-moment hora");
        assert_eq!(direct.hora, info.hora);
        assert_eq!(direct.hora_index, info.hora_index);
    }
}

// ---------------------------------------------------------------------------
// amsha_lagna_events
// ---------------------------------------------------------------------------

#[test]
fn amsha_lagna_events_d1_segments() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 15, 0, 0, 0.0);
    let to = UtcTime::new(2024, 1, 16, 2, 0, 0.0);
    let loc = new_delhi();
    let requests = [AmshaRequest::new(Amsha::D1)];

    let result = amsha_lagna_events(&engine, &eop, &from, &to, &loc, &aya(), &requests, 0)
        .expect("d1 events should compute");
    assert!(!result.truncated);
    assert_eq!(result.entries.len(), 1);
    let segments = &result.entries[0].segments;
    // 26 hours: every rashi rises roughly once (their durations vary).
    assert!(
        segments.len() >= 11 && segments.len() <= 16,
        "segment count {}",
        segments.len()
    );

    for pair in segments.windows(2) {
        assert_eq!(pair[0].end, pair[1].start, "segments must chain");
        assert_eq!(
            (pair[0].rashi_index + 1) % 12,
            pair[1].rashi_index,
            "D1 lagna rashis advance in order"
        );
    }

    // Classification matches the per-moment lagna at segment midpoints.
    for segment in segments.iter().take(6) {
        let mid_jd = 0.5 * (jd(&engine, &segment.start) + jd(&engine, &segment.end));
        let mid = UtcTime::from_jd_tdb(mid_jd, engine.lsk());
        let lagna =
            sidereal_lagna_for_date(&engine, &eop, &mid, &loc, &aya()).expect("per-moment lagna");
        let info = amsha_rashi_info(lagna, Amsha::D1, None);
        assert_eq!(info.rashi_index, segment.rashi_index);
    }
}

#[test]
fn amsha_lagna_events_d60_exact_boundaries() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 15, 6, 0, 0.0);
    let to = UtcTime::new(2024, 1, 15, 8, 0, 0.0);
    let loc = new_delhi();
    let requests = [AmshaRequest::new(Amsha::D60)];

    let result = amsha_lagna_events(&engine, &eop, &from, &to, &loc, &aya(), &requests, 0)
        .expect("d60 events should compute");
    assert!(!result.truncated);
    let segments = &result.entries[0].segments;
    // D60 rashi changes every 0.5 deg of D1 lagna (~2 min): a 2 h window has
    // dozens of segments — exactly the aliasing a sampling grid would hit.
    assert!(
        segments.len() >= 30 && segments.len() <= 90,
        "segment count {}",
        segments.len()
    );

    for pair in segments.windows(2) {
        assert_eq!(pair[0].end, pair[1].start, "segments must chain");
        assert_ne!(pair[0].rashi_index, pair[1].rashi_index);
    }

    // Each interior boundary lies on a D60 division boundary of the D1
    // ascendant longitude (multiples of 0.5 deg).
    for segment in segments.iter().take(10) {
        let lagna = sidereal_lagna_for_date(&engine, &eop, &segment.end, &loc, &aya())
            .expect("lagna at boundary");
        let frac = (lagna / 0.5).fract();
        let dist = frac.min(1.0 - frac) * 0.5;
        assert!(
            dist < 5e-4,
            "boundary lagna {lagna} not on 0.5 deg grid (dist {dist})"
        );
    }
}

#[test]
fn amsha_lagna_events_validation_and_truncation() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 15, 6, 0, 0.0);
    let to = UtcTime::new(2024, 1, 15, 8, 0, 0.0);
    let loc = new_delhi();

    let err = amsha_lagna_events(&engine, &eop, &from, &to, &loc, &aya(), &[], 0).unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));
    let err = amsha_lagna_events(
        &engine,
        &eop,
        &to,
        &from,
        &loc,
        &aya(),
        &[AmshaRequest::new(Amsha::D9)],
        0,
    )
    .unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));

    let capped = amsha_lagna_events(
        &engine,
        &eop,
        &from,
        &to,
        &loc,
        &aya(),
        &[AmshaRequest::new(Amsha::D60)],
        5,
    )
    .expect("capped sweep");
    assert!(capped.truncated);
    assert_eq!(capped.entries[0].segments.len(), 5);
    assert!(capped.next_from_utc.is_some());
}
