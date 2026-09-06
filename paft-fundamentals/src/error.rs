//! Error types for `paft-fundamentals`.

use thiserror::Error;

/// Errors produced by fundamentals models.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FundamentalsError {
    /// ESG methodology context contains an invalid supplied string.
    #[error("Invalid ESG context {field}: {reason} (value: {value:?})")]
    InvalidEsgContext {
        /// Context field that failed validation.
        field: &'static str,
        /// Original supplied value.
        value: String,
        /// Syntactic requirement that failed.
        reason: &'static str,
    },
    /// A reporting interval ends before it starts.
    #[error("Statement duration starts at {start} after its end at {end}")]
    InvalidStatementDuration {
        /// First included reporting date.
        start: chrono::NaiveDate,
        /// Last included reporting date.
        end: chrono::NaiveDate,
    },
    /// Invalid value provided for a fundamentals enum parser.
    #[error("Invalid {enum_name} value: '{value}'")]
    InvalidEnumValue {
        /// Enum type name for context.
        enum_name: &'static str,
        /// The offending input value.
        value: String,
    },
}
