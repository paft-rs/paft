//! Option response types for expirations.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Response containing available option expiration dates for an underlying.
pub struct OptionExpirationsResponse {
    /// Sorted list of expiration dates.
    pub dates: Vec<NaiveDate>,
}
