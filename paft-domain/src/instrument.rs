//! Instrument identifier and asset classification domain types.

use super::Exchange;
use crate::{
    DomainError,
    identifiers::{Figi, Isin, Symbol},
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[cfg(feature = "dataframe")]
mod dataframe;

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

/// Security identifiers and optional trading-listing context.
///
/// With the `dataframe` feature, exports include the structured identity fields
/// plus [`Self::security_key`], [`Self::listing_key`], the legacy `key` from
/// [`Self::unique_key`], and `display` from [`Self::display_key`]. Nested records
/// prefix these columns, for example `instrument.listing_key`. Choose the key
/// matching the entity being joined; missing identity context produces nulls.
/// These helpers do not resolve aliases, identifier changes, or ticker reuse.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    /// Canonical ticker symbol.
    pub symbol: Symbol,
    /// Optional trading venue context for disambiguation.
    pub exchange: Option<Exchange>,
    /// Optional venue-level FIGI identifying this tradable listing.
    ///
    /// Composite and share-class FIGIs are not accepted in this field. Adapters
    /// must establish the level from source metadata; [`Figi`] validates syntax
    /// and checksum only, because the identifier does not encode its level.
    /// See the [OpenFIGI identifier hierarchy](https://www.openfigi.com/api/documentation).
    pub figi: Option<Figi>,
    /// Optional ISIN identifying the securities issue across venues.
    pub isin: Option<Isin>,
    /// Asset class and behavior.
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

    /// Construct an `Instrument` from a venue-level FIGI and symbol.
    /// The caller must establish the FIGI level; see [`Self::figi`].
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

    /// Returns a security-issue key when an ISIN is available.
    ///
    /// Includes asset kind and ISIN, deliberately independent of venue, symbol,
    /// and venue-level FIGI. Returns `None` without an ISIN: neither a ticker nor
    /// a venue FIGI establishes a cross-venue security identity. Do not group
    /// missing keys together as if they identified one security.
    #[must_use]
    pub fn security_key(&self) -> Option<String> {
        let isin = self.isin.as_ref()?;
        let kind = self.kind.code();
        Some(format!(
            "SECURITY|{}:{kind}|ISIN|{}",
            kind.len(),
            isin.as_ref()
        ))
    }

    /// Returns a venue/listing key when exchange context is available.
    ///
    /// Includes asset kind, exchange, and venue FIGI when supplied, otherwise
    /// the symbol. ISIN cannot replace the listing symbol: one issue can have
    /// several quotation lines even on one exchange. Missing exchange returns
    /// `None`; callers must supply the actual venue before joining quotes or
    /// histories. Exchange codes must use the same venue granularity across
    /// inputs. Symbol fallback is scoped to the observation's context and does
    /// not account for ticker reuse over time. Adding a FIGI changes the key.
    #[must_use]
    pub fn listing_key(&self) -> Option<String> {
        let exchange = self.exchange.as_ref()?.code();
        let kind = self.kind.code();
        let (source, identifier) = self
            .figi
            .as_ref()
            .map_or(("SYMBOL", self.symbol.as_str()), |figi| {
                ("FIGI", figi.as_ref())
            });
        Some(format!(
            "LISTING|{}:{kind}|{source}|{}:{identifier}|EXCHANGE|{}:{exchange}",
            kind.len(),
            identifier.len(),
            exchange.len()
        ))
    }

    /// Returns the legacy best-available identifier key.
    ///
    /// This mixes identity levels: FIGI identifies a listing, ISIN identifies
    /// an issue, and the fallback is a symbol with optional exchange. Exchange
    /// is ignored when FIGI or ISIN exists. **It is not a universal primary key
    /// or suitable for venue-level joins.** Prefer [`Self::security_key`] or
    /// [`Self::listing_key`] with their explicit, narrower contracts.
    ///
    /// The key includes the asset kind and identifier source so instruments that
    /// share a raw symbol (for example, an equity and a crypto asset both named
    /// `BTC`) do not collapse to the same key. Symbol payloads include their
    /// byte length to avoid delimiter collisions with symbols that contain
    /// characters such as `@`.
    ///
    /// This is a synthetic composite key and is always returned as an owned
    /// [`String`]. Use [`Self::display_key`] when a compact display identifier
    /// is needed.
    #[must_use]
    pub fn unique_key(&self) -> String {
        let kind = self.kind.code();

        if let Some(figi) = &self.figi {
            return format!("{kind}|FIGI|{}", figi.as_ref());
        }
        if let Some(isin) = &self.isin {
            return format!("{kind}|ISIN|{}", isin.as_ref());
        }

        let symbol = self.symbol.as_str();
        let symbol_len = symbol.len();

        if let Some(exchange) = &self.exchange {
            return format!(
                "{kind}|SYMBOL|{symbol_len}:{symbol}|EXCHANGE|{}",
                exchange.code()
            );
        }

        format!("{kind}|SYMBOL|{symbol_len}:{symbol}")
    }

    /// Returns the best available compact identifier for display
    /// (FIGI > ISIN > SYMBOL@EXCHANGE > SYMBOL).
    ///
    /// This label is not unique. Choose [`Self::security_key`] or
    /// [`Self::listing_key`] for the intended identity comparison.
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
