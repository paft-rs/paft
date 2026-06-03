//! Error types for `paft-fundamentals`.

use thiserror::Error;

/// Errors produced by fundamentals models.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FundamentalsError {
    /// Invalid value provided for a fundamentals enum parser.
    #[error("Invalid {enum_name} value: '{value}'")]
    InvalidEnumValue {
        /// Enum type name for context.
        enum_name: &'static str,
        /// The offending input value.
        value: String,
    },
}
