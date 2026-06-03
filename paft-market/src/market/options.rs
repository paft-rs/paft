//! Option contracts and chains under the market namespace.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use serde::{Deserialize, Serialize};

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_decimal::{Decimal, NonNegativeDecimal};
use paft_domain::Instrument;
use paft_money::{Currency, Price, PriceAmount};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Primary first-order greeks for an option contract.
pub struct OptionGreeks {
    /// Dimensionless change in option price for a 1.0 change in underlying price.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub delta: Option<Decimal>,
    /// Change in `delta` per 1.0 change in underlying price (1/price units).
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub gamma: Option<Decimal>,
    /// Change in option price per calendar day.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub theta: Option<Decimal>,
    /// Change in option price for a 1 percentage point (0.01) change in IV.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub vega: Option<Decimal>,
    /// Change in option price for a 1 percentage point (0.01) change in rate.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub rho: Option<Decimal>,
}

/// Whether an option contract gives the right to buy or sell the underlying.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OptionSide {
    /// Call option: right to buy the underlying at the strike price.
    Call,
    /// Put option: right to sell the underlying at the strike price.
    Put,
}

impl OptionSide {
    /// Returns the canonical uppercase string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Call => "CALL",
            Self::Put => "PUT",
        }
    }
}

impl AsRef<str> for OptionSide {
    fn as_ref(&self) -> &str {
        (*self).as_str()
    }
}

impl fmt::Display for OptionSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

/// Economic identity of an option contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct OptionContractKey {
    /// Underlying instrument the option is written on.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub underlying: Instrument,
    /// Call or put side of the option contract.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub side: OptionSide,
    /// Strike price of the contract.
    pub strike: Price,
    /// Canonical expiration calendar date.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub expiration_date: NaiveDate,
}

impl OptionContractKey {
    /// Build an option contract identity from its required economic fields.
    #[must_use]
    pub const fn new(
        underlying: Instrument,
        side: OptionSide,
        strike: Price,
        expiration_date: NaiveDate,
    ) -> Self {
        Self {
            underlying,
            side,
            strike,
            expiration_date,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single option contract (call or put) at a given strike and expiration.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`OptionContract`] alias for the
/// standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericOptionContract<M = ()> {
    /// Contract identity fields.
    #[serde(flatten)]
    #[cfg_attr(feature = "dataframe", df_derive(flatten))]
    pub key: OptionContractKey,
    /// Provider or venue instrument identifier for the option contract, when known.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub contract_instrument: Option<Instrument>,
    /// Premium currency for `price`, `bid`, and `ask`.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub currency: Currency,
    /// Last traded price amount, denominated in `currency`.
    pub price: Option<PriceAmount>,
    /// Best bid amount, denominated in `currency`.
    pub bid: Option<PriceAmount>,
    /// Best ask amount, denominated in `currency`.
    pub ask: Option<PriceAmount>,
    /// Traded volume.
    pub volume: Option<u64>,
    /// Open interest at the time of fetch.
    pub open_interest: Option<u64>,
    /// Implied volatility as a non-negative fraction (e.g., 0.25 for 25%).
    #[cfg_attr(feature = "dataframe", df_derive(decimal(precision = 38, scale = 10)))]
    pub implied_volatility: Option<NonNegativeDecimal>,
    /// Whether the provider reports the option as currently in the money.
    ///
    /// `None` means the provider did not report this value.
    pub in_the_money: Option<bool>,
    /// Exact UTC expiration instant, if known.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub expiration_at: Option<DateTime<Utc>>,
    /// Exact UTC last trade instant, if known.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
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
    /// default to `None`, including `in_the_money`. `provider` is
    /// initialised via `M::default()`.
    #[must_use]
    pub fn new(key: OptionContractKey, currency: Currency) -> Self {
        Self {
            key,
            contract_instrument: None,
            currency,
            price: None,
            bid: None,
            ask: None,
            volume: None,
            open_interest: None,
            implied_volatility: None,
            in_the_money: None,
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
/// A full option chain for one or more expirations.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into each contract. Use the
/// [`OptionChain`] alias for the standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericOptionChain<M = ()> {
    /// Option contracts in the chain.
    pub contracts: Vec<GenericOptionContract<M>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M> GenericOptionChain<M> {
    /// Iterate over contracts for the requested option side.
    pub fn by_side(
        &self,
        side: OptionSide,
    ) -> impl Iterator<Item = &GenericOptionContract<M>> + '_ {
        self.contracts
            .iter()
            .filter(move |contract| contract.key.side == side)
    }

    /// Iterate over call contracts.
    pub fn calls(&self) -> impl Iterator<Item = &GenericOptionContract<M>> + '_ {
        self.by_side(OptionSide::Call)
    }

    /// Iterate over put contracts.
    pub fn puts(&self) -> impl Iterator<Item = &GenericOptionContract<M>> + '_ {
        self.by_side(OptionSide::Put)
    }
}

/// Standard `OptionChain` with no extra provider metadata.
pub type OptionChain = GenericOptionChain<()>;

/// A point-in-time update for an option contract.
///
/// This represents incremental changes to market data commonly used for options,
/// such as bid/ask, last price, and implied volatility, keyed by the option
/// contract identity.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`OptionUpdate`] alias for the
/// standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericOptionUpdate<M = ()> {
    /// Contract identity fields.
    #[serde(flatten)]
    #[cfg_attr(feature = "dataframe", df_derive(flatten))]
    pub key: OptionContractKey,
    /// Provider or venue instrument identifier for the option contract, when known.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub contract_instrument: Option<Instrument>,
    /// Timestamp of the update as Unix milliseconds.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub ts: DateTime<Utc>,
    /// Premium currency for `bid`, `ask`, and `last_price`.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub currency: Currency,
    /// Best bid amount for the contract, denominated in `currency`.
    pub bid: Option<PriceAmount>,
    /// Best ask amount for the contract, denominated in `currency`.
    pub ask: Option<PriceAmount>,
    /// Last traded price amount, denominated in `currency`.
    pub last_price: Option<PriceAmount>,
    /// Implied volatility estimate, if available.
    #[cfg_attr(feature = "dataframe", df_derive(decimal(precision = 38, scale = 10)))]
    pub implied_volatility: Option<NonNegativeDecimal>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericOptionUpdate<M> {
    /// Build an option update from its contract identity and timestamp; all
    /// quoting fields default to `None` and `provider` is initialised via
    /// `M::default()`.
    #[must_use]
    pub fn new(key: OptionContractKey, currency: Currency, ts: DateTime<Utc>) -> Self {
        Self {
            key,
            contract_instrument: None,
            ts,
            currency,
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
