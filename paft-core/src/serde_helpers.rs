//! Serde helper modules for custom serialization/deserialization.
//!
//! This module contains reusable serde helpers for common serialization patterns
//! used throughout the paft workspace.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serde helper for Vec<`DateTime`<`Utc`>> encoded as epoch seconds
pub mod ts_seconds_vec {
    use super::{DateTime, Deserialize, Deserializer, Serialize, Serializer, Utc};
    /// Serialize a vector of `DateTime<`Utc`>` as epoch seconds.
    ///
    /// # Errors
    /// This function delegates to the provided `serializer` and returns any
    /// serialization error emitted by it.
    pub fn serialize<S>(value: &[DateTime<Utc>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let seconds: Vec<i64> = value.iter().map(chrono::DateTime::timestamp).collect();
        seconds.serialize(serializer)
    }

    /// Deserialize a vector of epoch seconds into `DateTime<Utc>` values.
    ///
    /// # Errors
    /// Returns an error if the underlying deserializer fails or if any of the
    /// input timestamps are invalid and cannot be converted to a `DateTime<Utc>`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs: Vec<i64> = Vec::<i64>::deserialize(deserializer)?;
        secs.into_iter()
            .map(|s| {
                DateTime::from_timestamp(s, 0)
                    .ok_or_else(|| serde::de::Error::custom(format!("invalid timestamp: {s}")))
            })
            .collect()
    }
}
