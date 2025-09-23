//! Currency representation backed by `iso_currency` for ISO 4217 and extensible fallback.
//!
//! This module provides type-safe handling of ISO currencies via the upstream
//! `iso_currency` crate, while gracefully handling unknown or provider-specific
//! currencies (including crypto) through the `Other` variant and a few
//! well-known non-ISO variants.

// no module-level serde imports needed here
use std::str::FromStr;

use super::currency_utils::{MAX_MINOR_UNIT_DECIMALS, currency_minor_units};
use super::money::MoneyError;
use super::string_canonical::{Canonical, canonicalize};
use crate::error::PaftError;
use iso_currency::Currency as IsoCurrency;

/// Currency enumeration with major currencies and extensible fallback.
///
/// This enum provides type-safe handling of currency codes while gracefully
/// handling unknown or provider-specific currencies through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix) and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
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
    ///
    /// Returns an error if `input` is empty or contains only whitespace.
    pub fn try_from_str(input: &str) -> Result<Self, PaftError> {
        Self::from_str(input)
    }

    /// Returns true if this is a major reserve currency
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

    /// Returns the number of decimal places (minor units) for this currency according to ISO 4217.
    ///
    /// Most fiat currencies have 2 decimal places, but some notable exceptions include:
    /// - Japanese Yen (JPY): 0 decimal places
    /// - Korean Won (KRW): 0 decimal places  
    /// - Vietnamese Dong (VND): 0 decimal places  
    /// - Bahraini Dinar (BHD): 3 decimal places (not implemented yet)
    /// - Jordanian Dinar (JOD): 3 decimal places (not implemented yet)
    ///
    /// Most cryptocurrencies use 8 decimal places, but some have different precision:
    /// - Bitcoin (BTC): 8 decimal places (satoshis)
    /// - Ethereum (ETH): 18 decimal places (wei)
    /// - Monero (XMR): 12 decimal places
    ///
    /// Other cryptocurrencies will default to 2 decimal places unless the
    /// variant encodes a specific precision.
    /// Precision for `Currency::Other` values can be overridden at runtime via
    /// `currency_utils::set_currency_minor_units`.
    ///
    /// For `Currency::Other(s)`, this method defaults to 2 decimal places.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::Currency;
    /// use iso_currency::Currency as IsoCurrency;
    ///
    /// assert_eq!(Currency::Iso(IsoCurrency::USD).decimal_places(), 2);
    /// assert_eq!(Currency::Iso(IsoCurrency::JPY).decimal_places(), 0);
    /// assert_eq!(Currency::BTC.decimal_places(), 8);
    /// assert_eq!(Currency::ETH.decimal_places(), 18);
    /// assert_eq!(Currency::try_from_str("XYZ").unwrap().decimal_places(), 2);
    /// ```
    #[must_use]
    pub fn decimal_places(&self) -> u8 {
        // First honor explicit overrides for unknown/non-ISO codes
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

    /// Returns the scaling factor for converting between major and minor units.
    /// This is `10^decimal_places`.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::Currency;
    /// use iso_currency::Currency as IsoCurrency;
    ///
    /// assert_eq!(Currency::Iso(IsoCurrency::USD).minor_unit_scale().unwrap(), 100);  // 10^2
    /// assert_eq!(Currency::Iso(IsoCurrency::JPY).minor_unit_scale().unwrap(), 1);    // 10^0
    /// ```
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` if `decimal_places()` exceeds the
    /// maximum supported precision for minor units.
    pub fn minor_unit_scale(&self) -> Result<i64, MoneyError> {
        let decimals = self.decimal_places();
        if decimals > MAX_MINOR_UNIT_DECIMALS {
            return Err(MoneyError::ConversionError);
        }
        Ok(10_i64.pow(u32::from(decimals)))
    }
}

impl Currency {
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

    // No associated ISO constants; construct ISO as `Currency::Iso(IsoCurrency::XXX)`
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

impl serde::Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.code())
    }
}

impl<'de> serde::Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::from_str(&raw).map_err(serde::de::Error::custom)
    }
}

impl std::str::FromStr for Currency {
    type Err = PaftError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(PaftError::InvalidEnumValue {
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

        let other = Canonical::try_new(trimmed).map_err(|_| PaftError::InvalidEnumValue {
            enum_name: "Currency",
            value: input.to_string(),
        })?;
        Ok(Self::Other(other))
    }
}

impl crate::domain::string_canonical::StringCode for Currency {
    fn code(&self) -> &str {
        Self::code(self)
    }
}
