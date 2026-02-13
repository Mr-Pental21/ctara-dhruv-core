use std::error::Error;
use std::fmt::{Display, Formatter};

use dhruv_core::EngineError;
use dhruv_search::SearchError;
use dhruv_time::TimeError;

/// Unified error type for the convenience wrapper.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DhruvError {
    /// Global engine has not been initialized via [`crate::init`].
    NotInitialized,
    /// [`crate::init`] was called more than once.
    AlreadyInitialized,
    /// Failed to parse a date string.
    DateParse(String),
    /// Error from the underlying engine.
    Engine(EngineError),
    /// Error from time conversion.
    Time(TimeError),
    /// Error from search/panchang computation.
    Search(SearchError),
}

impl Display for DhruvError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => {
                write!(f, "engine not initialized; call dhruv_rs::init() first")
            }
            Self::AlreadyInitialized => write!(f, "engine already initialized"),
            Self::DateParse(msg) => write!(f, "date parse error: {msg}"),
            Self::Engine(e) => write!(f, "engine error: {e}"),
            Self::Time(e) => write!(f, "time error: {e}"),
            Self::Search(e) => write!(f, "search error: {e}"),
        }
    }
}

impl Error for DhruvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Engine(e) => Some(e),
            Self::Time(e) => Some(e),
            Self::Search(e) => Some(e),
            _ => None,
        }
    }
}

impl From<EngineError> for DhruvError {
    fn from(e: EngineError) -> Self {
        Self::Engine(e)
    }
}

impl From<TimeError> for DhruvError {
    fn from(e: TimeError) -> Self {
        Self::Time(e)
    }
}

impl From<SearchError> for DhruvError {
    fn from(e: SearchError) -> Self {
        Self::Search(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_not_initialized() {
        let e = DhruvError::NotInitialized;
        assert!(e.to_string().contains("not initialized"));
    }

    #[test]
    fn display_already_initialized() {
        let e = DhruvError::AlreadyInitialized;
        assert!(e.to_string().contains("already initialized"));
    }

    #[test]
    fn display_date_parse() {
        let e = DhruvError::DateParse("bad date".into());
        assert!(e.to_string().contains("bad date"));
    }

    #[test]
    fn from_engine_error() {
        let e: DhruvError = EngineError::InvalidConfig("test").into();
        assert!(matches!(e, DhruvError::Engine(_)));
    }

    #[test]
    fn from_time_error() {
        let e: DhruvError = TimeError::Pre1972Utc.into();
        assert!(matches!(e, DhruvError::Time(_)));
    }
}
