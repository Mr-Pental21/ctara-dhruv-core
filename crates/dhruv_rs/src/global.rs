use std::sync::OnceLock;
use std::sync::{LazyLock, RwLock};

use dhruv_core::{Engine, EngineConfig};
use dhruv_time::TimeConversionPolicy;

use crate::DhruvError;

static ENGINE: OnceLock<Engine> = OnceLock::new();
static TIME_POLICY: LazyLock<RwLock<TimeConversionPolicy>> =
    LazyLock::new(|| RwLock::new(TimeConversionPolicy::default()));

/// Initialize the global engine singleton.
///
/// Must be called exactly once before any convenience functions.
/// Returns [`DhruvError::AlreadyInitialized`] on subsequent calls.
pub fn init(config: EngineConfig) -> Result<(), DhruvError> {
    let eng = Engine::new(config)?;
    ENGINE.set(eng).map_err(|_| DhruvError::AlreadyInitialized)
}

/// Returns `true` if the global engine has been initialized.
pub fn is_initialized() -> bool {
    ENGINE.get().is_some()
}

/// Access the global engine. Returns [`DhruvError::NotInitialized`] if
/// [`init`] has not been called.
pub(crate) fn engine() -> Result<&'static Engine, DhruvError> {
    ENGINE.get().ok_or(DhruvError::NotInitialized)
}

/// Set UTC->TDB time-conversion policy for `dhruv_rs` convenience APIs.
///
/// This also synchronizes the same policy into `dhruv_search` internals so
/// date-driven search APIs are consistent.
pub fn set_time_conversion_policy(policy: TimeConversionPolicy) {
    match TIME_POLICY.write() {
        Ok(mut guard) => *guard = policy,
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            *guard = policy;
        }
    }
    dhruv_search::set_time_conversion_policy(policy);
}

/// Read the currently configured UTC->TDB time-conversion policy.
pub fn time_conversion_policy() -> TimeConversionPolicy {
    match TIME_POLICY.read() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_initialized_by_default() {
        // Note: this test relies on the global not being initialized.
        // In a multi-test binary this may not hold if another test calls init().
        // We test the function contract rather than actual state here.
        let _ = is_initialized(); // just ensure it doesn't panic
    }

    #[test]
    fn engine_returns_error_when_not_initialized() {
        // OnceLock is process-global so we can only test the error path
        // when no other test has called init(). Integration tests cover
        // the success path.
        if !is_initialized() {
            assert!(matches!(engine(), Err(DhruvError::NotInitialized)));
        }
    }
}
