//! Shared canonical string utilities for extensible enums.
//!
//! All extensible enum `Other` branches must construct their canonical token via
//! [`Canonical::try_new`] to guarantee we never serialize an empty string and thus
//! preserve serde/display round-trips.

use std::{
    borrow::{Borrow, Cow},
    fmt,
    str::FromStr,
};

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
        Ok(Self(token.into_owned()))
    }

    /// Returns the inner canonical string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `Canonical` and returns the inner `String`.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
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
/// # Canonical Form Contract
///
/// Canonical form is `[A-Z0-9]+(?:_[A-Z0-9]+)*`. Non-ASCII and non-alphanumeric characters
/// are treated as separators. Empty after normalization → error.
///
/// # Canonicalization Rules
///
/// - **ASCII-only**: Only ASCII uppercase letters (A-Z) and digits (0-9) are preserved as-is
/// - **Case normalization**: ASCII lowercase letters are converted to uppercase
/// - **Separators**: All non-alphanumeric ASCII characters and Unicode codepoints become separators
/// - **Separator handling**: Contiguous separators collapse to a single underscore `_`
/// - **Trimming**: Leading and trailing separators are removed
/// - **Underscores**: Multiple underscores collapse to single underscores; no leading/trailing/double underscores
///
/// Returns `Cow::Borrowed(input)` if `input` is already canonical; otherwise returns an owned, normalized string.
#[inline]
#[must_use]
pub fn canonicalize(input: &str) -> Cow<'_, str> {
    // Fast path: check if input is already canonical
    if is_canonical(input) {
        return Cow::Borrowed(input);
    }

    let mut out = String::with_capacity(input.len());
    let mut prev_sep = true; // treat start as "just saw a separator" to skip leading seps

    for ch in input.chars() {
        let c = ch.to_ascii_uppercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_sep = false;
        } else if !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }

    if out.ends_with('_') {
        out.pop(); // drop trailing separator without reallocation
    }

    Cow::Owned(out)
}

/// Checks if a string is already in canonical form.
///
/// A string is canonical if:
/// - All characters are ASCII uppercase letters or digits
/// - There are no consecutive non-alphanumeric characters
/// - There are no leading or trailing underscores
#[inline]
fn is_canonical(input: &str) -> bool {
    let b = input.as_bytes();
    if b.is_empty() || b[0] == b'_' || b[b.len() - 1] == b'_' {
        return false;
    }
    let mut prev = b'_';
    for &c in b {
        match c {
            b'A'..=b'Z' | b'0'..=b'9' => prev = c,
            b'_' if prev != b'_' => prev = c,
            _ => return false,
        }
    }
    true
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
