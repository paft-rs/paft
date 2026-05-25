//! Option contracts and chains under the market namespace.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use serde::{Deserialize, Serialize};

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_decimal::Decimal;
use paft_domain::Instrument;
use paft_money::Price;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Primary first-order greeks for an option contract.
pub struct OptionGreeks {
    /// Dimensionless change in option price for a 1.0 change in underlying price.
    pub delta: Option<Decimal>,
    /// Change in `delta` per 1.0 change in underlying price (1/price units).
    pub gamma: Option<Decimal>,
    /// Change in option price per calendar day.
    pub theta: Option<Decimal>,
    /// Change in option price for a 1 percentage point (0.01) change in IV.
    pub vega: Option<Decimal>,
    /// Change in option price for a 1 percentage point (0.01) change in rate.
    pub rho: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single option contract (call or put) at a given strike and expiration.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`OptionContract`] alias for the
/// standard shape (no extra metadata).
pub struct GenericOptionContract<M = ()> {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Strike price of the contract.
    pub strike: Price,
    /// Last traded price.
    pub price: Option<Price>,
    /// Best bid.
    pub bid: Option<Price>,
    /// Best ask.
    pub ask: Option<Price>,
    /// Traded volume.
    pub volume: Option<u64>,
    /// Open interest at the time of fetch.
    pub open_interest: Option<u64>,
    /// Implied volatility as a fraction (e.g., 0.25 for 25%).
    pub implied_volatility: Option<Decimal>,
    /// Whether the option is currently in the money.
    pub in_the_money: bool,
    /// Canonical expiration calendar date.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub expiration_date: NaiveDate,
    /// Exact UTC expiration instant, if known.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub expiration_at: Option<DateTime<Utc>>,
    /// Exact UTC last trade instant, if known.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub last_trade_at: Option<DateTime<Utc>>,
    /// Optional first-order greeks for the contract.
    pub greeks: Option<OptionGreeks>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericOptionContract<M> {
    /// Build an option contract from its required parts. All quoting fields
    /// (`price`, `bid`, `ask`, …), `volume`, `open_interest`,
    /// `implied_volatility`, `expiration_at`, `last_trade_at`, and `greeks`
    /// default to `None`. `in_the_money` defaults to `false`. `provider` is
    /// initialised via `M::default()`.
    #[must_use]
    pub fn new(instrument: Instrument, strike: Price, expiration_date: NaiveDate) -> Self {
        Self {
            instrument,
            strike,
            price: None,
            bid: None,
            ask: None,
            volume: None,
            open_interest: None,
            implied_volatility: None,
            in_the_money: false,
            expiration_date,
            expiration_at: None,
            last_trade_at: None,
            greeks: None,
            provider: M::default(),
        }
    }
}

/// Standard `OptionContract` with no extra provider metadata.
pub type OptionContract = GenericOptionContract<()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A full option chain split into calls and puts.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into each contract. Use the
/// [`OptionChain`] alias for the standard shape (no extra metadata).
pub struct GenericOptionChain<M = ()> {
    /// Call contracts.
    pub calls: Vec<GenericOptionContract<M>>,
    /// Put contracts.
    pub puts: Vec<GenericOptionContract<M>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard `OptionChain` with no extra provider metadata.
pub type OptionChain = GenericOptionChain<()>;

/// A point-in-time update for an option contract.
///
/// This represents incremental changes to market data commonly used for options,
/// such as bid/ask, last price, and implied volatility, keyed by the underlying
/// symbol for routing and session filtering.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`OptionUpdate`] alias for the
/// standard shape (no extra metadata).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericOptionUpdate<M = ()> {
    /// Underlying instrument used for routing and monotonic filtering.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Timestamp of the update (Unix seconds).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ts: DateTime<Utc>,
    /// Best bid for the contract, if available.
    pub bid: Option<Price>,
    /// Best ask for the contract, if available.
    pub ask: Option<Price>,
    /// Last traded price, if available.
    pub last_price: Option<Price>,
    /// Implied volatility estimate, if available.
    pub implied_volatility: Option<Decimal>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericOptionUpdate<M> {
    /// Build an option update from its instrument and timestamp; all
    /// quoting fields default to `None` and `provider` is initialised via
    /// `M::default()`.
    #[must_use]
    pub fn new(instrument: Instrument, ts: DateTime<Utc>) -> Self {
        Self {
            instrument,
            ts,
            bid: None,
            ask: None,
            last_price: None,
            implied_volatility: None,
            provider: M::default(),
        }
    }
}

/// Standard `OptionUpdate` with no extra provider metadata.
pub type OptionUpdate = GenericOptionUpdate<()>;
