//! Option response types for expirations.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Response containing available option expiration dates for an underlying.
pub struct OptionExpirationsResponse {
    /// Expiration dates as supplied by a provider.
    ///
    /// Providers are expected to return these sorted ascending and without
    /// duplicates, but direct struct construction and deserialization do not
    /// enforce that. Use [`Self::new_sorted`] to canonicalize caller-provided
    /// dates.
    pub dates: Vec<NaiveDate>,
}

impl OptionExpirationsResponse {
    /// Build a response with expiration dates sorted ascending and deduplicated.
    #[must_use]
    pub fn new_sorted(mut dates: Vec<NaiveDate>) -> Self {
        dates.sort_unstable();
        dates.dedup();
        Self { dates }
    }
}
