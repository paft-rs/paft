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

    if !normalized.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(invalid_figi(input));
    }

    if !figi_checksum_is_valid(&normalized) {
        return Err(invalid_figi(input));
    }

    Ok(normalized)
}

fn figi_checksum_is_valid(value: &str) -> bool {
    if value.len() != 12 {
        return false;
    }

    let body = &value[..11];
    let checksum_char = value.as_bytes()[11];
    if !(checksum_char as char).is_ascii_digit() {
        return false;
    }

    let mut digits = Vec::with_capacity(22);
    for ch in body.chars() {
        if ch.is_ascii_digit() {
            digits.push(u32::from(ch as u8 - b'0'));
        } else if ch.is_ascii_uppercase() {
            let value = (ch as u32 - 'A' as u32) + 10;
            digits.push(value / 10);
            digits.push(value % 10);
        } else {
            return false;
        }
    }

    let mut sum = 0u32;
    let mut double = false;
    for digit in digits.iter().rev() {
        let mut val = *digit;
        if double {
            val *= 2;
            if val > 9 {
                val = (val / 10) + (val % 10);
            }
        }
        sum += val;
        double = !double;
    }

    let expected = (10 - (sum % 10)) % 10;
    expected == u32::from(checksum_char - b'0')
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
