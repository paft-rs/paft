//! Option request types for expirations and chains.

use chrono::NaiveDate;
use paft_domain::Instrument;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Request to retrieve option expiration dates for the given underlying instrument.
pub struct OptionExpirationsRequest {
    /// Underlying instrument identifier.
    pub instrument: Instrument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Request to retrieve an option chain for an underlying instrument and expiration date.
pub struct OptionChainRequest {
    /// Underlying instrument identifier.
    pub instrument: Instrument,
    /// Option expiration date (naive date in the exchange's calendar).
    pub expiration: NaiveDate,
}
