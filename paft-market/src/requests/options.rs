//! Option request types for expirations and chains.

use chrono::NaiveDate;
use paft_domain::Symbol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Request to retrieve option expiration dates for the given underlying symbol.
pub struct OptionExpirationsRequest {
    /// Underlying symbol identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub symbol: Symbol,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Request to retrieve an option chain for an underlying symbol and expiration date.
pub struct OptionChainRequest {
    /// Underlying symbol identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub symbol: Symbol,
    /// Option expiration date (naive date in the exchange's calendar).
    pub expiration: NaiveDate,
}
