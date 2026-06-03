//! Error types specific to `paft-market` request validation.

use thiserror::Error;

/// Errors returned when validating market requests before execution.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MarketError {
    /// Search query must not be empty.
    #[error("Search query must not be empty")]
    EmptySearchQuery,

    /// Search limit must be greater than 0.
    #[error("Search limit must be greater than 0, but was {0}")]
    InvalidSearchLimit(u32),

    /// Search locale fields must not be empty when provided.
    #[error("Search {field} must not be empty when provided")]
    EmptySearchLocaleField {
        /// The locale field that was empty.
        field: &'static str,
    },

    /// `HistoryRequest`: 'period' start must be before end.
    #[error("HistoryRequest: 'period' start ({start}) must be before end ({end})")]
    InvalidPeriod {
        /// Start timestamp (milliseconds since epoch).
        start: i64,
        /// End timestamp (milliseconds since epoch).
        end: i64,
    },
}
