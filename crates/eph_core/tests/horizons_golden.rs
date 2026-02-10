//! Golden validation tests comparing engine output against JPL Horizons reference vectors.
//!
//! Reference data: Horizons API v1.2, fetched 2026-02-10.
//! Horizons uses DE441; our engine uses DE442s.
//! All vectors are geometric (no aberration/light-time), ICRF frame, km and km/s.
//!
//! Tolerance policy:
//! - Position: 1.0 km (accounts for DE441 vs DE442s cross-kernel differences)
//! - Velocity: 1.0e-5 km/s (proportional to position tolerance / orbital period)
//!
//! For Mars/Jupiter/Saturn, our engine returns the system barycenter position
//! (code 4/5/6) since DE442s does not include planet-body-center-to-barycenter
//! segments for these. Horizons references use matching barycenters.

// Keep exact Horizons output values verbatim for traceability.
#![allow(clippy::excessive_precision)]

use std::path::PathBuf;

use eph_core::{Body, Engine, EngineConfig, Frame, Observer, Query, StateVector};

// ---------------------------------------------------------------------------
// Tolerances (per-body tier)
// ---------------------------------------------------------------------------

/// Inner-planet/Moon/Sun position tolerance in km.
/// Bodies with direct body-center segments in DE442s (199, 299, 399, 301, 10).
/// DE441 vs DE442s differences are sub-km for these.
const POS_TOL_INNER_KM: f64 = 1.0;

/// Outer-planet barycenter position tolerance in km.
/// Bodies where DE442s has only barycenter segments (4, 5, 6, 7, 8, 9).
/// Cross-kernel differences are larger for outer barycenters (~3 km for Jupiter).
/// 5 km at Jupiter's ~750M km distance is still <10 ppb relative accuracy.
const POS_TOL_BARY_KM: f64 = 5.0;

/// Velocity tolerance in km/s (uniform across all bodies).
const VEL_TOL_KM_S: f64 = 1.0e-5;

// ---------------------------------------------------------------------------
// Test infrastructure
// ---------------------------------------------------------------------------

fn kernel_paths() -> (PathBuf, PathBuf) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../kernels/data");
    (base.join("de442s.bsp"), base.join("naif0012.tls"))
}

fn engine() -> Option<Engine> {
    let (spk, lsk) = kernel_paths();
    if !spk.exists() || !lsk.exists() {
        eprintln!("Skipping golden tests: kernel files not found");
        return None;
    }
    Some(
        Engine::new(EngineConfig::with_single_spk(spk, lsk, 256, true))
            .expect("engine should load"),
    )
}

/// Assert position and velocity match within tolerance, reporting per-component errors.
fn assert_state_within_tolerance(
    label: &str,
    actual: &StateVector,
    expected_pos: [f64; 3],
    expected_vel: [f64; 3],
    pos_tol: f64,
    vel_tol: f64,
) {
    for (i, (&act, &exp)) in actual
        .position_km
        .iter()
        .zip(expected_pos.iter())
        .enumerate()
    {
        let dp = (act - exp).abs();
        assert!(
            dp <= pos_tol,
            "{label}: position[{i}] error {dp:.6} km exceeds tolerance {pos_tol} km\n  \
             actual:   {act:.16e}\n  expected: {exp:.16e}",
        );
    }
    for (i, (&act, &exp)) in actual
        .velocity_km_s
        .iter()
        .zip(expected_vel.iter())
        .enumerate()
    {
        let dv = (act - exp).abs();
        assert!(
            dv <= vel_tol,
            "{label}: velocity[{i}] error {dv:.6e} km/s exceeds tolerance {vel_tol} km/s\n  \
             actual:   {act:.16e}\n  expected: {exp:.16e}",
        );
    }
}

// ===========================================================================
// Epoch: J2000.0 (JD 2451545.0 TDB)
// ===========================================================================

#[test]
fn golden_earth_ssb_j2000() {
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Earth,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_451_545.0,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Earth wrt SSB @ J2000",
        &state,
        [-2.756674048281145e+07, 1.323613811535491e+08, 5.741865328625385e+07],
        [-2.978494749851088e+01, -5.029753814928081e+00, -2.180645069035755e+00],
        POS_TOL_INNER_KM,
        VEL_TOL_KM_S,
    );
}

#[test]
fn golden_sun_ssb_j2000() {
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Sun,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_451_545.0,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Sun wrt SSB @ J2000",
        &state,
        [-1.067706805380953e+06, -3.960361847959462e+05, -1.380651842868809e+05],
        [9.312571926520472e-03, -1.170150612817771e-02, -5.251266205200356e-03],
        POS_TOL_INNER_KM,
        VEL_TOL_KM_S,
    );
}

#[test]
fn golden_mercury_ssb_j2000() {
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Mercury,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_451_545.0,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Mercury wrt SSB @ J2000",
        &state,
        [-2.052943316123468e+07, -6.032400395827633e+07, -3.013083786411830e+07],
        [3.700430442920571e+01, -8.541376789510446e+00, -8.398372409672424e+00],
        POS_TOL_INNER_KM,
        VEL_TOL_KM_S,
    );
}

#[test]
fn golden_venus_ssb_j2000() {
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Venus,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_451_545.0,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Venus wrt SSB @ J2000",
        &state,
        [-1.085242008575715e+08, -7.318564959678600e+06, 3.548121861333776e+06],
        [1.391218601189967e+00, -3.202951993781091e+01, -1.449708673947320e+01],
        POS_TOL_INNER_KM,
        VEL_TOL_KM_S,
    );
}

#[test]
fn golden_moon_earth_j2000() {
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Moon,
            observer: Observer::Body(Body::Earth),
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_451_545.0,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Moon wrt Earth @ J2000",
        &state,
        [-2.916083841877129e+05, -2.667168338540655e+05, -7.610248730658794e+04],
        [6.435313889889519e-01, -6.660876829565195e-01, -3.013257046610932e-01],
        POS_TOL_INNER_KM,
        VEL_TOL_KM_S,
    );
}

#[test]
fn golden_mars_bary_ssb_j2000() {
    // Our engine returns Mars barycenter (code 4) for Body::Mars since DE442s
    // has no 499-wrt-4 segment. Compare against Horizons barycenter reference.
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Mars,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_451_545.0,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Mars (bary) wrt SSB @ J2000",
        &state,
        [2.069804338364514e+08, -1.864170114371795e+05, -5.667227497526179e+06],
        [1.171985008531777e+00, 2.390670819417074e+01, 1.093392063330765e+01],
        POS_TOL_BARY_KM,
        VEL_TOL_KM_S,
    );
}

#[test]
fn golden_jupiter_bary_ssb_j2000() {
    // Our engine returns Jupiter barycenter (code 5) for Body::Jupiter.
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Jupiter,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_451_545.0,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Jupiter (bary) wrt SSB @ J2000",
        &state,
        [5.974998767925479e+08, 4.089903139317586e+08, 1.607562819387201e+08],
        [-7.900525116640771e+00, 1.017179630923791e+01, 4.552467787262923e+00],
        POS_TOL_BARY_KM,
        VEL_TOL_KM_S,
    );
}

#[test]
fn golden_saturn_bary_ssb_j2000() {
    // Our engine returns Saturn barycenter (code 6) for Body::Saturn.
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Saturn,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_451_545.0,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Saturn (bary) wrt SSB @ J2000",
        &state,
        [9.573174174143425e+08, 9.233196218969914e+08, 3.401628003886153e+08],
        [-7.422709426014511e+00, 6.097474815228996e+00, 2.837682288255575e+00],
        POS_TOL_BARY_KM,
        VEL_TOL_KM_S,
    );
}

// ===========================================================================
// Epoch: JD 2460000.5 TDB (2023-Feb-25)
// ===========================================================================

#[test]
fn golden_earth_ssb_2460000() {
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Earth,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_460_000.5,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Earth wrt SSB @ 2460000.5",
        &state,
        [-1.363822333172446e+08, 5.563795761699516e+07, 2.415399443522515e+07],
        [-1.268994499649647e+01, -2.505446050905924e+01, -1.086195937816694e+01],
        POS_TOL_INNER_KM,
        VEL_TOL_KM_S,
    );
}

#[test]
fn golden_moon_earth_2460000() {
    let engine = match engine() {
        Some(e) => e,
        None => return,
    };
    let state = engine
        .query(Query {
            target: Body::Moon,
            observer: Observer::Body(Body::Earth),
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: 2_460_000.5,
        })
        .expect("query should succeed");

    assert_state_within_tolerance(
        "Moon wrt Earth @ 2460000.5",
        &state,
        [2.996603954771215e+05, 2.166072841835021e+05, 9.569516645455429e+04],
        [-5.758810685692292e-01, 7.395419176970628e-01, 4.199150072161020e-01],
        POS_TOL_INNER_KM,
        VEL_TOL_KM_S,
    );
}
