//! Option contracts and chains under the market namespace.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_money::Money;
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single option contract (call or put) at a given strike and expiration.
pub struct OptionContract {
    /// Provider-specific contract identifier.
    pub contract_symbol: String,
    /// Strike price of the contract.
    pub strike: Money,
    /// Last traded price.
    pub price: Option<Money>,
    /// Best bid.
    pub bid: Option<Money>,
    /// Best ask.
    pub ask: Option<Money>,
    /// Traded volume.
    pub volume: Option<u64>,
    /// Open interest at the time of fetch.
    pub open_interest: Option<u64>,
    /// Implied volatility as a fraction (e.g., 0.25 for 25%).
    pub implied_volatility: Option<f64>,
    /// Whether the option is currently in the money.
    pub in_the_money: bool,
    /// Expiration timestamp (Unix seconds).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expiration: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A full option chain split into calls and puts.
pub struct OptionChain {
    /// Call contracts.
    pub calls: Vec<OptionContract>,
    /// Put contracts.
    pub puts: Vec<OptionContract>,
}
