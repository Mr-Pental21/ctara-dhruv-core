//! JPL kernel parsing and interpolation primitives.
//!
//! This crate reads NAIF DAF/SPK binary kernel files and evaluates
//! Chebyshev polynomial segments to produce position and velocity
//! state vectors.
//!
//! Reference: NAIF DAF/SPK Required Reading documents (public domain,
//! US Government work product). Implementation is original, written
//! from the public specifications.

pub mod chebyshev;
pub mod daf;
pub mod error;
pub mod spk;

use std::path::Path;

pub use error::KernelError;
pub use spk::{SpkEvaluation, SpkSegment};

/// Map a planet body code (x99) to its parent barycenter (x).
///
/// In DE ephemerides, planets without dedicated segments (Mars 499,
/// Jupiter 599, etc.) are treated as coincident with their barycenter.
/// This function returns the barycenter code, or the input unchanged
/// if the code is not a planet body (x99).
pub fn planet_body_to_barycenter(code: i32) -> i32 {
    // Planet body codes are x99 where x is the barycenter number (1–9).
    // Special cases: Sun=10, Moon=301, Earth=399 — these have their own segments.
    if code >= 100 && code % 100 == 99 {
        code / 100
    } else {
        code
    }
}

/// A loaded SPK kernel, ready for evaluation.
#[derive(Debug, Clone)]
pub struct SpkKernel {
    data: Vec<u8>,
    endianness: daf::Endianness,
    segments: Vec<SpkSegment>,
}

impl SpkKernel {
    /// Load an SPK kernel from a file path.
    ///
    /// Reads the entire file into memory, parses the DAF file record
    /// and summary records, and indexes all SPK segments.
    pub fn load(path: &Path) -> Result<Self, KernelError> {
        let data = std::fs::read(path)?;
        Self::from_bytes(data)
    }

    /// Load an SPK kernel from raw bytes (useful for testing).
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, KernelError> {
        let file_record = daf::parse_file_record(&data)?;

        if file_record.nd != 2 || file_record.ni != 6 {
            return Err(KernelError::BadFileId(format!(
                "expected SPK (ND=2, NI=6), got ND={}, NI={}",
                file_record.nd, file_record.ni
            )));
        }

        let summaries = daf::read_summaries(&data, &file_record)?;
        let mut segments = Vec::with_capacity(summaries.len());
        for summary in &summaries {
            segments.push(spk::segment_from_summary(summary)?);
        }

        Ok(Self {
            data,
            endianness: file_record.endianness,
            segments,
        })
    }

    /// All segments in this kernel.
    pub fn segments(&self) -> &[SpkSegment] {
        &self.segments
    }

    /// Find the first segment matching `target` whose center field matches,
    /// and whose time range covers `epoch_tdb_s`.
    fn find_segment(
        &self,
        target: i32,
        center: i32,
        epoch_tdb_s: f64,
    ) -> Result<&SpkSegment, KernelError> {
        self.segments
            .iter()
            .find(|seg| {
                seg.target == target
                    && seg.center == center
                    && epoch_tdb_s >= seg.start_epoch
                    && epoch_tdb_s <= seg.end_epoch
            })
            .ok_or(KernelError::EpochOutOfRange {
                target,
                center,
                epoch_tdb_s,
            })
    }

    /// Evaluate a segment for the given (target, center) pair at the epoch.
    ///
    /// `epoch_tdb_s` is TDB seconds past J2000.0.
    ///
    /// Returns position in km and velocity in km/s in the segment's native
    /// reference frame (typically ICRF/J2000 for DE kernels).
    pub fn evaluate(
        &self,
        target: i32,
        center: i32,
        epoch_tdb_s: f64,
    ) -> Result<SpkEvaluation, KernelError> {
        let segment = self.find_segment(target, center, epoch_tdb_s)?;

        match segment.data_type {
            2 => spk::evaluate_type2(&self.data, segment, epoch_tdb_s, self.endianness),
            other => Err(KernelError::UnsupportedDataType(other)),
        }
    }

    /// Look up the center body for a given target by inspecting segments.
    ///
    /// Returns `None` if no segment with that target is found.
    pub fn center_for(&self, target: i32) -> Option<i32> {
        self.segments
            .iter()
            .find(|seg| seg.target == target)
            .map(|seg| seg.center)
    }

    /// Resolve a body to SSB (code 0) by walking the segment chain,
    /// accumulating position and velocity.
    ///
    /// If a planet body code (x99) has no segment, it falls back to
    /// the planet's barycenter (x). This matches DE kernel conventions
    /// where Mars(499)=MarsBary(4), Jupiter(599)=JupiterBary(5), etc.
    ///
    /// Returns `[x, y, z, vx, vy, vz]` in km and km/s.
    pub fn resolve_to_ssb(
        &self,
        body_code: i32,
        epoch_tdb_s: f64,
    ) -> Result<[f64; 6], KernelError> {
        let mut code = body_code;
        let mut state = [0.0f64; 6];

        while code != 0 {
            let center = match self.center_for(code) {
                Some(c) => c,
                None => {
                    // Planet body x99 → barycenter x (e.g. 499→4, 599→5).
                    // If the barycenter also has no segment, error out.
                    let bary = planet_body_to_barycenter(code);
                    if bary != code {
                        code = bary;
                        continue;
                    }
                    return Err(KernelError::SegmentNotFound {
                        target: code,
                        center: -1,
                    });
                }
            };

            let eval = self.evaluate(code, center, epoch_tdb_s)?;
            state[0] += eval.position_km[0];
            state[1] += eval.position_km[1];
            state[2] += eval.position_km[2];
            state[3] += eval.velocity_km_s[0];
            state[4] += eval.velocity_km_s[1];
            state[5] += eval.velocity_km_s[2];

            code = center;
        }

        Ok(state)
    }
}
