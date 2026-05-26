//! FIGI newtype for instrument codes.

use crate::DomainError;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{convert::TryFrom, fmt, str::FromStr};

#[inline]
fn invalid_figi(value: &str) -> DomainError {
    DomainError::InvalidFigi {
        value: value.to_string(),
    }
}

fn normalize_figi(input: &str) -> Result<String, DomainError> {
    let candidate = input.trim();
    if candidate.is_empty() {
        return Err(invalid_figi(input));
    }

    let normalized = candidate.to_ascii_uppercase();
    if normalized.len() != 12 {
        return Err(invalid_figi(input));
    }

    if !figi_structure_is_valid(&normalized) {
        return Err(invalid_figi(input));
    }

    if !figi_checksum_is_valid(&normalized) {
        return Err(invalid_figi(input));
    }

    Ok(normalized)
}

fn figi_structure_is_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 12
        && bytes[..2].iter().all(|b| is_figi_consonant(*b))
        && bytes[2] == b'G'
        && bytes[3..11]
            .iter()
            .all(|b| b.is_ascii_digit() || is_figi_consonant(*b))
        && bytes[11].is_ascii_digit()
}

const fn is_figi_consonant(byte: u8) -> bool {
    byte.is_ascii_uppercase() && !matches!(byte, b'A' | b'E' | b'I' | b'O' | b'U')
}

fn figi_checksum_is_valid(value: &str) -> bool {
    if value.len() != 12 {
        return false;
    }

    let bytes = value.as_bytes();
    let checksum = bytes[11];
    if !checksum.is_ascii_digit() {
        return false;
    }

    let mut sum = 0u32;
    for (offset, byte) in bytes[..11].iter().rev().enumerate() {
        let Some(mut value) = figi_char_value(*byte) else {
            return false;
        };
        if offset % 2 == 1 {
            value *= 2;
        }
        sum += value / 10 + value % 10;
    }

    let expected = (10 - (sum % 10)) % 10;
    expected == u32::from(checksum - b'0')
}

fn figi_char_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'A'..=b'Z' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

/// Opaque wrapper for validated FIGI values.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Figi(String);

impl Figi {
    /// Construct a new validated FIGI.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidFigi` when `value` is empty, not exactly
    /// 12 ASCII alphanumeric characters, or fails the checksum.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn new(value: &str) -> Result<Self, DomainError> {
        let normalized = normalize_figi(value)?;
        Ok(Self(normalized))
    }
}

impl AsRef<str> for Figi {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Figi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Figi {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Figi {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Figi> for String {
    fn from(value: Figi) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for Figi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}
