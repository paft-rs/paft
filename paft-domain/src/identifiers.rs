//! Identifier newtypes for instrument codes (ISIN, FIGI, Symbol).

use crate::DomainError;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{convert::TryFrom, fmt, str::FromStr};

#[inline]
fn invalid_isin(value: &str) -> DomainError {
    DomainError::InvalidIsin {
        value: value.to_string(),
    }
}

#[inline]
fn invalid_figi(value: &str) -> DomainError {
    DomainError::InvalidFigi {
        value: value.to_string(),
    }
}

#[inline]
fn invalid_symbol(value: &str) -> DomainError {
    DomainError::InvalidSymbol {
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

fn normalize_symbol(input: &str) -> Result<String, DomainError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(invalid_symbol(input));
    }

    if trimmed.len() > 64 {
        return Err(invalid_symbol(input));
    }

    if trimmed
        .chars()
        .any(|c| c.is_ascii_whitespace() || c.is_ascii_control())
    {
        return Err(invalid_symbol(input));
    }

    let normalized = trimmed.to_ascii_uppercase();

    if normalized.len() > 64 {
        return Err(invalid_symbol(input));
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
        Self::new(&raw).map_err(de::Error::custom)
    }
}

/// Opaque wrapper for validated symbol strings used by markets and data providers.
///
/// Symbols are canonicalized to uppercase ASCII, must not contain whitespace
/// or ASCII control characters, and are limited to 64 bytes. Punctuation and
/// numerics are preserved verbatim so that provider-specific conventions
/// (class suffixes, exchange codes, contract metadata, etc.) round-trip without
/// transformation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Symbol(String);

impl Symbol {
    /// Construct a new validated symbol.
    ///
    /// Trims leading/trailing whitespace, uppercases ASCII letters, and enforces
    /// the following invariants:
    ///
    /// - Must be non-empty after trimming.
    /// - Must not contain ASCII whitespace (space, tab, newline, carriage return, form feed, vertical tab).
    /// - Must not contain ASCII control characters (0x00–0x1F or 0x7F).
    /// - Must be at most 64 bytes long.
    ///
    /// Punctuation and numerics are preserved as-is.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidSymbol` when invariants are violated.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn new(value: &str) -> Result<Self, DomainError> {
        let normalized = normalize_symbol(value)?;
        Ok(Self(normalized))
    }

    /// Returns the canonical symbol string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the byte length of the canonical symbol.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the symbol is empty. This should always be `false`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        debug_assert!(
            !self.0.is_empty(),
            "Symbol invariant violated: empty symbol"
        );
        self.0.is_empty()
    }
}

impl Default for Symbol {
    fn default() -> Self {
        // Safe unwrap: "DEFAULT" satisfies all invariants.
        Self::new("DEFAULT").expect("static default symbol is valid")
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Symbol {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Symbol {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Symbol> for String {
    fn from(value: Symbol) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::new(&raw).map_err(de::Error::custom)
    }
}
