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
}
