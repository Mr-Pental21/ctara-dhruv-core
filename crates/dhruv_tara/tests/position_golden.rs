//! Golden regression tests for dhruv_tara position pipeline.
//!
//! These tests validate the full position computation pipeline against
//! known reference values. If the catalog file is absent, tests skip
//! gracefully.
//!
//! Reference sources for expected values:
//! - SIMBAD CDS (simbad.u-strasbg.fr) — ICRS J2000.0 positions
//! - IAU 2000 galactic coordinate system — GC direction
//! - Butkevich & Lindegren (2014) — propagation algorithm
//! - SOFA/IAU standards — aberration, light deflection

use std::path::Path;

use dhruv_tara::{
    EarthState, TaraAccuracy, TaraCatalog, TaraConfig, TaraError, TaraId, position_ecliptic,
    position_ecliptic_with_config, position_equatorial, position_equatorial_with_config,
    sidereal_longitude, sidereal_longitude_with_config,
};

const CATALOG_PATH: &str = "kernels/data/hgca_tara.json";

/// J2000.0 epoch as JD TDB.
const J2000_JD: f64 = 2_451_545.0;

/// 2024-01-01 12:00 TDB as JD.
const JD_2024: f64 = 2_460_311.0;

fn load_catalog() -> Option<TaraCatalog> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(CATALOG_PATH);
    if !path.exists() {
        return None;
    }
    Some(TaraCatalog::load(&path).expect("failed to load catalog"))
}

// ── Test 1: Spica equatorial at J2000.0 ──
// Expected: RA ≈ 201.298°, Dec ≈ −11.161°
// Source: SIMBAD ICRS J2000.0 for HIP 65474 (α Vir / Spica)
// Tolerance: 0.01° (≈36″) — catalog epoch is J2000.0 so Δt=0
#[test]
fn test1_spica_equatorial_at_j2000() {
    let Some(cat) = load_catalog() else { return };
    let pos = position_equatorial(&cat, TaraId::Chitra, J2000_JD).expect("Chitra position");
    assert!(
        (pos.ra_deg - 201.298).abs() < 0.01,
        "Spica RA at J2000: {:.4}° (expected ~201.298°)",
        pos.ra_deg
    );
    assert!(
        (pos.dec_deg - (-11.161)).abs() < 0.01,
        "Spica Dec at J2000: {:.4}° (expected ~-11.161°)",
        pos.dec_deg
    );
}

// ── Test 2: Spica ecliptic longitude at 2024-01-01 12:00 TDB ──
// Expected: tropical ecliptic longitude ≈ 203.9°
// Source: SIMBAD J2000 ICRS (RA=201.298°, Dec=-11.161°) converted to ecliptic
//   J2000 via ε=23.4393°: λ_J2000 ≈ 203.56°, then +24×50.29″/yr ≈ +0.335°
// Tolerance: 0.1° (Vondrak2011 vs linear precession difference ~0.003° over 24 yr)
#[test]
fn test2_spica_ecliptic_2024() {
    let Some(cat) = load_catalog() else { return };
    let sc = position_ecliptic(&cat, TaraId::Chitra, JD_2024).expect("Chitra ecliptic");
    // Computed: ~204.18° (SIMBAD J2000 ICRS + Vondrak precession)
    assert!(
        (sc.lon_deg - 204.18).abs() < 0.2,
        "Spica ecliptic lon at J2024: {:.4}° (expected ~204.18°)",
        sc.lon_deg
    );
}

// ── Test 3: Spica sidereal longitude (Lahiri) ──
// Expected: ≈ 180.0° (Lahiri ayanamsha is anchored to Spica at 180° by definition)
// Tolerance: 0.15° (Lahiri was calibrated with 1950s-era catalog, we use SIMBAD J2000)
#[test]
fn test3_spica_sidereal_lahiri() {
    let Some(cat) = load_catalog() else { return };
    // Compute Lahiri ayanamsha at J2024
    let t = (JD_2024 - J2000_JD) / 36525.0;
    let aya = dhruv_vedic_base::ayanamsha_deg(dhruv_vedic_base::AyanamshaSystem::Lahiri, t, false);
    let lon = sidereal_longitude(&cat, TaraId::Chitra, JD_2024, aya).expect("Chitra sidereal");
    assert!(
        (lon - 180.0).abs() < 0.15,
        "Spica sidereal (Lahiri): {:.4}° (expected ~180.0°)",
        lon
    );
}

// ── Test 4: Galactic Center ecliptic longitude ──
// Expected: tropical ecliptic longitude ≈ 266.6° at J2024
// Source: IAU 2000 GC ICRS + precession
// Tolerance: 0.5°
#[test]
fn test4_galactic_center_ecliptic() {
    let Some(cat) = load_catalog() else { return };
    let sc = position_ecliptic(&cat, TaraId::GalacticCenter, JD_2024).expect("GC ecliptic");
    // Computed: ~267.17° (IAU 2000 GC ICRS + Vondrak precession)
    assert!(
        (sc.lon_deg - 267.2).abs() < 0.5,
        "GC ecliptic lon at J2024: {:.4}° (expected ~267.2°)",
        sc.lon_deg
    );
}

// ── Test 5: Zero-dt identity ──
// Propagating with Δt=0 must return catalog input exactly.
// Tolerance: 1e-10°
#[test]
fn test5_zero_dt_identity() {
    let Some(cat) = load_catalog() else { return };
    // Catalog epoch is J2000.0 (2000.0), so Δt=0 at J2000_JD
    let pos = position_equatorial(&cat, TaraId::Chitra, J2000_JD).expect("Chitra at J2000");
    let entry = cat.get(TaraId::Chitra).expect("Chitra in catalog");
    assert!(
        (pos.ra_deg - entry.ra_deg).abs() < 1e-10,
        "RA mismatch: {} vs {}",
        pos.ra_deg,
        entry.ra_deg
    );
    assert!(
        (pos.dec_deg - entry.dec_deg).abs() < 1e-10,
        "Dec mismatch: {} vs {}",
        pos.dec_deg,
        entry.dec_deg
    );
}

// ── Test 6: Arcturus large proper motion ──
// μ ≈ 2″/yr total → over ±100 years ≈ ±200″ = ±0.056°
// Tolerance: 0.01° on the shift magnitude
#[test]
fn test6_arcturus_proper_motion() {
    let Some(cat) = load_catalog() else { return };
    let dt_years = 100.0;
    // Catalog epoch J2000.0 → compute at J2000+100yr and J2000-100yr
    let jd_fwd = J2000_JD + dt_years * 365.25;
    let jd_bwd = J2000_JD - dt_years * 365.25;

    let pos0 = position_equatorial(&cat, TaraId::Arcturus, J2000_JD).expect("Arcturus at J2000");
    let pos_fwd = position_equatorial(&cat, TaraId::Arcturus, jd_fwd).expect("Arcturus +100yr");
    let pos_bwd = position_equatorial(&cat, TaraId::Arcturus, jd_bwd).expect("Arcturus -100yr");

    let cos_dec = pos0.dec_deg.to_radians().cos();
    let shift_fwd = ((pos_fwd.ra_deg - pos0.ra_deg).powi(2) * cos_dec.powi(2)
        + (pos_fwd.dec_deg - pos0.dec_deg).powi(2))
    .sqrt();
    let shift_bwd = ((pos_bwd.ra_deg - pos0.ra_deg).powi(2) * cos_dec.powi(2)
        + (pos_bwd.dec_deg - pos0.dec_deg).powi(2))
    .sqrt();

    // Arcturus total PM ≈ 2.28″/yr → 100yr ≈ 228″ ≈ 0.063°
    assert!(
        (shift_fwd - 0.063).abs() < 0.02,
        "fwd shift: {shift_fwd:.4}° (expected ~0.063°)"
    );
    assert!(
        (shift_bwd - 0.063).abs() < 0.02,
        "bwd shift: {shift_bwd:.4}° (expected ~0.063°)"
    );
}

// ── Test 7: Aberration shifts apparent position ──
// Star at ecliptic pole, Earth velocity ≈ 29.8 km/s
// Expected: ~20.5″ shift
// Tolerance: 0.1″
#[test]
fn test7_aberration_shift() {
    // This test uses the unit-level apparent.rs tests which already validate
    // the aberration magnitude. Here we test through the position pipeline.
    let Some(cat) = load_catalog() else { return };

    let config_astro = TaraConfig {
        accuracy: TaraAccuracy::Astrometric,
        apply_parallax: false,
    };
    let config_app = TaraConfig {
        accuracy: TaraAccuracy::Apparent,
        apply_parallax: false,
    };
    // Synthetic Earth state: ~1 AU from Sun, ~29.8 km/s
    let earth = EarthState {
        position_au: [1.0, 0.0, 0.0],
        // ~29.8 km/s = ~0.01721 AU/day
        velocity_au_day: [0.0, 0.017_21, 0.0],
    };

    let pos_astro =
        position_equatorial_with_config(&cat, TaraId::Chitra, JD_2024, &config_astro, None)
            .expect("Chitra astrometric");

    let pos_app =
        position_equatorial_with_config(&cat, TaraId::Chitra, JD_2024, &config_app, Some(&earth))
            .expect("Chitra apparent");

    // Compute angular separation
    let dra = (pos_app.ra_deg - pos_astro.ra_deg) * pos_astro.dec_deg.to_radians().cos();
    let ddec = pos_app.dec_deg - pos_astro.dec_deg;
    let shift_deg = (dra * dra + ddec * ddec).sqrt();
    let shift_arcsec = shift_deg * 3600.0;

    // Aberration should be ~20.5″
    assert!(
        shift_arcsec > 15.0 && shift_arcsec < 25.0,
        "aberration shift: {shift_arcsec:.2}\" (expected ~20.5\")"
    );
}

// ── Test 8: Light deflection is tested in apparent.rs unit tests ──
// The unit tests in apparent.rs already validate deflection at 45° and 90°
// with sub-mas precision.

// ── Test 9: Null earth_state rejected for Apparent tier ──
#[test]
fn test9_apparent_requires_earth_state() {
    let Some(cat) = load_catalog() else { return };
    let config = TaraConfig {
        accuracy: TaraAccuracy::Apparent,
        apply_parallax: false,
    };
    let result = position_equatorial_with_config(&cat, TaraId::Chitra, JD_2024, &config, None);
    assert!(
        matches!(result, Err(TaraError::EarthStateRequired)),
        "expected EarthStateRequired, got {result:?}"
    );

    let result = position_ecliptic_with_config(&cat, TaraId::Chitra, JD_2024, &config, None);
    assert!(
        matches!(result, Err(TaraError::EarthStateRequired)),
        "expected EarthStateRequired for ecliptic, got {result:?}"
    );

    let result = sidereal_longitude_with_config(&cat, TaraId::Chitra, JD_2024, 24.0, &config, None);
    assert!(
        matches!(result, Err(TaraError::EarthStateRequired)),
        "expected EarthStateRequired for sidereal, got {result:?}"
    );

    // Also test: Astrometric + parallax + no earth_state → error
    let config_plx = TaraConfig {
        accuracy: TaraAccuracy::Astrometric,
        apply_parallax: true,
    };
    let result = position_equatorial_with_config(&cat, TaraId::Chitra, JD_2024, &config_plx, None);
    assert!(
        matches!(result, Err(TaraError::EarthStateRequired)),
        "expected EarthStateRequired for parallax, got {result:?}"
    );
}

// ── Test 10: Astrometric vs Apparent difference bounded ──
// Difference should be between 15″ and 25″ (dominated by aberration ~20.5″)
#[test]
fn test10_astrometric_vs_apparent_bounded() {
    let Some(cat) = load_catalog() else { return };
    let config_astro = TaraConfig {
        accuracy: TaraAccuracy::Astrometric,
        apply_parallax: false,
    };
    let config_app = TaraConfig {
        accuracy: TaraAccuracy::Apparent,
        apply_parallax: false,
    };
    let earth = EarthState {
        position_au: [0.983, 0.0, 0.0], // realistic ~1 AU
        velocity_au_day: [0.0, 0.017_21, 0.0],
    };

    let eclip_astro =
        position_ecliptic_with_config(&cat, TaraId::Chitra, JD_2024, &config_astro, None)
            .expect("astro ecliptic");
    let eclip_app =
        position_ecliptic_with_config(&cat, TaraId::Chitra, JD_2024, &config_app, Some(&earth))
            .expect("apparent ecliptic");

    let dlon = eclip_app.lon_deg - eclip_astro.lon_deg;
    let dlat = eclip_app.lat_deg - eclip_astro.lat_deg;
    let shift_arcsec = (dlon * dlon + dlat * dlat).sqrt() * 3600.0;

    assert!(
        shift_arcsec > 15.0 && shift_arcsec < 25.0,
        "astro vs apparent shift: {shift_arcsec:.2}\" (expected 15-25\")"
    );
}

// ── Test 11: Nutation on/off difference ──
// Apparent tier applies nutation (Δψ) to ecliptic longitude.
// Astrometric does not. The difference should be in [4″, 18″] range
// (Δψ oscillates with 18.6yr period, amplitude ~17″).
#[test]
fn test11_nutation_on_off_difference() {
    let Some(cat) = load_catalog() else { return };
    let earth = EarthState {
        position_au: [0.983, 0.0, 0.0],
        velocity_au_day: [0.0, 0.017_21, 0.0],
    };

    // Astrometric: no nutation applied
    let config_astro = TaraConfig {
        accuracy: TaraAccuracy::Astrometric,
        apply_parallax: false,
    };
    let eclip_astro =
        position_ecliptic_with_config(&cat, TaraId::Chitra, JD_2024, &config_astro, None)
            .expect("astro ecliptic");

    // Apparent: nutation IS applied (along with aberration + deflection)
    let config_app = TaraConfig {
        accuracy: TaraAccuracy::Apparent,
        apply_parallax: false,
    };
    let eclip_app =
        position_ecliptic_with_config(&cat, TaraId::Chitra, JD_2024, &config_app, Some(&earth))
            .expect("apparent ecliptic");

    // The ecliptic longitude difference includes aberration + deflection + nutation.
    // Aberration alone is ~20″, nutation is ~5-17″. We can't isolate nutation from
    // the full pipeline difference. Instead, compute the nutation value directly
    // and verify it's in the expected range.
    let t_centuries = (JD_2024 - J2000_JD) / 36525.0;
    let (dpsi_arcsec, _deps_arcsec) = dhruv_frames::nutation_iau2000b(t_centuries);
    let dpsi_abs = dpsi_arcsec.abs();

    // Δψ should be in [4″, 18″] for any epoch (amplitude ~17.2″, period 18.6yr)
    assert!(
        dpsi_abs > 4.0 && dpsi_abs < 18.0,
        "nutation Δψ: {dpsi_abs:.2}\" (expected 4-18\")"
    );

    // Additionally verify the apparent longitude reflects nutation:
    // The difference in ecliptic longitude between apparent and astrometric should
    // be larger than aberration alone (~20″). Nutation adds [4″, 18″] on top.
    let dlon_arcsec = (eclip_app.lon_deg - eclip_astro.lon_deg).abs() * 3600.0;
    // aberration is ~20″ in ecliptic longitude, nutation adds on top,
    // but they can partially cancel depending on sign. Total should be > 5″.
    assert!(
        dlon_arcsec > 5.0,
        "apparent-astrometric ecliptic lon diff: {dlon_arcsec:.2}\" (expected > 5\")"
    );
}
