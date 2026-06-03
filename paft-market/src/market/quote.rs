//! Quote types under the `paft_market::market::quote` namespace.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_domain::{Instrument, MarketState};
use paft_money::{Currency, PriceAmount};

use crate::market::orderbook::GenericBookLevel;

#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot quote data for an instrument at a single point in time.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`Quote`] alias for the standard
/// shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericQuote<M = ()> {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Display name.
    pub name: Option<String>,
    /// Currency shared by every price amount in this quote.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub currency: Currency,
    /// Market price (most recent trade).
    pub price: Option<PriceAmount>,
    /// Best bid: top-of-book quoted price on the buy side, with optional size.
    pub bid: Option<GenericBookLevel<M>>,
    /// Best ask: top-of-book quoted price on the sell side, with optional size.
    pub ask: Option<GenericBookLevel<M>>,
    /// Previous close price.
    pub previous_close: Option<PriceAmount>,
    /// Day volume.
    pub day_volume: Option<u64>,
    /// Market state.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub market_state: Option<MarketState>,
    /// Timestamp (UTC) when this quote snapshot was observed.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub as_of: Option<DateTime<Utc>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericQuote<M> {
    /// Build a quote with the given instrument and all optional fields unset.
    /// `provider` is initialised via `M::default()`.
    #[must_use]
    pub fn new(instrument: Instrument, currency: Currency) -> Self {
        Self {
            instrument,
            name: None,
            currency,
            price: None,
            bid: None,
            ask: None,
            previous_close: None,
            day_volume: None,
            market_state: None,
            as_of: None,
            provider: M::default(),
        }
    }
}

/// Standard `Quote` with no extra provider metadata.
pub type Quote = GenericQuote<()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Incremental update for an instrument during streaming sessions.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`QuoteUpdate`] alias for the
/// standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericQuoteUpdate<M = ()> {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Currency shared by every price amount in this update.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub currency: Currency,
    /// Last traded price, if present.
    pub price: Option<PriceAmount>,
    /// Previous close price.
    pub previous_close: Option<PriceAmount>,
    /// Volume traded since the previous update.
    pub volume: Option<u64>,
    /// Event timestamp as Unix milliseconds.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub ts: DateTime<Utc>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericQuoteUpdate<M> {
    /// Build a quote update with the given instrument and timestamp; all other
    /// fields default to `None` and `provider` is initialised via `M::default()`.
    #[must_use]
    pub fn new(instrument: Instrument, currency: Currency, ts: DateTime<Utc>) -> Self {
        Self {
            instrument,
            currency,
            price: None,
            previous_close: None,
            volume: None,
            ts,
            provider: M::default(),
        }
    }
}

/// Standard `QuoteUpdate` with no extra provider metadata.
pub type QuoteUpdate = GenericQuoteUpdate<()>;
