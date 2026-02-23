//! Full multi-epoch comparison: tropical, ayanamsha, sidereal for all precession models.
//! Range: 1000–3000 CE, 100-year steps.
//! Uses DE441 (two-part) for full time coverage.

use std::path::{Path, PathBuf};

use dhruv_core::{Body, Engine, EngineConfig};
use dhruv_frames::PrecessionModel;
use dhruv_search::conjunction::body_ecliptic_lon_lat_with_model;

const SPK1_PATH: &str = "../../kernels/data/de441_part-1.bsp";
const SPK2_PATH: &str = "../../kernels/data/de441_part-2.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK1_PATH).exists()
        || !Path::new(SPK2_PATH).exists()
        || !Path::new(LSK_PATH).exists()
    {
        eprintln!("Skipping: DE441 kernel files not found");
        return None;
    }
    let config = EngineConfig {
        spk_paths: vec![PathBuf::from(SPK1_PATH), PathBuf::from(SPK2_PATH)],
        lsk_path: PathBuf::from(LSK_PATH),
        cache_capacity: 1024,
        strict_validation: false,
    };
    Engine::new(config).ok()
}

/// Lieske / IAU 1976 accumulated general precession in ecliptic longitude.
fn lieske_p_a(t: f64) -> f64 {
    dhruv_frames::general_precession_longitude_arcsec_with_model(t, PrecessionModel::Lieske1977)
}

/// IAU 2006 (Capitaine 2003) accumulated general precession.
fn iau2006_p_a(t: f64) -> f64 {
    dhruv_frames::general_precession_longitude_arcsec_with_model(t, PrecessionModel::Iau2006)
}

/// Vondrák 2011 accumulated general precession (via dhruv_frames).
fn vondrak_p_a(t: f64) -> f64 {
    dhruv_frames::general_precession_longitude_arcsec_with_model(t, PrecessionModel::Vondrak2011)
}

/// Self-consistent Lahiri ayanamsha: back-compute J2000 ref from 1956 anchor
/// using the given model, then forward to epoch T with same model.
fn lahiri_e2e(t: f64, p_a_fn: fn(f64) -> f64) -> f64 {
    let anchor_deg = 23.0 + 15.0 / 60.0 + 0.658 / 3600.0;
    let t_1956 = (2_435_553.5 - 2_451_545.0) / 36525.0;
    let ref_j2000 = anchor_deg - p_a_fn(t_1956) / 3600.0;
    ref_j2000 + p_a_fn(t) / 3600.0
}

/// 3D-consistent Lahiri ayanamsha on the ecliptic-of-date.
///
/// Instead of scalar `ref + p_A(t)`, this tracks the sidereal zero point
/// as a 3D vector through the full precession matrix:
/// 1. At the 1956 anchor, the sidereal zero sits at ecliptic-of-date
///    longitude = 23°15'00.658".
/// 2. Precess that direction back to J2000 ecliptic via P⁻¹(t_1956).
/// 3. At epoch t, precess forward via P(t) and read off the longitude.
///
/// This correctly accounts for the tilting ecliptic (π_A) that the scalar
/// p_A formula ignores.
fn lahiri_e2e_3d(t: f64, model: PrecessionModel) -> f64 {
    let anchor_deg: f64 = 23.0 + 15.0 / 60.0 + 0.658 / 3600.0;
    let t_1956 = (2_435_553.5 - 2_451_545.0) / 36525.0;

    // Sidereal zero point at 1956: ecliptic-of-date longitude = anchor_deg
    let anchor_rad = anchor_deg.to_radians();
    let v_1956 = [anchor_rad.cos(), anchor_rad.sin(), 0.0];

    // Back to J2000 ecliptic
    let v_j2000 =
        dhruv_frames::precess_ecliptic_date_to_j2000_with_model(&v_1956, t_1956, model);

    // Forward to ecliptic-of-date at epoch t
    let v_date = dhruv_frames::precess_ecliptic_j2000_to_date_with_model(&v_j2000, t, model);

    // Ayanamsha = longitude of sidereal zero on ecliptic-of-date
    v_date[1].atan2(v_date[0]).to_degrees().rem_euclid(360.0)
}

fn dms(deg: f64) -> String {
    let neg = deg < 0.0;
    let d_abs = deg.abs();
    let d = d_abs.floor() as u32;
    let m_f = (d_abs - d as f64) * 60.0;
    let m = m_f.floor() as u32;
    let s = (m_f - m as f64) * 60.0;
    if neg {
        format!("-{d:>3}°{m:02}'{s:05.2}\"")
    } else {
        format!("{d:>3}°{m:02}'{s:05.2}\"")
    }
}

/// Try to get tropical Sun longitude; return None if ephemeris doesn't cover epoch.
fn try_tropical(engine: &Engine, jd: f64, model: PrecessionModel) -> Option<f64> {
    body_ecliptic_lon_lat_with_model(engine, Body::Sun, jd, model)
        .ok()
        .map(|(lon, _)| lon)
}

#[test]
fn full_multi_epoch_comparison() {
    let engine = match load_engine() {
        Some(e) => e,
        None => return,
    };

    let years: Vec<i32> = (1000..=3000).step_by(100).collect();

    // ═══════════════════════════════════════════════════════════════════
    // TABLE 1: GENERAL PRECESSION p_A (arcseconds from J2000)
    // ═══════════════════════════════════════════════════════════════════
    println!("\n{}", "=".repeat(100));
    println!("TABLE 1: GENERAL PRECESSION p_A (arcseconds from J2000)");
    println!("{}", "=".repeat(100));
    println!(
        "{:>5} {:>7} {:>14} {:>14} {:>14} | {:>10} {:>10}",
        "Year", "T(cy)", "Lieske", "IAU2006", "Vondrák", "Δ(L−V)\"", "Δ(I−V)\""
    );
    println!("{}", "-".repeat(100));
    for &y in &years {
        let jd = dhruv_time::calendar_to_jd(y, 1, 1.0);
        let t = (jd - 2_451_545.0) / 36525.0;
        let l = lieske_p_a(t);
        let i = iau2006_p_a(t);
        let v = vondrak_p_a(t);
        println!(
            "{y:>5} {t:>7.3} {l:>14.3} {i:>14.3} {v:>14.3} | {:>+10.4} {:>+10.4}",
            l - v, i - v
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // TABLE 2: LAHIRI AYANAMSHA (degrees) — self-consistent end-to-end
    // ═══════════════════════════════════════════════════════════════════
    println!("\n{}", "=".repeat(105));
    println!("TABLE 2: LAHIRI AYANAMSHA (degrees) — self-consistent end-to-end, same model for ref + runtime");
    println!("{}", "=".repeat(105));
    println!(
        "{:>5} {:>7} {:>14} {:>14} {:>14} | {:>10} {:>10}",
        "Year", "T(cy)", "Lieske", "IAU2006", "Vondrák", "Δ(L−V)\"", "Δ(I−V)\""
    );
    println!("{}", "-".repeat(105));
    for &y in &years {
        let jd = dhruv_time::calendar_to_jd(y, 1, 1.0);
        let t = (jd - 2_451_545.0) / 36525.0;
        let l = lahiri_e2e(t, lieske_p_a);
        let i = lahiri_e2e(t, iau2006_p_a);
        let v = lahiri_e2e(t, vondrak_p_a);
        println!(
            "{y:>5} {t:>7.3} {l:>14.8} {i:>14.8} {v:>14.8} | {:>+10.4} {:>+10.4}",
            (l - v) * 3600.0, (i - v) * 3600.0
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // TABLE 3: TROPICAL SUN LONGITUDE (degrees) — 3D precession matrix
    // Using DE441 for full range coverage.
    // ═══════════════════════════════════════════════════════════════════
    println!("\n{}", "=".repeat(130));
    println!("TABLE 3: TROPICAL SUN LONGITUDE (degrees) — 3D precession matrix, Jan 1 0h TDB");
    println!("(Blank = out of DE441 range)");
    println!("{}", "=".repeat(130));
    println!(
        "{:>5} {:>7} {:>16} {:>16} {:>16} {:>16} | {:>10} {:>10}",
        "Year", "T(cy)", "Trop(Lieske)", "Trop(IAU2006)", "Trop(Vondrák)", "DMS(Vondrák)", "Δ(L−V)\"", "Δ(I−V)\""
    );
    println!("{}", "-".repeat(130));
    for &y in &years {
        let jd = dhruv_time::calendar_to_jd(y, 1, 1.0);
        let t = (jd - 2_451_545.0) / 36525.0;
        let tl = try_tropical(&engine, jd, PrecessionModel::Lieske1977);
        let ti = try_tropical(&engine, jd, PrecessionModel::Iau2006);
        let tv = try_tropical(&engine, jd, PrecessionModel::Vondrak2011);
        match (tl, ti, tv) {
            (Some(lon_l), Some(lon_i), Some(lon_v)) => {
                println!(
                    "{y:>5} {t:>7.3} {lon_l:>16.8} {lon_i:>16.8} {lon_v:>16.8} {:>16} | {:>+10.6} {:>+10.6}",
                    dms(lon_v), (lon_l - lon_v) * 3600.0, (lon_i - lon_v) * 3600.0
                );
            }
            _ => {
                println!("{y:>5} {t:>7.3} {:>16} {:>16} {:>16} {:>16} |", "", "", "", "");
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // TABLE 4: SIDEREAL SUN LONGITUDE (Lahiri, degrees)
    // Tropical (model 3D) − Ayanamsha (model 3D)
    // Both measured on ecliptic-of-date for full consistency.
    // ═══════════════════════════════════════════════════════════════════
    println!("\n{}", "=".repeat(130));
    println!("TABLE 4: SIDEREAL SUN (Lahiri, degrees) = Tropical(model 3D) − Ayanamsha(model 3D)");
    println!("(Both tropical and ayanamsha on ecliptic-of-date. Blank = out of DE441 range)");
    println!("{}", "=".repeat(130));
    println!(
        "{:>5} {:>7} {:>14} {:>14} {:>14} {:>16} | {:>10} {:>10}",
        "Year", "T(cy)", "Sid(Lieske)", "Sid(IAU2006)", "Sid(Vondrák)", "DMS(Vondrák)", "Δ(L−V)\"", "Δ(I−V)\""
    );
    println!("{}", "-".repeat(130));
    for &y in &years {
        let jd = dhruv_time::calendar_to_jd(y, 1, 1.0);
        let t = (jd - 2_451_545.0) / 36525.0;
        let trop_l = try_tropical(&engine, jd, PrecessionModel::Lieske1977);
        let trop_i = try_tropical(&engine, jd, PrecessionModel::Iau2006);
        let trop_v = try_tropical(&engine, jd, PrecessionModel::Vondrak2011);
        match (trop_l, trop_i, trop_v) {
            (Some(tl), Some(ti), Some(tv)) => {
                let a_l = lahiri_e2e_3d(t, PrecessionModel::Lieske1977);
                let a_i = lahiri_e2e_3d(t, PrecessionModel::Iau2006);
                let a_v = lahiri_e2e_3d(t, PrecessionModel::Vondrak2011);
                let s_l = (tl - a_l).rem_euclid(360.0);
                let s_i = (ti - a_i).rem_euclid(360.0);
                let s_v = (tv - a_v).rem_euclid(360.0);
                // Wrap-safe delta
                let mut dl = (s_l - s_v) * 3600.0;
                if dl > 648_000.0 { dl -= 1_296_000.0; }
                if dl < -648_000.0 { dl += 1_296_000.0; }
                let mut di = (s_i - s_v) * 3600.0;
                if di > 648_000.0 { di -= 1_296_000.0; }
                if di < -648_000.0 { di += 1_296_000.0; }
                println!(
                    "{y:>5} {t:>7.3} {s_l:>14.8} {s_i:>14.8} {s_v:>14.8} {:>16} | {:>+10.4} {:>+10.4}",
                    dms(s_v), dl, di
                );
            }
            _ => {
                println!("{y:>5} {t:>7.3} {:>14} {:>14} {:>14} {:>16} |", "", "", "", "");
            }
        }
    }
}
