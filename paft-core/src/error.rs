use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Domain-oriented errors shared across the paft workspace.
pub enum PaftError {
    /// Invalid value provided for an enum parser.
    #[error("Invalid {enum_name} value: '{value}'")]
    InvalidEnumValue {
        /// Enum type name for context (e.g., "Currency").
        enum_name: &'static str,
        /// The offending input value.
        value: String,
    },

    /// Invalid canonical token produced by normalization helpers.
    #[error("Invalid canonical token: '{value}' - canonicalized value must be non-empty")]
    InvalidCanonicalToken {
        /// The original input that failed to produce a canonical token.
        value: String,
    },

    /// `HistoryRequest`: 'period' start must be before end.
    #[error("HistoryRequest: 'period' start ({start}) must be before end ({end})")]
    InvalidPeriod {
        /// The start timestamp that was invalid.
        start: i64,
        /// The end timestamp that was invalid.
        end: i64,
    },

    /// Invalid period format provided for parsing.
    #[error(
        "Invalid period format: '{format}' - expected formats like '2023Q4', '2023', 'FY2023', '2023-12-31', or '12/31/2023'"
    )]
    InvalidPeriodFormat {
        /// The invalid format string that could not be parsed.
        format: String,
    },
}
