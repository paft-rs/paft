//! Order book and book-level types under the `paft_market::market::orderbook` namespace.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use chrono::{DateTime, Utc};
use paft_decimal::NonNegativeDecimal;
use paft_domain::Instrument;
use paft_money::Price;
use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

/// A single price level on one side of the market: a quoted price with an
/// optional displayed size.
///
/// Used both as one row of a depth snapshot in [`GenericOrderBook`] and as
/// the top-of-book bid/ask payload on [`crate::market::quote::GenericQuote`].
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`BookLevel`] alias for the standard
/// shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
///
/// `price` is mandatory: a level with no price isn't meaningful. `size` is
/// optional because real-world feeds frequently emit price-without-size —
/// delayed and aggregated equity feeds often strip sizes, and some real-time
/// venues' BBO updates routinely carry the size for only one side of the
/// market per message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericBookLevel<M = ()> {
    /// The price at this level.
    pub price: Price,

    /// The displayed size at this price, when reported by the source.
    #[cfg_attr(feature = "dataframe", df_derive(decimal(precision = 38, scale = 10)))]
    pub size: Option<NonNegativeDecimal>,

    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericBookLevel<M> {
    /// Build a book level with the given price and (optional) size; `provider`
    /// is initialised via `M::default()`.
    #[must_use]
    pub fn new(price: Price, size: Option<NonNegativeDecimal>) -> Self {
        Self {
            price,
            size,
            provider: M::default(),
        }
    }
}

/// Standard `BookLevel` with no extra provider metadata.
pub type BookLevel = GenericBookLevel<()>;

/// A snapshot of the order book for a specific instrument.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into each level. Use the
/// [`OrderBook`] alias for the standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericOrderBook<M = ()> {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,

    /// Timestamp (UTC) when this book snapshot was observed.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub as_of: Option<DateTime<Utc>>,

    /// A vector of ask (sell) levels, typically sorted by price ascending.
    pub asks: Vec<GenericBookLevel<M>>,

    /// A vector of bid (buy) levels, typically sorted by price descending.
    pub bids: Vec<GenericBookLevel<M>>,

    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericOrderBook<M> {
    /// Build an empty order book for the given instrument with no snapshot
    /// timestamp; `provider` is initialised via `M::default()`.
    #[must_use]
    pub fn new(instrument: Instrument) -> Self {
        Self {
            instrument,
            as_of: None,
            asks: Vec::new(),
            bids: Vec::new(),
            provider: M::default(),
        }
    }
}

/// Standard `OrderBook` with no extra provider metadata.
pub type OrderBook = GenericOrderBook<()>;
