//! ISIN newtype for instrument codes.

use crate::DomainError;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;
use std::{convert::TryFrom, fmt, str::FromStr};

#[inline]
fn invalid_isin(value: &str) -> DomainError {
    DomainError::InvalidIsin {
        value: value.to_string(),
    }
}

fn normalize_isin(input: &str) -> Result<SmolStr, DomainError> {
    let Some(normalized) = normalized_12_byte_code(input) else {
        return Err(invalid_isin(input));
    };

    match ::isin::parse(normalized.as_str()) {
        Ok(_) => Ok(normalized),
        Err(_) => Err(invalid_isin(input)),
    }
}

fn normalized_12_byte_code(input: &str) -> Option<SmolStr> {
    let trimmed = input.trim();
    if trimmed.len() != 12 {
        return None;
    }

    let mut bytes = [0; 12];
    for (dest, byte) in bytes.iter_mut().zip(trimmed.bytes()) {
        if !byte.is_ascii_alphanumeric() {
            return None;
        }
        *dest = byte.to_ascii_uppercase();
    }

    let normalized = std::str::from_utf8(&bytes).expect("ASCII bytes are valid UTF-8");
    Some(SmolStr::new(normalized))
}

/// Opaque wrapper for validated ISIN values.
///
/// Backed by [`SmolStr`], so standard 12-byte ISIN codes live inline without
/// heap allocation and clones stay cheap.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Isin(SmolStr);

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
        value.0.into()
    }
}

impl Serialize for Isin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for Isin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}
