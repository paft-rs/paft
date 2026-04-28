//! Instant-in-time market snapshot model.
//!
//! [`Snapshot`] is a strictly snapshot-shaped view of an instrument: identity,
//! the current session's prices, and the timestamp the snapshot was taken at.
//! Fundamentals, analyst coverage, and ESG fields are intentionally excluded
//! and live in `paft-fundamentals`.

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_domain::{Exchange, Instrument, MarketState};
use paft_money::{Currency, Money};
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Strictly instant-in-time snapshot for an instrument.
///
/// All fields except `instrument` are optional to accommodate partially
/// populated data from upstream sources.
pub struct Snapshot {
    /// Primary instrument as provided by the data source.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Human-friendly instrument name.
    pub name: Option<String>,
    /// Primary listing exchange, if known.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub exchange: Option<Exchange>,
    /// Quote currency used for monetary values in this snapshot.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub currency: Option<Currency>,
    /// Current market session state (for example: Pre, Regular, Post).
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub market_state: Option<MarketState>,
    /// Timestamp (UTC) when this snapshot was taken.
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub as_of: Option<DateTime<Utc>>,
    /// Most recent traded/quoted price.
    pub last: Option<Money>,
    /// Previous session's official close price.
    pub previous_close: Option<Money>,
    /// Opening price for the current session.
    pub open: Option<Money>,
    /// Highest traded price observed during the current session.
    pub day_high: Option<Money>,
    /// Lowest traded price observed during the current session.
    pub day_low: Option<Money>,
    /// Today's trading volume.
    pub volume: Option<u64>,
}
