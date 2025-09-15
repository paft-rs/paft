use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Errors shared across the paft workspace (request validation, parsing, etc.).
pub enum PaftError {
    /// Search query must not be empty.
    #[error("Search query must not be empty")]
    EmptySearchQuery,

    /// Search limit must be greater than 0.
    #[error("Search limit must be greater than 0, but was {0}")]
    InvalidSearchLimit(usize),

    /// `HistoryRequest`: 'range' and 'period' are mutually exclusive.
    #[error("HistoryRequest: 'range' and 'period' are mutually exclusive")]
    ExclusiveRangeAndPeriod,

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
