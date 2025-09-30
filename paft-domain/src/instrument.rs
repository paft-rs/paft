//! Instrument identifier and asset classification domain types.

use super::Exchange;
use crate::DomainError;
use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer},
};
use std::{borrow::Cow, str::FromStr};

fn scrub_isin(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
}

#[cfg(feature = "isin-validate")]
fn normalize_isin(input: &str) -> Result<String, DomainError> {
    let cleaned = scrub_isin(input);
    match ::isin::parse_loose(&cleaned) {
        Ok(_) => Ok(cleaned.to_ascii_uppercase()),
        Err(_) => Err(DomainError::InvalidIsin {
            value: input.to_string(),
        }),
    }
}

#[cfg(not(feature = "isin-validate"))]
fn normalize_isin(input: &str) -> Result<String, DomainError> {
    let cleaned = scrub_isin(input);
    if cleaned.is_empty() {
        return Err(DomainError::InvalidIsin {
            value: input.to_string(),
        });
    }

    Ok(cleaned.to_ascii_uppercase())
}

fn deserialize_isin<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    if let Some(value) = raw {
        let normalized = normalize_isin(&value).map_err(de::Error::custom)?;
        Ok(Some(normalized))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "isin-validate")]
/// Returns `true` if the input parses as a valid ISIN after separators are scrubbed.
pub fn is_valid_isin(s: &str) -> bool {
    let cleaned = scrub_isin(s);
    ::isin::parse_loose(&cleaned).is_ok()
}

#[cfg(not(feature = "isin-validate"))]
/// Returns `true` when the scrubbed input still contains ASCII alphanumeric characters.
pub fn is_valid_isin(s: &str) -> bool {
    let cleaned = scrub_isin(s);
    !cleaned.is_empty()
}

/// Normalizes an ISIN and applies validation when the `isin-validate` feature is enabled; without the feature, the value is scrubbed to uppercase ASCII alphanumerics and must not be empty.
pub fn normalize_isin_strict(s: &str) -> Result<String, DomainError> {
    normalize_isin(s)
}

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    figi: Option<String>,
    #[serde(default, deserialize_with = "deserialize_isin")]
    isin: Option<String>,
    symbol: String,
    exchange: Option<Exchange>,
    kind: AssetKind,
}

impl Instrument {
    /// `set_isin_unchecked` bypasses normalization/validation in all configurations.
    pub fn set_isin_unchecked(&mut self, isin: impl Into<String>) {
        self.isin = Some(isin.into());
    }

    /// When the `isin-validate` feature is enabled, values are validated and normalized; invalid values cause an error.
    /// When the feature is disabled, values are scrubbed to ASCII alphanumerics, uppercased, and must not be empty.
    pub fn try_set_isin(&mut self, isin: &str) -> Result<(), DomainError> {
        let normalized = normalize_isin(isin)?;
        self.isin = Some(normalized);
        Ok(())
    }

    /// Try to set the ISIN while consuming and returning the instrument.
    pub fn try_with_isin(mut self, isin: &str) -> Result<Self, DomainError> {
        self.try_set_isin(isin)?;
        Ok(self)
    }

    /// Construct a new `Instrument` with validation-aware ISIN handling.
    /// When the `isin-validate` feature is enabled, values are validated and normalized; invalid values cause an error.
    /// When the feature is disabled, values are scrubbed to ASCII alphanumerics, uppercased, and must not be empty.
    pub fn try_new(
        symbol: impl Into<String>,
        kind: AssetKind,
        figi: Option<&str>,
        isin: Option<&str>,
        exchange: Option<Exchange>,
    ) -> Result<Self, DomainError> {
        let mut instrument = Self {
            figi: figi.map(String::from),
            isin: None,
            symbol: symbol.into(),
            exchange,
            kind,
        };

        if let Some(isin_value) = isin {
            instrument.try_set_isin(isin_value)?;
        }

        Ok(instrument)
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
    ///
    /// # Future compatibility
    /// The `symbol@exchange` fallback currently uses the exchange display code (e.g. `NASDAQ`).
    /// These values are not MICs and should be treated as a legacy format until
    /// a canonical mapping is introduced in a future release.
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
