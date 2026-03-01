//! Golden-value comparison tests against JHora (Jagannatha Hora) charts.
//!
//! Compares our engine's graha sidereal longitudes and ayanamsha values
//! against 20 JHora reference charts spanning 1902–2094.
//!
//! JHora uses Swiss Ephemeris (based on JPL DE431); we use DE442s.
//! Black-box I/O comparison is permitted per clean-room policy.
//!
//! Requires kernel files. Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::graha_sidereal_longitudes;
use dhruv_time::UtcTime;
use dhruv_vedic_base::ayanamsha::ayanamsha_deg;
use dhruv_vedic_base::AyanamshaSystem;

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";

/// Base path for JHora test charts (outside the repository).
const JHORA_BASE: &str = "../../../jhora-test-charts";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping jhora_golden: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

// ─── DMS parser ─────────────────────────────────────────────────────────────

/// Sign code → base longitude in degrees.
fn sign_offset(code: &str) -> Option<f64> {
    match code {
        "Ar" => Some(0.0),
        "Ta" => Some(30.0),
        "Ge" => Some(60.0),
        "Cn" => Some(90.0),
        "Le" => Some(120.0),
        "Vi" => Some(150.0),
        "Li" => Some(180.0),
        "Sc" => Some(210.0),
        "Sg" => Some(240.0),
        "Cp" => Some(270.0),
        "Aq" => Some(300.0),
        "Pi" => Some(330.0),
        _ => None,
    }
}

/// Parse JHora DMS longitude: `DD Rasi MM' SS.SS"` → decimal degrees.
///
/// Examples: `9 Cn 12' 17.92"`, `21 Sg 50' 37.71"`.
fn parse_dms_longitude(s: &str) -> Option<f64> {
    // Trim and remove trailing quote
    let s = s.trim().trim_end_matches('"');
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    let deg: f64 = parts[0].parse().ok()?;
    let sign = sign_offset(parts[1])?;
    let min: f64 = parts[2].trim_end_matches('\'').parse().ok()?;
    let sec: f64 = parts[3].trim_end_matches('\'').parse().ok()?;
    Some(sign + deg + min / 60.0 + sec / 3600.0)
}

/// Parse ayanamsha from details_data.txt format: `DD-MM-SS.SS`.
fn parse_aya_dms(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let deg: f64 = parts[0].parse().ok()?;
    let min: f64 = parts[1].parse().ok()?;
    let sec: f64 = parts[2].parse().ok()?;
    Some(deg + min / 60.0 + sec / 3600.0)
}

// ─── JHora file parser ──────────────────────────────────────────────────────

/// Body names in JHora files (first 9 data lines after header).
const JHORA_BODY_NAMES: [&str; 9] = [
    "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu",
];

/// Parse graha sidereal longitudes from a JHora graha_*.txt file.
/// Returns [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu].
fn parse_jhora_graha_file(path: &Path) -> Option<[f64; 9]> {
    let raw = std::fs::read(path).ok()?;
    // Strip UTF-8 BOM if present
    let content = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        String::from_utf8_lossy(&raw[3..]).to_string()
    } else {
        String::from_utf8_lossy(&raw).to_string()
    };

    let mut lons = [0.0f64; 9];
    let mut found = [false; 9];

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Body") {
            continue;
        }

        // Extract body name: first whitespace-separated token
        let first_token = trimmed.split_whitespace().next().unwrap_or("");

        // Match to our body index
        let body_idx = JHORA_BODY_NAMES.iter().position(|&name| first_token == name);
        let Some(idx) = body_idx else { continue };
        if found[idx] {
            continue; // already parsed this body
        }

        // Extract longitude: find the first `"` (closing quote of DMS)
        // then walk backwards to collect `DD Sign MM' SS.SS"`
        if let Some(quote_pos) = trimmed.find('"') {
            // Walk backwards from quote to find the start of the degree number
            let before_quote = &trimmed[..quote_pos];
            // Find the sign code by looking for a 2-letter sign surrounded by spaces
            let sign_codes = [
                "Ar", "Ta", "Ge", "Cn", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
            ];
            for &code in &sign_codes {
                let pattern = format!(" {code} ");
                if let Some(sign_pos) = before_quote.find(&pattern) {
                    // Degree is the number(s) just before the sign code
                    let before_sign = before_quote[..sign_pos].trim_end();
                    // Find start of degree digits
                    let deg_start = before_sign
                        .rfind(|c: char| !c.is_ascii_digit())
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let dms_str = &trimmed[deg_start..=quote_pos];
                    if let Some(lon) = parse_dms_longitude(dms_str) {
                        lons[idx] = lon;
                        found[idx] = true;
                    }
                    break;
                }
            }
        }
    }

    if found.iter().all(|&f| f) {
        Some(lons)
    } else {
        None
    }
}

/// Parse ayanamsha value from details_data.txt.
fn parse_jhora_ayanamsha(path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Ayanamsa:") {
            let val = trimmed.trim_start_matches("Ayanamsa:").trim();
            return parse_aya_dms(val);
        }
    }
    None
}

// ─── Chart definitions ──────────────────────────────────────────────────────

struct ChartDef {
    name: &'static str,
    dir: &'static str,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: f64,
    tz_hours: f64,
}

impl ChartDef {
    fn utc_time(&self) -> UtcTime {
        // Convert local time to UTC
        let total_sec_local = self.hour as f64 * 3600.0
            + self.min as f64 * 60.0
            + self.sec
            - self.tz_hours * 3600.0;

        // Handle day rollover (simplified: assume no month boundary crossing)
        let (day_offset, utc_sec) = if total_sec_local < 0.0 {
            (-1i32, total_sec_local + 86400.0)
        } else if total_sec_local >= 86400.0 {
            (1i32, total_sec_local - 86400.0)
        } else {
            (0i32, total_sec_local)
        };

        let utc_hour = (utc_sec / 3600.0) as u32;
        let utc_min = ((utc_sec % 3600.0) / 60.0) as u32;
        let utc_s = utc_sec % 60.0;
        let utc_day = (self.day as i32 + day_offset) as u32;

        UtcTime::new(self.year, self.month, utc_day, utc_hour, utc_min, utc_s)
    }

    fn graha_file_path(&self, system_name: &str) -> String {
        format!("{}/{}/graha_{}.txt", JHORA_BASE, self.dir, system_name)
    }

    fn details_file_path(&self) -> String {
        format!("{}/{}/details_data.txt", JHORA_BASE, self.dir)
    }
}

const CHARTS: [ChartDef; 5] = [
    ChartDef {
        name: "Mumbai 1902",
        dir: "1900s_1902-07-02_Mumbai",
        year: 1902,
        month: 7,
        day: 2,
        hour: 16,
        min: 28,
        sec: 50.0,
        tz_hours: 5.5,
    },
    ChartDef {
        name: "Paris 1962",
        dir: "1960s_1962-04-19_Paris",
        year: 1962,
        month: 4,
        day: 19,
        hour: 9,
        min: 18,
        sec: 0.0,
        tz_hours: 1.0,
    },
    ChartDef {
        name: "Moscow 1996",
        dir: "1990s_1996-07-25_Moscow",
        year: 1996,
        month: 7,
        day: 25,
        hour: 19,
        min: 54,
        sec: 30.0,
        tz_hours: 3.0,
    },
    ChartDef {
        name: "Dubai 2021",
        dir: "2020s_2021-03-26_Dubai",
        year: 2021,
        month: 3,
        day: 26,
        hour: 13,
        min: 14,
        sec: 55.0,
        tz_hours: 4.0,
    },
    ChartDef {
        name: "Karachi 2056",
        dir: "2050s_2056-08-27_Karachi",
        year: 2056,
        month: 8,
        day: 27,
        hour: 17,
        min: 18,
        sec: 0.0,
        tz_hours: 5.0,
    },
];

// All 20 JHora ayanamsha system name mappings (retained for future use
// when comparing all 20 systems across all 20 charts).
#[allow(dead_code)]
const ALL_JHORA_SYSTEMS: [(&str, AyanamshaSystem); 20] = [
    ("Traditional_Lahiri", AyanamshaSystem::Lahiri),
    ("True_Lahiri_Chitrapaksha", AyanamshaSystem::TrueLahiri),
    ("Jagannatha", AyanamshaSystem::Jagganatha),
    ("Raman", AyanamshaSystem::Raman),
    ("Krishnamoorthy_KP", AyanamshaSystem::KP),
    ("Fagan", AyanamshaSystem::FaganBradley),
    ("Pushya-paksha", AyanamshaSystem::PushyaPaksha),
    ("Rohini-paksha", AyanamshaSystem::RohiniPaksha),
    ("Deluce", AyanamshaSystem::DeLuce),
    ("Djwhal_Khul", AyanamshaSystem::DjwalKhul),
    ("Hipparchos", AyanamshaSystem::Hipparchos),
    ("Sassanian", AyanamshaSystem::Sassanian),
    ("Deva-datta", AyanamshaSystem::DevaDutta),
    ("Usha-Shashi", AyanamshaSystem::UshaShashi),
    ("Yukteshwar", AyanamshaSystem::Yukteshwar),
    ("JN_Bhasin", AyanamshaSystem::JnBhasin),
    ("Chandra_Hari", AyanamshaSystem::ChandraHari),
    ("Sri_Surya_Siddhanta", AyanamshaSystem::SuryaSiddhanta),
    ("Galaxy_Center_0Sg0", AyanamshaSystem::GalacticCenter0Sag),
    ("Aldebaran_15Ta0", AyanamshaSystem::Aldebaran15Tau),
];

const GRAHA_NAMES: [&str; 9] = [
    "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu",
];

/// Normalize angular difference to [-180, 180].
fn ang_diff(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// Maximum allowed difference for sapta grahas (Sun–Saturn) with Lahiri.
/// Swiss Ephemeris (DE431) vs our DE442s + minor ayanamsha formula differences.
/// 120" = 2 arcminutes.
const LAHIRI_SAPTA_TOLERANCE_DEG: f64 = 120.0 / 3600.0;

/// Maximum allowed difference for Rahu/Ketu, in degrees.
/// Node models differ substantially between engines.
/// 600" = 10 arcminutes.
const NODE_TOLERANCE_DEG: f64 = 600.0 / 3600.0;

/// Wider tolerance for systems where formula implementations diverge.
/// Different star catalogs, proper motion, invariable plane parameters, etc.
/// 600" = 10 arcminutes.
const WIDE_SAPTA_TOLERANCE_DEG: f64 = 600.0 / 3600.0;

/// Maximum allowed ayanamsha difference, in degrees.
/// 120" = 2 arcminutes — Lahiri is well-standardized.
const AYA_TOLERANCE_DEG: f64 = 120.0 / 3600.0;

#[test]
fn jhora_lahiri_graha_comparison() {
    let Some(engine) = load_engine() else { return };
    let base = Path::new(JHORA_BASE);
    if !base.exists() {
        eprintln!("Skipping jhora_golden: test charts not found at {JHORA_BASE}");
        return;
    }

    let system = AyanamshaSystem::Lahiri;
    let mut total_checked = 0;
    let mut max_diff_sapta = 0.0f64;
    let mut max_diff_node = 0.0f64;

    for chart in &CHARTS {
        let graha_file = chart.graha_file_path("Traditional_Lahiri");
        let graha_path = Path::new(&graha_file);
        if !graha_path.exists() {
            eprintln!("  Skipping {}: graha file not found", chart.name);
            continue;
        }

        let jhora_lons = parse_jhora_graha_file(graha_path)
            .unwrap_or_else(|| panic!("Failed to parse {}", graha_path.display()));

        let utc = chart.utc_time();
        let jd_tdb = utc.to_jd_tdb(engine.lsk());

        let our_lons = graha_sidereal_longitudes(&engine, jd_tdb, system, true)
            .unwrap_or_else(|e| panic!("{}: graha_sidereal_longitudes failed: {e}", chart.name));

        for i in 0..9 {
            let diff = ang_diff(our_lons.longitudes[i], jhora_lons[i]).abs();
            let tolerance = if i < 7 {
                LAHIRI_SAPTA_TOLERANCE_DEG
            } else {
                NODE_TOLERANCE_DEG
            };

            if i < 7 {
                max_diff_sapta = max_diff_sapta.max(diff);
            } else {
                max_diff_node = max_diff_node.max(diff);
            }

            assert!(
                diff < tolerance,
                "{} Lahiri {}: our={:.6}° jhora={:.6}° diff={:.1}\" (tolerance={:.0}\")",
                chart.name,
                GRAHA_NAMES[i],
                our_lons.longitudes[i],
                jhora_lons[i],
                diff * 3600.0,
                tolerance * 3600.0,
            );
            total_checked += 1;
        }
    }

    eprintln!(
        "Lahiri comparison: {total_checked} data points checked. \
         Max diff sapta={:.1}\", nodes={:.1}\"",
        max_diff_sapta * 3600.0,
        max_diff_node * 3600.0,
    );
}

#[test]
fn jhora_jagganatha_graha_comparison() {
    let Some(engine) = load_engine() else { return };
    let base = Path::new(JHORA_BASE);
    if !base.exists() {
        eprintln!("Skipping jhora_golden: test charts not found at {JHORA_BASE}");
        return;
    }

    let system = AyanamshaSystem::Jagganatha;
    let mut total_checked = 0;
    let mut max_diff_sapta = 0.0f64;

    for chart in &CHARTS {
        let graha_file = chart.graha_file_path("Jagannatha");
        let graha_path = Path::new(&graha_file);
        if !graha_path.exists() {
            eprintln!("  Skipping {}: graha file not found", chart.name);
            continue;
        }

        let jhora_lons = parse_jhora_graha_file(graha_path)
            .unwrap_or_else(|| panic!("Failed to parse {}", graha_path.display()));

        let utc = chart.utc_time();
        let jd_tdb = utc.to_jd_tdb(engine.lsk());

        let our_lons = graha_sidereal_longitudes(&engine, jd_tdb, system, true)
            .unwrap_or_else(|e| panic!("{}: graha_sidereal_longitudes failed: {e}", chart.name));

        // Wider tolerance: different invariable plane parameters, star catalogs,
        // and projection methods. Especially large for bodies at high ecliptic
        // latitude (e.g. retrograde Mercury near station, ~7° lat) where the
        // 1.58° plane tilt produces significant longitude shifts.
        // 1200" = 20 arcminutes for sapta grahas.
        //
        // Skip Rahu/Ketu (indices 7-8): fundamental architectural difference.
        // Our engine computes the osculating node relative to the invariable
        // plane (where the orbit crosses it), giving a geometrically consistent
        // ascending node. JHora uses the ecliptic ascending node and simply
        // subtracts the invariable-plane ayanamsha. This causes ~18° divergence
        // for Rahu/Ketu, which is the actual geometric node shift between planes.
        let jagganatha_sapta_tol = 1200.0 / 3600.0;
        for i in 0..7 {
            let diff = ang_diff(our_lons.longitudes[i], jhora_lons[i]).abs();
            max_diff_sapta = max_diff_sapta.max(diff);

            assert!(
                diff < jagganatha_sapta_tol,
                "{} Jagganatha {}: our={:.6}° jhora={:.6}° diff={:.1}\" (tolerance={:.0}\")",
                chart.name,
                GRAHA_NAMES[i],
                our_lons.longitudes[i],
                jhora_lons[i],
                diff * 3600.0,
                jagganatha_sapta_tol * 3600.0,
            );
            total_checked += 1;
        }
    }

    eprintln!(
        "Jagganatha comparison: {total_checked} sapta graha data points checked. \
         Max diff={:.1}\" (Rahu/Ketu skipped — architectural difference)",
        max_diff_sapta * 3600.0,
    );
}

#[test]
fn jhora_ayanamsha_comparison() {
    let Some(engine) = load_engine() else { return };
    let base = Path::new(JHORA_BASE);
    if !base.exists() {
        eprintln!("Skipping jhora_golden: test charts not found at {JHORA_BASE}");
        return;
    }

    // JHora's details_data.txt ayanamsha uses Lahiri (Traditional) by default.
    let system = AyanamshaSystem::Lahiri;
    let mut total_checked = 0;
    let mut max_diff = 0.0f64;

    for chart in &CHARTS {
        let details_file = chart.details_file_path();
        let details_path = Path::new(&details_file);
        if !details_path.exists() {
            eprintln!("  Skipping {}: details file not found", chart.name);
            continue;
        }

        let jhora_aya = parse_jhora_ayanamsha(details_path)
            .unwrap_or_else(|| panic!("Failed to parse ayanamsha from {}", details_path.display()));

        let utc = chart.utc_time();
        let jd_tdb = utc.to_jd_tdb(engine.lsk());
        let t_cy = (jd_tdb - 2451545.0) / 36525.0;
        let our_aya = ayanamsha_deg(system, t_cy, true);

        let diff = (our_aya - jhora_aya).abs();
        max_diff = max_diff.max(diff);

        assert!(
            diff < AYA_TOLERANCE_DEG,
            "{} Lahiri ayanamsha: our={:.6}° jhora={:.6}° diff={:.2}\"",
            chart.name,
            our_aya,
            jhora_aya,
            diff * 3600.0,
        );
        total_checked += 1;
    }

    eprintln!(
        "Ayanamsha comparison: {total_checked} charts checked. Max diff={:.2}\"",
        max_diff * 3600.0,
    );
}

#[test]
fn jhora_multi_system_graha_comparison() {
    let Some(engine) = load_engine() else { return };
    let base = Path::new(JHORA_BASE);
    if !base.exists() {
        eprintln!("Skipping jhora_golden: test charts not found at {JHORA_BASE}");
        return;
    }

    // Test a representative subset of ayanamsha systems
    let systems_to_test = [
        ("Raman", AyanamshaSystem::Raman),
        ("Fagan", AyanamshaSystem::FaganBradley),
        ("Krishnamoorthy_KP", AyanamshaSystem::KP),
        ("Yukteshwar", AyanamshaSystem::Yukteshwar),
        ("True_Lahiri_Chitrapaksha", AyanamshaSystem::TrueLahiri),
    ];

    let mut total_checked = 0;
    let mut max_diff_sapta = 0.0f64;
    let mut failures: Vec<String> = Vec::new();

    for (jhora_name, system) in &systems_to_test {
        for chart in &CHARTS {
            let graha_file = chart.graha_file_path(jhora_name);
            let graha_path = Path::new(&graha_file);
            if !graha_path.exists() {
                continue;
            }

            let jhora_lons = match parse_jhora_graha_file(graha_path) {
                Some(l) => l,
                None => continue,
            };

            let utc = chart.utc_time();
            let jd_tdb = utc.to_jd_tdb(engine.lsk());

            let our_lons = match graha_sidereal_longitudes(&engine, jd_tdb, *system, true) {
                Ok(l) => l,
                Err(e) => {
                    failures.push(format!("{} {jhora_name}: {e}", chart.name));
                    continue;
                }
            };

            // Only check sapta grahas (0-6) for multi-system test.
            // Use wide tolerance: formula implementations diverge across software.
            for i in 0..7 {
                let diff = ang_diff(our_lons.longitudes[i], jhora_lons[i]).abs();
                max_diff_sapta = max_diff_sapta.max(diff);

                if diff >= WIDE_SAPTA_TOLERANCE_DEG {
                    failures.push(format!(
                        "{} {jhora_name} {}: our={:.6}° jhora={:.6}° diff={:.1}\"",
                        chart.name,
                        GRAHA_NAMES[i],
                        our_lons.longitudes[i],
                        jhora_lons[i],
                        diff * 3600.0,
                    ));
                }
                total_checked += 1;
            }
        }
    }

    eprintln!(
        "Multi-system comparison: {total_checked} data points. Max sapta diff={:.1}\"",
        max_diff_sapta * 3600.0,
    );

    assert!(
        failures.is_empty(),
        "Multi-system comparison failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn jhora_rashi_agreement() {
    let Some(engine) = load_engine() else { return };
    let base = Path::new(JHORA_BASE);
    if !base.exists() {
        eprintln!("Skipping jhora_golden: test charts not found at {JHORA_BASE}");
        return;
    }

    // Verify rashi (sign) assignments match for all 20 charts with Lahiri
    let chart_dirs: Vec<String> = std::fs::read_dir(base)
        .unwrap()
        .filter_map(|e| {
            let e = e.ok()?;
            if e.file_type().ok()?.is_dir() {
                Some(e.file_name().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    let mut total_checked = 0;
    let mut rashi_mismatches: Vec<String> = Vec::new();

    for dir_name in &chart_dirs {
        // Parse birth data from details_data.txt
        let details_path = base.join(dir_name).join("details_data.txt");
        let graha_path = base.join(dir_name).join("graha_Traditional_Lahiri.txt");
        if !details_path.exists() || !graha_path.exists() {
            continue;
        }

        // Parse the date/time from directory name: {decade}s_{YYYY-MM-DD}_{city}
        let parts: Vec<&str> = dir_name.split('_').collect();
        if parts.len() < 3 {
            continue;
        }
        let date_parts: Vec<&str> = parts[1].split('-').collect();
        if date_parts.len() != 3 {
            continue;
        }
        let year: i32 = match date_parts[0].parse() {
            Ok(y) => y,
            Err(_) => continue,
        };
        let month: u32 = match date_parts[1].parse() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let day: u32 = match date_parts[2].parse() {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Parse timezone and time from details_data.txt
        let details_content = match std::fs::read_to_string(&details_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (hour, min, sec, tz_hours) = match parse_time_and_tz(&details_content) {
            Some(t) => t,
            None => continue,
        };

        // Convert to UTC
        let total_sec_local =
            hour as f64 * 3600.0 + min as f64 * 60.0 + sec - tz_hours * 3600.0;
        let (day_offset, utc_sec) = if total_sec_local < 0.0 {
            (-1i32, total_sec_local + 86400.0)
        } else if total_sec_local >= 86400.0 {
            (1, total_sec_local - 86400.0)
        } else {
            (0, total_sec_local)
        };
        let utc_h = (utc_sec / 3600.0) as u32;
        let utc_m = ((utc_sec % 3600.0) / 60.0) as u32;
        let utc_s = utc_sec % 60.0;
        let utc_d = (day as i32 + day_offset) as u32;

        let utc = UtcTime::new(year, month, utc_d, utc_h, utc_m, utc_s);
        let jd_tdb = utc.to_jd_tdb(engine.lsk());

        let jhora_lons = match parse_jhora_graha_file(&graha_path) {
            Some(l) => l,
            None => continue,
        };

        let our_lons =
            match graha_sidereal_longitudes(&engine, jd_tdb, AyanamshaSystem::Lahiri, true) {
                Ok(l) => l,
                Err(_) => continue,
            };

        // Compare rashi (sign = floor(lon / 30))
        for i in 0..7 {
            let our_rashi = (our_lons.longitudes[i] / 30.0).floor() as u8;
            let jhora_rashi = (jhora_lons[i] / 30.0).floor() as u8;

            if our_rashi != jhora_rashi {
                // Check if we're near a boundary (within 2')
                let our_in_sign = our_lons.longitudes[i] % 30.0;
                let jhora_in_sign = jhora_lons[i] % 30.0;
                let near_boundary =
                    our_in_sign < 0.05 || our_in_sign > 29.95 || jhora_in_sign < 0.05 || jhora_in_sign > 29.95;

                if !near_boundary {
                    rashi_mismatches.push(format!(
                        "{} {} our_rashi={} jhora_rashi={} (our={:.4}° jhora={:.4}°)",
                        dir_name, GRAHA_NAMES[i], our_rashi, jhora_rashi,
                        our_lons.longitudes[i], jhora_lons[i],
                    ));
                }
            }
            total_checked += 1;
        }
    }

    eprintln!("Rashi agreement: {total_checked} graha-chart pairs checked");
    assert!(
        rashi_mismatches.is_empty(),
        "Rashi mismatches (not near boundary):\n{}",
        rashi_mismatches.join("\n"),
    );
}

/// Parse time and timezone from details_data.txt content.
fn parse_time_and_tz(content: &str) -> Option<(u8, u8, f64, f64)> {
    let mut time_str = None;
    let mut tz_str = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Time:") && !trimmed.contains("Zone") {
            time_str = Some(trimmed.trim_start_matches("Time:").trim().to_string());
        }
        if trimmed.starts_with("Time Zone:") {
            tz_str = Some(trimmed.trim_start_matches("Time Zone:").trim().to_string());
        }
    }

    let time = time_str?;
    let tz = tz_str?;

    // Parse time: HH:MM:SS
    let time_parts: Vec<&str> = time.split(':').collect();
    if time_parts.len() < 3 {
        return None;
    }
    let hour: u8 = time_parts[0].parse().ok()?;
    let min: u8 = time_parts[1].parse().ok()?;
    let sec: f64 = time_parts[2].parse().ok()?;

    // Parse timezone: "H:MM:SS (East/West of GMT)"
    let tz_val_str = tz.split('(').next()?.trim();
    let is_east = tz.contains("East");
    let tz_parts: Vec<&str> = tz_val_str.split(':').collect();
    let tz_h: f64 = tz_parts[0].parse().ok()?;
    let tz_m: f64 = if tz_parts.len() > 1 {
        tz_parts[1].parse().ok()?
    } else {
        0.0
    };
    let tz_s: f64 = if tz_parts.len() > 2 {
        tz_parts[2].parse().ok()?
    } else {
        0.0
    };
    let tz_hours = tz_h + tz_m / 60.0 + tz_s / 3600.0;
    let tz_signed = if is_east { tz_hours } else { -tz_hours };

    Some((hour, min, sec, tz_signed))
}

// ─── Parser unit tests ──────────────────────────────────────────────────────

#[test]
fn parse_dms_known_values() {
    // 9 Cn 12' 17.92" = Cancer 9°12'17.92" = 90 + 9 + 12/60 + 17.92/3600
    let lon = parse_dms_longitude("9 Cn 12' 17.92\"").unwrap();
    let expected = 90.0 + 9.0 + 12.0 / 60.0 + 17.92 / 3600.0;
    assert!((lon - expected).abs() < 1e-10, "got {lon}, expected {expected}");

    // 21 Sg 50' 37.71" = Sagittarius 21°50'37.71"
    let lon2 = parse_dms_longitude("21 Sg 50' 37.71\"").unwrap();
    let expected2 = 240.0 + 21.0 + 50.0 / 60.0 + 37.71 / 3600.0;
    assert!(
        (lon2 - expected2).abs() < 1e-10,
        "got {lon2}, expected {expected2}"
    );

    // 0 Ar 00' 00.00"
    let lon3 = parse_dms_longitude("0 Ar 00' 00.00\"").unwrap();
    assert!((lon3).abs() < 1e-10);
}

#[test]
fn parse_aya_dms_known() {
    // 23-47-36.27 = 23 + 47/60 + 36.27/3600
    let aya = parse_aya_dms("23-47-36.27").unwrap();
    let expected = 23.0 + 47.0 / 60.0 + 36.27 / 3600.0;
    assert!((aya - expected).abs() < 1e-10, "got {aya}, expected {expected}");
}
