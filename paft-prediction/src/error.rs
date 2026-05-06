//! Error types specific to `paft-prediction`.

use thiserror::Error;

/// Errors produced when constructing prediction-market identifiers.
///
/// Each variant carries the original input so callers can surface the offending
/// value back to the user. The `Display` impls match the historical messages
/// used in `paft-domain` so existing log scrapers and tests continue to work.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PredictionError {
    /// Invalid event ID encountered while parsing or validating.
    #[error(
        "Invalid event ID: '{0}' - expected 66 characters starting with '0x' followed by 64 hexadecimal characters"
    )]
    InvalidEventId(String),

    /// Invalid outcome ID encountered while parsing or validating.
    #[error(
        "Invalid outcome ID: '{0}' - expected 1-78 ASCII digits with no leading +, -, or whitespace"
    )]
    InvalidOutcomeId(String),
}
