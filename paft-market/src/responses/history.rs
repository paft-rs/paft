//! History response types.

use paft_core::domain::Money;
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
#[cfg(feature = "dataframe")]
use paft_core::dataframe::ToDataFrame;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single OHLCV bar at timestamp `ts` (Unix seconds).
///
/// Volume may be `None` when unavailable.
pub struct Candle {
    /// Timestamp for the bar (Unix seconds).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ts: DateTime<Utc>,
    /// Open price.
    pub open: Money,
    /// High price.
    pub high: Money,
    /// Low price.
    pub low: Money,
    /// Close or adjusted close depending on provider and request.
    pub close: Money,
    /// Volume if available.
    pub volume: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Optional metadata describing the history series.
pub struct HistoryMeta {
    /// IANA timezone identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub timezone: Option<Tz>,
    /// UTC offset in seconds.
    pub utc_offset_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// A complete history response including candles, actions, and metadata.
pub struct HistoryResponse {
    /// Ordered candles.
    pub candles: Vec<Candle>,
    /// Corporate actions aligned to candles.
    pub actions: Vec<crate::market::action::Action>,
    /// Whether prices are adjusted for splits/dividends.
    pub adjusted: bool,
    /// Optional metadata including timezone.
    pub meta: Option<HistoryMeta>,
    /// Original unadjusted close prices when adjusted close is provided.
    pub unadjusted_close: Option<Vec<Money>>,
}
