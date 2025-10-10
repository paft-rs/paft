//! Option request types for expirations and chains.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Request to retrieve option expiration dates for the given underlying symbol.
pub struct OptionExpirationsRequest {
    /// Underlying symbol identifier.
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Request to retrieve an option chain for an underlying symbol and expiration date.
pub struct OptionChainRequest {
    /// Underlying symbol identifier.
    pub symbol: String,
    /// Option expiration date (naive date in the exchange's calendar).
    pub expiration: NaiveDate,
}
