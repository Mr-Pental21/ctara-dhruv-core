//! Piecewise polynomial Delta-T model.
//!
//! Delta-T is defined as `TT - UT1` in seconds.
//! This module implements public-domain polynomial fits from the
//! Espenak/Meeus and Morrison/Stephenson lineage.

use crate::julian::{calendar_to_jd, jd_to_calendar};
use std::sync::OnceLock;

/// Parsed SMH2016 reconstruction points for `-720..1961`.
#[derive(Debug, Clone)]
pub struct Smh2016Reconstruction {
    points: Vec<(f64, f64)>, // (year_fraction, delta_t_seconds)
    segments: Vec<SmhCubicSegment>,
}

#[derive(Debug, Clone, Copy)]
struct SmhCubicSegment {
    k0: f64,
    k1: f64,
    a0: f64,
    a1: f64,
    a2: f64,
    a3: f64,
}

impl Smh2016Reconstruction {
    pub fn from_points(mut points: Vec<(f64, f64)>) -> Result<Self, String> {
        if points.len() < 2 {
            return Err("SMH2016 reconstruction needs at least 2 points".to_string());
        }
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for w in points.windows(2) {
            if (w[1].0 - w[0].0).abs() < 1e-12 {
                return Err(format!("duplicate SMH2016 year value: {}", w[0].0));
            }
        }
        Ok(Self {
            points,
            segments: Vec::new(),
        })
    }

    pub fn from_segments(mut rows: Vec<(f64, f64, f64, f64, f64, f64)>) -> Result<Self, String> {
        if rows.is_empty() {
            return Err("SMH2016 reconstruction needs at least 1 segment".to_string());
        }
        rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut segments = Vec::with_capacity(rows.len());
        let mut prev_k1 = rows[0].0;
        for (i, (k0, k1, a0, a1, a2, a3)) in rows.into_iter().enumerate() {
            if k1 <= k0 {
                return Err(format!("invalid SMH2016 segment [{k0}, {k1}]"));
            }
            if i > 0 && (k0 - prev_k1).abs() > 1e-6 {
                return Err(format!("non-contiguous SMH2016 segments near {k0}"));
            }
            prev_k1 = k1;
            segments.push(SmhCubicSegment {
                k0,
                k1,
                a0,
                a1,
                a2,
                a3,
            });
        }
        Ok(Self {
            points: Vec::new(),
            segments,
        })
    }

    pub fn delta_t_at_year_linear(&self, year: f64) -> Option<f64> {
        if self.points.is_empty() {
            return None;
        }
        if year < self.points[0].0 || year > self.points[self.points.len() - 1].0 {
            return None;
        }
        let idx = self
            .points
            .partition_point(|(y, _)| *y < year)
            .saturating_sub(1);
        if idx + 1 >= self.points.len() {
            return Some(self.points[idx].1);
        }
        let (y0, d0) = self.points[idx];
        let (y1, d1) = self.points[idx + 1];
        if (y1 - y0).abs() < 1e-12 {
            return Some(d0);
        }
        let f = (year - y0) / (y1 - y0);
        Some(d0 + f * (d1 - d0))
    }

    pub fn delta_t_at_year(&self, year: f64) -> Option<f64> {
        if !self.segments.is_empty() {
            let first = self.segments.first().copied()?;
            let last = self.segments.last().copied()?;
            if year < first.k0 || year > last.k1 {
                return None;
            }
            let idx = self
                .segments
                .partition_point(|s| s.k1 < year)
                .min(self.segments.len().saturating_sub(1));
            let s = self.segments[idx];
            if year < s.k0 || year > s.k1 {
                return None;
            }
            let t = (year - s.k0) / (s.k1 - s.k0);
            return Some(s.a0 + s.a1 * t + s.a2 * t * t + s.a3 * t * t * t);
        }
        self.delta_t_at_year_linear(year)
    }
}

static SMH2016_RECONSTRUCTION: OnceLock<Smh2016Reconstruction> = OnceLock::new();

/// Parse a plain-text SMH2016 table.
///
/// Supported row formats (delimiters: whitespace/comma/semicolon):
/// - points: `year delta_t_seconds`
/// - cubic segments: `Ki Ki+1 a0 a1 a2 a3`
/// - cubic segments with row index prefix: `row Ki Ki+1 a0 a1 a2 a3`
pub fn parse_smh2016_reconstruction(content: &str) -> Result<Smh2016Reconstruction, String> {
    let mut points = Vec::new();
    let mut segments = Vec::new();
    for (line_no, raw) in content.lines().enumerate() {
        let line = raw.trim().replace('\u{2212}', "-");
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("//")
            || line.starts_with('%')
            || line.starts_with("year")
            || line.starts_with("Year")
        {
            continue;
        }
        let cols: Vec<&str> = line
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
            .filter(|s| !s.is_empty())
            .collect();
        if cols.len() < 2 || cols.iter().all(|c| c.parse::<f64>().is_err()) {
            continue;
        }
        // Prefer spline rows when provided:
        // [Ki,Ki+1,a0,a1,a2,a3] or [row,Ki,Ki+1,a0,a1,a2,a3]
        if cols.len() >= 6 {
            let take = if cols.len() >= 7 { 1 } else { 0 };
            let nums: Result<Vec<f64>, _> = cols[take..take + 6]
                .iter()
                .map(|v| {
                    v.parse().map_err(|_| {
                        format!(
                            "SMH2016 parse error at line {}: invalid spline numeric value",
                            line_no + 1
                        )
                    })
                })
                .collect();
            if let Ok(v) = nums {
                segments.push((v[0], v[1], v[2], v[3], v[4], v[5]));
                continue;
            }
        }

        // Fallback to (year, delta_t) points.
        let year: f64 = cols[0].parse().map_err(|_| {
            format!(
                "SMH2016 parse error at line {}: invalid year/segment start",
                line_no + 1
            )
        })?;
        let dt: f64 = cols[1].parse().map_err(|_| {
            format!(
                "SMH2016 parse error at line {}: invalid delta_t_seconds/segment end",
                line_no + 1
            )
        })?;
        points.push((year, dt));
    }
    if !segments.is_empty() {
        Smh2016Reconstruction::from_segments(segments)
    } else {
        Smh2016Reconstruction::from_points(points)
    }
}

/// Install SMH2016 reconstruction points for runtime model dispatch.
///
/// Returns `true` if installed now, `false` if already installed.
pub fn install_smh2016_reconstruction(reconstruction: Smh2016Reconstruction) -> bool {
    SMH2016_RECONSTRUCTION.set(reconstruction).is_ok()
}

/// Whether SMH2016 reconstruction points are installed.
pub fn smh2016_reconstruction_installed() -> bool {
    SMH2016_RECONSTRUCTION.get().is_some()
}

/// Piecewise segment used by the Delta-T model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaTSegment {
    PreMinus720Quadratic,
    Smh2016Reconstruction,
    SmhAsymptoticFuture,
    BeforeMinus500,
    Minus500To500,
    Year500To1600,
    Year1600To1700,
    Year1700To1800,
    Year1800To1860,
    Year1860To1900,
    Year1900To1920,
    Year1920To1941,
    Year1941To1961,
    Year1961To1986,
    Year1986To2005,
    Year2005To2050,
    Year2050To2150,
    After2150,
}

/// Delta-T model selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaTModel {
    /// Legacy Espenak/Meeus-style piecewise polynomial currently used by Dhruv.
    LegacyEspenakMeeus2006,
    /// Target migration model (SMH2016 + pre-720 quadratic).
    ///
    /// Phase A scaffolding keeps output parity with the legacy model until
    /// SMH2016 data assets are integrated.
    Smh2016WithPre720Quadratic,
}

/// Future Delta-T strategy used for post-EOP asymptotic fallback.
///
/// Clean sources:
/// - Morrison et al. (2021) Addendum 2020, Eq. (5) + Table 1.
/// - NASA GSFC Delta-T help page `deltaT2.html` (Stephenson 1997 summary).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmhFutureParabolaFamily {
    /// Piecewise family from Addendum 2020 Table 1:
    /// - `c=-20.0` for `-720..2019`
    /// - `c=-17.52` for `2019..3000`
    /// - `c=-15.32` for `3000..10000`
    Addendum2020Piecewise,
    /// Forced family member `c=-20.0`.
    ConstantCMinus20,
    /// Forced family member `c=-17.52`.
    ConstantCMinus17p52,
    /// Forced family member `c=-15.32`.
    ConstantCMinus15p32,
    /// Stephenson (1997) long-term parabola used by some eclipse toolchains:
    /// `ΔT = -20 + 31*t²`, `t = (year - 1820)/100`.
    ///
    /// This is intended only for post-EOP future fallback strategy selection.
    Stephenson1997,
}

impl Default for SmhFutureParabolaFamily {
    fn default() -> Self {
        Self::Addendum2020Piecewise
    }
}

impl Default for DeltaTModel {
    fn default() -> Self {
        Self::Smh2016WithPre720Quadratic
    }
}

/// Compute Delta-T (`TT-UT1`) in seconds for a UT1 Julian date.
///
/// Returns `(delta_t_seconds, segment_used)`.
pub fn delta_t_seconds(jd_ut1: f64) -> (f64, DeltaTSegment) {
    delta_t_seconds_with_model(jd_ut1, DeltaTModel::LegacyEspenakMeeus2006)
}

/// Compute Delta-T (`TT-UT1`) in seconds for a UT1 Julian date and model.
///
/// Returns `(delta_t_seconds, segment_used)`.
pub fn delta_t_seconds_with_model(jd_ut1: f64, model: DeltaTModel) -> (f64, DeltaTSegment) {
    match model {
        DeltaTModel::LegacyEspenakMeeus2006 => delta_t_seconds_legacy(jd_ut1),
        DeltaTModel::Smh2016WithPre720Quadratic => {
            delta_t_seconds_smh_model(jd_ut1, SMH2016_RECONSTRUCTION.get())
        }
    }
}

/// Long-term asymptotic Delta-T branch (SMH/Horizons family), seconds.
///
/// Default is the Addendum 2020 piecewise parabola family.
pub fn smh_asymptotic_delta_t_seconds(year: f64) -> f64 {
    smh_asymptotic_delta_t_seconds_with_family(year, SmhFutureParabolaFamily::default())
}

/// Long-term asymptotic Delta-T branch (strategy-selectable), seconds.
pub fn smh_asymptotic_delta_t_seconds_with_family(
    year: f64,
    family: SmhFutureParabolaFamily,
) -> f64 {
    match family {
        SmhFutureParabolaFamily::Stephenson1997 => {
            let t = (year - 1820.0) / 100.0;
            -20.0 + 31.0 * t * t
        }
        _ => {
            let c = smh_future_parabola_c_for_year(year, family);
            let t = (year - 1825.0) / 100.0;
            c + 32.5 * t * t
        }
    }
}

/// Resolve the SMH future parabola-family constant `c` for a given year.
pub fn smh_future_parabola_c_for_year(year: f64, family: SmhFutureParabolaFamily) -> f64 {
    match family {
        SmhFutureParabolaFamily::Addendum2020Piecewise => {
            if year < 2019.0 {
                -20.0
            } else if year < 3000.0 {
                -17.52
            } else {
                -15.32
            }
        }
        SmhFutureParabolaFamily::ConstantCMinus20 => -20.0,
        SmhFutureParabolaFamily::ConstantCMinus17p52 => -17.52,
        SmhFutureParabolaFamily::ConstantCMinus15p32 => -15.32,
        // Kept for API compatibility: this helper returns the additive constant
        // for c+32.5*t^2 families. Stephenson1997 uses a different polynomial.
        SmhFutureParabolaFamily::Stephenson1997 => -20.0,
    }
}

/// Evaluate asymptotic Delta-T branch for a JD epoch.
pub fn smh_asymptotic_delta_t_seconds_for_jd(jd: f64) -> (f64, DeltaTSegment) {
    smh_asymptotic_delta_t_seconds_for_jd_with_family(jd, SmhFutureParabolaFamily::default())
}

/// Evaluate asymptotic Delta-T branch for a JD epoch with selected family.
pub fn smh_asymptotic_delta_t_seconds_for_jd_with_family(
    jd: f64,
    family: SmhFutureParabolaFamily,
) -> (f64, DeltaTSegment) {
    (
        smh_asymptotic_delta_t_seconds_with_family(year_fraction_from_jd(jd), family),
        DeltaTSegment::SmhAsymptoticFuture,
    )
}

fn delta_t_seconds_smh_model(
    jd_ut1: f64,
    reconstruction: Option<&Smh2016Reconstruction>,
) -> (f64, DeltaTSegment) {
    let y = year_fraction_from_jd(jd_ut1);
    if y < -720.0 {
        // Horizons-style long-term quadratic branch used by the target
        // hybrid model for deep antiquity.
        let t = (y - 1825.0) / 100.0;
        let dt = -75.62 + 31.35 * t * t;
        return (dt, DeltaTSegment::PreMinus720Quadratic);
    }

    if y <= 1961.0
        && let Some(table) = reconstruction
        && let Some(dt) = table.delta_t_at_year(y)
    {
        return (dt, DeltaTSegment::Smh2016Reconstruction);
    }

    // Fallback path until SMH2016 assets are installed for this runtime.
    delta_t_seconds_legacy(jd_ut1)
}

fn delta_t_seconds_legacy(jd_ut1: f64) -> (f64, DeltaTSegment) {
    let y = year_fraction_from_jd(jd_ut1);

    if y < -500.0 {
        let u = (y - 1820.0) / 100.0;
        return (-20.0 + 32.0 * u * u, DeltaTSegment::BeforeMinus500);
    }

    if y < 500.0 {
        let u = y / 100.0;
        let dt = 10583.6 - 1014.41 * u + 33.78311 * u.powi(2)
            - 5.952053 * u.powi(3)
            - 0.1798452 * u.powi(4)
            + 0.022174192 * u.powi(5)
            + 0.0090316521 * u.powi(6);
        return (dt, DeltaTSegment::Minus500To500);
    }

    if y < 1600.0 {
        let u = (y - 1000.0) / 100.0;
        let dt = 1574.2 - 556.01 * u + 71.23472 * u.powi(2) + 0.319781 * u.powi(3)
            - 0.8503463 * u.powi(4)
            - 0.005050998 * u.powi(5)
            + 0.0083572073 * u.powi(6);
        return (dt, DeltaTSegment::Year500To1600);
    }

    if y < 1700.0 {
        let t = y - 1600.0;
        let dt = 120.0 - 0.9808 * t - 0.01532 * t.powi(2) + t.powi(3) / 7129.0;
        return (dt, DeltaTSegment::Year1600To1700);
    }

    if y < 1800.0 {
        let t = y - 1700.0;
        let dt = 8.83 + 0.1603 * t - 0.0059285 * t.powi(2) + 0.00013336 * t.powi(3)
            - t.powi(4) / 1_174_000.0;
        return (dt, DeltaTSegment::Year1700To1800);
    }

    if y < 1860.0 {
        let t = y - 1800.0;
        let dt = 13.72 - 0.332447 * t + 0.0068612 * t.powi(2) + 0.0041116 * t.powi(3)
            - 0.00037436 * t.powi(4)
            + 0.0000121272 * t.powi(5)
            - 0.0000001699 * t.powi(6)
            + 0.000000000875 * t.powi(7);
        return (dt, DeltaTSegment::Year1800To1860);
    }

    if y < 1900.0 {
        let t = y - 1860.0;
        let dt = 7.62 + 0.5737 * t - 0.251754 * t.powi(2) + 0.01680668 * t.powi(3)
            - 0.0004473624 * t.powi(4)
            + t.powi(5) / 233_174.0;
        return (dt, DeltaTSegment::Year1860To1900);
    }

    if y < 1920.0 {
        let t = y - 1900.0;
        let dt = -2.79 + 1.494119 * t - 0.0598939 * t.powi(2) + 0.0061966 * t.powi(3)
            - 0.000197 * t.powi(4);
        return (dt, DeltaTSegment::Year1900To1920);
    }

    if y < 1941.0 {
        let t = y - 1920.0;
        let dt = 21.20 + 0.84493 * t - 0.0761 * t.powi(2) + 0.0020936 * t.powi(3);
        return (dt, DeltaTSegment::Year1920To1941);
    }

    if y < 1961.0 {
        let t = y - 1950.0;
        let dt = 29.07 + 0.407 * t - t.powi(2) / 233.0 + t.powi(3) / 2547.0;
        return (dt, DeltaTSegment::Year1941To1961);
    }

    if y < 1986.0 {
        let t = y - 1975.0;
        let dt = 45.45 + 1.067 * t - t.powi(2) / 260.0 - t.powi(3) / 718.0;
        return (dt, DeltaTSegment::Year1961To1986);
    }

    if y < 2005.0 {
        let t = y - 2000.0;
        let dt = 63.86 + 0.3345 * t - 0.060374 * t.powi(2)
            + 0.0017275 * t.powi(3)
            + 0.000651814 * t.powi(4)
            + 0.00002373599 * t.powi(5);
        return (dt, DeltaTSegment::Year1986To2005);
    }

    if y < 2050.0 {
        let t = y - 2000.0;
        let dt = 62.92 + 0.32217 * t + 0.005589 * t.powi(2);
        return (dt, DeltaTSegment::Year2005To2050);
    }

    if y < 2150.0 {
        let u = (y - 1820.0) / 100.0;
        let dt = -20.0 + 32.0 * u.powi(2) - 0.5628 * (2150.0 - y);
        return (dt, DeltaTSegment::Year2050To2150);
    }

    let u = (y - 1820.0) / 100.0;
    (-20.0 + 32.0 * u.powi(2), DeltaTSegment::After2150)
}

fn year_fraction_from_jd(jd: f64) -> f64 {
    let (year, _month, _day) = jd_to_calendar(jd);
    let jd_year_start = calendar_to_jd(year, 1, 1.0);
    let jd_next_year_start = calendar_to_jd(year + 1, 1, 1.0);
    let year_span_days = jd_next_year_start - jd_year_start;
    year as f64 + (jd - jd_year_start) / year_span_days
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julian::calendar_to_jd;

    #[test]
    fn delta_t_2000_reasonable() {
        let jd = calendar_to_jd(2000, 1, 1.0);
        let (dt, seg) = delta_t_seconds(jd);
        assert_eq!(seg, DeltaTSegment::Year1986To2005);
        assert!(
            (dt - 63.8).abs() < 2.0,
            "expected near 63.8s around year 2000, got {dt}"
        );
    }

    #[test]
    fn delta_t_2024_reasonable() {
        let jd = calendar_to_jd(2024, 1, 1.0);
        let (dt, seg) = delta_t_seconds(jd);
        assert_eq!(seg, DeltaTSegment::Year2005To2050);
        assert!(
            (dt - 74.0).abs() < 6.0,
            "expected modern-era delta-T around ~70-80s, got {dt}"
        );
    }

    #[test]
    fn delta_t_piecewise_boundaries_are_continuous_enough() {
        // Adjacent segments should not produce discontinuities larger than a few seconds.
        let boundaries = [
            -500, 500, 1600, 1700, 1800, 1860, 1900, 1920, 1941, 1961, 1986, 2005, 2050, 2150,
        ];
        for year in boundaries {
            let jd_before = calendar_to_jd(year, 1, 0.999); // slightly before boundary
            let jd_after = calendar_to_jd(year, 1, 1.001); // slightly after boundary
            let (dt_before, _) = delta_t_seconds(jd_before);
            let (dt_after, _) = delta_t_seconds(jd_after);
            let jump = (dt_after - dt_before).abs();
            assert!(
                jump < 10.0,
                "delta-T jump too large at {year}: {jump:.6}s (before={dt_before:.6}, after={dt_after:.6})"
            );
        }
    }

    #[test]
    fn model_dispatch_defaults_to_smh() {
        let jd = calendar_to_jd(2024, 1, 1.0);
        let smh = delta_t_seconds_with_model(jd, DeltaTModel::Smh2016WithPre720Quadratic);
        let defaulted = delta_t_seconds_with_model(jd, DeltaTModel::default());
        assert_eq!(smh.1, defaulted.1);
        assert!(
            (smh.0 - defaulted.0).abs() < 1e-12,
            "smh/default mismatch: smh={} default={}",
            smh.0,
            defaulted.0
        );
    }

    #[test]
    fn smh_placeholder_keeps_legacy_output_for_phase_a() {
        let jd = calendar_to_jd(1600, 6, 15.25);
        let legacy = delta_t_seconds_with_model(jd, DeltaTModel::LegacyEspenakMeeus2006);
        let smh = delta_t_seconds_with_model(jd, DeltaTModel::Smh2016WithPre720Quadratic);
        assert_eq!(legacy.1, smh.1);
        assert!(
            (legacy.0 - smh.0).abs() < 1e-12,
            "legacy/smh placeholder mismatch: legacy={} smh={}",
            legacy.0,
            smh.0
        );
    }

    #[test]
    fn smh_model_uses_pre_minus_720_quadratic() {
        let jd = calendar_to_jd(-1000, 1, 1.0);
        let legacy = delta_t_seconds_with_model(jd, DeltaTModel::LegacyEspenakMeeus2006);
        let smh = delta_t_seconds_with_model(jd, DeltaTModel::Smh2016WithPre720Quadratic);
        assert_eq!(smh.1, DeltaTSegment::PreMinus720Quadratic);
        assert!(
            (smh.0 - legacy.0).abs() > 1e-6,
            "expected pre-720 quadratic to differ from legacy at year -1000; both={}",
            smh.0
        );
    }

    #[test]
    fn parse_smh_table_and_interpolate() {
        let txt = r#"
# year,delta_t
-720 17100
-719 17090
-718 17080
"#;
        let table = parse_smh2016_reconstruction(txt).unwrap();
        let dt = table.delta_t_at_year_linear(-719.5).unwrap();
        assert!((dt - 17095.0).abs() < 1e-9);
    }

    #[test]
    fn parse_smh_segments_and_evaluate() {
        let txt = r#"
# Ki Ki+1 a0 a1 a2 a3
-720 400 20550.593 -21268.478 11863.418 -4541.129
"#;
        let table = parse_smh2016_reconstruction(txt).unwrap();
        let dt0 = table.delta_t_at_year(-720.0).unwrap();
        let dt1 = table.delta_t_at_year(400.0).unwrap();
        assert!((dt0 - 20550.593).abs() < 1e-6);
        // At t=1, polynomial sum should match endpoint in this segment.
        let expected = 20550.593 - 21268.478 + 11863.418 - 4541.129;
        assert!((dt1 - expected).abs() < 1e-6);
    }

    #[test]
    fn smh_model_uses_reconstruction_when_available() {
        let table =
            Smh2016Reconstruction::from_points(vec![(-720.0, 10.0), (1961.0, 20.0)]).unwrap();
        let jd = calendar_to_jd(1000, 1, 1.0);
        let (dt, seg) = delta_t_seconds_smh_model(jd, Some(&table));
        assert_eq!(seg, DeltaTSegment::Smh2016Reconstruction);
        assert!(dt >= 10.0 && dt <= 20.0);
    }

    #[test]
    fn asymptotic_future_segment_exposed() {
        let jd = calendar_to_jd(2500, 1, 1.0);
        let (_dt, seg) = smh_asymptotic_delta_t_seconds_for_jd(jd);
        assert_eq!(seg, DeltaTSegment::SmhAsymptoticFuture);
    }

    #[test]
    fn smh_addendum_piecewise_c_switches_by_year_band() {
        assert_eq!(
            smh_future_parabola_c_for_year(2018.999, SmhFutureParabolaFamily::default()),
            -20.0
        );
        assert_eq!(
            smh_future_parabola_c_for_year(2019.0, SmhFutureParabolaFamily::default()),
            -17.52
        );
        assert_eq!(
            smh_future_parabola_c_for_year(3000.0, SmhFutureParabolaFamily::default()),
            -15.32
        );
    }

    #[test]
    fn smh_forced_family_members_shift_delta_t_as_expected() {
        let year = 2500.0;
        let dt20 = smh_asymptotic_delta_t_seconds_with_family(
            year,
            SmhFutureParabolaFamily::ConstantCMinus20,
        );
        let dt17 = smh_asymptotic_delta_t_seconds_with_family(
            year,
            SmhFutureParabolaFamily::ConstantCMinus17p52,
        );
        let dt15 = smh_asymptotic_delta_t_seconds_with_family(
            year,
            SmhFutureParabolaFamily::ConstantCMinus15p32,
        );
        assert!((dt17 - dt20 - 2.48).abs() < 1e-12);
        assert!((dt15 - dt17 - 2.20).abs() < 1e-12);
    }

    #[test]
    fn stephenson1997_future_formula_matches_reference() {
        let year = 2500.0;
        let dt = smh_asymptotic_delta_t_seconds_with_family(
            year,
            SmhFutureParabolaFamily::Stephenson1997,
        );
        let t = (year - 1820.0) / 100.0;
        let expected = -20.0 + 31.0 * t * t;
        assert!((dt - expected).abs() < 1e-12);
    }

    #[test]
    fn stephenson1997_future_differs_from_addendum2020_family() {
        let year = 2500.0;
        let dt_stephenson = smh_asymptotic_delta_t_seconds_with_family(
            year,
            SmhFutureParabolaFamily::Stephenson1997,
        );
        let dt_addendum = smh_asymptotic_delta_t_seconds_with_family(
            year,
            SmhFutureParabolaFamily::Addendum2020Piecewise,
        );
        assert!(
            (dt_stephenson - dt_addendum).abs() > 1e-6,
            "expected distinct future strategy outputs"
        );
    }
}
