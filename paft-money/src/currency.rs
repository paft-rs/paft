//! Currency enumeration with ISO 4217 support and extensible fallback.

use std::str::FromStr;

use iso_currency::Currency as IsoCurrency;
use paft_utils::{Canonical, StringCode, canonicalize};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::currency_utils::{MAX_MINOR_UNIT_DECIMALS, currency_minor_units};
use crate::error::MoneyParseError;
use crate::money::MoneyError;

/// Currency enumeration with major currencies and extensible fallback.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Currency {
    /// ISO 4217 currency code (backed by `iso_currency` crate)
    Iso(IsoCurrency),
    /// Bitcoin (non-ISO)
    BTC,
    /// Ethereum (non-ISO)
    ETH,
    /// Monero (non-ISO)
    XMR,
    /// Unknown or provider-specific currency
    Other(Canonical),
}

impl Default for Currency {
    fn default() -> Self {
        Self::Iso(IsoCurrency::USD)
    }
}

impl Currency {
    /// Attempts to parse a currency from the provided string, enforcing canonical aliases.
    ///
    /// # Errors
    /// Returns `MoneyParseError::InvalidEnumValue` when the input is empty or cannot be canonicalized.
    pub fn try_from_str(input: &str) -> Result<Self, MoneyParseError> {
        Self::from_str(input)
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
    #[must_use]
    pub fn decimal_places(&self) -> u8 {
        if let Self::Other(code) = self
            && let Some(decimals) = currency_minor_units(code.as_ref())
        {
            return decimals;
        }

        match self {
            Self::Iso(iso) => iso
                .exponent()
                .and_then(|e| u8::try_from(e).ok())
                .unwrap_or(2),
            Self::ETH => 18,
            Self::XMR => 12,
            Self::BTC => 8,
            Self::Other(_) => 2,
        }
    }

    /// Returns the human-readable name for this currency.
    #[must_use]
    pub fn full_name(&self) -> &str {
        match self {
            Self::Iso(iso) => iso.name(),
            Self::BTC => "Bitcoin",
            Self::ETH => "Ethereum",
            Self::XMR => "Monero",
            Self::Other(code) => code.as_ref(),
        }
    }

    /// Returns the scaling factor for converting between major and minor units (`10^decimal_places`).
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` when the required precision exceeds `MAX_MINOR_UNIT_DECIMALS`.
    pub fn minor_unit_scale(&self) -> Result<i64, MoneyError> {
        let decimals = self.decimal_places();
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
            Self::Other(c) => c.as_ref(),
        }
    }

    /// Whether this value is a canonical variant (not an `Other` payload).
    #[must_use]
    pub const fn is_canonical(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
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
        let canon = token.as_str();

        if canon == "BTC" {
            return Ok(Self::BTC);
        }
        if canon == "ETH" {
            return Ok(Self::ETH);
        }
        if canon == "XMR" {
            return Ok(Self::XMR);
        }

        if let Some(iso) = IsoCurrency::from_code(canon) {
            return Ok(Self::Iso(iso));
        }

        let other = Canonical::try_new(trimmed).map_err(|_| MoneyParseError::InvalidEnumValue {
            enum_name: "Currency",
            value: input.to_string(),
        })?;
        Ok(Self::Other(other))
    }
}

impl StringCode for Currency {
    fn code(&self) -> &str {
        match self {
            Self::Iso(c) => c.code(),
            Self::BTC => "BTC",
            Self::ETH => "ETH",
            Self::XMR => "XMR",
            Self::Other(canon) => canon.as_ref(),
        }
    }
}
