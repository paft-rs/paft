//! Currency enumeration with ISO 4217 support and extensible fallback.

use std::{borrow::Cow, str::FromStr};

use paft_utils::{Canonical, StringCode, canonicalize, has_canonical_token_boundaries};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::IsoCurrency;
use crate::currency_utils::{MAX_MINOR_UNIT_DECIMALS, currency_metadata};
use crate::error::{MoneyError, MoneyParseError};
#[cfg(feature = "money-formatting")]
use crate::locale::Locale;

/// Provider-specific currency code that is not modeled by [`Currency`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OtherCurrency(Canonical);

impl OtherCurrency {
    /// Builds an unknown currency code, rejecting tokens modeled by [`Currency`].
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`Currency`] variant.
    pub fn new(input: &str) -> Result<Self, MoneyParseError> {
        match Currency::try_from_str(input)? {
            Currency::Other(code) => Ok(code),
            _ => Err(MoneyParseError::InvalidEnumValue {
                enum_name: "Currency",
                value: input.to_string(),
            }),
        }
    }

    /// Returns this unknown currency's canonical code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    #[doc(hidden)]
    pub(crate) const fn from_canonical_unchecked(code: Canonical) -> Self {
        Self(code)
    }
}

impl AsRef<str> for OtherCurrency {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for OtherCurrency {
    type Err = MoneyParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(input)
    }
}

impl std::fmt::Display for OtherCurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Currency enumeration with major currencies and extensible fallback.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Currency {
    /// ISO 4217 currency code (backed by `iso_currency` crate)
    Iso(IsoCurrency),
    /// Bitcoin (non-ISO)
    BTC,
    /// Ethereum (non-ISO)
    ETH,
    /// Monero (non-ISO)
    XMR,
    /// USDC
    USDC,
    /// USDT
    USDT,
    /// Unknown or provider-specific currency
    Other(OtherCurrency),
}

impl Currency {
    /// Attempts to parse a currency from the provided string, enforcing canonical aliases.
    ///
    /// # Errors
    /// Returns `MoneyParseError::InvalidEnumValue` when the input is empty or cannot be canonicalized.
    pub fn try_from_str(input: &str) -> Result<Self, MoneyParseError> {
        Self::from_str(input)
    }

    /// Builds an unknown currency value, rejecting tokens modeled by [`Currency`].
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`Currency`] variant.
    pub fn other(input: &str) -> Result<Self, MoneyParseError> {
        OtherCurrency::new(input).map(Self::Other)
    }

    /// Returns true if this is a major reserve currency.
    #[must_use]
    pub const fn is_reserve_currency(&self) -> bool {
        matches!(
            self,
            Self::Iso(
                IsoCurrency::USD
                    | IsoCurrency::EUR
                    | IsoCurrency::GBP
                    | IsoCurrency::JPY
                    | IsoCurrency::CHF
            )
        )
    }

    /// Returns the number of decimal places (minor units) for this currency.
    ///
    /// Policy: If ISO defines a minor unit exponent, we use it. Otherwise (for
    /// ISO currencies without a registered exponent such as `XAU`/`XDR`, for
    /// the built-in non-ISO variants, and for `Other` codes), we consult the
    /// metadata registry by canonical code. If metadata is present, its
    /// `minor_units` value is used. Otherwise, an error is returned.
    ///
    /// Routing every non-ISO variant through the registry keeps the source of
    /// truth in one place (`BUILTIN_CURRENCY_METADATA`) so a future drift
    /// between hard-coded arms and registered metadata cannot occur.
    ///
    /// # Errors
    /// - Returns `MoneyError::MetadataNotFound` when no metadata can be
    ///   resolved for the currency (e.g. an `Other` code without a registered
    ///   overlay, or an ISO code whose ISO entry has no exponent and which has
    ///   no overlay registered).
    pub fn decimal_places(&self) -> Result<u8, MoneyError> {
        if let Self::Iso(iso) = self
            && let Some(exp) = iso.exponent().and_then(|e| u8::try_from(e).ok())
        {
            return Ok(exp);
        }

        currency_metadata(self.code())
            .map(|meta| meta.minor_units)
            .ok_or_else(|| MoneyError::MetadataNotFound {
                currency: self.clone(),
            })
    }

    /// Returns the human-readable name for this currency.
    ///
    /// ISO currencies use the canonical ISO 4217 name. Non-ISO currencies use
    /// the metadata registry, so display-name overlays affect modeled non-ISO
    /// variants and `Other` codes consistently.
    #[must_use]
    pub fn full_name(&self) -> Cow<'static, str> {
        match self {
            Self::Iso(iso) => Cow::Owned(iso.name().to_string()),
            Self::BTC | Self::ETH | Self::XMR | Self::USDC | Self::USDT | Self::Other(_) => {
                currency_metadata(self.code()).map_or_else(
                    || Cow::Owned(self.code().to_string()),
                    |meta| meta.full_name,
                )
            }
        }
    }

    /// Returns the scaling factor for converting between major and minor units (`10^decimal_places`).
    ///
    /// The result is computed as `10_i64.pow(decimal_places)`, which is why
    /// `MAX_MINOR_UNIT_DECIMALS` is capped at 18 — beyond that, `10^scale`
    /// no longer fits in `i64`.
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` when the required precision exceeds `MAX_MINOR_UNIT_DECIMALS`.
    pub fn minor_unit_scale(&self) -> Result<i64, MoneyError> {
        let decimals = self.decimal_places()?;
        if decimals > MAX_MINOR_UNIT_DECIMALS {
            return Err(MoneyError::ConversionError);
        }
        Ok(10_i64.pow(u32::from(decimals)))
    }

    /// Returns the canonical string code for this value.
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Iso(iso) => iso.code(),
            Self::BTC => "BTC",
            Self::ETH => "ETH",
            Self::XMR => "XMR",
            Self::USDC => "USDC",
            Self::USDT => "USDT",
            Self::Other(c) => c.as_ref(),
        }
    }

    /// Whether this value is a canonical variant (not an `Other` payload).
    #[must_use]
    pub const fn is_canonical(&self) -> bool {
        !matches!(self, Self::Other(_))
    }

    /// Returns the preferred currency symbol.
    #[cfg(feature = "money-formatting")]
    #[must_use]
    pub fn symbol(&self) -> Option<Cow<'static, str>> {
        if let Some(meta) = currency_metadata(self.code()) {
            let symbol = meta.symbol.clone();
            if symbol.is_empty() {
                return None;
            }
            return Some(symbol);
        }

        Some(Cow::Owned(self.code().to_string()))
    }

    /// Returns whether the symbol should precede (`true`) or follow (`false`) the amount.
    #[cfg(feature = "money-formatting")]
    #[must_use]
    pub fn symbol_first(&self) -> bool {
        currency_metadata(self.code()).is_none_or(|meta| meta.symbol_first)
    }

    /// Returns the default locale for formatting this currency.
    #[cfg(feature = "money-formatting")]
    #[must_use]
    pub fn default_locale(&self) -> Locale {
        currency_metadata(self.code()).map_or(Locale::EnUs, |meta| meta.default_locale)
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

impl AsRef<str> for Currency {
    fn as_ref(&self) -> &str {
        self.code()
    }
}

impl Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.code())
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::from_str(&raw).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Currency {
    type Err = MoneyParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(MoneyParseError::InvalidEnumValue {
                enum_name: "Currency",
                value: input.to_string(),
            });
        }
        let token = canonicalize(trimmed);
        let canon = token.as_ref();

        let known = if canon == "BTC" {
            Some(Self::BTC)
        } else if canon == "ETH" {
            Some(Self::ETH)
        } else if canon == "XMR" {
            Some(Self::XMR)
        } else if canon == "USDC" {
            Some(Self::USDC)
        } else if canon == "USDT" {
            Some(Self::USDT)
        } else {
            IsoCurrency::from_code(canon).map(Self::Iso)
        };

        if let Some(currency) = known {
            if has_canonical_token_boundaries(trimmed) {
                return Ok(currency);
            }
            return Err(MoneyParseError::InvalidEnumValue {
                enum_name: "Currency",
                value: input.to_string(),
            });
        }

        let other = Canonical::try_new(trimmed).map_err(|_| MoneyParseError::InvalidEnumValue {
            enum_name: "Currency",
            value: input.to_string(),
        })?;
        Ok(Self::Other(OtherCurrency::from_canonical_unchecked(other)))
    }
}

impl StringCode for Currency {
    fn code(&self) -> &str {
        Self::code(self)
    }

    fn is_canonical(&self) -> bool {
        Self::is_canonical(self)
    }
}
