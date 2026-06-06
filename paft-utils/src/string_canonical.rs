//! Shared canonical string utilities for extensible enums.
//!
//! All extensible enum `Other` branches must construct their canonical token via
//! [`Canonical::try_new`] to guarantee we never serialize an empty string and thus
//! preserve serde/display round-trips.

use smol_str::SmolStr;
use std::{
    borrow::{Borrow, Cow},
    fmt,
    str::FromStr,
};

/// Canonical string wrapper used for `Other` variants.
///
/// Canonical tokens are capped at [`MAX_CANONICAL_TOKEN_LEN`] bytes. The cap is
/// intentionally generous for provider enum codes while preventing unbounded
/// unknown-token storage from untrusted inputs.
///
/// Invariants:
/// - Trimmed
/// - ASCII uppercased
/// - Separator runs collapsed to single underscores
/// - Non-empty and no longer than [`MAX_CANONICAL_TOKEN_LEN`] bytes
///
/// Backed by [`SmolStr`] so canonical tokens that fit inline (≤ 23 bytes on
/// 64-bit targets) avoid heap allocation entirely, and longer tokens use an
/// `Arc<str>` so clones are O(1) refcount bumps. Most canonical tokens in
/// this workspace — currency codes, exchange codes, period codes — are short
/// enough to be stored inline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Canonical(SmolStr);

/// Maximum byte length of a canonical `Other` enum token.
///
/// Canonical tokens only contain ASCII uppercase letters, digits, and
/// underscores, so byte length and character count are equivalent.
pub const MAX_CANONICAL_TOKEN_LEN: usize = 256;

impl Canonical {
    /// Attempts to create a new canonical string from arbitrary input, rejecting
    /// values that would canonicalize to an empty token (e.g., strings composed
    /// solely of separators or non-alphanumeric characters), or whose
    /// canonical form exceeds [`MAX_CANONICAL_TOKEN_LEN`] bytes.
    ///
    /// This should be used by all enum `Other` variants to ensure the emitted
    /// string is always non-empty and round-trips via serde and `Display`.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError::InvalidCanonicalToken`] when the canonicalized
    /// token would be empty, or [`CanonicalError::CanonicalTokenTooLong`] when
    /// it would exceed [`MAX_CANONICAL_TOKEN_LEN`] bytes.
    pub fn try_new(input: &str) -> Result<Self, CanonicalError> {
        let token = canonicalize_bounded(input)?;
        if token.is_empty() {
            return Err(CanonicalError::InvalidCanonicalToken {
                value: input.to_string(),
            });
        }
        Ok(Self(SmolStr::new(token.as_ref())))
    }

    /// Returns the inner canonical string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `Canonical` and returns the inner value as a `String`.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0.to_string()
    }
}

fn canonicalize_bounded(input: &str) -> Result<Cow<'_, str>, CanonicalError> {
    if is_canonical(input) {
        if input.len() > MAX_CANONICAL_TOKEN_LEN {
            return Err(CanonicalError::CanonicalTokenTooLong {
                max_len: MAX_CANONICAL_TOKEN_LEN,
            });
        }
        return Ok(Cow::Borrowed(input));
    }

    let mut out = String::with_capacity(input.len().min(MAX_CANONICAL_TOKEN_LEN));
    let mut pending_sep = false;

    for ch in input.chars() {
        let c = ch.to_ascii_uppercase();
        if c.is_ascii_alphanumeric() {
            let sep_len = usize::from(pending_sep);
            if out.len() + sep_len + 1 > MAX_CANONICAL_TOKEN_LEN {
                return Err(CanonicalError::CanonicalTokenTooLong {
                    max_len: MAX_CANONICAL_TOKEN_LEN,
                });
            }
            if pending_sep {
                out.push('_');
            }
            out.push(c);
            pending_sep = false;
        } else if !out.is_empty() {
            pending_sep = true;
        }
    }

    Ok(Cow::Owned(out))
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

/// Returns true when `input` can safely be matched against a modeled enum token.
///
/// String enum parsers use this as a boundary check before resolving a
/// canonicalized token to a known variant or alias. It allows ordinary
/// case/separator normalization inside the token while rejecting leading or
/// trailing separators such as `"$USD"` or `"CLOSED!"`, which would otherwise
/// canonicalize into modeled values and lose their original identity.
#[inline]
#[must_use]
pub fn has_canonical_token_boundaries(input: &str) -> bool {
    let trimmed = input.trim();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let last = chars.next_back().unwrap_or(first);

    first.is_ascii_alphanumeric() && last.is_ascii_alphanumeric()
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
#[non_exhaustive]
pub enum CanonicalError {
    /// Invalid canonical token produced by normalization helpers.
    #[error("Invalid canonical token: '{value}' - canonicalized value must be non-empty")]
    InvalidCanonicalToken {
        /// The original input that failed to produce a canonical token.
        value: String,
    },
    /// Canonical token exceeded the configured maximum length.
    #[error("canonical token exceeds maximum length of {max_len} bytes")]
    CanonicalTokenTooLong {
        /// Maximum accepted canonical token length in bytes.
        max_len: usize,
    },
}
