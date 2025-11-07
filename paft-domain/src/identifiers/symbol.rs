//! Identifier newtypes for instrument codes (ISIN, FIGI, Symbol).

use crate::DomainError;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{convert::TryFrom, fmt, str::FromStr};

#[inline]
fn invalid_symbol(value: &str) -> DomainError {
    DomainError::InvalidSymbol {
        value: value.to_string(),
    }
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

/// Opaque wrapper for validated symbol strings used by markets and data providers.
///
/// Symbols are canonicalized to uppercase ASCII, must not contain whitespace
/// or ASCII control characters, and are limited to 64 bytes. Punctuation and
/// numerics are preserved verbatim so that provider-specific conventions
/// (class suffixes, exchange codes, contract metadata, etc.) round-trip without
/// transformation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
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
