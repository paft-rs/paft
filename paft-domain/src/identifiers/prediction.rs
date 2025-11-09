//! Generic prediction-market identifier newtypes.
use crate::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Opaque wrapper for validated prediction event ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct EventID(String);

impl EventID {
    /// Construct a new validated prediction event ID.
    ///
    /// Validates that the ID is exactly 66 characters, starts with "0x",
    /// and the remaining 64 characters are hexadecimal (0-9, a-f, A-F).
    ///
    /// # Errors
    /// Returns `DomainError::InvalidEventId` if validation fails.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        if value.len() != 66 {
            return Err(DomainError::InvalidEventId {
                value: value.to_string(),
            });
        }
        if !value.starts_with("0x") {
            return Err(DomainError::InvalidEventId {
                value: value.to_string(),
            });
        }
        for byte in value[2..].bytes() {
            match byte {
                b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {}
                _ => {
                    return Err(DomainError::InvalidEventId {
                        value: value.to_string(),
                    });
                }
            }
        }
        Ok(Self(value.to_string()))
    }
}

impl AsRef<str> for EventID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EventID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Opaque wrapper for validated prediction outcome ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct OutcomeID(String);

impl OutcomeID {
    /// Construct a new validated outcome ID.
    ///
    /// Validates that the outcome ID:
    /// - Is non-empty
    /// - Contains only ASCII digits (0-9)
    /// - Has no leading +, -, or whitespace
    /// - Is between 1 and 78 characters long
    ///
    /// # Errors
    /// Returns `DomainError::InvalidOutcomeId` if validation fails.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        if value.is_empty() || value.len() > 78 {
            return Err(DomainError::InvalidOutcomeId {
                value: value.to_string(),
            });
        }
        if let Some(first_char) = value.chars().next()
            && (first_char == '+' || first_char == '-' || first_char.is_whitespace())
        {
            return Err(DomainError::InvalidOutcomeId {
                value: value.to_string(),
            });
        }
        if !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(DomainError::InvalidOutcomeId {
                value: value.to_string(),
            });
        }
        Ok(Self(value.to_string()))
    }
}

impl AsRef<str> for OutcomeID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OutcomeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
