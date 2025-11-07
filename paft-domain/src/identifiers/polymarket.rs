//! Identifier newtypes for Polymarket.

use crate::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Opaque wrapper for validated Polymarket condition ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct ConditionID(String);

impl ConditionID {
    /// Construct a new (currently not validated!) Polymarket condition ID.
    ///
    /// # Errors
    /// Currently this is infallible, the error type is for API consistency
    pub fn new(value: &str) -> Result<Self, DomainError> {
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
    /// Construct a new (currently not validated!) Polymarket condition ID.
    ///
    /// # Errors
    /// Currently this is infallible, the error type is for API consistency
    pub fn new(value: &str) -> Result<Self, DomainError> {
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
