//! Integration tests for Surya grahan field products: the local-circumstance
//! grid, visibility/duration/magnitude isolines, and the swept central
//! corridor. Covers the polar, antimeridian, and hybrid validation events.
//! Requires kernel files (de442s.bsp, naif0012.tls); skips gracefully if
//! absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::{
    EclipseGeoPoint, GeoLocation, GrahanConfig, PoleSide, SuryaCentrality, SuryaContactKind,
    SuryaGrahan, SuryaGrahanType, SuryaIsolineRing, besselian_elements_at, next_surya_grahan,
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
        include_contact_footprints: true,
        include_umbra_footprints: true,
        instantaneous_magnitude_levels: vec![0.25, 0.5, 0.75],
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

/// Change 8 on the 2026-08-12 Arctic total event: sampled penumbral
/// footprints are terminator-clipped — no vertex beyond the day side, some
/// vertices on the terminator where the region is truncated, and the
/// central path stays inside its timestamp-matched footprint.
#[test]
fn arctic_total_2026_footprints_are_terminator_clipped() {
    let Some(engine) = load_engine() else { return };
    let eop = load_eop();
    // The 1-minute cadence the consumer's server runs: the near-contact
    // samples (17:01-17:02 and 18:31-18:32 UTC) are the grazing ellipses
    // whose raw form crossed the terminator.
    let config = GrahanConfig {
        include_path: true,
        path_step_minutes: 1,
        boundary_step_deg: 5,
        include_umbra_footprints: true,
        ..GrahanConfig::default()
    };
    let event = next_surya_grahan(&engine, eop.as_ref(), jd(2026, 8, 1.0), None, &config)
        .expect("surya search")
        .expect("surya event");
    assert!(!event.footprints.is_empty());
    let mut any_on_terminator = false;
    for footprint in &event.footprints {
        // Subsolar point from the derived shadow-axis elements (the axis
        // points at the Sun to within the lunar parallax, ~1 degree).
        let elements = besselian_elements_at(&engine, eop.as_ref(), footprint.jd_tdb)
            .expect("besselian elements");
        let subsolar = EclipseGeoPoint {
            latitude_deg: elements.d_deg,
            longitude_deg: {
                let lon = (-elements.mu_deg).rem_euclid(360.0);
                if lon > 180.0 { lon - 360.0 } else { lon }
            },
        };
        for vertex in &footprint.boundary {
            let distance_deg = great_circle_km(*vertex, subsolar) / 111.19;
            assert!(
                distance_deg <= 92.5,
                "footprint vertex {vertex:?} lies {distance_deg:.1} deg from the subsolar point"
            );
            if distance_deg >= 88.0 {
                any_on_terminator = true;
            }
        }
    }
    assert!(
        any_on_terminator,
        "no footprint vertex near the terminator; clipping looks inactive"
    );
    // Change 4 invariant: every timestamp-matched central path point stays
    // inside its clipped footprint.
    for point in &event.path {
        let footprint = event
            .footprints
            .iter()
            .find(|footprint| (footprint.jd_tdb - point.jd_tdb).abs() < 1.0e-9)
            .expect("timestamp-matched footprint");
        assert!(
            spherical_ring_contains(&footprint.boundary, point.center),
            "path center {:?} outside its clipped footprint",
            point.center
        );
    }

    // Change 8b: umbral outlines are terminator-clipped too. The C2/C3
    // entries are grazing ellipses that previously reached ~99-100 degrees
    // from the subsolar point; clipped, they end on the terminator.
    let mut umbra_on_terminator = false;
    for footprint in &event.umbra_footprints {
        let elements = besselian_elements_at(&engine, eop.as_ref(), footprint.jd_tdb)
            .expect("besselian elements");
        let subsolar = EclipseGeoPoint {
            latitude_deg: elements.d_deg,
            longitude_deg: {
                let lon = (-elements.mu_deg).rem_euclid(360.0);
                if lon > 180.0 { lon - 360.0 } else { lon }
            },
        };
        for vertex in &footprint.boundary {
            let distance_deg = great_circle_km(*vertex, subsolar) / 111.19;
            assert!(
                distance_deg <= 92.5,
                "umbra vertex {vertex:?} lies {distance_deg:.1} deg from the subsolar point"
            );
            if distance_deg >= 88.0 {
                umbra_on_terminator = true;
            }
        }
        // Nested inside the timestamp-matched clipped penumbral footprint.
        if let Some(penumbral) = event
            .footprints
            .iter()
            .find(|candidate| (candidate.jd_tdb - footprint.jd_tdb).abs() < 1.0e-9)
        {
            for vertex in footprint.boundary.iter().step_by(4) {
                assert!(
                    spherical_ring_contains(&penumbral.boundary, *vertex)
                        || penumbral
                            .boundary
                            .iter()
                            .map(|point| great_circle_km(*point, *vertex))
                            .fold(f64::INFINITY, f64::min)
                            < 130.0,
                    "umbra vertex {vertex:?} outside its clipped penumbral footprint"
                );
            }
        }
    }
    assert!(
        umbra_on_terminator,
        "no umbra vertex near the terminator; the C2/C3 grazing ellipses should touch it"
    );
}

/// Change 6 on the 2026-08-12 Arctic total event: contact and umbral
/// footprints with producer-decided pole containment.
#[test]
fn arctic_total_2026_contact_and_umbra_footprints() {
    let Some(engine) = load_engine() else { return };
    let eop = load_eop();
    let event = event_with_fields(&engine, eop.as_ref(), jd(2026, 8, 1.0));
    let isolines = event.isolines.as_ref().expect("isolines");
    let corridor = event.central_corridor.as_ref().expect("corridor");

    // 6b: sampled penumbral footprints carry contains_pole; the one nearest
    // greatest eclipse encloses the north pole for this Arctic event.
    let greatest_jd = event.greatest_grahan_jd;
    let nearest_sample = event
        .footprints
        .iter()
        .min_by(|a, b| {
            (a.jd_tdb - greatest_jd)
                .abs()
                .total_cmp(&(b.jd_tdb - greatest_jd).abs())
        })
        .expect("sampled footprints");
    assert_eq!(nearest_sample.contains_pole, Some(PoleSide::North));

    // 6a: all five contacts present for a central event, in order.
    let contacts: Vec<SuryaContactKind> = event
        .contact_footprints
        .iter()
        .map(|footprint| footprint.contact)
        .collect();
    assert_eq!(
        contacts,
        vec![
            SuryaContactKind::C1,
            SuryaContactKind::C2,
            SuryaContactKind::Greatest,
            SuryaContactKind::C3,
            SuryaContactKind::C4,
        ]
    );
    let greatest_footprint = event
        .contact_footprints
        .iter()
        .find(|footprint| footprint.contact == SuryaContactKind::Greatest)
        .expect("greatest contact footprint");
    // Acceptance 1: producer-decided pole containment.
    assert_eq!(greatest_footprint.contains_pole, Some(PoleSide::North));
    // Acceptance 2: contains the greatest location and stays inside the
    // visibility boundary.
    let greatest_location = event.greatest_location.expect("greatest location");
    assert!(spherical_ring_contains(
        &greatest_footprint.boundary,
        greatest_location
    ));
    for footprint in &event.contact_footprints {
        if footprint.boundary.is_empty() {
            continue; // exact-tangency convention at C1/C4
        }
        let ring = SuryaIsolineRing {
            boundary: footprint.boundary.clone(),
            contains_pole: footprint.contains_pole,
        };
        assert_closed_ring(&ring, "contact footprint");
        // The instantaneous visibility region is a subset of the
        // max-over-time visibility region by construction; the tolerance
        // only covers the polyline chords of the coarse test isolines.
        for vertex in footprint.boundary.iter().step_by(8) {
            assert!(
                inside_any_ring(&isolines.visibility_boundary, *vertex)
                    || distance_to_rings_km(&isolines.visibility_boundary, *vertex) < 150.0,
                "contact footprint vertex {vertex:?} outside visibility boundary"
            );
        }
    }
    // Acceptance 3: the greatest contact footprint agrees with the nearest
    // 5-minute sample within sampling error. The contact ring is the
    // Sun-up-clipped region, a subset of the sample's geometric
    // cone-ellipsoid region; the margin covers the shadow's travel over
    // half a sampling step plus the sample ring's own vertex spacing (the
    // distance check measures to vertices and a 5-degree boundary step
    // spaces them up to ~550 km apart on the ground).
    let sampling_error_km = 2.5 * 60.0 * 1.0 + 550.0;
    for vertex in greatest_footprint.boundary.iter().step_by(8) {
        assert!(
            spherical_ring_contains(&nearest_sample.boundary, *vertex)
                || nearest_sample
                    .boundary
                    .iter()
                    .map(|point| great_circle_km(*point, *vertex))
                    .fold(f64::INFINITY, f64::min)
                    < sampling_error_km,
            "greatest contact footprint vertex {vertex:?} departs from nearest sample"
        );
    }

    // Instantaneous magnitude rings: per-timestamp nesting
    // umbra ⊆ 0.75 ⊆ 0.5 ⊆ 0.25 ⊆ penumbral boundary.
    let level_rings = |footprint: &dhruv_search::SuryaGrahanFootprint,
                       level: f64|
     -> Vec<SuryaIsolineRing> {
        footprint
            .magnitude_rings
            .iter()
            .filter(|ring| (ring.level - level).abs() < 1.0e-9)
            .map(|ring| SuryaIsolineRing {
                boundary: ring.boundary.clone(),
                contains_pole: ring.contains_pole,
            })
            .collect()
    };
    let mut nested_checks = 0usize;
    for footprint in event.footprints.iter().step_by(10) {
        if footprint.magnitude_rings.is_empty() {
            continue;
        }
        for ring in &footprint.magnitude_rings {
            let as_ring = SuryaIsolineRing {
                boundary: ring.boundary.clone(),
                contains_pole: ring.contains_pole,
            };
            assert_closed_ring(&as_ring, "instantaneous magnitude ring");
            // Terminator-clipped magnitude region is a subset of the
            // geometric penumbral ring region; tolerance covers the
            // geometric ring's ~550 km vertex spacing at 5-degree steps.
            for vertex in ring.boundary.iter().step_by(6) {
                assert!(
                    spherical_ring_contains(&footprint.boundary, *vertex)
                        || footprint
                            .boundary
                            .iter()
                            .map(|point| great_circle_km(*point, *vertex))
                            .fold(f64::INFINITY, f64::min)
                            < 600.0,
                    "magnitude ring vertex {vertex:?} outside penumbral footprint"
                );
            }
        }
        for pair in [(0.5, 0.25), (0.75, 0.5)] {
            let inner = level_rings(footprint, pair.0);
            let outer = level_rings(footprint, pair.1);
            if inner.is_empty() || outer.is_empty() {
                continue;
            }
            for ring in &inner {
                for vertex in ring.boundary.iter().step_by(6) {
                    assert!(
                        inside_any_ring(&outer, *vertex)
                            || distance_to_rings_km(&outer, *vertex) < 60.0,
                        "level {} ring vertex {vertex:?} outside level {} region",
                        pair.0,
                        pair.1
                    );
                }
            }
        }
        // Umbra outline at the same timestamp sits inside the 0.75 region.
        let rings_075 = level_rings(footprint, 0.75);
        if let Some(umbra) = event
            .umbra_footprints
            .iter()
            .find(|umbra| (umbra.jd_tdb - footprint.jd_tdb).abs() < 1.0e-9)
            && !rings_075.is_empty()
        {
            for vertex in umbra.boundary.iter().step_by(6) {
                assert!(
                    inside_any_ring(&rings_075, *vertex)
                        || distance_to_rings_km(&rings_075, *vertex) < 60.0,
                    "umbra vertex {vertex:?} outside the 0.75 magnitude region"
                );
            }
            nested_checks += 1;
        }
    }
    assert!(nested_checks > 0, "no umbra-in-0.75 nesting checks ran");
    // Contact footprints carry magnitude rings too (greatest reaches all
    // requested levels for this total eclipse).
    assert!(
        greatest_footprint
            .magnitude_rings
            .iter()
            .any(|ring| (ring.level - 0.75).abs() < 1.0e-9),
        "greatest contact footprint missing the 0.75 magnitude ring"
    );
    // The central path passes close to the north pole mid-event, so some
    // timestamp's 0.5-level instantaneous region encloses it and the
    // producer-decided flag must say so.
    assert!(
        event.footprints.iter().any(|footprint| {
            footprint.magnitude_rings.iter().any(|ring| {
                (ring.level - 0.5).abs() < 1.0e-9
                    && ring.contains_pole == Some(PoleSide::North)
            })
        }),
        "no 0.5 magnitude ring encloses the north pole for the Arctic event"
    );

    // 6c: umbral outlines at every path timestamp plus the central contacts.
    assert!(
        event.umbra_footprints.len() >= event.path.len(),
        "expected umbra footprints at every path timestamp"
    );
    let corridor_rings: Vec<SuryaIsolineRing> = corridor
        .segments
        .iter()
        .flat_map(|segment| segment.rings.iter().cloned())
        .collect();
    for footprint in &event.umbra_footprints {
        assert_eq!(footprint.grahan_type, SuryaGrahanType::Total);
        let ring = SuryaIsolineRing {
            boundary: footprint.boundary.clone(),
            contains_pole: footprint.contains_pole,
        };
        assert_closed_ring(&ring, "umbra footprint");
    }
    // Umbral outlines sweep the corridor: sampled vertices stay inside (or
    // within refinement distance of) the swept corridor rings.
    for footprint in event.umbra_footprints.iter().step_by(5) {
        for vertex in footprint.boundary.iter().step_by(6) {
            assert!(
                inside_any_ring(&corridor_rings, *vertex)
                    || distance_to_rings_km(&corridor_rings, *vertex) < 30.0,
                "umbra footprint vertex {vertex:?} outside corridor"
            );
        }
    }
}

/// Change 6 on a partial-only event (2025-03-29): exactly C1/greatest/C4
/// contact footprints and no umbral outlines.
#[test]
fn partial_2025_contact_footprints_only() {
    let Some(engine) = load_engine() else { return };
    let eop = load_eop();
    let config = GrahanConfig {
        include_path: true,
        path_step_minutes: 5,
        boundary_step_deg: 5,
        include_contact_footprints: true,
        include_umbra_footprints: true,
        ..GrahanConfig::default()
    };
    let event = next_surya_grahan(&engine, eop.as_ref(), jd(2025, 3, 1.0), None, &config)
        .expect("surya search")
        .expect("surya event");
    assert_eq!(event.grahan_type, SuryaGrahanType::Partial);
    assert_eq!(event.centrality, SuryaCentrality::None);
    let contacts: Vec<SuryaContactKind> = event
        .contact_footprints
        .iter()
        .map(|footprint| footprint.contact)
        .collect();
    assert_eq!(
        contacts,
        vec![
            SuryaContactKind::C1,
            SuryaContactKind::Greatest,
            SuryaContactKind::C4,
        ]
    );
    assert!(event.umbra_footprints.is_empty());
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
        include_contact_footprints: true,
        include_umbra_footprints: true,
        instantaneous_magnitude_levels: vec![0.5],
        ..GrahanConfig::default()
    };
    let event = next_surya_grahan(&engine, eop.as_ref(), jd(2026, 2, 1.0), None, &config)
        .expect("surya search")
        .expect("surya event");
    assert_eq!(event.grahan_type, SuryaGrahanType::Annular);
    assert_eq!(event.centrality, SuryaCentrality::Full);
    // Acceptance 1 (southern counterpart): the greatest contact footprint
    // encloses the south pole, and antumbral outlines are annular.
    let greatest_footprint = event
        .contact_footprints
        .iter()
        .find(|footprint| footprint.contact == SuryaContactKind::Greatest)
        .expect("greatest contact footprint");
    assert_eq!(greatest_footprint.contains_pole, Some(PoleSide::South));
    assert!(!event.umbra_footprints.is_empty());
    for footprint in &event.umbra_footprints {
        assert_eq!(footprint.grahan_type, SuryaGrahanType::Annular);
    }
    // Instantaneous magnitude rings close near the pole. The 0.5 region at
    // greatest is far smaller than the visibility region and need not reach
    // the pole itself, so the flag is only constrained to south-or-absent.
    let greatest_ring = greatest_footprint
        .magnitude_rings
        .iter()
        .find(|ring| (ring.level - 0.5).abs() < 1.0e-9)
        .expect("0.5 magnitude ring at greatest");
    let as_ring = SuryaIsolineRing {
        boundary: greatest_ring.boundary.clone(),
        contains_pole: greatest_ring.contains_pole,
    };
    assert_closed_ring(&as_ring, "antarctic magnitude ring");
    assert_ne!(greatest_ring.contains_pole, Some(PoleSide::North));
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
        instantaneous_magnitude_levels: vec![0.75, 0.25, 0.25, 2.0, f64::NAN],
        ..GrahanConfig::default()
    };
    let effective = config.effective();
    assert_eq!(effective.local_grid_step_deg, 0.5);
    assert_eq!(effective.duration_isoline_fractions, vec![0.25, 0.75]);
    assert_eq!(effective.magnitude_isoline_levels, vec![0.5, 1.0]);
    assert_eq!(effective.instantaneous_magnitude_levels, vec![0.25, 0.75]);
}
