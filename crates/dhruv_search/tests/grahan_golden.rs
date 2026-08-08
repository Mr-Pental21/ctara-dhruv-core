//! Golden-value integration tests for grahan computation.
//!
//! Validates against NASA Five Millennium Eclipse Catalog data.
//! Requires kernel files (de442s.bsp, naif0012.tls). Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::{
    ChandraGrahan, ChandraGrahanType, EclipseGeoPoint, GeoLocation, GrahanConfig, SuryaGrahanType,
    next_chandra_grahan, next_surya_grahan, prev_chandra_grahan, prev_surya_grahan,
    search_chandra_grahan, search_surya_grahan,
};
use dhruv_time::EopKernel;

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";
const EOP_PATH: &str = "../../kernels/data/finals2000A.all";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping grahan_golden: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn jd_from_date(year: i32, month: u32, day: f64) -> f64 {
    dhruv_time::calendar_to_jd(year, month, day)
}

fn surface_unit_vector(point: EclipseGeoPoint) -> [f64; 3] {
    let lat = point.latitude_deg.to_radians();
    let lon = point.longitude_deg.to_radians();
    [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
}

fn vector_dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vector_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vector_norm(v: [f64; 3]) -> f64 {
    vector_dot(v, v).sqrt()
}

fn spherical_ring_contains(ring: &[EclipseGeoPoint], point: EclipseGeoPoint) -> bool {
    if ring.len() < 4 {
        return false;
    }
    let point_vector = surface_unit_vector(point);
    let mut tangent_vertices = Vec::with_capacity(ring.len() - 1);
    for vertex in &ring[..ring.len() - 1] {
        let vertex = surface_unit_vector(*vertex);
        let projected = [
            vertex[0] - point_vector[0] * vector_dot(vertex, point_vector),
            vertex[1] - point_vector[1] * vector_dot(vertex, point_vector),
            vertex[2] - point_vector[2] * vector_dot(vertex, point_vector),
        ];
        let length = vector_norm(projected);
        if length < 1.0e-12 {
            return vector_dot(vertex, point_vector) > 0.0;
        }
        tangent_vertices.push([
            projected[0] / length,
            projected[1] / length,
            projected[2] / length,
        ]);
    }

    let mut winding = 0.0;
    for index in 0..tangent_vertices.len() {
        let a = tangent_vertices[index];
        let b = tangent_vertices[(index + 1) % tangent_vertices.len()];
        winding += vector_dot(point_vector, vector_cross(a, b)).atan2(vector_dot(a, b));
    }
    winding.abs() > std::f64::consts::PI
}

fn great_circle_km(a: EclipseGeoPoint, b: EclipseGeoPoint) -> f64 {
    let dot = vector_dot(surface_unit_vector(a), surface_unit_vector(b)).clamp(-1.0, 1.0);
    6378.137 * dot.acos()
}

fn assert_closed_continuous_ring(boundary: &[EclipseGeoPoint]) {
    assert!(
        boundary.len() >= 4,
        "footprint must contain at least three vertices plus closure, got {}",
        boundary.len()
    );
    let first = boundary[0];
    let last = boundary[boundary.len() - 1];
    assert!(
        (first.latitude_deg - last.latitude_deg).abs() < 1.0e-9
            && (first.longitude_deg - last.longitude_deg).abs() < 1.0e-9,
        "footprint boundary is open: first={first:?}, last={last:?}"
    );
    let longest_edge = boundary
        .windows(2)
        .map(|edge| great_circle_km(edge[0], edge[1]))
        .fold(0.0, f64::max);
    assert!(
        longest_edge < 2_500.0,
        "footprint contains a discontinuous {longest_edge:.1} km edge"
    );
}

fn assert_central_path_inside_matching_footprints(
    path: &[dhruv_search::SuryaGrahanPathPoint],
    footprints: &[dhruv_search::SuryaGrahanFootprint],
) {
    for path_point in path {
        let footprint = footprints
            .iter()
            .find(|footprint| (footprint.jd_tdb - path_point.jd_tdb).abs() < 1.0e-9)
            .expect("timestamp-matched footprint");
        assert!(
            spherical_ring_contains(&footprint.boundary, path_point.center),
            "central point {:?} at JD {} lies outside its penumbral footprint",
            path_point.center,
            path_point.jd_tdb
        );
    }
}

fn assert_path_limits_are_local(path: &[dhruv_search::SuryaGrahanPathPoint]) {
    for point in path {
        let maximum_local_distance_km = point.width_km * 1.5 + 1.0;
        for (name, limit) in [
            ("northern", point.northern_limit),
            ("southern", point.southern_limit),
        ] {
            let limit = limit.expect("central path point must have both limits");
            let distance_km = great_circle_km(point.center, limit);
            assert!(
                distance_km <= maximum_local_distance_km,
                "{name} limit is a distant cone branch: center={:?}, limit={limit:?}, distance={distance_km:.1} km, width={:.1} km at JD {}",
                point.center,
                point.width_km,
                point.jd_tdb
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Chandra grahan (lunar eclipses)
// ---------------------------------------------------------------------------

/// 2024-Mar-25: Penumbral chandra grahan
/// NASA catalog: Greatest eclipse 07:13 UTC
#[test]
fn chandra_grahan_2024_mar_penumbral() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 3, 1.0);
    let config = GrahanConfig::default();
    let result = next_chandra_grahan(&engine, None, jd_start, None, &config).expect("search should succeed");
    let grahan = result.expect("should find a chandra grahan");

    // Should be in March 2024
    let expected_jd = jd_from_date(2024, 3, 25.3); // ~07:13 UTC
    let diff_hours = (grahan.greatest_grahan_jd - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 12.0,
        "chandra grahan off by {diff_hours:.1}h, got JD {}, expected ~JD {}",
        grahan.greatest_grahan_jd,
        expected_jd
    );
    assert_eq!(grahan.grahan_type, ChandraGrahanType::Penumbral);
}

/// 2025-Mar-14: Total chandra grahan
/// NASA catalog: Greatest eclipse ~06:59 UTC, magnitude 1.178
#[test]
fn chandra_grahan_2025_mar_total() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2025, 3, 1.0);
    let config = GrahanConfig::default();
    let result = next_chandra_grahan(&engine, None, jd_start, None, &config).expect("search should succeed");
    let grahan = result.expect("should find a chandra grahan");

    let expected_jd = jd_from_date(2025, 3, 14.29); // ~06:59 UTC
    let diff_hours = (grahan.greatest_grahan_jd - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 12.0,
        "chandra grahan off by {diff_hours:.1}h, got JD {}",
        grahan.greatest_grahan_jd
    );
    assert_eq!(grahan.grahan_type, ChandraGrahanType::Total);
    // Magnitude should be > 1 for total
    assert!(
        grahan.magnitude > 1.0,
        "total chandra grahan magnitude = {}, expected > 1",
        grahan.magnitude
    );

    // Moon apparent equatorial position at maximum. 2025-Mar-14 the eclipsed
    // Moon sits near the Leo/Virgo border: RA ~ 170-175°, dec ~ +2..+7°.
    assert!(
        grahan.moon_right_ascension_deg >= 0.0 && grahan.moon_right_ascension_deg < 360.0,
        "moon RA out of range: {}",
        grahan.moon_right_ascension_deg
    );
    assert!(
        (168.0..=178.0).contains(&grahan.moon_right_ascension_deg),
        "moon RA {} outside golden band",
        grahan.moon_right_ascension_deg
    );
    assert!(
        (0.0..=9.0).contains(&grahan.moon_declination_deg),
        "moon declination {} outside golden band",
        grahan.moon_declination_deg
    );
}

/// Search for chandra grahan in 2024 — should find 2 (Mar and Sep).
#[test]
fn chandra_grahan_2024_count() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 1.0);
    let jd_end = jd_from_date(2025, 1, 1.0);
    let config = GrahanConfig::default();
    let results =
        search_chandra_grahan(&engine, None, jd_start, jd_end, None, &config).expect("search should succeed");

    // 2024 has 2 chandra grahan: Mar 25 (penumbral) and Sep 18 (partial)
    assert!(
        results.len() >= 2,
        "found {} chandra grahan in 2024, expected at least 2",
        results.len()
    );
}

/// Penumbral-only filter: exclude penumbral grahan.
#[test]
fn penumbral_filter() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 1.0);
    let jd_end = jd_from_date(2025, 1, 1.0);
    let config = GrahanConfig {
        include_penumbral: false,
        ..Default::default()
    };
    let results =
        search_chandra_grahan(&engine, None, jd_start, jd_end, None, &config).expect("search should succeed");

    // With penumbral excluded, should have fewer grahan
    for e in &results {
        assert_ne!(
            e.grahan_type,
            ChandraGrahanType::Penumbral,
            "penumbral grahan should be filtered"
        );
    }
}

/// Backward search for previous chandra grahan.
#[test]
fn prev_chandra_grahan_from_2024() {
    let Some(engine) = load_engine() else { return };
    let jd = jd_from_date(2024, 3, 1.0);
    let config = GrahanConfig::default();
    let result = prev_chandra_grahan(&engine, None, jd, None, &config).expect("search should succeed");
    let grahan = result.expect("should find previous chandra grahan");

    // Previous chandra grahan should be before our search date
    assert!(grahan.greatest_grahan_jd < jd);
    // Contact times should be ordered: P1 < greatest < P4
    assert!(grahan.p1_jd < grahan.greatest_grahan_jd);
    assert!(grahan.greatest_grahan_jd < grahan.p4_jd);
}

// ---------------------------------------------------------------------------
// Chandra grahan local circumstances
// ---------------------------------------------------------------------------

/// Minutes elapsed in the UTC day. Every eclipse asserted below begins and
/// ends inside one UTC date, so this is a safe way to compare against
/// published clock times.
fn utc_minutes_of_day(t: dhruv_time::UtcTime) -> f64 {
    t.hour as f64 * 60.0 + t.minute as f64 + t.second / 60.0
}

fn hm(hour: u32, minute: f64) -> f64 {
    hour as f64 * 60.0 + minute
}

fn load_eop() -> Option<EopKernel> {
    EopKernel::load(Path::new(EOP_PATH)).ok()
}

/// Contact times of the 2025-Mar-14 total lunar eclipse, NASA/Espenak canon
/// (UTC): P1 03:57:28, U1 05:09:40, U2 06:26:06, greatest 06:58:43,
/// U3 07:31:26, U4 08:47:52, P4 10:00:11.
///
/// Umbral contacts agree to ~15 s and penumbral contacts to ~3 min; the
/// residual is the shadow-enlargement convention, not the contact solver.
#[test]
fn chandra_grahan_2025_mar_contacts_match_canon() {
    let Some(engine) = load_engine() else { return };
    let config = GrahanConfig::default();
    let grahan = next_chandra_grahan(&engine, None, jd_from_date(2025, 3, 10.0), None, &config)
        .expect("search")
        .expect("eclipse");

    assert_eq!(grahan.grahan_type, ChandraGrahanType::Total);
    for (label, actual, published, tolerance_min) in [
        ("P1", utc_minutes_of_day(grahan.p1_utc), hm(3, 57.47), 4.0),
        (
            "U1",
            utc_minutes_of_day(grahan.u1_utc.expect("u1")),
            hm(5, 9.67),
            1.0,
        ),
        (
            "U2",
            utc_minutes_of_day(grahan.u2_utc.expect("u2")),
            hm(6, 26.10),
            1.0,
        ),
        (
            "greatest",
            utc_minutes_of_day(grahan.greatest_grahan_utc),
            hm(6, 58.72),
            2.0,
        ),
        (
            "U3",
            utc_minutes_of_day(grahan.u3_utc.expect("u3")),
            hm(7, 31.43),
            2.0,
        ),
        (
            "U4",
            utc_minutes_of_day(grahan.u4_utc.expect("u4")),
            hm(8, 47.87),
            2.0,
        ),
        ("P4", utc_minutes_of_day(grahan.p4_utc), hm(10, 0.18), 4.0),
    ] {
        let error = (actual - published).abs();
        assert!(
            error < tolerance_min,
            "{label} off by {error:.2} min (got {actual:.2}, published {published:.2})"
        );
    }

    // Contacts must be strictly ordered. This is what catches an inverted
    // limb sign, which silently swaps U1 with U2 and U3 with U4.
    let ordered = [
        grahan.p1_jd,
        grahan.u1_jd.expect("u1"),
        grahan.u2_jd.expect("u2"),
        grahan.greatest_grahan_jd,
        grahan.u3_jd.expect("u3"),
        grahan.u4_jd.expect("u4"),
        grahan.p4_jd,
    ];
    for pair in ordered.windows(2) {
        assert!(
            pair[0] < pair[1],
            "contacts out of order: {ordered:?} (P1, U1, U2, greatest, U3, U4, P4)"
        );
    }
}

/// Fully visible: the 2025-Mar-14 total eclipse from Denver, where the Moon
/// stays well above the horizon from P1 to P4. The visible window must be the
/// whole event.
#[test]
fn chandra_local_fully_visible_denver_2025() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = GrahanConfig::default();
    let location = GeoLocation::new(39.7392, -104.9903, 0.0);
    let grahan = next_chandra_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2025, 3, 10.0),
        Some(location),
        &config,
    )
    .expect("search")
    .expect("eclipse");

    let local = grahan.local.expect("local circumstances");
    assert!(local.visible);
    assert_eq!(local.location.latitude_deg, 39.7392);

    // Moon is up throughout, so every contact altitude is positive.
    for (label, altitude) in [
        ("P1", local.p1_altitude_deg),
        ("U1", local.u1_altitude_deg.expect("u1 altitude")),
        ("U2", local.u2_altitude_deg.expect("u2 altitude")),
        ("U3", local.u3_altitude_deg.expect("u3 altitude")),
        ("U4", local.u4_altitude_deg.expect("u4 altitude")),
        ("P4", local.p4_altitude_deg),
    ] {
        assert!(altitude > 30.0, "{label} altitude {altitude:.2} too low");
    }

    // Greatest eclipse occurs near the meridian around local midnight.
    assert!(
        (45.0..60.0).contains(&local.moon_altitude_deg),
        "altitude at greatest = {:.2}",
        local.moon_altitude_deg
    );
    assert!(
        (150.0..200.0).contains(&local.moon_azimuth_deg),
        "azimuth at greatest = {:.2}",
        local.moon_azimuth_deg
    );

    // Nothing is clipped: the window is exactly [P1, P4].
    assert!((local.visible_start_jd.expect("start") - grahan.p1_jd).abs() < 1.0e-9);
    assert!((local.visible_end_jd.expect("end") - grahan.p4_jd).abs() < 1.0e-9);
    let full_span_seconds = (grahan.p4_jd - grahan.p1_jd) * 86_400.0;
    assert!(
        (local.visible_duration_seconds - full_span_seconds).abs() < 1.0,
        "duration {} vs full span {}",
        local.visible_duration_seconds,
        full_span_seconds
    );
}

/// Entirely below the horizon: the same eclipse from New Delhi, where it
/// happens in broad daylight. Nothing is observable, and the global contact
/// times must be untouched by the location.
#[test]
fn chandra_local_below_horizon_new_delhi_2025() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = GrahanConfig::default();
    let at = jd_from_date(2025, 3, 10.0);
    let location = GeoLocation::new(28.6139, 77.2090, 0.0);

    let grahan = next_chandra_grahan(&engine, Some(&eop), at, Some(location), &config)
        .expect("search")
        .expect("eclipse");
    let local = grahan.local.expect("local circumstances");

    assert!(!local.visible, "eclipse should be below the horizon");
    assert_eq!(local.visible_start_jd, None);
    assert_eq!(local.visible_start_utc, None);
    assert_eq!(local.visible_end_jd, None);
    assert_eq!(local.visible_end_utc, None);
    assert_eq!(local.visible_duration_seconds, 0.0);
    // Positive zero, not -0.0: a float `Sum` over no intervals starts from
    // -0.0, which would reach JSON consumers as a negative duration.
    assert!(local.visible_duration_seconds.is_sign_positive());

    for (label, altitude) in [
        ("P1", local.p1_altitude_deg),
        ("U1", local.u1_altitude_deg.expect("u1 altitude")),
        ("U4", local.u4_altitude_deg.expect("u4 altitude")),
        ("P4", local.p4_altitude_deg),
        ("greatest", local.moon_altitude_deg),
    ] {
        assert!(
            altitude < 0.0,
            "{label} altitude {altitude:.2} should be below the horizon"
        );
    }

    // A lunar eclipse is seen at the same instants everywhere: supplying a
    // location must add `local` and change nothing else.
    let global = next_chandra_grahan(&engine, Some(&eop), at, None, &config)
        .expect("search")
        .expect("eclipse");
    assert!(global.local.is_none());
    assert_eq!(global.p1_jd, grahan.p1_jd);
    assert_eq!(global.u1_jd, grahan.u1_jd);
    assert_eq!(global.u2_jd, grahan.u2_jd);
    assert_eq!(global.greatest_grahan_jd, grahan.greatest_grahan_jd);
    assert_eq!(global.u3_jd, grahan.u3_jd);
    assert_eq!(global.u4_jd, grahan.u4_jd);
    assert_eq!(global.p4_jd, grahan.p4_jd);
    assert_eq!(global.magnitude, grahan.magnitude);
    assert_eq!(ChandraGrahan { local: None, ..grahan }, global);
}

/// Moon sets mid-eclipse: the 2025-Mar-14 eclipse from London begins with the
/// Moon up and is cut short by moonset during the partial phase. Published
/// local circumstances put London's moonset at 06:23 UTC that morning, with
/// the Moon setting while eclipsed.
///
/// This is the case a naive implementation gets wrong, by reporting the
/// geometric P4 as the local end.
#[test]
fn chandra_local_moonset_mid_eclipse_london_2025() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = GrahanConfig::default();
    let location = GeoLocation::new(51.5074, -0.1278, 0.0);
    let grahan = next_chandra_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2025, 3, 10.0),
        Some(location),
        &config,
    )
    .expect("search")
    .expect("eclipse");

    let local = grahan.local.expect("local circumstances");
    assert!(local.visible, "the opening phase is visible from London");

    // Starts at P1 (Moon already up), ends at moonset well before P4.
    let start_jd = local.visible_start_jd.expect("start");
    let end_jd = local.visible_end_jd.expect("end");
    assert!((start_jd - grahan.p1_jd).abs() < 1.0e-9, "should start at P1");
    assert!(
        end_jd < grahan.p4_jd - 1.0 / 1440.0,
        "visible end must precede P4"
    );
    assert!(
        end_jd > grahan.u1_jd.expect("u1"),
        "the Moon sets after the umbral phase begins"
    );

    // Published moonset for London on 2025-03-14 is 06:23 UTC.
    let end_minutes = utc_minutes_of_day(local.visible_end_utc.expect("end utc"));
    assert!(
        (end_minutes - hm(6, 23.0)).abs() < 12.0,
        "visible end {end_minutes:.2} min-of-day, expected ~06:23 UTC"
    );

    // The Moon crosses the horizon between U1 and U2: still up for first
    // umbral contact, already set by the start of totality.
    assert!(local.p1_altitude_deg > 0.0);
    assert!(local.u1_altitude_deg.expect("u1 altitude") > 0.0);
    assert!(local.u2_altitude_deg.expect("u2 altitude") < 0.0);
    assert!(local.p4_altitude_deg < 0.0);
    assert!(
        local.moon_altitude_deg < 0.0,
        "greatest eclipse is below the horizon from London"
    );

    // The window closes exactly at the horizon crossing.
    let duration_days = end_jd - start_jd;
    assert!(
        (local.visible_duration_seconds - duration_days * 86_400.0).abs() < 1.0,
        "duration must match the single visible interval"
    );
    assert!(
        (140.0..155.0).contains(&(local.visible_duration_seconds / 60.0)),
        "visible duration {:.1} min",
        local.visible_duration_seconds / 60.0
    );
}

/// Moon rises mid-eclipse: the 2022-Nov-08 total eclipse from Bangkok, where
/// the Moon rises already totally eclipsed. Published moonrise for Bangkok on
/// 2022-11-08 is 17:44 local time (ICT, UTC+7) — 10:44 UTC, between U2 and U3.
#[test]
fn chandra_local_moonrise_mid_eclipse_bangkok_2022() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = GrahanConfig::default();
    let location = GeoLocation::new(13.7563, 100.5018, 0.0);
    let grahan = next_chandra_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2022, 11, 5.0),
        Some(location),
        &config,
    )
    .expect("search")
    .expect("eclipse");

    assert_eq!(grahan.grahan_type, ChandraGrahanType::Total);
    let local = grahan.local.expect("local circumstances");
    assert!(local.visible);

    let start_jd = local.visible_start_jd.expect("start");
    let end_jd = local.visible_end_jd.expect("end");

    // Rises during totality, then stays up through P4.
    assert!(
        start_jd > grahan.u2_jd.expect("u2") && start_jd < grahan.u3_jd.expect("u3"),
        "moonrise should fall inside totality"
    );
    assert!((end_jd - grahan.p4_jd).abs() < 1.0e-9, "should end at P4");

    let start_minutes = utc_minutes_of_day(local.visible_start_utc.expect("start utc"));
    assert!(
        (start_minutes - hm(10, 44.0)).abs() < 12.0,
        "visible start {start_minutes:.2} min-of-day, expected ~10:44 UTC"
    );

    // Below the horizon for the opening contacts, above it for the closing.
    assert!(local.p1_altitude_deg < 0.0);
    assert!(local.u1_altitude_deg.expect("u1 altitude") < 0.0);
    assert!(local.u2_altitude_deg.expect("u2 altitude") < 0.0);
    assert!(local.u3_altitude_deg.expect("u3 altitude") > 0.0);
    assert!(local.u4_altitude_deg.expect("u4 altitude") > 0.0);
    assert!(local.p4_altitude_deg > 0.0);

    let duration_days = end_jd - start_jd;
    assert!(
        (local.visible_duration_seconds - duration_days * 86_400.0).abs() < 1.0,
        "duration must match the single visible interval"
    );
    assert!(
        local.visible_duration_seconds < (grahan.p4_jd - grahan.p1_jd) * 86_400.0 - 60.0,
        "the clipped window must be shorter than the whole event"
    );
}

/// A penumbral eclipse has no umbral contacts, so the matching altitudes are
/// absent rather than zero.
#[test]
fn chandra_local_penumbral_has_no_umbral_altitudes() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = GrahanConfig::default();
    let grahan = next_chandra_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2024, 3, 1.0),
        Some(GeoLocation::new(39.7392, -104.9903, 0.0)),
        &config,
    )
    .expect("search")
    .expect("eclipse");

    assert_eq!(grahan.grahan_type, ChandraGrahanType::Penumbral);
    let local = grahan.local.expect("local circumstances");
    assert_eq!(local.u1_altitude_deg, None);
    assert_eq!(local.u2_altitude_deg, None);
    assert_eq!(local.u3_altitude_deg, None);
    assert_eq!(local.u4_altitude_deg, None);
    assert!(local.p1_altitude_deg.is_finite());
    assert!(local.p4_altitude_deg.is_finite());
}

/// A location supplied to a range search must reach every event in it.
#[test]
fn chandra_local_applies_across_range_search() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = GrahanConfig::default();
    let location = GeoLocation::new(19.0760, 72.8777, 0.0);
    let results = search_chandra_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2025, 1, 1.0),
        jd_from_date(2026, 1, 1.0),
        Some(location),
        &config,
    )
    .expect("search");

    assert!(!results.is_empty(), "2025 has lunar eclipses");
    for grahan in &results {
        let local = grahan.local.expect("every event carries local circumstances");
        assert_eq!(local.location.longitude_deg, 72.8777);
        if local.visible {
            assert!(local.visible_duration_seconds > 0.0);
            assert!(local.visible_start_jd.expect("start") >= grahan.p1_jd - 1.0e-9);
            assert!(local.visible_end_jd.expect("end") <= grahan.p4_jd + 1.0e-9);
        } else {
            assert_eq!(local.visible_duration_seconds, 0.0);
        }
    }

    // Omitting the location leaves `local` unset, which is what the cached
    // global catalogue relies on.
    let global = search_chandra_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2025, 1, 1.0),
        jd_from_date(2026, 1, 1.0),
        None,
        &config,
    )
    .expect("search");
    assert_eq!(global.len(), results.len());
    assert!(global.iter().all(|grahan| grahan.local.is_none()));
}

// ---------------------------------------------------------------------------
// Surya grahan (solar eclipses)
// ---------------------------------------------------------------------------

/// 2024-Apr-08: total eclipse with a North American central path.
#[test]
fn surya_grahan_2024_apr() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 3, 1.0);
    let config = GrahanConfig::default();
    let result =
        next_surya_grahan(&engine, None, jd_start, None, &config).expect("search should succeed");
    let grahan = result.expect("should find a surya grahan");

    let expected_jd = jd_from_date(2024, 4, 8.763); // ~18:18 UTC
    let diff_hours = (grahan.greatest_grahan_jd - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 12.0,
        "surya grahan off by {diff_hours:.1}h, got JD {}, expected ~JD {}",
        grahan.greatest_grahan_jd,
        expected_jd
    );
    assert_eq!(grahan.grahan_type, SuryaGrahanType::Total);
    assert!(
        grahan.magnitude > 0.90,
        "surya grahan magnitude = {}, expected > 0.90",
        grahan.magnitude
    );

    // Sun apparent equatorial position at maximum. On 2024-Apr-08 the Sun
    // is in Aries: RA ~ 17.4°, dec ~ +7.4°.
    assert!(
        (14.0..=21.0).contains(&grahan.sun_right_ascension_deg),
        "sun RA {} outside golden band",
        grahan.sun_right_ascension_deg
    );
    assert!(
        (5.5..=9.5).contains(&grahan.sun_declination_deg),
        "sun declination {} outside golden band",
        grahan.sun_declination_deg
    );
    assert!(
        (grahan.gamma - 0.343).abs() < 0.02,
        "gamma={}",
        grahan.gamma
    );
    let point = grahan.greatest_location.expect("greatest location");
    assert!((point.latitude_deg - 25.3).abs() < 1.0);
    assert!((point.longitude_deg + 104.3).abs() < 1.0);
}

/// 2024-Oct-02: annular eclipse.
#[test]
fn surya_grahan_2024_oct() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 9, 1.0);
    let config = GrahanConfig::default();
    let result =
        next_surya_grahan(&engine, None, jd_start, None, &config).expect("search should succeed");
    let grahan = result.expect("should find a surya grahan");

    let expected_jd = jd_from_date(2024, 10, 2.78); // ~18:45 UTC
    let diff_hours = (grahan.greatest_grahan_jd - expected_jd).abs() * 24.0;
    assert!(
        diff_hours < 12.0,
        "surya grahan off by {diff_hours:.1}h, got JD {}",
        grahan.greatest_grahan_jd
    );
    assert_eq!(grahan.grahan_type, SuryaGrahanType::Annular);
    assert!(
        grahan.magnitude > 0.90,
        "surya grahan magnitude = {}, expected > 0.90",
        grahan.magnitude
    );
}

#[test]
fn surya_grahan_path_and_local_circumstances() {
    let Some(engine) = load_engine() else { return };
    let Ok(eop) = EopKernel::load(Path::new(EOP_PATH)) else {
        return;
    };
    let config = GrahanConfig {
        include_path: true,
        path_step_minutes: 5,
        boundary_step_deg: 10,
        ..Default::default()
    };
    let location = GeoLocation::new(25.2854, -104.3, 0.0);
    let grahan = next_surya_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2024, 3, 1.0),
        Some(location),
        &config,
    )
    .expect("search")
    .expect("eclipse");
    assert!(grahan.path.len() > 30);
    assert!(grahan.footprints.len() > 50);
    for footprint in &grahan.footprints {
        assert_closed_continuous_ring(&footprint.boundary);
    }
    assert_central_path_inside_matching_footprints(&grahan.path, &grahan.footprints);
    let peak = &grahan.path[grahan.path.len() / 2];
    assert!(
        (50.0..400.0).contains(&peak.width_km),
        "width={}",
        peak.width_km
    );
    assert!((120.0..420.0).contains(&peak.central_duration_seconds));
    let local = grahan.local.expect("local circumstances");
    assert!(local.visible);
    assert_eq!(local.grahan_type, Some(SuryaGrahanType::Total));
    assert!((240.0..300.0).contains(&local.central_duration_seconds));

    // Torreon sees the whole eclipse with the Sun up, so the Sun-up-clipped
    // window coincides with the geometric contacts.
    let c1 = local.c1_jd.expect("c1");
    let c4 = local.c4_jd.expect("c4");
    let first_visible = local.first_visible_contact_jd.expect("first visible");
    let last_visible = local.last_visible_contact_jd.expect("last visible");
    assert!(
        (first_visible - c1).abs() < 1.0 / 1440.0,
        "first visible contact {first_visible} should match C1 {c1}"
    );
    assert!(
        (last_visible - c4).abs() < 1.0 / 1440.0,
        "last visible contact {last_visible} should match C4 {c4}"
    );
    assert!(local.first_visible_contact_utc.is_some());
    assert!(local.last_visible_contact_utc.is_some());
    assert!(
        (local.visible_duration_seconds - (c4 - c1) * 86_400.0).abs() < 60.0,
        "visible duration {} vs C1-C4 span {}",
        local.visible_duration_seconds,
        (c4 - c1) * 86_400.0
    );
}

/// A solar eclipse that is entirely below the horizon reports no visible
/// window at all, rather than surfacing geometric contacts as local timings.
///
/// The 2024-Apr-08 total eclipse crossed North America; from Perth, Western
/// Australia it is the middle of the night and nothing is observable.
#[test]
fn surya_local_visible_window_absent_below_horizon() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = GrahanConfig::default();
    let grahan = next_surya_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2024, 3, 1.0),
        Some(GeoLocation::new(-31.9523, 115.8613, 0.0)),
        &config,
    )
    .expect("search")
    .expect("eclipse");

    let local = grahan.local.expect("local circumstances");
    assert!(!local.visible, "the eclipse is below Perth's horizon");
    assert_eq!(local.first_visible_contact_jd, None);
    assert_eq!(local.first_visible_contact_utc, None);
    assert_eq!(local.last_visible_contact_jd, None);
    assert_eq!(local.last_visible_contact_utc, None);
    assert_eq!(local.visible_duration_seconds, 0.0);
    assert!(local.visible_duration_seconds.is_sign_positive());
}

#[test]
fn surya_hybrid_and_noncentral_classification() {
    let Some(engine) = load_engine() else { return };
    let config = GrahanConfig::default();
    let hybrid = next_surya_grahan(&engine, None, jd_from_date(2013, 10, 1.0), None, &config)
        .expect("search")
        .expect("hybrid");
    assert_eq!(hybrid.grahan_type, SuryaGrahanType::Hybrid);

    let noncentral = next_surya_grahan(&engine, None, jd_from_date(2014, 4, 1.0), None, &config)
        .expect("search")
        .expect("noncentral annular");
    assert_eq!(noncentral.grahan_type, SuryaGrahanType::Annular);
}

#[test]
fn surya_antimeridian_and_polar_footprint_geometry() {
    let Some(engine) = load_engine() else { return };
    let config = GrahanConfig {
        include_path: true,
        path_step_minutes: 10,
        boundary_step_deg: 10,
        ..Default::default()
    };
    let antimeridian = next_surya_grahan(&engine, None, jd_from_date(2002, 6, 1.0), None, &config)
        .expect("search")
        .expect("annular eclipse");
    assert_eq!(antimeridian.grahan_type, SuryaGrahanType::Annular);
    assert!(antimeridian.path.windows(2).any(|pair| {
        (pair[1].center.longitude_deg - pair[0].center.longitude_deg).abs() > 180.0
    }));
    assert!(antimeridian.path.iter().all(|point| {
        (-180.0..=180.0).contains(&point.center.longitude_deg)
            && (-90.0..=90.0).contains(&point.center.latitude_deg)
    }));
    for footprint in &antimeridian.footprints {
        assert_closed_continuous_ring(&footprint.boundary);
    }
    assert_central_path_inside_matching_footprints(&antimeridian.path, &antimeridian.footprints);

    let polar = next_surya_grahan(&engine, None, jd_from_date(2025, 3, 1.0), None, &config)
        .expect("search")
        .expect("partial eclipse");
    assert_eq!(polar.grahan_type, SuryaGrahanType::Partial);
    assert!(polar.path.is_empty());
    for footprint in &polar.footprints {
        assert_closed_continuous_ring(&footprint.boundary);
    }
    assert!(polar.footprints.iter().any(|footprint| {
        footprint
            .boundary
            .iter()
            .any(|point| point.latitude_deg > 80.0)
    }));
}

#[test]
fn surya_footprints_are_closed_continuous_rings_containing_the_central_path() {
    let Some(engine) = load_engine() else { return };
    let Ok(eop) = EopKernel::load(Path::new(EOP_PATH)) else {
        return;
    };
    let config = GrahanConfig {
        include_path: true,
        path_step_minutes: 5,
        boundary_step_deg: 5,
        ..Default::default()
    };
    let grahan = next_surya_grahan(
        &engine,
        Some(&eop),
        jd_from_date(2026, 2, 1.0),
        None,
        &config,
    )
    .expect("search")
    .expect("2026 annular eclipse");

    assert_eq!(grahan.grahan_type, SuryaGrahanType::Annular);
    assert!(!grahan.path.is_empty());
    assert!(!grahan.footprints.is_empty());
    for footprint in &grahan.footprints {
        assert_closed_continuous_ring(&footprint.boundary);
    }
    assert_central_path_inside_matching_footprints(&grahan.path, &grahan.footprints);
    assert_path_limits_are_local(&grahan.path);
}

#[test]
fn surya_2001_2100_catalog_distribution() {
    let Some(engine) = load_engine() else { return };
    let events = search_surya_grahan(
        &engine,
        None,
        jd_from_date(2001, 1, 1.0),
        jd_from_date(2101, 1, 1.0),
        None,
        &GrahanConfig::default(),
    )
    .expect("century search");
    let count = |kind| {
        events
            .iter()
            .filter(|event| event.grahan_type == kind)
            .count()
    };
    assert_eq!(events.len(), 224);
    assert_eq!(count(SuryaGrahanType::Partial), 77);
    assert_eq!(count(SuryaGrahanType::Annular), 72);
    assert_eq!(count(SuryaGrahanType::Total), 68);
    assert_eq!(count(SuryaGrahanType::Hybrid), 7);
}

/// Search for surya grahan in 2024 — should find 2 (Apr total, Oct annular).
#[test]
fn surya_grahan_2024_count() {
    let Some(engine) = load_engine() else { return };
    let jd_start = jd_from_date(2024, 1, 1.0);
    let jd_end = jd_from_date(2025, 1, 1.0);
    let config = GrahanConfig::default();
    let results = search_surya_grahan(&engine, None, jd_start, jd_end, None, &config)
        .expect("search should succeed");

    assert!(
        results.len() >= 2,
        "found {} surya grahan in 2024, expected at least 2",
        results.len()
    );
}

/// Backward search for previous surya grahan.
#[test]
fn prev_surya_grahan_from_2024() {
    let Some(engine) = load_engine() else { return };
    let jd = jd_from_date(2024, 3, 1.0);
    let config = GrahanConfig::default();
    let result =
        prev_surya_grahan(&engine, None, jd, None, &config).expect("search should succeed");
    let grahan = result.expect("should find previous surya grahan");

    assert!(grahan.greatest_grahan_jd < jd);
    // Contact times C1 < greatest < C4 (if present)
    if let Some(c1) = grahan.c1_jd {
        assert!(c1 < grahan.greatest_grahan_jd);
    }
    if let Some(c4) = grahan.c4_jd {
        assert!(grahan.greatest_grahan_jd < c4);
    }
}
