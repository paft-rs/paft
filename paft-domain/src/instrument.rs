//! Instrument identifier and asset classification domain types.

use super::Exchange;
use crate::{
    DomainError,
    identifiers::{Figi, Isin, Symbol},
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

paft_core::other_string_code_type!(
    /// Provider-specific asset kind that is not modeled by [`AssetKind`].
    pub struct OtherAssetKind for AssetKind;
    type Error = DomainError;
    parse(input) => input.parse::<AssetKind>();
    invalid(input) => DomainError::InvalidAssetKindValue {
        value: input.to_string(),
    };
);

/// Kinds of financial instruments.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix)
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AssetKind {
    /// Common stock or equity-like instruments.
    Equity,
    /// Cryptocurrency assets.
    Crypto,
    /// Funds and ETFs.
    Fund,
    /// Market indexes.
    Index,
    /// Foreign exchange currency pairs.
    Forex,
    /// Bonds and fixed income.
    Bond,
    /// Commodities.
    Commodity,
    /// Option contracts.
    Option,
    /// Commodity futures.
    Future,
    /// Real Estate Investment Trusts.
    REIT,
    /// Warrants.
    Warrant,
    /// Convertible bonds/securities.
    Convertible,
    /// Non-fungible tokens.
    NFT,
    /// Perpetual futures contracts (no expiration date).
    PerpetualFuture,
    /// Leveraged tokens (e.g., 3x leveraged Bitcoin tokens).
    LeveragedToken,
    /// Liquidity provider tokens (`DeFi` protocol tokens).
    LPToken,
    /// Liquid staking tokens (e.g., stETH, rETH).
    LST,
    /// Real-world assets (tokenized physical assets).
    RWA,
    /// Provider-specific asset kind not modeled as a canonical variant.
    Other(OtherAssetKind),
}

crate::string_enum_with_code!(
    AssetKind, Other(OtherAssetKind),
    "AssetKind",
    type Error = DomainError;
    invalid(input) => DomainError::InvalidAssetKindValue {
        value: input.to_string(),
    };
    {
        "EQUITY" => AssetKind::Equity,
        "CRYPTO" => AssetKind::Crypto,
        "FUND" => AssetKind::Fund,
        "INDEX" => AssetKind::Index,
        "FOREX" => AssetKind::Forex,
        "BOND" => AssetKind::Bond,
        "COMMODITY" => AssetKind::Commodity,
        "OPTION" => AssetKind::Option,
        "FUTURE" => AssetKind::Future,
        "REIT" => AssetKind::REIT,
        "WARRANT" => AssetKind::Warrant,
        "CONVERTIBLE" => AssetKind::Convertible,
        "NFT" => AssetKind::NFT,
        "PERPETUAL_FUTURE" => AssetKind::PerpetualFuture,
        "LEVERAGED_TOKEN" => AssetKind::LeveragedToken,
        "LP_TOKEN" => AssetKind::LPToken,
        "LST" => AssetKind::LST,
        "RWA" => AssetKind::RWA,
    },
    {
        "STOCK" => AssetKind::Equity,
        "FX" => AssetKind::Forex,
    }
);

crate::impl_display_via_code!(AssetKind);

impl AssetKind {
    /// Builds an unknown asset kind, rejecting tokens modeled by [`AssetKind`].
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`AssetKind`] variant.
    pub fn other(input: &str) -> Result<Self, DomainError> {
        OtherAssetKind::new(input).map(Self::Other)
    }

    /// Human-readable label for displaying this asset kind.
    #[must_use]
    pub fn full_name(&self) -> Cow<'static, str> {
        match self {
            Self::Equity => Cow::Borrowed("Equity"),
            Self::Crypto => Cow::Borrowed("Crypto"),
            Self::Fund => Cow::Borrowed("Fund"),
            Self::Index => Cow::Borrowed("Index"),
            Self::Forex => Cow::Borrowed("Forex"),
            Self::Bond => Cow::Borrowed("Bond"),
            Self::Commodity => Cow::Borrowed("Commodity"),
            Self::Option => Cow::Borrowed("Option"),
            Self::Future => Cow::Borrowed("Future"),
            Self::REIT => Cow::Borrowed("REIT"),
            Self::Warrant => Cow::Borrowed("Warrant"),
            Self::Convertible => Cow::Borrowed("Convertible"),
            Self::NFT => Cow::Borrowed("NFT"),
            Self::PerpetualFuture => Cow::Borrowed("Perpetual Future"),
            Self::LeveragedToken => Cow::Borrowed("Leveraged Token"),
            Self::LPToken => Cow::Borrowed("LP Token"),
            Self::LST => Cow::Borrowed("Liquid Staking Token"),
            Self::RWA => Cow::Borrowed("Real-World Asset"),
            Self::Other(code) => Cow::Owned(code.as_ref().to_string()),
        }
    }
}

/// Logical instrument identity for a security.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(df_derive_macros::ToDataFrame))]
pub struct Instrument {
    /// Canonical ticker symbol.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub symbol: Symbol,
    /// Optional trading venue context for disambiguation.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub exchange: Option<Exchange>,
    /// Optional global identifier (preferred).
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub figi: Option<Figi>,
    /// Optional global identifier (fallback).
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub isin: Option<Isin>,
    /// Asset class and behavior.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub kind: AssetKind,
}

impl Instrument {
    /// Construct an instrument from a validated symbol and asset kind.
    #[must_use]
    pub const fn new(symbol: Symbol, kind: AssetKind) -> Self {
        Self {
            symbol,
            exchange: None,
            figi: None,
            isin: None,
            kind,
        }
    }

    /// Construct a new `Instrument` with just a symbol and kind.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidSymbol` if the provided symbol violates canonical invariants.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(symbol), err)
    )]
    pub fn from_symbol(symbol: impl AsRef<str>, kind: AssetKind) -> Result<Self, DomainError> {
        Ok(Self {
            symbol: Symbol::new(symbol.as_ref())?,
            exchange: None,
            figi: None,
            isin: None,
            kind,
        })
    }

    /// Construct a new `Instrument` with symbol, exchange, and kind.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidSymbol` if the provided symbol violates canonical invariants.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(symbol), err)
    )]
    pub fn from_symbol_and_exchange(
        symbol: impl AsRef<str>,
        exchange: Exchange,
        kind: AssetKind,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            symbol: Symbol::new(symbol.as_ref())?,
            exchange: Some(exchange),
            figi: None,
            isin: None,
            kind,
        })
    }

    /// Construct a new `Instrument` for a security identified by a FIGI and symbol.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidFigi` if FIGI validation fails.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn from_figi(figi: &str, symbol: Symbol, kind: AssetKind) -> Result<Self, DomainError> {
        Ok(Self {
            symbol,
            exchange: None,
            figi: Some(Figi::new(figi)?),
            isin: None,
            kind,
        })
    }

    /// Returns a stable, namespaced identity key for this instrument.
    ///
    /// The key includes the asset kind and identifier source so instruments that
    /// share a raw symbol (for example, an equity and a crypto asset both named
    /// `BTC`) do not collapse to the same key. Symbol payloads include their
    /// byte length to avoid delimiter collisions with symbols that contain
    /// characters such as `@`.
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        let kind = self.kind.code();

        if let Some(figi) = &self.figi {
            return Cow::Owned(format!("{kind}|FIGI|{}", figi.as_ref()));
        }
        if let Some(isin) = &self.isin {
            return Cow::Owned(format!("{kind}|ISIN|{}", isin.as_ref()));
        }

        let symbol = self.symbol.as_str();
        let symbol_len = symbol.len();

        if let Some(exchange) = &self.exchange {
            return Cow::Owned(format!(
                "{kind}|SYMBOL|{symbol_len}:{symbol}|EXCHANGE|{}",
                exchange.code()
            ));
        }

        Cow::Owned(format!("{kind}|SYMBOL|{symbol_len}:{symbol}"))
    }

    /// Returns the best available compact identifier for display
    /// (FIGI > ISIN > SYMBOL@EXCHANGE > SYMBOL).
    #[must_use]
    pub fn display_key(&self) -> Cow<'_, str> {
        if let Some(figi) = &self.figi {
            return Cow::Borrowed(figi.as_ref());
        }
        if let Some(isin) = &self.isin {
            return Cow::Borrowed(isin.as_ref());
        }
        if let Some(exchange) = &self.exchange {
            return Cow::Owned(format!("{}@{}", self.symbol, exchange.code()));
        }
        Cow::Borrowed(self.symbol.as_str())
    }
}

impl std::fmt::Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_key())
    }
}
