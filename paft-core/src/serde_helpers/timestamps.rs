//! PAFT's exact UTC-instant text and nanosecond export boundaries.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Identifiable reasons for rejecting a timestamp at an ingestion/export boundary.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum TimestampErrorKind {
    /// The input is outside PAFT's deliberately narrow text grammar.
    #[error("invalid timestamp text syntax")]
    InvalidSyntax,
    /// More than nine fractional digits were supplied, including trailing zeros.
    #[error("timestamp fractional seconds exceed nine digits")]
    FractionalPrecision,
    /// Chrono rejected the calendar, clock, offset, or resulting UTC range.
    #[error("invalid timestamp calendar, clock, offset, or UTC range: {0}")]
    InvalidDateTime(chrono::ParseError),
    /// Leap seconds are outside PAFT's canonical instant contract.
    #[error("timestamp leap seconds are unsupported")]
    LeapSecond,
    /// The instant cannot be encoded as a signed i64 Unix nanosecond count.
    #[error("timestamp is outside the DataFrame Unix nanosecond range")]
    OutOfDataFrameRange,
}

/// Timestamp failure with the original input/value and available field context.
///
/// Core has no Polars dependency. Export integrations translate this error at
/// their boundary, retaining its reason and context.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{field}: {kind}: {timestamp}")]
pub struct TimestampError {
    /// Field name or collection element when supplied by the caller.
    pub field: String,
    /// Original text input, or an exact text rendering of an in-memory value.
    pub timestamp: String,
    /// Why this boundary rejected the timestamp.
    pub kind: TimestampErrorKind,
}

impl TimestampError {
    fn new(timestamp: impl Into<String>, kind: TimestampErrorKind) -> Self {
        Self {
            field: "timestamp".into(),
            timestamp: timestamp.into(),
            kind,
        }
    }

    /// Attach the field or collection-element name known by the caller.
    #[must_use]
    pub fn at_field(mut self, field: impl Into<String>) -> Self {
        self.field = field.into();
        self
    }
}

// This checks spelling only; Chrono remains responsible for calendar arithmetic,
// clock/offset validity, and checked conversion at its representable limits.
fn check_text_shape(input: &str) -> Result<(), TimestampErrorKind> {
    use TimestampErrorKind::{FractionalPrecision, InvalidSyntax};
    let bytes = input.as_bytes();
    if !input.is_ascii() {
        return Err(InvalidSyntax);
    }
    let signed = matches!(bytes.first(), Some(b'+' | b'-'));
    let first_digit = usize::from(signed);
    let digits = bytes[first_digit..]
        .iter()
        .take_while(|c| c.is_ascii_digit())
        .count();
    let year = &bytes[first_digit..first_digit + digits];
    let year_valid = match bytes.first() {
        Some(b'+') => (5..=6).contains(&digits) && year[0] != b'0',
        Some(b'-') => {
            (4..=6).contains(&digits)
                && (digits == 4 || year[0] != b'0')
                && year.iter().any(|c| *c != b'0')
        }
        _ => digits == 4,
    };
    if !year_valid {
        return Err(InvalidSyntax);
    }
    let rest = &bytes[first_digit + digits..];
    if rest.len() < 16
        || rest[0] != b'-'
        || rest[3] != b'-'
        || !matches!(rest[6], b'T' | b't')
        || rest[9] != b':'
        || rest[12] != b':'
        || ![1, 2, 4, 5, 7, 8, 10, 11, 13, 14]
            .iter()
            .all(|&i| rest[i].is_ascii_digit())
    {
        return Err(InvalidSyntax);
    }
    let mut zone = &rest[15..];
    if zone.first() == Some(&b'.') {
        let digits = zone[1..].iter().take_while(|c| c.is_ascii_digit()).count();
        if digits > 9 {
            return Err(FractionalPrecision);
        }
        if digits == 0 {
            return Err(InvalidSyntax);
        }
        zone = &zone[digits + 1..];
    }
    if matches!(zone, [b'Z' | b'z'])
        || (zone.len() == 6
            && matches!(zone[0], b'+' | b'-')
            && zone[3] == b':'
            && [1, 2, 4, 5].iter().all(|&i| zone[i].is_ascii_digit()))
    {
        Ok(())
    } else {
        Err(InvalidSyntax)
    }
}

/// Parse PAFT UTC ISO-8601-style timestamp text without discarding precision.
///
/// Ordinary years use four unsigned digits and RFC 3339 syntax. Expanded
/// positive years use `+` and five or six digits; negative years use `-` and
/// four to six digits. Four-digit negative years may be padded. Expanded forms
/// have no redundant leading zeros; negative zero is rejected. Chrono determines
/// the supported calendar range, including checked offset-to-UTC conversion.
///
/// Components are padded, separated by `T`/`t`, with optional one-to-nine
/// fractional digits and `Z`/`z` or `±HH:MM`. Whitespace, `UTC`, colonless offsets,
/// and leap seconds are rejected. The digit limit is checked **before** parsing;
/// even excess trailing zeros are invalid. This is not Chrono's relaxed grammar.
///
/// # Errors
/// Returns syntax, fractional-precision, calendar/clock/offset/range, or leap-second
/// errors. The smaller `DataFrame` nanosecond range does not restrict this parser.
pub fn parse_timestamp(input: &str) -> Result<DateTime<Utc>, TimestampError> {
    check_text_shape(input).map_err(|kind| TimestampError::new(input, kind))?;
    // FromStr supports Chrono's signed-year extension. Its checked timezone
    // mapping returns a parse error when the normalized UTC date would overflow.
    let value = input
        .parse::<DateTime<Utc>>()
        .map_err(|error| TimestampError::new(input, TimestampErrorKind::InvalidDateTime(error)))?;
    validate_timestamp(&value).map_err(|error| TimestampError::new(input, error.kind))?;
    Ok(value)
}

/// Validate an in-memory instant for canonical JSON, independently of export range.
///
/// # Errors
/// Rejects Chrono's exceptional leap-second representation.
pub fn validate_timestamp(value: &DateTime<Utc>) -> Result<(), TimestampError> {
    if value.timestamp_subsec_nanos() >= 1_000_000_000 {
        return Err(TimestampError::new(
            value.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            TimestampErrorKind::LeapSecond,
        ));
    }
    Ok(())
}

/// Convert to exact signed i64 Unix nanoseconds without panicking or rounding.
///
/// # Errors
/// Rejects leap seconds separately from values outside the nanosecond range.
/// JSON can still represent non-leap-second values outside this export range.
pub fn timestamp_nanos_exact(value: &DateTime<Utc>) -> Result<i64, TimestampError> {
    validate_timestamp(value)?;
    value.timestamp_nanos_opt().ok_or_else(|| {
        TimestampError::new(
            value.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            TimestampErrorKind::OutOfDataFrameRange,
        )
    })
}

/// Validate an export field while retaining its name in any timestamp error.
///
/// # Errors
/// Returns the same reasons as [`timestamp_nanos_exact`], with field context.
pub fn validate_timestamp_nanos(field: &str, value: &DateTime<Utc>) -> Result<(), TimestampError> {
    timestamp_nanos_exact(value)
        .map(|_| ())
        .map_err(|error| error.at_field(field))
}

fn timestamp_text(value: &DateTime<Utc>) -> Result<String, TimestampError> {
    validate_timestamp(value)?;
    Ok(value.to_rfc3339_opts(SecondsFormat::AutoSi, true))
}

/// Canonical string-only serde adapter for UTC instants.
///
/// Output uses UTC `Z` and `AutoSi`'s shortest exact width among zero, three, six,
/// or nine fractional digits. Ordinary years follow RFC 3339 syntax; other years
/// use Chrono's signed-year ISO 8601 extension. Canonicalization preserves the
/// instant, not the source offset, spelling, or declared precision. Numeric JSON
/// timestamps require an explicit legacy adapter or source-schema migration;
/// no epoch unit is inferred. See [`parse_timestamp`] for the accepted grammar.
pub mod ts_iso8601 {
    use super::{
        DateTime, Deserialize, Deserializer, Serialize, Serializer, Utc, parse_timestamp,
        timestamp_text,
    };

    /// Serialize a validated UTC instant as canonical text.
    /// # Errors
    /// Rejects leap seconds and serializer errors.
    pub fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        timestamp_text(value)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    /// Deserialize timestamp strings only, preserving the exact instant.
    /// # Errors
    /// Rejects non-string input and violations of [`parse_timestamp`]'s contract.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        parse_timestamp(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Nullable version of [`ts_iso8601`]. Add `serde(default)` to allow omission.
pub mod ts_iso8601_option {
    use super::{
        DateTime, Deserialize, Deserializer, Serialize, Serializer, Utc, parse_timestamp,
        timestamp_text,
    };

    /// Serialize `None` as null and supplied instants as exact canonical text.
    /// # Errors
    /// Rejects leap seconds and serializer errors.
    pub fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value
            .as_ref()
            .map(timestamp_text)
            .transpose()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    /// Deserialize a string or null, with no numeric timestamp compatibility mode.
    /// # Errors
    /// Rejects non-string/non-null input and invalid timestamp text.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        Option::<String>::deserialize(deserializer)?
            .map(|text| parse_timestamp(&text))
            .transpose()
            .map_err(serde::de::Error::custom)
    }
}

/// List version of [`ts_iso8601`], preserving each supplied instant exactly.
pub mod ts_iso8601_vec {
    use super::{
        DateTime, Deserialize, Deserializer, Serialize, Serializer, Utc, parse_timestamp,
        timestamp_text,
    };

    /// Serialize a timestamp list as canonical text, with indexed errors.
    /// # Errors
    /// Rejects leap seconds and serializer errors.
    pub fn serialize<S: Serializer>(
        value: &[DateTime<Utc>],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value
            .iter()
            .enumerate()
            .map(|(i, ts)| {
                timestamp_text(ts).map_err(|error| error.at_field(format!("timestamps[{i}]")))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    /// Deserialize timestamp strings, retaining an invalid element's index.
    /// # Errors
    /// Rejects non-string elements and invalid timestamp text.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<DateTime<Utc>>, D::Error> {
        Vec::<String>::deserialize(deserializer)?
            .iter()
            .enumerate()
            .map(|(i, text)| {
                parse_timestamp(text).map_err(|error| error.at_field(format!("timestamps[{i}]")))
            })
            .collect::<Result<_, _>>()
            .map_err(serde::de::Error::custom)
    }
}
