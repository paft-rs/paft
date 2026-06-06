//! Order book and book-level types under the `paft_market::market::orderbook` namespace.

use chrono::{DateTime, Utc};
use paft_domain::Instrument;
use paft_money::{Currency, PriceAmount, QuantityAmount};
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericBookLevel<M = ()> {
    /// The price at this level.
    pub price: PriceAmount,

    /// The displayed size at this price, when reported by the source.
    #[cfg_attr(feature = "dataframe", df_derive(decimal(precision = 38, scale = 10)))]
    pub size: Option<QuantityAmount>,

    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericBookLevel<M> {
    /// Build a book level with the given price and (optional) size; `provider`
    /// is initialised via `M::default()`.
    #[must_use]
    pub fn new(price: PriceAmount, size: Option<QuantityAmount>) -> Self {
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
/// Generic over a provider metadata payload `B`, which is flattened into the
/// serialized representation, and a per-level metadata payload `L`. Use the
/// [`OrderBook`] alias for the standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericOrderBook<B = (), L = ()> {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,

    /// Timestamp (UTC) when this book snapshot was observed.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub as_of: Option<DateTime<Utc>>,

    /// Currency shared by every price amount in this book.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub currency: Currency,

    /// A vector of ask (sell) levels, typically sorted by price ascending.
    /// Direct construction and deserialization preserve provider order; use
    /// [`Self::is_sorted`] to check the advisory ordering or
    /// [`Self::sort_levels`] to canonicalize caller-owned data.
    pub asks: Vec<GenericBookLevel<L>>,

    /// A vector of bid (buy) levels, typically sorted by price descending.
    /// Direct construction and deserialization preserve provider order; use
    /// [`Self::is_sorted`] to check the advisory ordering or
    /// [`Self::sort_levels`] to canonicalize caller-owned data.
    pub bids: Vec<GenericBookLevel<L>>,

    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: B,
}

impl<B: Default, L> GenericOrderBook<B, L> {
    /// Build an empty order book for the given instrument with no snapshot
    /// timestamp; `provider` is initialised via `B::default()`.
    #[must_use]
    pub fn new(instrument: Instrument, currency: Currency) -> Self {
        Self {
            instrument,
            as_of: None,
            currency,
            asks: Vec::new(),
            bids: Vec::new(),
            provider: B::default(),
        }
    }
}

impl<B, L> GenericOrderBook<B, L> {
    /// Return `true` when ask levels are sorted by non-decreasing price and
    /// bid levels are sorted by non-increasing price.
    ///
    /// Duplicate prices are considered sorted; this method validates ordering
    /// only and does not enforce level uniqueness.
    #[must_use]
    pub fn is_sorted(&self) -> bool {
        let asks_sorted = self
            .asks
            .windows(2)
            .all(|pair| pair[0].price.as_decimal() <= pair[1].price.as_decimal());
        let bids_sorted = self
            .bids
            .windows(2)
            .all(|pair| pair[0].price.as_decimal() >= pair[1].price.as_decimal());

        asks_sorted && bids_sorted
    }

    /// Sort ask levels by ascending price and bid levels by descending price.
    ///
    /// Sorting is stable, so levels with equal prices keep their existing
    /// relative order. This method does not deduplicate levels.
    pub fn sort_levels(&mut self) {
        self.asks
            .sort_by(|lhs, rhs| lhs.price.as_decimal().cmp(rhs.price.as_decimal()));
        self.bids
            .sort_by(|lhs, rhs| rhs.price.as_decimal().cmp(lhs.price.as_decimal()));
    }
}

/// Standard `OrderBook` with no extra provider metadata.
pub type OrderBook = GenericOrderBook<(), ()>;
