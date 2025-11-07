//! Identifier newtypes for Polymarket.

use crate::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Opaque wrapper for validated Polymarket condition ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct ConditionID(String);

impl ConditionID {
    /// Construct a new validated Polymarket condition ID.
    ///
    /// Validates that the ID is exactly 66 characters, starts with "0x",
    /// and the remaining 64 characters are hexadecimal (0-9, a-f, A-F).
    ///
    /// # Errors
    /// Returns `DomainError::InvalidtConditionId` if validation fails.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        if value.len() != 66 {
            return Err(DomainError::InvalidConditionId {
                value: value.to_string(),
            });
        }

        if !value.starts_with("0x") {
            return Err(DomainError::InvalidConditionId {
                value: value.to_string(),
            });
        }

        for byte in value[2..].bytes() {
            match byte {
                b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {}
                _ => {
                    return Err(DomainError::InvalidConditionId {
                        value: value.to_string(),
                    });
                }
            }
        }

        Ok(Self(value.to_string()))
    }
}

impl AsRef<str> for ConditionID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ConditionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Opaque wrapper for validated Polymarket token ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TokenID(String);

impl TokenID {
    /// Construct a new validated Polymarket token ID.
    ///
    /// Validates that the token ID:
    /// - Is non-empty
    /// - Contains only ASCII digits (0-9)
    /// - Has no leading +, -, or whitespace
    /// - Is between 1 and 78 characters long
    ///
    /// # Errors
    /// Returns `DomainError::InvalidTokenId` if validation fails.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        // Check length bounds
        if value.is_empty() || value.len() > 78 {
            return Err(DomainError::InvalidTokenId {
                value: value.to_string(),
            });
        }

        // Check for leading +, -, or whitespace
        if let Some(first_char) = value.chars().next()
            && (first_char == '+' || first_char == '-' || first_char.is_whitespace())
        {
            return Err(DomainError::InvalidTokenId {
                value: value.to_string(),
            });
        }

        // Check that all characters are ASCII digits
        if !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(DomainError::InvalidTokenId {
                value: value.to_string(),
            });
        }

        Ok(Self(value.to_string()))
    }
}

impl AsRef<str> for TokenID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TokenID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
