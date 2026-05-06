//! Generic prediction-market identifier newtypes.
//!
//! Both [`EventID`] and [`OutcomeID`] are validated newtypes with normalization
//! applied at construction time, so two equivalent inputs (modulo case for hex
//! values, modulo surrounding whitespace, etc.) compare equal.
//!
//! # Hex normalization
//! Inputs starting with `0x` or `0X` are normalised to lowercase `0x...`. The
//! lowercase convention follows the de-facto standard used by EVM tooling, so
//! IDs round-trip stably regardless of how upstream providers spell them.

use crate::error::PredictionError;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{convert::TryFrom, fmt, str::FromStr};

/// Length of a canonical hex event ID (`0x` prefix plus 64 hex digits).
const EVENT_ID_LEN: usize = 66;

/// Maximum length of a canonical outcome ID after normalization.
const OUTCOME_ID_MAX_LEN: usize = 78;

#[inline]
fn invalid_event_id(value: &str) -> PredictionError {
    PredictionError::InvalidEventId(value.to_string())
}

#[inline]
fn invalid_outcome_id(value: &str) -> PredictionError {
    PredictionError::InvalidOutcomeId(value.to_string())
}

/// Returns `true` if `value` contains any ASCII control character (newline,
/// tab, NUL, etc.) or any Unicode control character.
fn contains_control_char(value: &str) -> bool {
    value.chars().any(char::is_control)
}

/// Trim, hex-case-fold, and validate an event id.
///
/// Any leading/trailing ASCII whitespace is removed first, then the hex digits
/// after `0x` are folded to lowercase. The length check is applied to the
/// normalized form so callers cannot bypass it by padding the input.
fn normalize_event_id(input: &str) -> Result<String, PredictionError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(invalid_event_id(input));
    }
    if contains_control_char(trimmed) {
        return Err(invalid_event_id(input));
    }

    let normalized = trimmed.to_ascii_lowercase();
    if normalized.len() != EVENT_ID_LEN {
        return Err(invalid_event_id(input));
    }
    if !normalized.starts_with("0x") {
        return Err(invalid_event_id(input));
    }
    if !normalized.as_bytes()[2..]
        .iter()
        .all(u8::is_ascii_hexdigit)
    {
        return Err(invalid_event_id(input));
    }

    Ok(normalized)
}

/// Trim and validate an outcome id.
///
/// Whitespace is stripped from the ends only; embedded whitespace and any
/// control characters are still rejected. The length check runs on the trimmed
/// value so a 78-digit id surrounded by spaces is accepted, while one that is
/// 78 digits *plus* embedded spaces is not.
fn normalize_outcome_id(input: &str) -> Result<String, PredictionError> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.len() > OUTCOME_ID_MAX_LEN {
        return Err(invalid_outcome_id(input));
    }
    if contains_control_char(trimmed) {
        return Err(invalid_outcome_id(input));
    }

    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return Err(invalid_outcome_id(input));
    };
    if first == '+' || first == '-' || first.is_whitespace() {
        return Err(invalid_outcome_id(input));
    }
    if !first.is_ascii_digit() || !chars.all(|c| c.is_ascii_digit()) {
        return Err(invalid_outcome_id(input));
    }

    Ok(trimmed.to_string())
}

/// Opaque wrapper for a validated prediction event ID.
///
/// Stores the canonical lowercase `0x...` form, so two inputs that differ only
/// in hex letter case or surrounding whitespace compare equal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct EventID(String);

impl EventID {
    /// Construct a new validated prediction event ID.
    ///
    /// Surrounding ASCII whitespace is stripped and the hex digits are folded
    /// to lowercase before validation. The normalized value must be exactly 66
    /// characters: the literal prefix `0x` followed by 64 hexadecimal digits.
    ///
    /// # Errors
    /// Returns [`PredictionError::InvalidEventId`] if the input is empty after
    /// trimming, contains control characters, or does not match the expected
    /// shape after normalization.
    pub fn new(value: &str) -> Result<Self, PredictionError> {
        let normalized = normalize_event_id(value)?;
        Ok(Self(normalized))
    }
}

impl FromStr for EventID {
    type Err = PredictionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for EventID {
    type Error = PredictionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<EventID> for String {
    fn from(id: EventID) -> Self {
        id.0
    }
}

impl<'de> Deserialize<'de> for EventID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}

impl AsRef<str> for EventID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EventID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Opaque wrapper for a validated prediction outcome ID.
///
/// Outcome IDs are decimal integers (often very large, hence the 78-digit
/// upper bound) used by upstream providers to refer to a specific tradeable
/// outcome of a [`crate::Market`]. Surrounding whitespace is stripped on
/// construction; embedded whitespace and control characters remain invalid.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct OutcomeID(String);

impl OutcomeID {
    /// Construct a new validated outcome ID.
    ///
    /// After trimming surrounding whitespace, the value must be a non-empty
    /// run of ASCII digits no longer than 78 characters, with no leading
    /// sign and no embedded whitespace or control characters.
    ///
    /// # Errors
    /// Returns [`PredictionError::InvalidOutcomeId`] when the trimmed value is
    /// empty, exceeds the 78-character bound, contains control characters, or
    /// includes any non-digit character.
    pub fn new(value: &str) -> Result<Self, PredictionError> {
        let normalized = normalize_outcome_id(value)?;
        Ok(Self(normalized))
    }
}

impl FromStr for OutcomeID {
    type Err = PredictionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for OutcomeID {
    type Error = PredictionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<OutcomeID> for String {
    fn from(id: OutcomeID) -> Self {
        id.0
    }
}

impl<'de> Deserialize<'de> for OutcomeID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}

impl AsRef<str> for OutcomeID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OutcomeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
