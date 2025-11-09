//! Domain-specific error types for `paft-domain`.

use thiserror::Error;

/// Errors produced by domain models.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

    /// Invalid ISIN encountered while parsing or validating.
    #[error("Invalid ISIN: '{value}'")]
    InvalidIsin {
        /// The original invalid ISIN input.
        value: String,
    },

    /// Invalid FIGI encountered while parsing or validating.
    #[error("Invalid FIGI: '{value}'")]
    InvalidFigi {
        /// The original invalid FIGI input.
        value: String,
    },

    /// Invalid symbol encountered while parsing or validating.
    #[error("Invalid symbol: '{value}'")]
    InvalidSymbol {
        /// The original invalid symbol input.
        value: String,
    },

    /// Invalid event ID encountered while parsing or validating.
    #[error(
        "Invalid event ID: '{value}' - expected 66 characters starting with '0x' followed by 64 hexadecimal characters"
    )]
    InvalidEventId {
        /// The original invalid event ID input.
        value: String,
    },

    /// Invalid outcome ID encountered while parsing or validating.
    #[error(
        "Invalid outcome ID: '{value}' - expected 1-78 ASCII digits with no leading +, -, or whitespace"
    )]
    InvalidOutcomeId {
        /// The original invalid outcome ID input.
        value: String,
    },
}
