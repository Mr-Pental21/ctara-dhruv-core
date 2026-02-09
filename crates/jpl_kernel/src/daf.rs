//! DAF (Double precision Array File) binary format parser.
//!
//! Reference: NAIF DAF Required Reading (public domain, US Government work product).
//! Implementation is original, written from the public specification.

use crate::error::KernelError;

/// Size of every DAF record in bytes.
const RECORD_BYTES: usize = 1024;

/// Byte-order marker for little-endian DAF files.
const LTL_IEEE: &[u8; 8] = b"LTL-IEEE";

/// Byte-order marker for big-endian DAF files.
const BIG_IEEE: &[u8; 8] = b"BIG-IEEE";

/// Detected byte order of a DAF file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

/// Parsed DAF file record (the first 1024-byte record).
#[derive(Debug, Clone)]
pub struct FileRecord {
    pub file_id: String,
    pub nd: i32,
    pub ni: i32,
    pub internal_name: String,
    pub fward: i32,
    pub bward: i32,
    pub free: i32,
    pub endianness: Endianness,
}

/// A single DAF summary, containing the double and integer components.
#[derive(Debug, Clone)]
pub struct DafSummary {
    pub doubles: Vec<f64>,
    pub integers: Vec<i32>,
}

// ---------------------------------------------------------------------------
// Byte-reading helpers
// ---------------------------------------------------------------------------

fn read_f64(data: &[u8], offset: usize, endian: Endianness) -> f64 {
    let bytes: [u8; 8] = data[offset..offset + 8].try_into().unwrap();
    match endian {
        Endianness::Little => f64::from_le_bytes(bytes),
        Endianness::Big => f64::from_be_bytes(bytes),
    }
}

fn read_i32(data: &[u8], offset: usize, endian: Endianness) -> i32 {
    let bytes: [u8; 4] = data[offset..offset + 4].try_into().unwrap();
    match endian {
        Endianness::Little => i32::from_le_bytes(bytes),
        Endianness::Big => i32::from_be_bytes(bytes),
    }
}

// ---------------------------------------------------------------------------
// File record
// ---------------------------------------------------------------------------

/// Parse the DAF file record (record 1, bytes 0..1024).
pub fn parse_file_record(data: &[u8]) -> Result<FileRecord, KernelError> {
    if data.len() < RECORD_BYTES {
        return Err(KernelError::FileTooSmall {
            expected: RECORD_BYTES,
            actual: data.len(),
        });
    }

    // Endianness from LOCFMT at bytes 88..96.
    let locfmt = &data[88..96];
    let endianness = if locfmt == LTL_IEEE {
        Endianness::Little
    } else if locfmt == BIG_IEEE {
        Endianness::Big
    } else {
        let s = String::from_utf8_lossy(locfmt).to_string();
        return Err(KernelError::BadEndianness(s));
    };

    let file_id = String::from_utf8_lossy(&data[0..8]).trim().to_string();
    let nd = read_i32(data, 8, endianness);
    let ni = read_i32(data, 12, endianness);
    let internal_name = String::from_utf8_lossy(&data[16..76]).trim().to_string();
    let fward = read_i32(data, 76, endianness);
    let bward = read_i32(data, 80, endianness);
    let free = read_i32(data, 84, endianness);

    if !file_id.starts_with("DAF/") {
        return Err(KernelError::BadFileId(file_id));
    }

    Ok(FileRecord {
        file_id,
        nd,
        ni,
        internal_name,
        fward,
        bward,
        free,
        endianness,
    })
}

// ---------------------------------------------------------------------------
// Summary records
// ---------------------------------------------------------------------------

/// Summary size in doubles: ND + (NI + 1) / 2.
fn summary_size(nd: i32, ni: i32) -> usize {
    nd as usize + (ni as usize).div_ceil(2)
}

/// Walk the summary-record linked list and collect all summaries.
pub fn read_summaries(
    data: &[u8],
    file_record: &FileRecord,
) -> Result<Vec<DafSummary>, KernelError> {
    let nd = file_record.nd as usize;
    let ni = file_record.ni as usize;
    let ss = summary_size(file_record.nd, file_record.ni);
    let endian = file_record.endianness;

    let mut summaries = Vec::new();
    let mut record_num = file_record.fward as usize;

    while record_num != 0 {
        let rec_offset = (record_num - 1) * RECORD_BYTES;
        if rec_offset + RECORD_BYTES > data.len() {
            return Err(KernelError::BadSummaryRecord(format!(
                "summary record {record_num} extends past end of file"
            )));
        }

        let next = read_f64(data, rec_offset, endian);
        let nsum = read_f64(data, rec_offset + 16, endian) as usize;

        // Summaries start at double index 3 within the record (byte offset 24).
        for i in 0..nsum {
            let sum_offset = rec_offset + 24 + i * ss * 8;
            if sum_offset + ss * 8 > rec_offset + RECORD_BYTES {
                return Err(KernelError::BadSummaryRecord(format!(
                    "summary {i} in record {record_num} overflows record boundary"
                )));
            }

            // Read ND doubles.
            let mut doubles = Vec::with_capacity(nd);
            for d in 0..nd {
                doubles.push(read_f64(data, sum_offset + d * 8, endian));
            }

            // Read NI integers — packed into the bytes following the doubles.
            let int_base = sum_offset + nd * 8;
            let mut integers = Vec::with_capacity(ni);
            for j in 0..ni {
                integers.push(read_i32(data, int_base + j * 4, endian));
            }

            summaries.push(DafSummary { doubles, integers });
        }

        // Follow linked list. NEXT == 0.0 means end.
        record_num = next as usize;
    }

    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_size_spk() {
        // SPK: ND=2, NI=6 → SS = 2 + (6+1)/2 = 5
        assert_eq!(summary_size(2, 6), 5);
    }

    #[test]
    fn bad_endianness_rejected() {
        let mut data = vec![0u8; RECORD_BYTES];
        data[0..8].copy_from_slice(b"DAF/SPK ");
        data[88..96].copy_from_slice(b"UNKNOWN!");
        let result = parse_file_record(&data);
        assert!(matches!(result, Err(KernelError::BadEndianness(_))));
    }

    #[test]
    fn bad_file_id_rejected() {
        let mut data = vec![0u8; RECORD_BYTES];
        data[0..8].copy_from_slice(b"NOTADAF!");
        data[88..96].copy_from_slice(LTL_IEEE);
        let result = parse_file_record(&data);
        assert!(matches!(result, Err(KernelError::BadFileId(_))));
    }
}
