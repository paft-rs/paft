//! Serde helper modules for custom serialization/deserialization.
//!
//! This module contains reusable serde helpers for common serialization patterns
//! used throughout the paft workspace.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serde helper for `Vec<DateTime<Utc>>` encoded as epoch milliseconds.
pub mod ts_milliseconds_vec {
    use super::{DateTime, Deserialize, Deserializer, Serialize, Serializer, Utc};
    /// Serialize a vector of `DateTime<Utc>` as epoch milliseconds.
    ///
    /// # Errors
    /// This function delegates to the provided `serializer` and returns any
    /// serialization error emitted by it.
    pub fn serialize<S>(value: &[DateTime<Utc>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let milliseconds: Vec<i64> = value
            .iter()
            .map(chrono::DateTime::timestamp_millis)
            .collect();
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
