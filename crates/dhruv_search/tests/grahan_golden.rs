//! Golden-value integration tests for grahan computation.
//!
//! Validates against NASA Five Millennium Eclipse Catalog data.
//! Requires kernel files (de442s.bsp, naif0012.tls). Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::{
    ChandraGrahanType, EclipseGeoPoint, GeoLocation, GrahanConfig, SuryaGrahanType,
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
    let result = next_chandra_grahan(&engine, jd_start, &config).expect("search should succeed");
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
    let result = next_chandra_grahan(&engine, jd_start, &config).expect("search should succeed");
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
        search_chandra_grahan(&engine, jd_start, jd_end, &config).expect("search should succeed");

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
        search_chandra_grahan(&engine, jd_start, jd_end, &config).expect("search should succeed");

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
    let result = prev_chandra_grahan(&engine, jd, &config).expect("search should succeed");
    let grahan = result.expect("should find previous chandra grahan");

    // Previous chandra grahan should be before our search date
    assert!(grahan.greatest_grahan_jd < jd);
    // Contact times should be ordered: P1 < greatest < P4
    assert!(grahan.p1_jd < grahan.greatest_grahan_jd);
    assert!(grahan.greatest_grahan_jd < grahan.p4_jd);
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
