//! Quote types under the `paft_market::market::quote` namespace.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
use paft_domain::{Instrument, MarketState};
use paft_money::{Currency, PriceAmount, QuantityAmount};

use crate::market::orderbook::GenericBookLevel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Snapshot quote data for an instrument at a single point in time.
///
/// Generic over a quote-level provider metadata payload `Q`, which is
/// flattened into the serialized representation, and a top-of-book level
/// metadata payload `L`. Use the [`Quote`] alias for the standard shape
/// (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericQuote<Q = (), L = ()> {
    /// Instrument identifier.
    pub instrument: Instrument,
    /// Display name.
    pub name: Option<String>,
    /// Currency shared by every price amount in this quote.
    pub currency: Currency,
    /// Market price (most recent trade).
    pub price: Option<PriceAmount>,
    /// Best bid: top-of-book quoted price on the buy side, with optional size.
    pub bid: Option<GenericBookLevel<L>>,
    /// Best ask: top-of-book quoted price on the sell side, with optional size.
    pub ask: Option<GenericBookLevel<L>>,
    /// Previous close price.
    pub previous_close: Option<PriceAmount>,
    /// Day volume in the provider's stated quantity unit.
    pub day_volume: Option<QuantityAmount>,
    /// Market state.
    pub market_state: Option<MarketState>,
    /// Timestamp (UTC) when this quote snapshot was observed.
    /// Serialization rejects leap seconds. `DataFrame` export uses exact nanoseconds
    /// and independently rejects instants outside the signed i64 nanosecond range.
    #[serde(default, with = "paft_core::serde_helpers::ts_iso8601_option")]
    pub as_of: Option<DateTime<Utc>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: Q,
}

#[cfg(feature = "dataframe")]
paft_utils::impl_checked_dataframe! {
    GenericQuote<Q, L> {
        instrument: [Instrument],
        name: [Option<String>],
        #[df_derive(as_str)]
        currency: [Currency],
        price: [Option<PriceAmount>],
        bid: [Option<GenericBookLevel<L>>],
        ask: [Option<GenericBookLevel<L>>],
        previous_close: [Option<PriceAmount>],
        day_volume: [Option<QuantityAmount>],
        #[df_derive(as_str)]
        market_state: [Option<MarketState>],
        #[df_derive(time_unit = "ns")]
        as_of: [Option<DateTime<Utc>>],
        provider: [Q],
    }
    validate |row| row.as_of.iter().try_for_each(|ts| paft_core::serde_helpers::validate_timestamp_nanos("as_of", ts))
}

impl<Q: Default, L> GenericQuote<Q, L> {
    /// Build a quote with the given instrument and all optional fields unset.
    /// `provider` is initialised via `Q::default()`.
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
            provider: Q::default(),
        }
    }
}

/// Standard `Quote` with no extra provider metadata.
pub type Quote = GenericQuote<(), ()>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Streaming quote update payload for an instrument.
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
    pub instrument: Instrument,
    /// Currency shared by every price amount in this update.
    pub currency: Currency,
    /// Last traded price, if present.
    pub price: Option<PriceAmount>,
    /// Previous close price.
    pub previous_close: Option<PriceAmount>,
    /// Latest known cumulative traded volume for this instrument in the
    /// provider's stated quantity unit.
    ///
    /// For equity feeds this is usually current session/day volume. For crypto
    /// and some derivatives feeds this may be a provider-defined rolling or
    /// trading-day window. This field is a snapshot value, not a per-update
    /// delta.
    pub volume: Option<QuantityAmount>,
    /// Event UTC instant, encoded as canonical ISO-8601-style text.
    /// Serialization rejects leap seconds. `DataFrame` export uses exact nanoseconds
    /// and independently rejects instants outside the signed i64 nanosecond range.
    #[serde(with = "paft_core::serde_helpers::ts_iso8601")]
    pub ts: DateTime<Utc>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

#[cfg(feature = "dataframe")]
paft_utils::impl_checked_dataframe! {
    GenericQuoteUpdate<M> {
        instrument: [Instrument],
        #[df_derive(as_str)]
        currency: [Currency],
        price: [Option<PriceAmount>],
        previous_close: [Option<PriceAmount>],
        volume: [Option<QuantityAmount>],
        #[df_derive(time_unit = "ns")]
        ts: [DateTime<Utc>],
        provider: [M],
    }
    validate |row| paft_core::serde_helpers::validate_timestamp_nanos("ts", &row.ts)
}

impl<M: Default> GenericQuoteUpdate<M> {
    /// Build a quote update with the given instrument and timestamp; all other
    /// fields default to `None` and `provider` is initialised via `M::default()`.
    /// The timestamp is retained unchanged in memory; serialization requires
    /// a non-leap-second instant, including after public mutation.
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
