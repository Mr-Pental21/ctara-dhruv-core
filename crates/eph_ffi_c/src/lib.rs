//! C-facing adapter types for `ctara-eph-core`.

use std::path::PathBuf;
use std::ptr;

use eph_core::{Body, Engine, EngineConfig, EngineError, Frame, Observer, Query, StateVector};

/// ABI version for downstream bindings.
pub const EPH_API_VERSION: u32 = 2;

/// Fixed UTF-8 buffer size for path fields in C-compatible structs.
pub const EPH_PATH_CAPACITY: usize = 512;

/// Maximum number of SPK kernel paths in a C-compatible config.
pub const EPH_MAX_SPK_PATHS: usize = 8;

/// C-facing status codes.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EphStatus {
    Ok = 0,
    InvalidConfig = 1,
    InvalidQuery = 2,
    KernelLoad = 3,
    TimeConversion = 4,
    UnsupportedQuery = 5,
    EpochOutOfRange = 6,
    NullPointer = 7,
    Internal = 255,
}

impl From<&EngineError> for EphStatus {
    fn from(value: &EngineError) -> Self {
        match value {
            EngineError::InvalidConfig(_) => Self::InvalidConfig,
            EngineError::InvalidQuery(_) => Self::InvalidQuery,
            EngineError::KernelLoad(_) => Self::KernelLoad,
            EngineError::TimeConversion(_) => Self::TimeConversion,
            EngineError::UnsupportedQuery(_) => Self::UnsupportedQuery,
            EngineError::EpochOutOfRange { .. } => Self::EpochOutOfRange,
            EngineError::Internal(_) => Self::Internal,
            _ => Self::Internal,
        }
    }
}

/// C-compatible engine configuration.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EphEngineConfig {
    pub spk_path_count: u32,
    pub spk_paths_utf8: [[u8; EPH_PATH_CAPACITY]; EPH_MAX_SPK_PATHS],
    pub lsk_path_utf8: [u8; EPH_PATH_CAPACITY],
    pub cache_capacity: u64,
    pub strict_validation: u8,
}

impl EphEngineConfig {
    /// Convenience constructor for a single SPK path (most common case).
    pub fn try_new(
        spk_path_utf8: &str,
        lsk_path_utf8: &str,
        cache_capacity: u64,
        strict_validation: bool,
    ) -> Result<Self, EphStatus> {
        Self::try_new_multi(&[spk_path_utf8], lsk_path_utf8, cache_capacity, strict_validation)
    }

    /// Constructor for multiple SPK paths.
    pub fn try_new_multi(
        spk_paths: &[&str],
        lsk_path_utf8: &str,
        cache_capacity: u64,
        strict_validation: bool,
    ) -> Result<Self, EphStatus> {
        if spk_paths.is_empty() || spk_paths.len() > EPH_MAX_SPK_PATHS {
            return Err(EphStatus::InvalidConfig);
        }

        let mut spk_paths_utf8 = [[0_u8; EPH_PATH_CAPACITY]; EPH_MAX_SPK_PATHS];
        for (i, path) in spk_paths.iter().enumerate() {
            spk_paths_utf8[i] = encode_c_utf8(path)?;
        }

        Ok(Self {
            spk_path_count: spk_paths.len() as u32,
            spk_paths_utf8,
            lsk_path_utf8: encode_c_utf8(lsk_path_utf8)?,
            cache_capacity,
            strict_validation: u8::from(strict_validation),
        })
    }
}

impl TryFrom<&EphEngineConfig> for EngineConfig {
    type Error = EngineError;

    fn try_from(value: &EphEngineConfig) -> Result<Self, Self::Error> {
        let count = value.spk_path_count as usize;
        if count == 0 || count > EPH_MAX_SPK_PATHS {
            return Err(EngineError::InvalidConfig(
                "spk_path_count must be between 1 and 8",
            ));
        }

        let mut spk_paths = Vec::with_capacity(count);
        for buf in &value.spk_paths_utf8[..count] {
            let path_str = decode_c_utf8(buf)
                .map_err(|_| EngineError::InvalidConfig("invalid UTF-8 in spk_path"))?;
            spk_paths.push(PathBuf::from(path_str));
        }

        let lsk_path = decode_c_utf8(&value.lsk_path_utf8)
            .map_err(|_| EngineError::InvalidConfig("invalid UTF-8 in lsk_path"))?;

        let cache_capacity = usize::try_from(value.cache_capacity)
            .map_err(|_| EngineError::InvalidConfig("cache_capacity exceeds platform usize"))?;

        Ok(EngineConfig {
            spk_paths,
            lsk_path: PathBuf::from(lsk_path),
            cache_capacity,
            strict_validation: value.strict_validation != 0,
        })
    }
}

/// C-compatible query shape.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EphQuery {
    pub target: i32,
    pub observer: i32,
    pub frame: i32,
    pub epoch_tdb_jd: f64,
}

impl TryFrom<EphQuery> for Query {
    type Error = EngineError;

    fn try_from(value: EphQuery) -> Result<Self, Self::Error> {
        let target = Body::from_code(value.target)
            .ok_or(EngineError::InvalidQuery("target code is unsupported"))?;
        let observer = Observer::from_code(value.observer)
            .ok_or(EngineError::InvalidQuery("observer code is unsupported"))?;
        let frame = Frame::from_code(value.frame)
            .ok_or(EngineError::InvalidQuery("frame code is unsupported"))?;

        Ok(Query {
            target,
            observer,
            frame,
            epoch_tdb_jd: value.epoch_tdb_jd,
        })
    }
}

/// C-compatible output state vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EphStateVector {
    pub position_km: [f64; 3],
    pub velocity_km_s: [f64; 3],
}

impl From<StateVector> for EphStateVector {
    fn from(value: StateVector) -> Self {
        Self {
            position_km: value.position_km,
            velocity_km_s: value.velocity_km_s,
        }
    }
}

/// Opaque engine handle type for ABI consumers.
pub type EphEngineHandle = Engine;

/// Build a core engine from C-compatible config.
pub fn eph_engine_new_internal(config: &EphEngineConfig) -> Result<Engine, EphStatus> {
    let core_config = EngineConfig::try_from(config).map_err(|err| EphStatus::from(&err))?;
    Engine::new(core_config).map_err(|err| EphStatus::from(&err))
}

/// Query the engine using C-compatible types.
pub fn eph_engine_query_internal(
    engine: &Engine,
    query: EphQuery,
) -> Result<EphStateVector, EphStatus> {
    let core_query = Query::try_from(query).map_err(|err| EphStatus::from(&err))?;
    let state = engine
        .query(core_query)
        .map_err(|err| EphStatus::from(&err))?;
    Ok(EphStateVector::from(state))
}

/// Convenience helper for one-shot callers.
pub fn eph_query_once_internal(
    config: &EphEngineConfig,
    query: EphQuery,
) -> Result<EphStateVector, EphStatus> {
    let engine = eph_engine_new_internal(config)?;
    eph_engine_query_internal(&engine, query)
}

/// Return ABI version of the exported C API.
#[unsafe(no_mangle)]
pub extern "C" fn eph_api_version() -> u32 {
    EPH_API_VERSION
}

/// Create an engine handle.
///
/// # Safety
/// `config` and `out_engine` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eph_engine_new(
    config: *const EphEngineConfig,
    out_engine: *mut *mut EphEngineHandle,
) -> EphStatus {
    ffi_boundary(|| {
        if config.is_null() || out_engine.is_null() {
            return EphStatus::NullPointer;
        }

        // SAFETY: Pointers are checked for null and are only borrowed for this call.
        let config_ref = unsafe { &*config };
        // SAFETY: Pointer is checked for null and we only write a single pointer value.
        let out_engine_ref = unsafe { &mut *out_engine };

        match eph_engine_new_internal(config_ref) {
            Ok(engine) => {
                *out_engine_ref = Box::into_raw(Box::new(engine));
                EphStatus::Ok
            }
            Err(status) => {
                *out_engine_ref = ptr::null_mut();
                status
            }
        }
    })
}

/// Query an existing engine handle.
///
/// # Safety
/// `engine`, `query`, and `out_state` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eph_engine_query(
    engine: *const EphEngineHandle,
    query: *const EphQuery,
    out_state: *mut EphStateVector,
) -> EphStatus {
    ffi_boundary(|| {
        if engine.is_null() || query.is_null() || out_state.is_null() {
            return EphStatus::NullPointer;
        }

        // SAFETY: Pointers are checked for null and only borrowed for this call.
        let engine_ref = unsafe { &*engine };
        // SAFETY: Pointer is checked for null and copied by value.
        let query_value = unsafe { *query };

        match eph_engine_query_internal(engine_ref, query_value) {
            Ok(state) => {
                // SAFETY: Pointer is checked for null and written once.
                unsafe { *out_state = state };
                EphStatus::Ok
            }
            Err(status) => status,
        }
    })
}

/// Destroy an engine handle allocated by [`eph_engine_new`].
///
/// # Safety
/// `engine` must be either null or a pointer returned by `eph_engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eph_engine_free(engine: *mut EphEngineHandle) -> EphStatus {
    ffi_boundary(|| {
        if engine.is_null() {
            return EphStatus::Ok;
        }

        // SAFETY: Ownership is transferred back from a pointer created by Box::into_raw.
        unsafe { drop(Box::from_raw(engine)) };
        EphStatus::Ok
    })
}

/// One-shot query helper (constructs and tears down engine internally).
///
/// # Safety
/// `config`, `query`, and `out_state` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eph_query_once(
    config: *const EphEngineConfig,
    query: *const EphQuery,
    out_state: *mut EphStateVector,
) -> EphStatus {
    ffi_boundary(|| {
        if config.is_null() || query.is_null() || out_state.is_null() {
            return EphStatus::NullPointer;
        }

        // SAFETY: Pointer checks performed above; references are ephemeral.
        let config_ref = unsafe { &*config };
        // SAFETY: Pointer checks performed above; copied by value.
        let query_value = unsafe { *query };

        match eph_query_once_internal(config_ref, query_value) {
            Ok(state) => {
                // SAFETY: Pointer checks performed above; write one value.
                unsafe { *out_state = state };
                EphStatus::Ok
            }
            Err(status) => status,
        }
    })
}

fn ffi_boundary(f: impl FnOnce() -> EphStatus) -> EphStatus {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(status) => status,
        Err(_) => EphStatus::Internal,
    }
}

fn encode_c_utf8(input: &str) -> Result<[u8; EPH_PATH_CAPACITY], EphStatus> {
    if input.is_empty() {
        return Err(EphStatus::InvalidConfig);
    }

    let bytes = input.as_bytes();
    if bytes.len() >= EPH_PATH_CAPACITY {
        return Err(EphStatus::InvalidConfig);
    }
    if bytes.contains(&0) {
        return Err(EphStatus::InvalidConfig);
    }

    let mut out = [0_u8; EPH_PATH_CAPACITY];
    out[..bytes.len()].copy_from_slice(bytes);
    Ok(out)
}

fn decode_c_utf8(buffer: &[u8; EPH_PATH_CAPACITY]) -> Result<&str, std::str::Utf8Error> {
    let end = buffer
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(EPH_PATH_CAPACITY);
    std::str::from_utf8(&buffer[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn kernel_base() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../kernels/data")
    }

    fn kernels_available() -> bool {
        let base = kernel_base();
        base.join("de442s.bsp").exists() && base.join("naif0012.tls").exists()
    }

    fn real_config() -> Option<EphEngineConfig> {
        if !kernels_available() {
            eprintln!("Skipping: kernel files not found");
            return None;
        }
        let base = kernel_base();
        Some(
            EphEngineConfig::try_new(
                base.join("de442s.bsp").to_str().unwrap(),
                base.join("naif0012.tls").to_str().unwrap(),
                256,
                true,
            )
            .expect("config should be valid"),
        )
    }

    #[test]
    fn status_maps_from_core_error() {
        let status = EphStatus::from(&EngineError::InvalidQuery("bad"));
        assert_eq!(status, EphStatus::InvalidQuery);
    }

    #[test]
    fn query_once_successfully_maps_through_core_contract() {
        let config = match real_config() {
            Some(c) => c,
            None => return,
        };
        let query = EphQuery {
            target: Body::Mars.code(),
            observer: Observer::Body(Body::Earth).code(),
            frame: Frame::IcrfJ2000.code(),
            epoch_tdb_jd: 2_460_000.25,
        };

        let result = eph_query_once_internal(&config, query).expect("query should succeed");
        assert!(result.position_km[0].is_finite());
    }

    #[test]
    fn query_rejects_invalid_body_code() {
        let config = match real_config() {
            Some(c) => c,
            None => return,
        };
        let query = EphQuery {
            target: -999,
            observer: Observer::SolarSystemBarycenter.code(),
            frame: Frame::IcrfJ2000.code(),
            epoch_tdb_jd: 2_460_000.25,
        };

        let result = eph_query_once_internal(&config, query);
        assert_eq!(result, Err(EphStatus::InvalidQuery));
    }

    #[test]
    fn ffi_lifecycle_create_query_free() {
        let config = match real_config() {
            Some(c) => c,
            None => return,
        };
        let query = EphQuery {
            target: Body::Mars.code(),
            observer: Observer::Body(Body::Earth).code(),
            frame: Frame::IcrfJ2000.code(),
            epoch_tdb_jd: 2_460_000.5,
        };

        let mut engine_ptr: *mut EphEngineHandle = ptr::null_mut();
        // SAFETY: Passing valid pointers created in this test scope.
        let create_status = unsafe { eph_engine_new(&config, &mut engine_ptr) };
        assert_eq!(create_status, EphStatus::Ok);
        assert!(!engine_ptr.is_null());

        let mut out_state = EphStateVector {
            position_km: [0.0; 3],
            velocity_km_s: [0.0; 3],
        };
        // SAFETY: Engine handle and output buffers are valid in this test.
        let query_status = unsafe { eph_engine_query(engine_ptr, &query, &mut out_state) };
        assert_eq!(query_status, EphStatus::Ok);
        assert!(out_state.position_km[0].is_finite());

        // SAFETY: Pointer was returned by eph_engine_new and not yet freed.
        let free_status = unsafe { eph_engine_free(engine_ptr) };
        assert_eq!(free_status, EphStatus::Ok);
    }

    #[test]
    fn ffi_new_rejects_null_output_pointer() {
        let config = match real_config() {
            Some(c) => c,
            None => return,
        };
        // SAFETY: Passing null out pointer intentionally to verify validation.
        let status = unsafe { eph_engine_new(&config, ptr::null_mut()) };
        assert_eq!(status, EphStatus::NullPointer);
    }

    #[test]
    fn ffi_query_rejects_null_input_pointer() {
        let mut out_state = EphStateVector {
            position_km: [0.0; 3],
            velocity_km_s: [0.0; 3],
        };
        // SAFETY: Null engine pointer is intentional for this validation test.
        let status = unsafe { eph_engine_query(ptr::null(), ptr::null(), &mut out_state) };
        assert_eq!(status, EphStatus::NullPointer);
    }
}
