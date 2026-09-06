//! Serde helper modules for custom serialization/deserialization.
//!
//! This module contains reusable serde helpers for common serialization patterns
//! used throughout the paft workspace.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod timestamps;
pub use timestamps::{
    TimestampError, TimestampErrorKind, parse_timestamp, timestamp_nanos_exact, ts_iso8601,
    ts_iso8601_option, ts_iso8601_vec, validate_timestamp, validate_timestamp_nanos,
};

/// Returns Unix milliseconds only when decoding them restores the same timestamp.
///
/// Rejects sub-millisecond precision and leap seconds, including leap-second
/// values with a millisecond-aligned nanosecond component. No rounding occurs.
#[must_use]
pub fn timestamp_millis_exact(value: &DateTime<Utc>) -> Option<i64> {
    let millis = value.timestamp_millis();
    (DateTime::from_timestamp_millis(millis) == Some(*value)).then_some(millis)
}

fn serialize_millis<E: serde::ser::Error>(value: &DateTime<Utc>) -> Result<i64, E> {
    timestamp_millis_exact(value).ok_or_else(|| {
        E::custom("timestamp cannot be preserved as Unix milliseconds (sub-millisecond precision or leap second)")
    })
}

/// Exact epoch-millisecond serde adapter for `DateTime<Utc>`.
///
/// Serialization rejects values that the integer wire format cannot preserve.
/// Public payload fields may hold higher precision in memory; callers must
/// explicitly normalize such values before choosing this wire format.
pub mod ts_milliseconds {
    use super::{DateTime, Deserializer, Serializer, Utc};

    /// Serialize a timestamp without silently discarding precision.
    ///
    /// # Errors
    /// Rejects sub-millisecond precision, leap seconds, and serializer failures.
    pub fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(super::serialize_millis(value)?)
    }

    /// Deserialize epoch milliseconds.
    ///
    /// # Errors
    /// Rejects invalid integers or timestamps outside Chrono's supported range.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        chrono::serde::ts_milliseconds::deserialize(deserializer)
    }
}

/// Exact epoch-millisecond serde adapter for `Option<DateTime<Utc>>`.
///
/// `None` is encoded as null. Use `#[serde(default, with = "...")]` when an
/// absent field should also deserialize as `None`.
pub mod ts_milliseconds_option {
    use super::{DateTime, Deserializer, Serialize, Serializer, Utc};

    /// Serialize an optional timestamp, preserving every supplied value exactly.
    ///
    /// # Errors
    /// Rejects sub-millisecond precision, leap seconds, and serializer failures.
    pub fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value
            .as_ref()
            .map(super::serialize_millis::<S::Error>)
            .transpose()?
            .serialize(serializer)
    }

    /// Deserialize nullable epoch milliseconds.
    ///
    /// # Errors
    /// Rejects invalid integers or timestamps outside Chrono's supported range.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        chrono::serde::ts_milliseconds_option::deserialize(deserializer)
    }
}

/// Exact epoch-millisecond serde adapter for `Vec<DateTime<Utc>>`.
pub mod ts_milliseconds_vec {
    use super::{DateTime, Deserialize, Deserializer, Serialize, Serializer, Utc};
    /// Serialize a vector of `DateTime<Utc>` as epoch milliseconds.
    ///
    /// # Errors
    /// Rejects sub-millisecond precision, leap seconds, and serializer failures.
    pub fn serialize<S>(value: &[DateTime<Utc>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let milliseconds: Vec<i64> = value
            .iter()
            .map(super::serialize_millis::<S::Error>)
            .collect::<Result<_, _>>()?;
        milliseconds.serialize(serializer)
    }

    /// Deserialize a vector of epoch milliseconds into `DateTime<Utc>` values.
    ///
    /// # Errors
    /// Returns an error if the underlying deserializer fails or if any of the
    /// input timestamps are invalid and cannot be converted to a `DateTime<Utc>`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis: Vec<i64> = Vec::<i64>::deserialize(deserializer)?;
        millis
            .into_iter()
            .map(|ms| {
                DateTime::from_timestamp_millis(ms)
                    .ok_or_else(|| serde::de::Error::custom(format!("invalid timestamp: {ms}")))
            })
            .collect()
    }
}
