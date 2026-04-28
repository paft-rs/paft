//! Order book types under the `paft_market::market::orderbook` namespace.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use paft_decimal::Decimal; // Decimal for size
use paft_money::Money; // Money for priced values
use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

/// A single entry (bid or ask) in an order book.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`OrderBookEntry`] alias for the
/// standard shape (no extra metadata).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericOrderBookEntry<M = ()> {
    /// The price point for this entry.
    pub price: Money,

    /// The total quantity (size) available at this price point.
    pub size: Decimal,

    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericOrderBookEntry<M> {
    /// Build an order-book entry with the given price and size; `provider` is
    /// initialised via `M::default()`.
    #[must_use]
    pub fn new(price: Money, size: Decimal) -> Self {
        Self {
            price,
            size,
            provider: M::default(),
        }
    }
}

/// Standard `OrderBookEntry` with no extra provider metadata.
pub type OrderBookEntry = GenericOrderBookEntry<()>;

/// A snapshot of the order book for a specific instrument.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into each entry. Use the
/// [`OrderBook`] alias for the standard shape (no extra metadata).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericOrderBook<M = ()> {
    /// A vector of ask (sell) entries, typically sorted by price ascending.
    pub asks: Vec<GenericOrderBookEntry<M>>,

    /// A vector of bid (buy) entries, typically sorted by price descending.
    pub bids: Vec<GenericOrderBookEntry<M>>,

    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard `OrderBook` with no extra provider metadata.
pub type OrderBook = GenericOrderBook<()>;
