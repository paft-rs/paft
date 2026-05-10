//! Quote types under the `paft_market::market::quote` namespace.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_decimal::Decimal;
use paft_domain::{Exchange, Instrument, MarketState};
use paft_money::Money;
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;

#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Snapshot quote data for an instrument at a single point in time.
pub struct Quote {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Short display name.
    pub shortname: Option<String>,
    /// Market price.
    pub price: Option<Money>,
    /// Opening price for the current session.
    pub open: Option<Money>,
    /// Intraday high.
    pub day_range_high: Option<Money>,
    /// Intraday low.
    pub day_range_low: Option<Money>,
    /// 52-week high.
    pub fifty_two_week_high: Option<Money>,
    /// 52-week low.
    pub fifty_two_week_low: Option<Money>,
    /// Previous close price.
    pub previous_close: Option<Money>,
    /// Day volume.
    pub day_volume: Option<u64>,
    /// Average daily volume (3-month).
    pub average_volume: Option<u64>,
    /// Market capitalisation.
    pub market_cap: Option<Money>,
    /// Shares outstanding.
    pub shares_outstanding: Option<u64>,
    /// Earnings per share, trailing twelve months.
    pub eps_ttm: Option<Money>,
    /// Price-to-earnings ratio, trailing twelve months.
    pub pe_ttm: Option<Decimal>,
    /// Trailing annual dividend yield as a fraction.
    pub dividend_yield: Option<Decimal>,
    /// Most recent ex-dividend date.
    pub ex_dividend_date: Option<NaiveDate>,
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
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
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
