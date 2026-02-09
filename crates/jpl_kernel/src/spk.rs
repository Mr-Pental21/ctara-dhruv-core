//! SPK (Spacecraft and Planet Kernel) segment types and evaluation.
//!
//! Reference: NAIF SPK Required Reading (public domain, US Government work product).
//! Implementation is original, written from the public specification.

use crate::chebyshev;
use crate::daf::{DafSummary, Endianness};
use crate::error::KernelError;

/// Metadata for a single SPK segment, extracted from a DAF summary.
#[derive(Debug, Clone)]
pub struct SpkSegment {
    pub start_epoch: f64,
    pub end_epoch: f64,
    pub target: i32,
    pub center: i32,
    pub frame: i32,
    pub data_type: i32,
    /// First word address (1-based, 8 bytes per word).
    pub start_addr: i32,
    /// Last word address (1-based, 8 bytes per word).
    pub end_addr: i32,
}

/// Result of evaluating an SPK segment at a single epoch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpkEvaluation {
    pub position_km: [f64; 3],
    pub velocity_km_s: [f64; 3],
}

/// SPK Type 2 segment descriptor (stored at the end of segment data).
#[derive(Debug, Clone, Copy)]
struct Type2Descriptor {
    init: f64,
    intlen: f64,
    rsize: f64,
    n: f64,
}

// ---------------------------------------------------------------------------
// Segment extraction from DAF summary
// ---------------------------------------------------------------------------

/// Convert a DAF summary (with ND=2, NI=6) into an SPK segment descriptor.
pub fn segment_from_summary(summary: &DafSummary) -> Result<SpkSegment, KernelError> {
    if summary.doubles.len() < 2 || summary.integers.len() < 6 {
        return Err(KernelError::BadSummaryRecord(
            "SPK summary requires ND>=2, NI>=6".into(),
        ));
    }

    Ok(SpkSegment {
        start_epoch: summary.doubles[0],
        end_epoch: summary.doubles[1],
        target: summary.integers[0],
        center: summary.integers[1],
        frame: summary.integers[2],
        data_type: summary.integers[3],
        start_addr: summary.integers[4],
        end_addr: summary.integers[5],
    })
}

// ---------------------------------------------------------------------------
// Byte helpers
// ---------------------------------------------------------------------------

fn read_f64(data: &[u8], offset: usize, endian: Endianness) -> f64 {
    let bytes: [u8; 8] = data[offset..offset + 8].try_into().unwrap();
    match endian {
        Endianness::Little => f64::from_le_bytes(bytes),
        Endianness::Big => f64::from_be_bytes(bytes),
    }
}

// ---------------------------------------------------------------------------
// Type 2 evaluation
// ---------------------------------------------------------------------------

/// Read the Type 2 descriptor from the last 4 doubles of the segment data.
fn read_type2_descriptor(
    data: &[u8],
    segment: &SpkSegment,
    endian: Endianness,
) -> Result<Type2Descriptor, KernelError> {
    // The descriptor occupies the last 4 doubles (32 bytes) of the segment.
    let end_byte = segment.end_addr as usize * 8;
    if end_byte > data.len() || end_byte < 32 {
        return Err(KernelError::BadSegmentData(
            "segment end_addr extends past file".into(),
        ));
    }
    let desc_offset = end_byte - 32; // 4 doubles * 8 bytes

    Ok(Type2Descriptor {
        init: read_f64(data, desc_offset, endian),
        intlen: read_f64(data, desc_offset + 8, endian),
        rsize: read_f64(data, desc_offset + 16, endian),
        n: read_f64(data, desc_offset + 24, endian),
    })
}

/// Evaluate an SPK Type 2 (Chebyshev position-only) segment.
///
/// Returns position (km) and velocity (km/s) in the segment's reference frame.
pub fn evaluate_type2(
    data: &[u8],
    segment: &SpkSegment,
    epoch_tdb_s: f64,
    endian: Endianness,
) -> Result<SpkEvaluation, KernelError> {
    let desc = read_type2_descriptor(data, segment, endian)?;

    let n = desc.n as usize;
    let rsize = desc.rsize as usize;
    let intlen = desc.intlen;

    if rsize < 3 || !(rsize - 2).is_multiple_of(3) {
        return Err(KernelError::BadSegmentData(format!(
            "invalid RSIZE {rsize}: must satisfy (RSIZE-2) mod 3 == 0"
        )));
    }
    let n_coeffs = (rsize - 2) / 3;

    // Find the record index.
    let record_index = ((epoch_tdb_s - desc.init) / intlen).floor() as usize;
    let record_index = record_index.min(n.saturating_sub(1));

    // Byte offset of this record within the file.
    let seg_start_byte = (segment.start_addr as usize - 1) * 8;
    let record_byte = seg_start_byte + record_index * rsize * 8;

    if record_byte + rsize * 8 > data.len() {
        return Err(KernelError::BadSegmentData(
            "record extends past end of file".into(),
        ));
    }

    // Read MID and RADIUS.
    let mid = read_f64(data, record_byte, endian);
    let radius = read_f64(data, record_byte + 8, endian);

    if radius == 0.0 {
        return Err(KernelError::BadSegmentData("RADIUS is zero".into()));
    }

    // Normalised time in [-1, 1].
    let s = (epoch_tdb_s - mid) / radius;

    // Read coefficients for X, Y, Z and evaluate.
    let coeff_base = record_byte + 16; // skip MID + RADIUS

    let mut position_km = [0.0f64; 3];
    let mut velocity_km_s = [0.0f64; 3];

    for axis in 0..3 {
        let axis_offset = coeff_base + axis * n_coeffs * 8;
        let mut coeffs = Vec::with_capacity(n_coeffs);
        for c in 0..n_coeffs {
            coeffs.push(read_f64(data, axis_offset + c * 8, endian));
        }

        position_km[axis] = chebyshev::clenshaw(&coeffs, s);
        velocity_km_s[axis] = chebyshev::clenshaw_derivative(&coeffs, s) / radius;
    }

    Ok(SpkEvaluation {
        position_km,
        velocity_km_s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_from_summary_rejects_short() {
        let summary = DafSummary {
            doubles: vec![0.0],
            integers: vec![1, 2, 3],
        };
        assert!(segment_from_summary(&summary).is_err());
    }

    #[test]
    fn segment_from_summary_roundtrip() {
        let summary = DafSummary {
            doubles: vec![-1e9, 1e9],
            integers: vec![499, 4, 1, 2, 100, 200],
        };
        let seg = segment_from_summary(&summary).unwrap();
        assert_eq!(seg.target, 499);
        assert_eq!(seg.center, 4);
        assert_eq!(seg.data_type, 2);
        assert_eq!(seg.start_addr, 100);
        assert_eq!(seg.end_addr, 200);
    }
}
