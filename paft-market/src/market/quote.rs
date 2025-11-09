//! Quote types under the `paft_market::market::quote` namespace.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_domain::{Exchange, Instrument, MarketState};
use paft_money::Money;
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;

#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Snapshot quote data for an instrument at a single point in time.
pub struct Quote {
    /// Instrument identifier.
    pub instrument: Instrument,
    /// Short display name.
    pub shortname: Option<String>,
    /// Market price.
    pub price: Option<Money>,
    /// Previous close price.
    pub previous_close: Option<Money>,
    /// Day volume.
    pub day_volume: Option<u64>,
    /// Exchange identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub exchange: Option<Exchange>,
    /// Market state.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub market_state: Option<MarketState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Incremental update for an instrument during streaming sessions.
pub struct QuoteUpdate {
    /// Instrument identifier.
    pub instrument: Instrument,
    /// Last traded price, if present.
    pub price: Option<Money>,
    /// Previous close price.
    pub previous_close: Option<Money>,
    /// Volume traded since the previous update.
    pub volume: Option<u64>,
    /// Event timestamp (Unix seconds).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ts: DateTime<Utc>,
}
