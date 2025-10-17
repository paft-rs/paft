//! Instrument identity and market snapshot models.
//!
//! These types provide both a lightweight and a full-featured view of an
//! instrument that downstream consumers can use for rendering UIs, building
//! reports, or persisting aggregated snapshots.
//!
//! - `FastInfo` focuses on the most frequently accessed fields for
//!   low-latency use cases.
//! - `Info` provides an extended, richer view including ranges and
//!   fundamentals.

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_domain::{Exchange, Isin, MarketState, Symbol};
use paft_money::{Currency, Money};
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Lightweight snapshot of commonly requested fields for an instrument.
///
/// Prefer `FastInfo` for list views and latency-sensitive paths. For
/// extended snapshots, see [`Info`].
pub struct FastInfo {
    /// Primary trading symbol/ticker as provided by the data source.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub symbol: Symbol,
    /// Human-friendly instrument name.
    pub name: Option<String>,
    /// Primary listing exchange, if known.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub exchange: Option<Exchange>,
    /// Current market session state (for example: Pre, Regular, Post).
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub market_state: Option<MarketState>,
    /// Quote currency used for monetary values in this snapshot.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub currency: Option<Currency>,
    /// Most recent traded/quoted price.
    pub last: Option<Money>,
    /// Previous session's official close price.
    pub previous_close: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Detailed instrument profile and market snapshot.
///
/// Includes identification fields, real-time snapshot metrics, intraday and
/// 52-week ranges, as well as a subset of fundamentals. All values are
/// optional to accommodate partially populated data from upstream sources.
pub struct Info {
    // Identity
    /// Primary trading symbol/ticker as provided by the data source.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub symbol: Symbol,
    /// Human-friendly instrument name.
    pub name: Option<String>,
    /// International Securities Identification Number.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub isin: Option<Isin>,
    /// Primary listing exchange, if known.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub exchange: Option<Exchange>,

    // Market snapshot
    /// Current market session state (for example: Pre, Regular, Post).
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub market_state: Option<MarketState>,
    /// Quote currency for all monetary values in this snapshot.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub currency: Option<Currency>,
    /// Most recent traded/quoted price.
    pub last: Option<Money>,
    /// Opening price for the current session.
    pub open: Option<Money>,
    /// Highest traded price observed during the current session.
    pub high: Option<Money>,
    /// Lowest traded price observed during the current session.
    pub low: Option<Money>,
    /// Previous session's official close price.
    pub previous_close: Option<Money>,

    // Ranges & volumes
    /// Intraday low for the current session.
    pub day_range_low: Option<Money>,
    /// Intraday high for the current session.
    pub day_range_high: Option<Money>,
    /// 52-week low.
    pub fifty_two_week_low: Option<Money>,
    /// 52-week high.
    pub fifty_two_week_high: Option<Money>,
    /// Today's trading volume.
    pub volume: Option<u64>,
    /// Average daily trading volume (commonly 30D or 90D average, depending on source).
    pub average_volume: Option<u64>,

    // Fundamentals (generic)
    /// Market capitalization (price × shares outstanding) in the quote currency.
    pub market_cap: Option<Money>,
    /// Number of shares currently outstanding.
    pub shares_outstanding: Option<u64>,
    /// Earnings per share, trailing twelve months.
    pub eps_ttm: Option<Money>,
    /// Price-to-earnings ratio, trailing twelve months.
    pub pe_ttm: Option<f64>,
    /// Dividend yield as a fraction (for example: `0.025` = 2.5%).
    pub dividend_yield: Option<f64>, // 0.025 = 2.5%
    /// Most recent ex-dividend date.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub ex_dividend_date: Option<NaiveDate>,

    // Timestamp of snapshot
    #[serde(with = "chrono::serde::ts_seconds_option")]
    /// Timestamp (UTC) when this snapshot was taken.
    pub as_of: Option<DateTime<Utc>>,
}
