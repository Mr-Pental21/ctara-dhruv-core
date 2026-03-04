//! IERS Earth Orientation Parameters (EOP) — UT1−UTC lookup.
//!
//! Supports:
//! - `finals2000A` fixed-width parsing (historical + rapid + predicted tail)
//! - `C04` whitespace-format parsing (final series; 1962+)
//! - deterministic merged precedence for overlaps:
//!   1. C04 final
//!   2. finals2000A `I`
//!   3. finals2000A `P`
//!
//! The merged table is interpolated linearly in MJD space.

use crate::diagnostics::TimeWarning;
use crate::error::TimeError;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static EOP_PRE_RANGE_WARNED: AtomicBool = AtomicBool::new(false);
static EOP_FUTURE_WARNED: AtomicBool = AtomicBool::new(false);

/// Source used for a DUT1 value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EopSource {
    C04Final,
    FinalsFinal,
    FinalsPredicted,
    FallbackPreRange,
    FallbackFutureFrozen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct EopEntry {
    mjd: f64,
    dut1: f64,
    source: EopSource,
}

/// Options for DUT1 lookup behavior when date is outside table coverage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EopLookupOptions {
    /// Freeze DUT1 at last known value for future dates.
    pub freeze_future_dut1: bool,
    /// Use this DUT1 value for pre-range dates.
    pub pre_range_dut1: f64,
    /// Emit warnings in the lookup result.
    pub warn_on_fallback: bool,
}

impl Default for EopLookupOptions {
    fn default() -> Self {
        Self {
            freeze_future_dut1: true,
            pre_range_dut1: 0.0,
            warn_on_fallback: true,
        }
    }
}

/// DUT1 lookup result with diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct Dut1LookupResult {
    pub dut1_seconds: f64,
    pub source: EopSource,
    pub warnings: Vec<TimeWarning>,
}

/// Parsed IERS Earth Orientation Parameters (DUT1 lookup table).
#[derive(Debug, Clone)]
pub struct EopData {
    /// Daily entries sorted ascending by MJD.
    entries: Vec<EopEntry>,
    /// Last MJD whose selected source is a predicted finals row.
    prediction_end_mjd: Option<f64>,
}

impl EopData {
    /// Parse IERS finals2000A fixed-width format.
    ///
    /// Extracts MJD (col 8-15), source flag (col 58), and DUT1 (col 59-68).
    pub fn parse_finals(content: &str) -> Result<Self, TimeError> {
        let entries = Self::parse_finals_entries(content)?;
        Ok(Self::from_sorted_entries(entries))
    }

    /// Parse IERS C04 format (whitespace-delimited operational/reprocessed files).
    ///
    /// Expected row head: `year month day mjd x y ut1_utc ...`
    pub fn parse_c04(content: &str) -> Result<Self, TimeError> {
        let entries = Self::parse_c04_entries(content)?;
        Ok(Self::from_sorted_entries(entries))
    }

    /// Build merged EOP table from finals primary plus optional C04 and daily finals files.
    ///
    /// Precedence at identical MJD (deterministic):
    /// 1. C04 final
    /// 2. finals `I`
    /// 3. finals `P`
    pub fn parse_merged(
        finals_primary_content: &str,
        c04_content: Option<&str>,
        finals_daily_content: Option<&str>,
    ) -> Result<Self, TimeError> {
        let mut by_mjd_key: BTreeMap<i64, EopEntry> = BTreeMap::new();

        let mut ingest = |entries: Vec<EopEntry>| {
            for mut entry in entries {
                let key = (entry.mjd * 100.0).round() as i64;
                entry.mjd = key as f64 / 100.0;
                match by_mjd_key.get(&key).copied() {
                    Some(current) if source_rank(current.source) <= source_rank(entry.source) => {}
                    _ => {
                        by_mjd_key.insert(key, entry);
                    }
                }
            }
        };

        ingest(Self::parse_finals_entries(finals_primary_content)?);
        if let Some(content) = finals_daily_content {
            ingest(Self::parse_finals_entries(content)?);
        }
        if let Some(content) = c04_content {
            ingest(Self::parse_c04_entries(content)?);
        }

        if by_mjd_key.is_empty() {
            return Err(TimeError::EopParse(
                "no valid DUT1 entries found in merged sources".to_string(),
            ));
        }

        let mut entries = Vec::with_capacity(by_mjd_key.len());
        for (_k, v) in by_mjd_key {
            entries.push(v);
        }
        Ok(Self::from_sorted_entries(entries))
    }

    fn parse_finals_entries(content: &str) -> Result<Vec<EopEntry>, TimeError> {
        let mut entries = Vec::new();

        for line in content.lines() {
            let bytes = line.as_bytes();
            if bytes.len() < 68 {
                continue;
            }

            let mjd: f64 = match line[7..15].trim().parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let dut1: f64 = match line[58..68].trim().parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let flag = bytes[57] as char;
            let source = if flag == 'P' {
                EopSource::FinalsPredicted
            } else {
                EopSource::FinalsFinal
            };

            entries.push(EopEntry { mjd, dut1, source });
        }

        if entries.is_empty() {
            return Err(TimeError::EopParse(
                "no valid DUT1 entries found".to_string(),
            ));
        }
        entries.sort_by(|a, b| {
            a.mjd
                .partial_cmp(&b.mjd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(entries)
    }

    fn parse_c04_entries(content: &str) -> Result<Vec<EopEntry>, TimeError> {
        let mut entries = Vec::new();

        for line in content.lines() {
            let t = line.trim();
            if t.is_empty()
                || t.starts_with('#')
                || t.starts_with('!')
                || t.starts_with('*')
                || t.starts_with("DATE")
            {
                continue;
            }
            let cols: Vec<&str> = t.split_whitespace().collect();
            if cols.len() < 7 {
                continue;
            }
            if cols[0].parse::<i32>().is_err()
                || cols[1].parse::<u32>().is_err()
                || cols[2].parse::<u32>().is_err()
            {
                continue;
            }

            let mjd: f64 = match cols[3].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let dut1: f64 = match cols[6].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            entries.push(EopEntry {
                mjd,
                dut1,
                source: EopSource::C04Final,
            });
        }

        if entries.is_empty() {
            return Err(TimeError::EopParse(
                "no valid C04 DUT1 entries found".to_string(),
            ));
        }
        entries.sort_by(|a, b| {
            a.mjd
                .partial_cmp(&b.mjd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(entries)
    }

    fn from_sorted_entries(mut entries: Vec<EopEntry>) -> Self {
        entries.sort_by(|a, b| {
            a.mjd
                .partial_cmp(&b.mjd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let prediction_end_mjd = entries
            .iter()
            .filter(|e| e.source == EopSource::FinalsPredicted)
            .map(|e| e.mjd)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Self {
            entries,
            prediction_end_mjd,
        }
    }

    /// Number of entries in the table.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// MJD range covered by the table: (first, last).
    pub fn range(&self) -> (f64, f64) {
        (
            self.entries[0].mjd,
            self.entries[self.entries.len() - 1].mjd,
        )
    }

    /// Last MJD where selected source is finals predicted (`P`) if present.
    pub fn prediction_end_mjd(&self) -> Option<f64> {
        self.prediction_end_mjd
    }

    fn interpolate_dut1_and_source(&self, mjd: f64) -> (f64, EopSource) {
        let idx = self
            .entries
            .partition_point(|e| e.mjd < mjd)
            .saturating_sub(1);

        if idx + 1 >= self.entries.len() {
            let e = self.entries[idx];
            return (e.dut1, e.source);
        }

        let e0 = self.entries[idx];
        let e1 = self.entries[idx + 1];
        if (e1.mjd - e0.mjd).abs() < 1e-12 {
            return (e0.dut1, e0.source);
        }

        let frac = (mjd - e0.mjd) / (e1.mjd - e0.mjd);
        let dut1 = e0.dut1 + frac * (e1.dut1 - e0.dut1);
        let source = if frac < 0.5 { e0.source } else { e1.source };
        (dut1, source)
    }

    /// DUT1 (UT1−UTC) in seconds at a given MJD, linearly interpolated.
    ///
    /// Default behavior:
    /// - pre-range dates use `DUT1=0` fallback
    /// - future out-of-range dates freeze to last known DUT1.
    pub fn dut1_at_mjd(&self, mjd: f64) -> Result<f64, TimeError> {
        let out = self.dut1_at_mjd_with_options(mjd, EopLookupOptions::default())?;
        for warning in &out.warnings {
            emit_warning_once(warning);
        }
        Ok(out.dut1_seconds)
    }

    /// Strict lookup:
    /// - pre-range dates use `DUT1=0` fallback
    /// - future out-of-range dates return `TimeError::EopOutOfRange`.
    pub fn dut1_at_mjd_strict(&self, mjd: f64) -> Result<f64, TimeError> {
        let (mjd_start, mjd_end) = self.range();
        if mjd < mjd_start {
            return Ok(0.0);
        }
        if mjd > mjd_end {
            return Err(TimeError::EopOutOfRange);
        }

        let (dut1, _source) = self.interpolate_dut1_and_source(mjd);
        Ok(dut1)
    }

    /// DUT1 lookup with configurable fallback policy and diagnostics.
    pub fn dut1_at_mjd_with_options(
        &self,
        mjd: f64,
        options: EopLookupOptions,
    ) -> Result<Dut1LookupResult, TimeError> {
        let (mjd_start, mjd_end) = self.range();
        let mut warnings = Vec::new();

        if mjd < mjd_start {
            if options.warn_on_fallback {
                warnings.push(TimeWarning::EopPreRangeFallback {
                    mjd,
                    first_entry_mjd: mjd_start,
                    used_dut1_seconds: options.pre_range_dut1,
                });
            }
            return Ok(Dut1LookupResult {
                dut1_seconds: options.pre_range_dut1,
                source: EopSource::FallbackPreRange,
                warnings,
            });
        }

        if mjd > mjd_end {
            if !options.freeze_future_dut1 {
                return Err(TimeError::EopOutOfRange);
            }
            let last = self.entries[self.entries.len() - 1];
            if options.warn_on_fallback {
                warnings.push(TimeWarning::EopFutureFrozen {
                    mjd,
                    last_entry_mjd: mjd_end,
                    used_dut1_seconds: last.dut1,
                });
            }
            return Ok(Dut1LookupResult {
                dut1_seconds: last.dut1,
                source: EopSource::FallbackFutureFrozen,
                warnings,
            });
        }

        let (dut1, source) = self.interpolate_dut1_and_source(mjd);
        Ok(Dut1LookupResult {
            dut1_seconds: dut1,
            source,
            warnings,
        })
    }

    /// Convert a UTC Julian Date to a UT1 Julian Date.
    ///
    /// `jd_ut1 = jd_utc + dut1 / 86400.0`
    pub fn utc_to_ut1_jd(&self, jd_utc: f64) -> Result<f64, TimeError> {
        let mjd = jd_utc - 2_400_000.5;
        let out = self.dut1_at_mjd_with_options(mjd, EopLookupOptions::default())?;
        for warning in &out.warnings {
            emit_warning_once(warning);
        }
        Ok(jd_utc + out.dut1_seconds / 86_400.0)
    }

    /// Strict UTC->UT1 conversion (errors for future out-of-range dates).
    pub fn utc_to_ut1_jd_strict(&self, jd_utc: f64) -> Result<f64, TimeError> {
        let mjd = jd_utc - 2_400_000.5;
        let dut1 = self.dut1_at_mjd_strict(mjd)?;
        Ok(jd_utc + dut1 / 86_400.0)
    }

    /// UTC->UT1 conversion with configurable fallback and diagnostics.
    pub fn utc_to_ut1_jd_with_options(
        &self,
        jd_utc: f64,
        options: EopLookupOptions,
    ) -> Result<(f64, Vec<TimeWarning>), TimeError> {
        let mjd = jd_utc - 2_400_000.5;
        let out = self.dut1_at_mjd_with_options(mjd, options)?;
        Ok((jd_utc + out.dut1_seconds / 86_400.0, out.warnings))
    }
}

fn source_rank(source: EopSource) -> u8 {
    match source {
        EopSource::C04Final => 0,
        EopSource::FinalsFinal => 1,
        EopSource::FinalsPredicted => 2,
        EopSource::FallbackPreRange | EopSource::FallbackFutureFrozen => 3,
    }
}

fn emit_warning_once(warning: &TimeWarning) {
    match warning {
        TimeWarning::EopPreRangeFallback { .. } => {
            if !EOP_PRE_RANGE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("Warning: {warning}");
            }
        }
        TimeWarning::EopFutureFrozen { .. } => {
            if !EOP_FUTURE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("Warning: {warning}");
            }
        }
        _ => {}
    }
}

/// Loaded IERS EOP file(s), ready for UT1 conversions.
///
/// Follows the same load-from-file pattern as [`crate::LeapSecondKernel`].
#[derive(Debug, Clone)]
pub struct EopKernel {
    data: EopData,
}

impl EopKernel {
    /// Load a finals2000A file from disk.
    pub fn load(path: &Path) -> Result<Self, TimeError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse finals2000A from string content.
    pub fn parse(content: &str) -> Result<Self, TimeError> {
        let data = EopData::parse_finals(content)?;
        Ok(Self { data })
    }

    /// Load merged EOP sources:
    /// - required `finals_primary_path` (typically `finals2000A.all`)
    /// - optional `c04_path`
    /// - optional `finals_daily_path` (e.g., `finals2000A.daily.extended`)
    pub fn load_merged(
        finals_primary_path: &Path,
        c04_path: Option<&Path>,
        finals_daily_path: Option<&Path>,
    ) -> Result<Self, TimeError> {
        let finals_primary = std::fs::read_to_string(finals_primary_path)?;
        let c04 = if let Some(p) = c04_path {
            Some(std::fs::read_to_string(p)?)
        } else {
            None
        };
        let daily = if let Some(p) = finals_daily_path {
            Some(std::fs::read_to_string(p)?)
        } else {
            None
        };
        Self::parse_merged(&finals_primary, c04.as_deref(), daily.as_deref())
    }

    /// Parse merged sources from content strings.
    pub fn parse_merged(
        finals_primary_content: &str,
        c04_content: Option<&str>,
        finals_daily_content: Option<&str>,
    ) -> Result<Self, TimeError> {
        let data =
            EopData::parse_merged(finals_primary_content, c04_content, finals_daily_content)?;
        Ok(Self { data })
    }

    /// Access the parsed EOP data.
    pub fn data(&self) -> &EopData {
        &self.data
    }

    /// Convert a UTC Julian Date to a UT1 Julian Date.
    pub fn utc_to_ut1_jd(&self, jd_utc: f64) -> Result<f64, TimeError> {
        self.data.utc_to_ut1_jd(jd_utc)
    }

    /// Strict UTC->UT1 conversion (errors for future out-of-range dates).
    pub fn utc_to_ut1_jd_strict(&self, jd_utc: f64) -> Result<f64, TimeError> {
        self.data.utc_to_ut1_jd_strict(jd_utc)
    }

    /// UTC->UT1 conversion with configurable fallback and diagnostics.
    pub fn utc_to_ut1_jd_with_options(
        &self,
        jd_utc: f64,
        options: EopLookupOptions,
    ) -> Result<(f64, Vec<TimeWarning>), TimeError> {
        self.data.utc_to_ut1_jd_with_options(jd_utc, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finals_line(mjd: f64, flag: char, dut1: f64) -> String {
        let mut line = vec![b' '; 70];
        let mjd_s = format!("{mjd:8.2}");
        line[7..15].copy_from_slice(mjd_s.as_bytes());
        line[57] = flag as u8;
        let dut1_s = format!("{dut1:10.7}");
        line[58..68].copy_from_slice(dut1_s.as_bytes());
        String::from_utf8(line).unwrap()
    }

    // A minimal hand-crafted finals snippet (3 lines).
    fn finals_snippet() -> String {
        vec![
            finals_line(60000.0, 'I', 0.1234567),
            finals_line(60001.0, 'I', 0.2345678),
            finals_line(60002.0, 'P', -0.1000000),
        ]
        .join("\n")
    }

    fn c04_snippet() -> String {
        // year month day mjd x y ut1_utc ...
        [
            "1962 01 01 37665 0.0 0.0 0.300000 0 0 0 0 0",
            "1962 01 02 37666 0.0 0.0 0.310000 0 0 0 0 0",
        ]
        .join("\n")
    }

    #[test]
    fn parse_small_finals_snippet() {
        let data = EopData::parse_finals(&finals_snippet()).unwrap();
        assert_eq!(data.len(), 3);
        let (start, end) = data.range();
        assert!((start - 60000.0).abs() < 0.01);
        assert!((end - 60002.0).abs() < 0.01);
        assert_eq!(data.prediction_end_mjd(), Some(60002.0));
    }

    #[test]
    fn parse_small_c04_snippet() {
        let data = EopData::parse_c04(&c04_snippet()).unwrap();
        assert_eq!(data.len(), 2);
        let (start, end) = data.range();
        assert!((start - 37665.0).abs() < 0.01);
        assert!((end - 37666.0).abs() < 0.01);
        assert_eq!(data.prediction_end_mjd(), None);
    }

    #[test]
    fn interpolation_exact() {
        let data = EopData::parse_finals(&finals_snippet()).unwrap();
        let dut1 = data.dut1_at_mjd(60000.0).unwrap();
        assert!((dut1 - 0.1234567).abs() < 1e-7);
    }

    #[test]
    fn interpolation_midpoint() {
        let data = EopData::parse_finals(&finals_snippet()).unwrap();
        let dut1 = data.dut1_at_mjd(60000.5).unwrap();
        let expected = (0.1234567 + 0.2345678) / 2.0;
        assert!(
            (dut1 - expected).abs() < 1e-7,
            "midpoint: got {dut1}, expected {expected}"
        );
    }

    #[test]
    fn out_of_range_before() {
        let data = EopData::parse_finals(&finals_snippet()).unwrap();
        assert_eq!(data.dut1_at_mjd(59999.0), Ok(0.0));
    }

    #[test]
    fn out_of_range_after() {
        let data = EopData::parse_finals(&finals_snippet()).unwrap();
        let dut1 = data.dut1_at_mjd(60003.0).unwrap();
        assert!((dut1 - (-0.1000000)).abs() < 1e-7);
    }

    #[test]
    fn out_of_range_after_strict() {
        let data = EopData::parse_finals(&finals_snippet()).unwrap();
        assert_eq!(
            data.dut1_at_mjd_strict(60003.0),
            Err(TimeError::EopOutOfRange)
        );
    }

    #[test]
    fn blank_dut1_skipped() {
        let mut snippet = finals_snippet();
        let mut blank_line = vec![b' '; 70];
        let mjd_s = format!("{:8.2}", 60003.00);
        blank_line[7..15].copy_from_slice(mjd_s.as_bytes());
        snippet.push('\n');
        snippet.push_str(&String::from_utf8(blank_line).unwrap());

        let data = EopData::parse_finals(&snippet).unwrap();
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn utc_to_ut1_offset() {
        let data = EopData::parse_finals(&finals_snippet()).unwrap();
        let dut1 = data.dut1_at_mjd(60000.0).unwrap();
        assert!((dut1 - 0.1234567).abs() < 1e-7, "dut1 = {dut1}");

        let jd_utc = 2_460_000.5;
        let jd_ut1 = data.utc_to_ut1_jd(jd_utc).unwrap();
        assert!(
            (jd_ut1 - jd_utc).abs() < 1e-5,
            "UT1 offset too large: {}",
            jd_ut1 - jd_utc
        );
        assert!(jd_ut1 > jd_utc, "UT1 should be ahead of UTC when DUT1 > 0");
    }

    #[test]
    fn merged_precedence_prefers_c04_over_finals_i_and_p() {
        let finals = vec![
            finals_line(60000.0, 'I', 0.2000000),
            finals_line(60001.0, 'P', 0.5000000),
        ]
        .join("\n");
        let c04 = "1962 01 01 60000 0.0 0.0 0.1000000 0 0 0 0 0\n1962 01 02 60001 0.0 0.0 0.1100000 0 0 0 0 0";
        let data = EopData::parse_merged(&finals, Some(c04), None).unwrap();
        assert_eq!(data.dut1_at_mjd(60000.0).unwrap(), 0.1);
        assert_eq!(data.dut1_at_mjd(60001.0).unwrap(), 0.11);
    }

    #[test]
    fn merged_uses_daily_predicted_when_primary_older() {
        let finals_primary = finals_line(61000.0, 'I', 0.1000000);
        let finals_daily = vec![
            finals_line(61000.0, 'I', 0.1000000),
            finals_line(61001.0, 'P', 0.1200000),
        ]
        .join("\n");
        let data = EopData::parse_merged(&finals_primary, None, Some(&finals_daily)).unwrap();
        assert_eq!(data.range().1, 61001.0);
        assert_eq!(data.prediction_end_mjd(), Some(61001.0));
    }
}
