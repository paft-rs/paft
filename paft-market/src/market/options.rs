//! Option contracts and chains under the market namespace.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_domain::Symbol;
use paft_money::Money;
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Primary first-order greeks for an option contract.
pub struct OptionGreeks {
    /// Dimensionless change in option price for a 1.0 change in underlying price.
    pub delta: Option<f64>,
    /// Change in `delta` per 1.0 change in underlying price (1/price units).
    pub gamma: Option<f64>,
    /// Change in option price per calendar day.
    pub theta: Option<f64>,
    /// Change in option price for a 1 percentage point (0.01) change in IV.
    pub vega: Option<f64>,
    /// Change in option price for a 1 percentage point (0.01) change in rate.
    pub rho: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single option contract (call or put) at a given strike and expiration.
pub struct OptionContract {
    /// Provider-specific contract identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub contract_symbol: Symbol,
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
    /// Canonical expiration calendar date.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub expiration_date: NaiveDate,
    /// Exact UTC expiration instant, if known.
    #[serde(with = "paft_core::serde_helpers::ts_seconds_option")]
    pub expiration_at: Option<DateTime<Utc>>,
    /// Exact UTC last trade instant, if known.
    #[serde(with = "paft_core::serde_helpers::ts_seconds_option")]
    pub last_trade_at: Option<DateTime<Utc>>,
    /// Optional first-order greeks for the contract.
    pub greeks: Option<OptionGreeks>,
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
