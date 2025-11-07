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

    /// Invalid condition ID encountered while parsing or validating.
    #[error(
        "Invalid condition ID: '{value}' - expected 66 characters starting with '0x' followed by 64 hexadecimal characters"
    )]
    InvalidConditionId {
        /// The original invalid condition ID input.
        value: String,
    },

    /// Invalid token ID encountered while parsing or validating.
    #[error(
        "Invalid token ID: '{value}' - expected 1-78 ASCII digits with no leading +, -, or whitespace"
    )]
    InvalidTokenId {
        /// The original invalid token ID input.
        value: String,
    },
}
