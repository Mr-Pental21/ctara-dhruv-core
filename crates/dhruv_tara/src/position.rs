//! High-level position queries: equatorial, ecliptic, sidereal.
//!
//! All public APIs accept `jd_tdb: f64` (Julian Date in TDB).

use dhruv_frames::{
    SphericalCoords, cartesian_to_spherical, icrf_to_ecliptic, nutation_iau2000b,
    precess_ecliptic_j2000_to_date,
};

use crate::apparent::{apply_aberration, apply_light_deflection};
use crate::catalog::{TaraCatalog, TaraEntry};
use crate::config::{EarthState, TaraAccuracy, TaraConfig};
use crate::error::TaraError;
use crate::galactic;
use crate::propagation::{
    AU_KM, EquatorialPosition, apply_parallax, cartesian_au_to_equatorial, propagate_cartesian_au,
    propagate_position,
};
use crate::tara_id::TaraId;

/// J2000.0 Julian Date.
const J2000_JD: f64 = 2_451_545.0;

/// Days per Julian year.
const DAYS_PER_YEAR: f64 = 365.25;

/// Compute equatorial position (ICRS RA/Dec) of a star at the given epoch.
///
/// Uses default config (Astrometric, no parallax).
pub fn position_equatorial(
    catalog: &TaraCatalog,
    id: TaraId,
    jd_tdb: f64,
) -> Result<EquatorialPosition, TaraError> {
    position_equatorial_with_config(catalog, id, jd_tdb, &TaraConfig::default(), None)
}

/// Compute equatorial position with full configuration.
pub fn position_equatorial_with_config(
    catalog: &TaraCatalog,
    id: TaraId,
    jd_tdb: f64,
    config: &TaraConfig,
    earth_state: Option<&EarthState>,
) -> Result<EquatorialPosition, TaraError> {
    validate_earth_state(config, earth_state)?;

    if id.is_galactic_reference() {
        let dir = galactic_direction(id);
        let dir = apply_apparent_corrections(config, &dir, earth_state);
        return Ok(cartesian_au_to_equatorial(&dir));
    }

    let entry = catalog
        .get(id)
        .ok_or_else(|| TaraError::StarNotFound(id.as_str().to_string()))?;

    let dt_years = (jd_tdb - epoch_to_jd(catalog.reference_epoch_jy)) / DAYS_PER_YEAR;

    if config.apply_parallax || config.accuracy == TaraAccuracy::Apparent {
        let mut pos_au = propagate_cartesian_au(
            entry.ra_deg,
            entry.dec_deg,
            entry.parallax_mas,
            entry.pm_ra_mas_yr,
            entry.pm_dec_mas_yr,
            entry.radial_velocity_km_s,
            dt_years,
        );

        if config.apply_parallax {
            let earth = earth_state.unwrap(); // validated above
            pos_au = direction_scaled(
                &apply_parallax(&pos_au, &earth.position_au),
                cartesian_au_to_equatorial(&pos_au).distance_au,
            );
        }

        if config.accuracy == TaraAccuracy::Apparent {
            let r = vec_len(&pos_au);
            let mut dir = if r > 0.0 {
                [pos_au[0] / r, pos_au[1] / r, pos_au[2] / r]
            } else {
                pos_au
            };
            dir = apply_apparent_corrections(config, &dir, earth_state);
            return Ok(cartesian_au_to_equatorial(&dir));
        }

        return Ok(cartesian_au_to_equatorial(&pos_au));
    }

    Ok(propagate_position(
        entry.ra_deg,
        entry.dec_deg,
        entry.parallax_mas,
        entry.pm_ra_mas_yr,
        entry.pm_dec_mas_yr,
        entry.radial_velocity_km_s,
        dt_years,
    ))
}

/// Compute ecliptic position (tropical, of-date) of a star.
///
/// Uses default config (Astrometric, no parallax).
pub fn position_ecliptic(
    catalog: &TaraCatalog,
    id: TaraId,
    jd_tdb: f64,
) -> Result<SphericalCoords, TaraError> {
    position_ecliptic_with_config(catalog, id, jd_tdb, &TaraConfig::default(), None)
}

/// Compute ecliptic position with full configuration.
pub fn position_ecliptic_with_config(
    catalog: &TaraCatalog,
    id: TaraId,
    jd_tdb: f64,
    config: &TaraConfig,
    earth_state: Option<&EarthState>,
) -> Result<SphericalCoords, TaraError> {
    validate_earth_state(config, earth_state)?;

    let t_centuries = (jd_tdb - J2000_JD) / 36525.0;

    let icrs_dir = if id.is_galactic_reference() {
        let dir = galactic_direction(id);
        apply_apparent_corrections(config, &dir, earth_state)
    } else {
        let entry = catalog
            .get(id)
            .ok_or_else(|| TaraError::StarNotFound(id.as_str().to_string()))?;

        icrs_direction(catalog, entry, jd_tdb, config, earth_state)
    };

    // Convert to ecliptic J2000 (in km)
    let icrs_km = [
        icrs_dir[0] * AU_KM,
        icrs_dir[1] * AU_KM,
        icrs_dir[2] * AU_KM,
    ];
    let ecl_j2000 = icrf_to_ecliptic(&icrs_km);

    // Precess to ecliptic of date
    let mut ecl_of_date = precess_ecliptic_j2000_to_date(&ecl_j2000, t_centuries);

    // For Apparent tier, apply nutation to the ecliptic longitude
    if config.accuracy == TaraAccuracy::Apparent {
        let (dpsi_arcsec, _deps_arcsec) = nutation_iau2000b(t_centuries);
        let sc = cartesian_to_spherical(&ecl_of_date);
        // dpsi is in arcseconds; convert to radians
        let dpsi_rad = dpsi_arcsec * std::f64::consts::PI / 648_000.0;
        let lon_rad = sc.lon_deg.to_radians() + dpsi_rad;
        let lat_rad = sc.lat_deg.to_radians();
        let cos_lat = lat_rad.cos();
        ecl_of_date = [
            sc.distance_km * cos_lat * lon_rad.cos(),
            sc.distance_km * cos_lat * lon_rad.sin(),
            sc.distance_km * lat_rad.sin(),
        ];
    }

    Ok(cartesian_to_spherical(&ecl_of_date))
}

/// Compute sidereal longitude of a star.
///
/// Returns the ecliptic longitude minus the given ayanamsha value.
///
/// Uses default config (Astrometric, no parallax).
///
/// # Arguments
/// * `ayanamsha_deg` — ayanamsha value in degrees (caller computes this,
///   keeping dhruv_tara independent of dhruv_vedic_base)
pub fn sidereal_longitude(
    catalog: &TaraCatalog,
    id: TaraId,
    jd_tdb: f64,
    ayanamsha_deg: f64,
) -> Result<f64, TaraError> {
    sidereal_longitude_with_config(
        catalog,
        id,
        jd_tdb,
        ayanamsha_deg,
        &TaraConfig::default(),
        None,
    )
}

/// Compute sidereal longitude with full configuration.
pub fn sidereal_longitude_with_config(
    catalog: &TaraCatalog,
    id: TaraId,
    jd_tdb: f64,
    ayanamsha_deg: f64,
    config: &TaraConfig,
    earth_state: Option<&EarthState>,
) -> Result<f64, TaraError> {
    let ecliptic = position_ecliptic_with_config(catalog, id, jd_tdb, config, earth_state)?;
    let sidereal = (ecliptic.lon_deg - ayanamsha_deg).rem_euclid(360.0);
    Ok(sidereal)
}

// ---- Internal helpers ----

fn validate_earth_state(
    config: &TaraConfig,
    earth_state: Option<&EarthState>,
) -> Result<(), TaraError> {
    if earth_state.is_none() && (config.accuracy == TaraAccuracy::Apparent || config.apply_parallax)
    {
        return Err(TaraError::EarthStateRequired);
    }
    Ok(())
}

fn galactic_direction(id: TaraId) -> [f64; 3] {
    match id {
        TaraId::GalacticCenter => galactic::galactic_center_icrs(),
        TaraId::GalacticAntiCenter => galactic::galactic_anticenter_icrs(),
        _ => [1.0, 0.0, 0.0], // unreachable for galactic refs
    }
}

/// Get ICRS unit direction vector for a catalog star, with optional corrections.
fn icrs_direction(
    catalog: &TaraCatalog,
    entry: &TaraEntry,
    jd_tdb: f64,
    config: &TaraConfig,
    earth_state: Option<&EarthState>,
) -> [f64; 3] {
    let dt_years = (jd_tdb - epoch_to_jd(catalog.reference_epoch_jy)) / DAYS_PER_YEAR;

    let pos_au = propagate_cartesian_au(
        entry.ra_deg,
        entry.dec_deg,
        entry.parallax_mas,
        entry.pm_ra_mas_yr,
        entry.pm_dec_mas_yr,
        entry.radial_velocity_km_s,
        dt_years,
    );

    let r = vec_len(&pos_au);
    let mut dir = if r > 0.0 {
        [pos_au[0] / r, pos_au[1] / r, pos_au[2] / r]
    } else {
        pos_au
    };

    if config.apply_parallax {
        if let Some(earth) = earth_state {
            dir = apply_parallax(&pos_au, &earth.position_au);
        }
    }

    if config.accuracy == TaraAccuracy::Apparent {
        dir = apply_apparent_corrections(config, &dir, earth_state);
    }

    dir
}

fn apply_apparent_corrections(
    config: &TaraConfig,
    direction: &[f64; 3],
    earth_state: Option<&EarthState>,
) -> [f64; 3] {
    if config.accuracy != TaraAccuracy::Apparent {
        return *direction;
    }

    let earth = match earth_state {
        Some(e) => e,
        None => return *direction, // shouldn't happen after validation
    };

    let mut dir = *direction;

    // 1. Annual aberration
    dir = apply_aberration(&dir, &earth.velocity_au_day);

    // 2. Gravitational light deflection by the Sun
    // Sun ≈ −earth_pos (barycenter is near Sun)
    let earth_r = vec_len(&earth.position_au);
    if earth_r > 1e-10 {
        let e_sun_to_obs = [
            earth.position_au[0] / earth_r,
            earth.position_au[1] / earth_r,
            earth.position_au[2] / earth_r,
        ];
        dir = apply_light_deflection(&dir, &e_sun_to_obs, earth_r);
    }

    dir
}

fn epoch_to_jd(epoch_jy: f64) -> f64 {
    J2000_JD + (epoch_jy - 2000.0) * DAYS_PER_YEAR
}

fn vec_len(v: &[f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn direction_scaled(unit: &[f64; 3], scale: f64) -> [f64; 3] {
    [unit[0] * scale, unit[1] * scale, unit[2] * scale]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_catalog() -> TaraCatalog {
        TaraCatalog::parse(
            r#"{
  "source": "TEST",
  "reference_epoch_jy": 2016.0,
  "reference_frame": "ICRS",
  "stars": [
    {
      "id": "Chitra",
      "bayer": "alf Vir",
      "common_name": "Spica",
      "hip_id": 65474,
      "ra_deg": 201.29825,
      "dec_deg": -11.16132,
      "parallax_mas": 13.06,
      "pm_ra_mas_yr": -42.50,
      "pm_dec_mas_yr": -31.73,
      "radial_velocity_km_s": 1.0,
      "v_mag": 0.97
    }
  ]
}"#,
        )
        .unwrap()
    }

    #[test]
    fn equatorial_basic() {
        let cat = test_catalog();
        let result = position_equatorial(&cat, TaraId::Chitra, J2000_JD).unwrap();
        // At J2000.0, Spica should be near its catalog position (propagated back 16 yr)
        assert!((result.ra_deg - 201.3).abs() < 0.1, "RA: {}", result.ra_deg);
        assert!(
            (result.dec_deg - (-11.16)).abs() < 0.1,
            "Dec: {}",
            result.dec_deg
        );
    }

    #[test]
    fn ecliptic_basic() {
        let cat = test_catalog();
        let jd = 2460311.0; // 2024-01-01 12:00 TDB
        let result = position_ecliptic(&cat, TaraId::Chitra, jd).unwrap();
        // Spica ecliptic longitude should be around 203-204° (tropical)
        assert!(
            (result.lon_deg - 204.0).abs() < 1.0,
            "lon: {}°",
            result.lon_deg
        );
    }

    #[test]
    fn sidereal_basic() {
        let cat = test_catalog();
        let jd = 2460311.0;
        // Lahiri ayanamsha at 2024 ≈ 24.17°
        let ayan = 24.17;
        let result = sidereal_longitude(&cat, TaraId::Chitra, jd, ayan).unwrap();
        // Spica sidereal should be near 180° (Chitra = beginning of Libra = 180°)
        assert!(
            (result - 180.0).abs() < 1.0,
            "sidereal: {}° (expected ~180°)",
            result
        );
    }

    #[test]
    fn star_not_found() {
        let cat = test_catalog();
        let result = position_equatorial(&cat, TaraId::Polaris, J2000_JD);
        assert!(matches!(result, Err(TaraError::StarNotFound(_))));
    }

    #[test]
    fn galactic_center_ecliptic() {
        let cat = test_catalog(); // needed for API but GC doesn't use it
        let jd = 2460311.0;
        let result = position_ecliptic(&cat, TaraId::GalacticCenter, jd).unwrap();
        // GC ecliptic longitude at 2024 ≈ 266-267°
        assert!(
            (result.lon_deg - 267.0).abs() < 1.0,
            "GC lon: {}°",
            result.lon_deg
        );
    }

    #[test]
    fn earth_state_required_for_apparent() {
        let cat = test_catalog();
        let config = TaraConfig {
            accuracy: TaraAccuracy::Apparent,
            apply_parallax: false,
        };
        let result = position_equatorial_with_config(&cat, TaraId::Chitra, J2000_JD, &config, None);
        assert!(matches!(result, Err(TaraError::EarthStateRequired)));
    }

    #[test]
    fn earth_state_required_for_parallax() {
        let cat = test_catalog();
        let config = TaraConfig {
            accuracy: TaraAccuracy::Astrometric,
            apply_parallax: true,
        };
        let result = position_equatorial_with_config(&cat, TaraId::Chitra, J2000_JD, &config, None);
        assert!(matches!(result, Err(TaraError::EarthStateRequired)));
    }

    #[test]
    fn epoch_to_jd_j2016() {
        let jd = epoch_to_jd(2016.0);
        // J2016.0 = JD 2457389.0 (approx)
        assert!((jd - 2457389.0).abs() < 1.0, "J2016.0 JD: {jd}");
    }
}
