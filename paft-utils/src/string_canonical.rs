//! Shared canonical string utilities for extensible enums.
//!
//! All extensible enum `Other` branches must construct their canonical token via
//! [`Canonical::try_new`] to guarantee we never serialize an empty string and thus
//! preserve serde/display round-trips.

use std::{borrow::Borrow, fmt, str::FromStr};

/// Canonical string wrapper used for `Other` variants.
///
/// Invariants:
/// - Trimmed
/// - ASCII uppercased
/// - Whitespace collapsed to single underscores
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Canonical(String);

impl Canonical {
    /// Attempts to create a new canonical string from arbitrary input, rejecting
    /// values that would canonicalize to an empty token (e.g., strings composed
    /// solely of separators or non-alphanumeric characters).
    ///
    /// This should be used by all enum `Other` variants to ensure the emitted
    /// string is always non-empty and round-trips via serde and `Display`.
    ///
    /// # Errors
    ///
    /// Returns `CanonicalError::InvalidCanonicalToken` when the canonicalized token would
    /// be empty.
    pub fn try_new(input: &str) -> Result<Self, CanonicalError> {
        let token = canonicalize(input);
        if token.is_empty() {
            return Err(CanonicalError::InvalidCanonicalToken {
                value: input.to_string(),
            });
        }
        Ok(Self(token))
    }

    /// Returns the inner canonical string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Canonical {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for Canonical {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for Canonical {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl FromStr for Canonical {
    type Err = CanonicalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

/// Produces the canonical representation of an input string used across enums.
///
/// All `Display`/serde string forms across enums are canonical tokens produced by this function.
///
/// Rules:
/// - Uppercase ASCII letters
/// - Convert every contiguous run of non-alphanumeric characters to a single underscore `_`
/// - Trim leading and trailing underscores
#[must_use]
pub fn canonicalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut previous_was_separator = false;

    for ch in input.chars() {
        let mut c = ch;
        if c.is_ascii_lowercase() {
            c = c.to_ascii_uppercase();
        }

        if c.is_ascii_alphanumeric() {
            out.push(c);
            previous_was_separator = false;
        } else if !previous_was_separator {
            out.push('_');
            previous_was_separator = true;
        }
    }

    let trimmed = out.trim_matches('_');
    if trimmed.len() == out.len() {
        out
    } else {
        trimmed.to_string()
    }
}

/// Trait for enums that have a canonical string code.
///
/// Implemented via macros across the paft workspace.
pub trait StringCode {
    /// Returns the canonical string code for this value.
    fn code(&self) -> &str;

    /// Whether this value is a canonical enum variant (not an `Other` payload).
    fn is_canonical(&self) -> bool {
        true
    }
}

/// Errors that can occur when constructing canonical strings.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum CanonicalError {
    /// Invalid canonical token produced by normalization helpers.
    #[error("Invalid canonical token: '{value}' - canonicalized value must be non-empty")]
    InvalidCanonicalToken {
        /// The original input that failed to produce a canonical token.
        value: String,
    },
}
