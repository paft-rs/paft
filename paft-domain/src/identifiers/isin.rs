//! ISIN newtype for instrument codes.

use crate::DomainError;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{convert::TryFrom, fmt, str::FromStr};

#[inline]
fn invalid_isin(value: &str) -> DomainError {
    DomainError::InvalidIsin {
        value: value.to_string(),
    }
}

fn scrub_isin(input: &str) -> String {
    input
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
}

fn normalize_isin(input: &str) -> Result<String, DomainError> {
    let cleaned = scrub_isin(input);
    match ::isin::parse_loose(&cleaned) {
        Ok(_) => Ok(cleaned.to_ascii_uppercase()),
        Err(_) => Err(invalid_isin(input)),
    }
}

/// Opaque wrapper for validated ISIN values.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Isin(String);

impl Isin {
    /// Construct a new validated ISIN.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidIsin` when `value` is empty, malformed,
    /// or fails checksum validation after normalization.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn new(value: &str) -> Result<Self, DomainError> {
        let normalized = normalize_isin(value)?;
        Ok(Self(normalized))
    }
}

impl AsRef<str> for Isin {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Isin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Isin {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Isin {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Isin> for String {
    fn from(value: Isin) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for Isin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::new(&raw).map_err(de::Error::custom)
    }
}
