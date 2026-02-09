//! Error types for kernel parsing and evaluation.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors produced during kernel loading or segment evaluation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum KernelError {
    /// File is too small or truncated.
    FileTooSmall { expected: usize, actual: usize },
    /// The file ID word is not a recognised DAF type.
    BadFileId(String),
    /// Endianness marker is unrecognised.
    BadEndianness(String),
    /// A summary record is internally inconsistent.
    BadSummaryRecord(String),
    /// The requested segment data type is not implemented.
    UnsupportedDataType(i32),
    /// No segment covers the requested (target, center) pair.
    SegmentNotFound { target: i32, center: i32 },
    /// The epoch is outside the time range of all matching segments.
    EpochOutOfRange {
        target: i32,
        center: i32,
        epoch_tdb_s: f64,
    },
    /// A segment's internal metadata is inconsistent.
    BadSegmentData(String),
    /// I/O error message (we store the string, not the io::Error, to keep Clone + PartialEq).
    Io(String),
}

impl Display for KernelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileTooSmall { expected, actual } => {
                write!(f, "file too small: need {expected} bytes, got {actual}")
            }
            Self::BadFileId(id) => write!(f, "bad DAF file ID: {id:?}"),
            Self::BadEndianness(s) => write!(f, "bad endianness marker: {s:?}"),
            Self::BadSummaryRecord(msg) => write!(f, "bad summary record: {msg}"),
            Self::UnsupportedDataType(dt) => write!(f, "unsupported SPK data type: {dt}"),
            Self::SegmentNotFound { target, center } => {
                write!(f, "no segment for target={target} center={center}")
            }
            Self::EpochOutOfRange {
                target,
                center,
                epoch_tdb_s,
            } => {
                write!(
                    f,
                    "epoch {epoch_tdb_s} s out of range for target={target} center={center}"
                )
            }
            Self::BadSegmentData(msg) => write!(f, "bad segment data: {msg}"),
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl Error for KernelError {}

impl From<std::io::Error> for KernelError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
