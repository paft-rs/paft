//! Instrument identifier and asset classification domain types.

use super::Exchange;
use crate::{
    DomainError,
    identifiers::{Figi, Isin, Symbol},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, str::FromStr};

/// Kinds of financial instruments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum AssetKind {
    /// Common stock or equity-like instruments.
    #[default]
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
}

crate::string_enum_closed_with_code!(
    AssetKind,
    "AssetKind",
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
        "RWA" => AssetKind::RWA
    },
    {
        "STOCK" => AssetKind::Equity,
        "FX" => AssetKind::Forex,
    }
);

crate::impl_display_via_code!(AssetKind);

impl AssetKind {
    /// Human-readable label for displaying this asset kind.
    #[must_use]
    pub const fn full_name(&self) -> &'static str {
        match self {
            Self::Equity => "Equity",
            Self::Crypto => "Crypto",
            Self::Fund => "Fund",
            Self::Index => "Index",
            Self::Forex => "Forex",
            Self::Bond => "Bond",
            Self::Commodity => "Commodity",
            Self::Option => "Option",
            Self::Future => "Future",
            Self::REIT => "REIT",
            Self::Warrant => "Warrant",
            Self::Convertible => "Convertible",
            Self::NFT => "NFT",
            Self::PerpetualFuture => "Perpetual Future",
            Self::LeveragedToken => "Leveraged Token",
            Self::LPToken => "LP Token",
            Self::LST => "Liquid Staking Token",
            Self::RWA => "Real-World Asset",
        }
    }
}

/// Logical instrument identifier with hierarchical identifier support and asset classification.
///
/// This struct serves as a container for multiple types of identifiers, prioritizing
/// stable, universal identifiers like FIGI while maintaining backward compatibility
/// with symbol-based identification. The hierarchical approach allows providers to
/// populate the identifiers they have access to, while encouraging the use of
/// better identifiers when available.
///
/// Symbol values are canonicalized into the [`Symbol`] newtype, preserving casing
/// and punctuation semantics required by upstream data sources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    figi: Option<Figi>,
    isin: Option<Isin>,
    symbol: Symbol,
    exchange: Option<Exchange>,
    kind: AssetKind,
}

impl Instrument {
    /// # Errors
    /// Returns `DomainError::InvalidIsin` when `isin` is empty, malformed,
    /// or fails normalization/validation.
    pub fn try_set_isin(&mut self, isin: &str) -> Result<(), DomainError> {
        self.isin = Some(Isin::new(isin)?);
        Ok(())
    }

    /// Try to set the FIGI while ensuring validation.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidFigi` when `figi` is empty, not exactly
    /// 12 ASCII alphanumeric characters, or fails the checksum.
    pub fn try_set_figi(&mut self, figi: &str) -> Result<(), DomainError> {
        self.figi = Some(Figi::new(figi)?);
        Ok(())
    }

    /// Try to set the ISIN while consuming and returning the instrument.
    ///
    /// # Errors
    /// Propagates `DomainError::InvalidIsin` from [`Isin::new`].
    pub fn try_with_isin(mut self, isin: &str) -> Result<Self, DomainError> {
        self.try_set_isin(isin)?;
        Ok(self)
    }

    /// Try to set the FIGI while consuming and returning the instrument.
    ///
    /// # Errors
    /// Propagates `DomainError::InvalidFigi` from [`Figi::new`].
    pub fn try_with_figi(mut self, figi: &str) -> Result<Self, DomainError> {
        self.try_set_figi(figi)?;
        Ok(self)
    }

    /// # Errors
    /// Returns `DomainError::InvalidSymbol`, `DomainError::InvalidFigi`, or
    /// `DomainError::InvalidIsin` if the provided identifiers fail
    /// validation/normalization.
    pub fn try_new(
        symbol: impl AsRef<str>,
        kind: AssetKind,
        figi: Option<&str>,
        isin: Option<&str>,
        exchange: Option<Exchange>,
    ) -> Result<Self, DomainError> {
        let symbol = Symbol::new(symbol.as_ref())?;
        let mut instrument = Self {
            figi: None,
            isin: None,
            symbol,
            exchange,
            kind,
        };

        if let Some(figi_value) = figi {
            instrument.try_set_figi(figi_value)?;
        }
        if let Some(isin_value) = isin {
            instrument.try_set_isin(isin_value)?;
        }

        Ok(instrument)
    }

    /// Construct a new `Instrument` with just a symbol and kind (backward compatibility).
    /// This is useful for providers that only have basic symbol information.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidSymbol` if the provided symbol violates canonical invariants.
    pub fn from_symbol(symbol: impl AsRef<str>, kind: AssetKind) -> Result<Self, DomainError> {
        Ok(Self {
            figi: None,
            isin: None,
            symbol: Symbol::new(symbol.as_ref())?,
            exchange: None,
            kind,
        })
    }

    /// Construct a new `Instrument` with symbol, exchange, and kind.
    /// This is useful for providers that have exchange information but no global identifiers.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidSymbol` if the provided symbol violates canonical invariants.
    pub fn from_symbol_and_exchange(
        symbol: impl AsRef<str>,
        exchange: Exchange,
        kind: AssetKind,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            figi: None,
            isin: None,
            symbol: Symbol::new(symbol.as_ref())?,
            exchange: Some(exchange),
            kind,
        })
    }

    /// Returns the best available unique identifier for this instrument.
    ///
    /// Priority order:
    /// 1. FIGI (if available)
    /// 2. ISIN (if available)
    /// 3. `SYMBOL@EXCHANGE` (if the exchange is available)
    /// 4. Symbol only (fallback; ambiguous across venues/data vendors)
    ///
    /// This method returns a `Cow<str>` to avoid unnecessary allocations:
    /// - Returns `Cow::Borrowed` for FIGI, ISIN, and symbol-only cases
    /// - Returns `Cow::Owned` only for the symbol@exchange case that requires formatting
    ///
    /// Bare symbols are not globally unique; callers should prefer FIGI/ISIN
    /// when present and treat the symbol fallback as legacy.
    ///
    /// # Future compatibility
    /// The `symbol@exchange` fallback currently uses the exchange display code (e.g. `NASDAQ`).
    /// These values are not MICs and should be treated as a legacy format until
    /// a canonical mapping is introduced in a future release.
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
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

    /// Returns true if this instrument has a globally unique identifier (FIGI or ISIN).
    #[must_use]
    pub const fn is_globally_identified(&self) -> bool {
        self.figi.is_some() || self.isin.is_some()
    }

    /// Returns the FIGI identifier if available.
    #[must_use]
    pub const fn figi(&self) -> Option<&Figi> {
        self.figi.as_ref()
    }

    /// Returns the FIGI as a string slice if available.
    #[must_use]
    pub fn figi_str(&self) -> Option<&str> {
        self.figi.as_ref().map(AsRef::as_ref)
    }

    /// Returns the ISIN identifier if available.
    #[must_use]
    pub const fn isin(&self) -> Option<&Isin> {
        self.isin.as_ref()
    }

    /// Returns the ISIN as a string slice if available.
    #[must_use]
    pub fn isin_str(&self) -> Option<&str> {
        self.isin.as_ref().map(AsRef::as_ref)
    }

    /// Returns the canonical instrument symbol.
    #[must_use]
    pub const fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    /// Returns the ticker symbol as a string slice.
    #[must_use]
    pub fn symbol_str(&self) -> &str {
        self.symbol.as_str()
    }

    /// Returns the exchange if available.
    #[must_use]
    pub const fn exchange(&self) -> Option<&Exchange> {
        self.exchange.as_ref()
    }

    /// Returns the asset kind.
    #[must_use]
    pub const fn kind(&self) -> &AssetKind {
        &self.kind
    }

    /// Returns true if this instrument has a FIGI identifier.
    #[must_use]
    pub const fn has_figi(&self) -> bool {
        self.figi.is_some()
    }

    /// Returns true if this instrument has an ISIN identifier.
    #[must_use]
    pub const fn has_isin(&self) -> bool {
        self.isin.is_some()
    }

    /// Returns true if this instrument has exchange information.
    #[must_use]
    pub const fn has_exchange(&self) -> bool {
        self.exchange.is_some()
    }
}
