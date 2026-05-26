//! Symbol newtype for instrument codes.

use crate::DomainError;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;
use std::{convert::TryFrom, fmt, str::FromStr};

#[inline]
fn invalid_symbol(value: &str) -> DomainError {
    DomainError::InvalidSymbol {
        value: value.to_string(),
    }
}

fn normalize_symbol(input: &str) -> Result<SmolStr, DomainError> {
    let trimmed = input.trim_ascii();
    if trimmed.is_empty() {
        return Err(invalid_symbol(input));
    }

    if trimmed.len() > 64 {
        return Err(invalid_symbol(input));
    }

    let mut has_lowercase = false;
    for byte in trimmed.bytes() {
        if !byte.is_ascii() || byte.is_ascii_whitespace() || byte.is_ascii_control() {
            return Err(invalid_symbol(input));
        }
        has_lowercase |= byte.is_ascii_lowercase();
    }

    // ASCII-uppercase normalization preserves byte length, so the 64-byte cap
    // already enforced on `trimmed` carries through to the canonical form.
    // We avoid the extra allocation a `String::to_ascii_uppercase()` would
    // require when the input is already canonical (a common case for tickers
    // that arrive pre-uppercased from providers).
    if has_lowercase {
        let mut buf = trimmed.to_owned();
        buf.make_ascii_uppercase();
        Ok(SmolStr::new(buf))
    } else {
        // Already canonical: SmolStr::new copies the bytes inline when ≤ 23.
        Ok(SmolStr::new(trimmed))
    }
}

/// Opaque wrapper for validated symbol strings used by markets and data providers.
///
/// Symbols are canonicalized to uppercase ASCII, must contain ASCII bytes only,
/// must not contain whitespace or control characters, and are limited to 64
/// bytes. ASCII punctuation and numerics are preserved verbatim so that
/// provider-specific conventions
/// (class suffixes, exchange codes, contract metadata, etc.) round-trip without
/// transformation.
///
/// Backed by [`SmolStr`] so that typical equity tickers (≤ 23 bytes) live
/// inline without heap allocation, and longer symbols share an `Arc<str>` so
/// clones are O(1) refcount bumps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(SmolStr);

impl Symbol {
    /// Construct a new validated symbol.
    ///
    /// Trims leading/trailing ASCII whitespace, uppercases ASCII letters, and
    /// enforces the following invariants:
    ///
    /// - Must be non-empty after trimming.
    /// - Must contain ASCII characters only.
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
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the byte length of the canonical symbol.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `false` unconditionally.
    ///
    /// `Symbol` is constructor-validated to be non-empty (see [`Symbol::new`]),
    /// so the only reason this method exists is to satisfy the lint that
    /// expects every `len()` method to be paired with `is_empty()`.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
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
        value.0.into()
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}
