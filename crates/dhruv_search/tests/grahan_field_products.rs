//! Integration tests for Surya grahan field products: the local-circumstance
//! grid, visibility/duration/magnitude isolines, and the swept central
//! corridor. Covers the polar, antimeridian, and hybrid validation events.
//! Requires kernel files (de442s.bsp, naif0012.tls); skips gracefully if
//! absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::{
    EclipseGeoPoint, GeoLocation, GrahanConfig, SuryaCentrality, SuryaGrahan, SuryaGrahanType,
    SuryaIsolineRing, next_surya_grahan,
};
use dhruv_time::EopKernel;

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";
const EOP_PATH: &str = "../../kernels/data/finals2000A.all";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping grahan_field_products: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn load_eop() -> Option<EopKernel> {
    EopKernel::load(Path::new(EOP_PATH)).ok()
}

fn field_config() -> GrahanConfig {
    GrahanConfig {
        include_path: true,
        path_step_minutes: 5,
        boundary_step_deg: 5,
        include_local_grid: true,
        local_grid_step_deg: 10.0,
        include_isolines: true,
        include_central_corridor: true,
        ..GrahanConfig::default()
    }
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

fn inside_any_ring(rings: &[SuryaIsolineRing], point: EclipseGeoPoint) -> bool {
    rings
        .iter()
        .any(|ring| spherical_ring_contains(&ring.boundary, point))
}

fn distance_to_rings_km(rings: &[SuryaIsolineRing], point: EclipseGeoPoint) -> f64 {
    rings
        .iter()
        .flat_map(|ring| ring.boundary.iter())
        .map(|vertex| great_circle_km(*vertex, point))
        .fold(f64::INFINITY, f64::min)
}

fn assert_closed_ring(ring: &SuryaIsolineRing, what: &str) {
    let boundary = &ring.boundary;
    assert!(
        boundary.len() >= 4,
        "{what}: ring needs at least three vertices plus closure, got {}",
        boundary.len()
    );
    let first = boundary[0];
    let last = boundary[boundary.len() - 1];
    assert!(
        (first.latitude_deg - last.latitude_deg).abs() < 1.0e-9
            && (first.longitude_deg - last.longitude_deg).abs() < 1.0e-9,
        "{what}: ring is open (first={first:?}, last={last:?})"
    );
    let longest_edge = boundary
        .windows(2)
        .map(|edge| great_circle_km(edge[0], edge[1]))
        .fold(0.0, f64::max);
    assert!(
        longest_edge < 2_500.0,
        "{what}: discontinuous {longest_edge:.1} km edge"
    );
    for point in boundary {
        assert!(
            point.latitude_deg.is_finite()
                && (-90.0..=90.0).contains(&point.latitude_deg)
                && point.longitude_deg.is_finite()
                && (-180.0..=180.0).contains(&point.longitude_deg),
            "{what}: vertex out of range: {point:?}"
        );
    }
}

fn event_with_fields(engine: &Engine, eop: Option<&EopKernel>, from_jd: f64) -> SuryaGrahan {
    next_surya_grahan(engine, eop, from_jd, None, &field_config())
        .expect("surya search")
        .expect("surya event")
}

fn jd(year: i32, month: u32, day: f64) -> f64 {
    dhruv_time::calendar_to_jd(year, month, day)
}

/// 2026-08-12 Arctic total eclipse: acceptance checks 1-3, 5, 6 plus ring
/// contracts.
#[test]
fn arctic_total_2026_field_products() {
    let Some(engine) = load_engine() else { return };
    let eop = load_eop();
    let event = event_with_fields(&engine, eop.as_ref(), jd(2026, 8, 1.0));
    assert_eq!(event.grahan_type, SuryaGrahanType::Total);
    assert_eq!(event.centrality, SuryaCentrality::Full);
    assert!(!event.local_grid.is_empty());
    let isolines = event.isolines.as_ref().expect("isolines");
    let corridor = event.central_corridor.as_ref().expect("corridor");

    // Check 1: the polar cells that Phoenix-side planar tests used to skip
    // are present with the per-location magnitudes.
    for (lat, lon, expected_magnitude) in [(85.0, -95.0, 0.9215), (85.0, -85.0, 0.9315)] {
        let sample = event
            .local_grid
            .iter()
            .find(|sample| {
                (sample.latitude_deg - lat).abs() < 1.0e-9
                    && (sample.longitude_deg - lon).abs() < 1.0e-9
            })
            .unwrap_or_else(|| panic!("grid sample ({lat}, {lon}) missing"));
        assert!(
            (sample.magnitude - expected_magnitude).abs() < 2.0e-3,
            "grid magnitude at ({lat}, {lon}) = {} (expected ~{expected_magnitude})",
            sample.magnitude
        );
        // Cross-check against the per-location circumstances.
        let local_event = next_surya_grahan(
            &engine,
            eop.as_ref(),
            jd(2026, 8, 1.0),
            Some(GeoLocation::new(lat, lon, 0.0)),
            &GrahanConfig::default(),
        )
        .expect("local search")
        .expect("local event");
        let local = local_event.local.expect("local circumstances");
        assert!(local.visible, "({lat}, {lon}) must be visible");
        assert!(
            (sample.magnitude - local.magnitude).abs() < 1.0e-3,
            "grid vs local magnitude at ({lat}, {lon}): {} vs {}",
            sample.magnitude,
            local.magnitude
        );
    }

    // Ring contracts for every isoline family.
    for ring in &isolines.visibility_boundary {
        assert_closed_ring(ring, "visibility boundary");
    }
    for level in &isolines.duration_isolines {
        for ring in &level.rings {
            assert_closed_ring(ring, "duration isoline");
        }
    }
    for level in &isolines.magnitude_isolines {
        for ring in &level.rings {
            assert_closed_ring(ring, "magnitude isoline");
        }
    }

    // Check 2: every visible grid sample lies inside (or within one grid
    // step of) the visibility boundary.
    let step_km = 10.0 * 111.2;
    for sample in &event.local_grid {
        let point = EclipseGeoPoint {
            latitude_deg: sample.latitude_deg,
            longitude_deg: sample.longitude_deg,
        };
        assert!(
            inside_any_ring(&isolines.visibility_boundary, point)
                || distance_to_rings_km(&isolines.visibility_boundary, point) < step_km,
            "visible sample ({}, {}) outside visibility boundary",
            sample.latitude_deg,
            sample.longitude_deg
        );
    }

    // Check 3: grid magnitude near the greatest location matches the event
    // magnitude.
    let greatest = event.greatest_location.expect("greatest location");
    let nearest = event
        .local_grid
        .iter()
        .min_by(|a, b| {
            let da = great_circle_km(
                EclipseGeoPoint {
                    latitude_deg: a.latitude_deg,
                    longitude_deg: a.longitude_deg,
                },
                greatest,
            );
            let db = great_circle_km(
                EclipseGeoPoint {
                    latitude_deg: b.latitude_deg,
                    longitude_deg: b.longitude_deg,
                },
                greatest,
            );
            da.total_cmp(&db)
        })
        .expect("nearest grid sample");
    assert!(
        (nearest.magnitude - event.magnitude).abs() < 2.0e-2,
        "grid magnitude near greatest = {} vs event magnitude {}",
        nearest.magnitude,
        event.magnitude
    );

    // Check 5: duration fractions from the grid agree with the isolines at
    // mid-latitudes (well away from the ring, to stay clear of sampling
    // error).
    let span_days = event.c4_jd.expect("c4") - event.c1_jd.expect("c1");
    for level in &isolines.duration_isolines {
        if level.rings.is_empty() {
            continue;
        }
        let threshold_seconds = level.fraction * span_days * 86_400.0;
        for sample in &event.local_grid {
            if sample.latitude_deg.abs() > 66.0 {
                continue;
            }
            let point = EclipseGeoPoint {
                latitude_deg: sample.latitude_deg,
                longitude_deg: sample.longitude_deg,
            };
            if distance_to_rings_km(&level.rings, point) < 1.5 * step_km {
                continue;
            }
            let inside = inside_any_ring(&level.rings, point);
            let above = sample.visible_duration_seconds > threshold_seconds;
            assert_eq!(
                inside,
                above,
                "duration fraction {} disagrees at ({}, {}): inside={inside}, duration={}s, threshold={}s",
                level.fraction,
                sample.latitude_deg,
                sample.longitude_deg,
                sample.visible_duration_seconds,
                threshold_seconds
            );
        }
    }

    // Check 6: single total corridor segment; path centers inside it;
    // corridor rings inside the visibility boundary.
    assert!(!corridor.segments.is_empty(), "corridor missing");
    for segment in &corridor.segments {
        assert_eq!(segment.grahan_type, SuryaGrahanType::Total);
        for ring in &segment.rings {
            assert_closed_ring(ring, "corridor");
        }
    }
    let corridor_rings: Vec<SuryaIsolineRing> = corridor
        .segments
        .iter()
        .flat_map(|segment| segment.rings.iter().cloned())
        .collect();
    for point in &event.path {
        assert!(
            inside_any_ring(&corridor_rings, point.center),
            "path center {:?} outside corridor",
            point.center
        );
    }
    for ring in &corridor_rings {
        for vertex in ring.boundary.iter().step_by(10) {
            assert!(
                inside_any_ring(&isolines.visibility_boundary, *vertex)
                    || distance_to_rings_km(&isolines.visibility_boundary, *vertex) < 100.0,
                "corridor vertex {vertex:?} outside visibility boundary"
            );
        }
    }
}

/// 2023-04-20 hybrid: both annular and total corridor segments (check 6).
#[test]
fn hybrid_2023_corridor_has_both_segment_types() {
    let Some(engine) = load_engine() else { return };
    let eop = load_eop();
    let config = GrahanConfig {
        include_path: true,
        path_step_minutes: 5,
        boundary_step_deg: 5,
        include_central_corridor: true,
        ..GrahanConfig::default()
    };
    let event = next_surya_grahan(&engine, eop.as_ref(), jd(2023, 4, 1.0), None, &config)
        .expect("surya search")
        .expect("surya event");
    assert_eq!(event.grahan_type, SuryaGrahanType::Hybrid);
    assert_eq!(event.centrality, SuryaCentrality::Full);
    let corridor = event.central_corridor.as_ref().expect("corridor");
    let has_total = corridor
        .segments
        .iter()
        .any(|segment| segment.grahan_type == SuryaGrahanType::Total);
    let has_annular = corridor
        .segments
        .iter()
        .any(|segment| segment.grahan_type == SuryaGrahanType::Annular);
    assert!(has_total, "hybrid corridor missing total segment");
    assert!(has_annular, "hybrid corridor missing annular segment");
    for segment in &corridor.segments {
        for ring in &segment.rings {
            assert_closed_ring(ring, "hybrid corridor");
        }
    }
    // The dominant total band contains the total-phase path centers.
    let total_rings: Vec<SuryaIsolineRing> = corridor
        .segments
        .iter()
        .filter(|segment| segment.grahan_type == SuryaGrahanType::Total)
        .flat_map(|segment| segment.rings.iter().cloned())
        .collect();
    let contained = event
        .path
        .iter()
        .filter(|point| point.grahan_type == SuryaGrahanType::Total)
        .filter(|point| inside_any_ring(&total_rings, point.center))
        .count();
    let total_points = event
        .path
        .iter()
        .filter(|point| point.grahan_type == SuryaGrahanType::Total)
        .count();
    assert!(
        total_points > 0 && contained * 10 >= total_points * 9,
        "total corridor contains {contained}/{total_points} total-phase centers"
    );
}

/// 2026-02-17 Antarctic annular: the corridor ring closes correctly near the
/// pole (check 6) and the path stays inside it.
#[test]
fn antarctic_annular_2026_corridor_closes() {
    let Some(engine) = load_engine() else { return };
    let eop = load_eop();
    let config = GrahanConfig {
        include_path: true,
        path_step_minutes: 5,
        boundary_step_deg: 5,
        include_central_corridor: true,
        ..GrahanConfig::default()
    };
    let event = next_surya_grahan(&engine, eop.as_ref(), jd(2026, 2, 1.0), None, &config)
        .expect("surya search")
        .expect("surya event");
    assert_eq!(event.grahan_type, SuryaGrahanType::Annular);
    assert_eq!(event.centrality, SuryaCentrality::Full);
    let corridor = event.central_corridor.as_ref().expect("corridor");
    assert!(!corridor.segments.is_empty());
    let rings: Vec<SuryaIsolineRing> = corridor
        .segments
        .iter()
        .flat_map(|segment| segment.rings.iter().cloned())
        .collect();
    for segment in &corridor.segments {
        assert_eq!(segment.grahan_type, SuryaGrahanType::Annular);
        for ring in &segment.rings {
            assert_closed_ring(ring, "antarctic corridor");
        }
    }
    for point in &event.path {
        assert!(
            inside_any_ring(&rings, point.center),
            "path center {:?} outside corridor",
            point.center
        );
    }
}

/// The effective-config echo sanitizes grid step and isoline levels.
#[test]
fn effective_config_sanitizes() {
    let config = GrahanConfig {
        local_grid_step_deg: 0.01,
        duration_isoline_fractions: vec![0.75, 0.25, 0.25, -1.0, 2.0, f64::NAN],
        magnitude_isoline_levels: vec![1.6, 1.0, 0.5, 0.5],
        ..GrahanConfig::default()
    };
    let effective = config.effective();
    assert_eq!(effective.local_grid_step_deg, 0.5);
    assert_eq!(effective.duration_isoline_fractions, vec![0.25, 0.75]);
    assert_eq!(effective.magnitude_isoline_levels, vec![0.5, 1.0]);
}
