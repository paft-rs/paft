use thiserror::Error;

/// Errors emitted by the paft-money crate.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MoneyParseError {
    /// Invalid value provided for an enum parser.
    #[error("Invalid {enum_name} value: '{value}'")]
    InvalidEnumValue {
        /// Enum type name for context (e.g., "Currency").
        enum_name: &'static str,
        /// The offending input value.
        value: String,
    },
}
