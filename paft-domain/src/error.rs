//! Domain-specific error types for `paft-domain`.

use thiserror::Error;

/// Errors produced by domain models.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Invalid period format provided for parsing.
    #[error(
        "Invalid period format: '{format}' - expected formats like '2023Q4', '2023', 'FY2023', '2023-12-31', or '12/31/2023'"
    )]
    InvalidPeriodFormat {
        /// The invalid format string that could not be parsed.
        format: String,
    },

    /// Invalid exchange token encountered while parsing.
    #[error("Invalid exchange value: '{value}'")]
    InvalidExchangeValue {
        /// The invalid exchange token.
        value: String,
    },
}
