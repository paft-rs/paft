//! History response types.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use paft_money::Price;
use serde::{Deserialize, Serialize};

use crate::requests::history::Interval;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_domain::Instrument;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single OHLCV bar at timestamp `ts` (Unix seconds).
///
/// Volume may be `None` when unavailable.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`Candle`] alias for the standard
/// shape (no extra metadata).
pub struct GenericCandle<M = ()> {
    /// Timestamp for the bar (Unix seconds).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ts: DateTime<Utc>,
    /// Open price.
    pub open: Price,
    /// High price.
    pub high: Price,
    /// Low price.
    pub low: Price,
    /// Close or adjusted close depending on provider and request.
    pub close: Price,
    /// Original unadjusted close price, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_unadj: Option<Price>,
    /// Volume if available.
    pub volume: Option<u64>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericCandle<M> {
    /// Build a candle from the OHLC quadruple at the given timestamp.
    /// `close_unadj` and `volume` default to `None`; `provider` is initialised
    /// via `M::default()`.
    #[must_use]
    pub fn new(ts: DateTime<Utc>, open: Price, high: Price, low: Price, close: Price) -> Self {
        Self {
            ts,
            open,
            high,
            low,
            close,
            close_unadj: None,
            volume: None,
            provider: M::default(),
        }
    }
}

/// Standard `Candle` with no extra provider metadata.
pub type Candle = GenericCandle<()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Streaming candle update event.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into the inner candle. Use
/// the [`CandleUpdate`] alias for the standard shape (no extra metadata).
pub struct GenericCandleUpdate<M = ()> {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Interval represented by the candle.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub interval: Interval,
    /// The candle payload for the interval.
    pub candle: GenericCandle<M>,
    /// true when the bar is closed/final as per the upstream provider WebSocket
    pub is_final: bool,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericCandleUpdate<M> {
    /// Build a candle-update event from its required parts.
    /// `provider` is initialised via `M::default()`.
    #[must_use]
    pub fn new(
        instrument: Instrument,
        interval: Interval,
        candle: GenericCandle<M>,
        is_final: bool,
    ) -> Self {
        Self {
            instrument,
            interval,
            candle,
            is_final,
            provider: M::default(),
        }
    }
}

/// Standard `CandleUpdate` with no extra provider metadata.
pub type CandleUpdate = GenericCandleUpdate<()>;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// A complete history response including candles, actions, and metadata.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into each candle. Use the
/// [`HistoryResponse`] alias for the standard shape (no extra metadata).
pub struct GenericHistoryResponse<M = ()> {
    /// Ordered candles.
    pub candles: Vec<GenericCandle<M>>,
    /// Corporate actions aligned to candles.
    pub actions: Vec<crate::market::action::Action>,
    /// Whether prices are adjusted for splits/dividends.
    pub adjusted: bool,
    /// Optional metadata including timezone.
    pub meta: Option<HistoryMeta>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard `HistoryResponse` with no extra provider metadata.
pub type HistoryResponse = GenericHistoryResponse<()>;
