//! Option contracts and chains under the market namespace.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_decimal::{Decimal, NonNegativeDecimal};
use paft_domain::Instrument;
use paft_money::{Currency, Price, PriceAmount};
use std::fmt;
use std::str::FromStr;

use crate::error::MarketError;

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

    fn from_code(value: &str) -> Option<Self> {
        match value {
            "CALL" => Some(Self::Call),
            "PUT" => Some(Self::Put),
            _ => None,
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

impl FromStr for OptionSide {
    type Err = MarketError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_code(value).ok_or_else(|| MarketError::InvalidEnumValue {
            enum_name: "OptionSide",
            value: value.to_string(),
        })
    }
}

/// Identity of an option contract.
///
/// The required fields form the option's economic key. When
/// `contract_instrument` is present, it participates in equality and hashing so
/// adjusted contracts, venue-specific listings, and other distinct listed
/// contracts with the same economic terms do not collapse to the same key.
/// Without a contract instrument this remains an economic key, not a guaranteed
/// unique listed-contract identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct OptionContractKey {
    /// Provider or venue instrument identifier for the option contract, when known.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub contract_instrument: Option<Instrument>,
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
    /// Build an option contract key from its required economic fields.
    #[must_use]
    pub const fn new(
        underlying: Instrument,
        side: OptionSide,
        strike: Price,
        expiration_date: NaiveDate,
    ) -> Self {
        Self {
            contract_instrument: None,
            underlying,
            side,
            strike,
            expiration_date,
        }
    }

    /// Add the listed contract instrument identifier to this key.
    #[must_use]
    pub fn with_contract_instrument(mut self, contract_instrument: Instrument) -> Self {
        self.contract_instrument = Some(contract_instrument);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Contract key fields.
    #[serde(flatten)]
    #[cfg_attr(feature = "dataframe", df_derive(flatten))]
    pub key: OptionContractKey,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A full option chain for one or more expirations.
///
/// Generic over a chain-level provider metadata payload `R`, which is
/// flattened into the serialized representation, and a contract-level
/// metadata payload `C`. Use the [`OptionChain`] alias for the standard shape
/// (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericOptionChain<R = (), C = ()> {
    /// Option contracts in the chain.
    pub contracts: Vec<GenericOptionContract<C>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: R,
}

impl<R, C> GenericOptionChain<R, C> {
    /// Iterate over contracts for the requested option side.
    pub fn by_side(
        &self,
        side: OptionSide,
    ) -> impl Iterator<Item = &GenericOptionContract<C>> + '_ {
        self.contracts
            .iter()
            .filter(move |contract| contract.key.side == side)
    }

    /// Iterate over call contracts.
    pub fn calls(&self) -> impl Iterator<Item = &GenericOptionContract<C>> + '_ {
        self.by_side(OptionSide::Call)
    }

    /// Iterate over put contracts.
    pub fn puts(&self) -> impl Iterator<Item = &GenericOptionContract<C>> + '_ {
        self.by_side(OptionSide::Put)
    }
}

/// Standard `OptionChain` with no extra provider metadata.
pub type OptionChain = GenericOptionChain<(), ()>;

/// A point-in-time update for an option contract.
///
/// This represents incremental changes to market data commonly used for options,
/// such as bid/ask, last price, and implied volatility, keyed by the option
/// contract key.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`OptionUpdate`] alias for the
/// standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct GenericOptionUpdate<M = ()> {
    /// Contract key fields.
    #[serde(flatten)]
    #[cfg_attr(feature = "dataframe", df_derive(flatten))]
    pub key: OptionContractKey,
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
