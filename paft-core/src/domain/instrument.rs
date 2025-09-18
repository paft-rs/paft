//! Instrument identifier and asset classification domain types.

use super::Exchange;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString};

/// Default string used for unknown asset kinds.
pub const UNKNOWN_ASSET_KIND: &str = "UNKNOWN";

/// Kinds of financial instruments with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of asset kinds while gracefully
/// handling unknown or provider-specific asset types through the `Other` variant.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    AsRefStr,
    EnumString,
    Default,
)]
#[strum(ascii_case_insensitive)]
#[serde(from = "String", into = "String")]
pub enum AssetKind {
    /// Common stock or equity-like instruments.
    #[strum(serialize = "EQUITY", serialize = "STOCK")]
    #[default]
    Equity,
    /// Cryptocurrency assets.
    #[strum(serialize = "CRYPTO", serialize = "CRYPTOCURRENCY")]
    Crypto,
    /// Funds and ETFs.
    #[strum(serialize = "FUND", serialize = "ETF")]
    Fund,
    /// Market indexes.
    #[strum(serialize = "INDEX")]
    Index,
    /// Foreign exchange currency pairs.
    #[strum(serialize = "FOREX", serialize = "FX", serialize = "CURRENCY")]
    Forex,
    /// Bonds and fixed income.
    #[strum(serialize = "BOND", serialize = "FIXED_INCOME")]
    Bond,
    /// Commodities.
    #[strum(serialize = "COMMODITY")]
    Commodity,
    /// Option contracts.
    #[strum(serialize = "OPTION")]
    Option,
    /// Unknown or provider-specific asset kind.
    Other(String),
}

impl From<String> for AssetKind {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        Self::from_str(&s).unwrap_or_else(|_| Self::Other(s.to_uppercase()))
    }
}

impl From<AssetKind> for String {
    fn from(kind: AssetKind) -> Self {
        match kind {
            AssetKind::Other(s) => s,
            _ => kind.to_string(),
        }
    }
}

impl AssetKind {
    /// Returns true if this is a known/canonical asset kind
    #[must_use]
    pub const fn is_canonical(&self) -> bool {
        !matches!(self, Self::Other(_))
    }

    /// Returns the asset kind as a string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Other(s) => s,
            _ => self.as_ref(),
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    /// Financial Instrument Global Identifier (FIGI) - a free, open, stable identifier.
    /// This is the preferred identifier for cross-provider data aggregation.
    figi: Option<String>,
    /// International Securities Identification Number (ISIN) - globally unique but may have licensing restrictions.
    /// Second choice for global identification when FIGI is not available.
    isin: Option<String>,
    /// Ticker symbol (e.g., "AAPL", "BRK.A").
    /// While ubiquitous, symbols can be ambiguous and vary by exchange/provider.
    /// This is the only required field to maintain backward compatibility.
    symbol: String,
    /// Exchange identifier with canonical variants and extensible fallback.
    /// Optional but recommended for disambiguating symbols.
    exchange: Option<Exchange>,
    /// Asset kind classification.
    kind: AssetKind,
}

impl Instrument {
    /// Construct a new `Instrument` with all available identifiers.
    /// This is the primary constructor for creating instruments with complete identification.
    pub fn new(
        symbol: impl Into<String>,
        kind: AssetKind,
        figi: Option<String>,
        isin: Option<String>,
        exchange: Option<Exchange>,
    ) -> Self {
        Self {
            figi,
            isin,
            symbol: symbol.into(),
            exchange,
            kind,
        }
    }

    /// Construct a new `Instrument` with just a symbol and kind (backward compatibility).
    /// This is useful for providers that only have basic symbol information.
    pub fn from_symbol(symbol: impl Into<String>, kind: AssetKind) -> Self {
        Self {
            figi: None,
            isin: None,
            symbol: symbol.into(),
            exchange: None,
            kind,
        }
    }

    /// Construct a new `Instrument` with symbol, exchange, and kind.
    /// This is useful for providers that have exchange information but no global identifiers.
    pub fn from_symbol_and_exchange(
        symbol: impl Into<String>,
        exchange: Exchange,
        kind: AssetKind,
    ) -> Self {
        Self {
            figi: None,
            isin: None,
            symbol: symbol.into(),
            exchange: Some(exchange),
            kind,
        }
    }

    /// Returns the best available unique identifier for this instrument.
    ///
    /// Priority order:
    /// 1. FIGI (if available)
    /// 2. ISIN (if available)
    /// 3. Symbol + Exchange (if exchange is available)
    /// 4. Symbol only (fallback)
    ///
    /// This method returns a `Cow<str>` to avoid unnecessary allocations:
    /// - Returns `Cow::Borrowed` for FIGI, ISIN, and symbol-only cases
    /// - Returns `Cow::Owned` only for the symbol@exchange case that requires formatting
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        if let Some(figi) = &self.figi {
            return Cow::Borrowed(figi);
        }
        if let Some(isin) = &self.isin {
            return Cow::Borrowed(isin);
        }
        if let Some(exchange) = &self.exchange {
            return Cow::Owned(format!("{}@{}", self.symbol, exchange.code()));
        }
        Cow::Borrowed(&self.symbol)
    }

    /// Returns true if this instrument has a globally unique identifier (FIGI or ISIN).
    #[must_use]
    pub const fn is_globally_identified(&self) -> bool {
        self.figi.is_some() || self.isin.is_some()
    }

    /// Returns the FIGI identifier if available.
    #[must_use]
    pub fn figi(&self) -> Option<&str> {
        self.figi.as_deref()
    }

    /// Returns the ISIN identifier if available.
    #[must_use]
    pub fn isin(&self) -> Option<&str> {
        self.isin.as_deref()
    }

    /// Returns the ticker symbol.
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
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
