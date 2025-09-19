//! Currency enumeration with major currencies and extensible fallback.
//!
//! This module provides type-safe handling of currency codes while gracefully
//! handling unknown or provider-specific currencies through the `Other` variant.

// no module-level serde imports needed here
use std::str::FromStr;

use super::currency_utils::{MAX_MINOR_UNIT_DECIMALS, currency_minor_units};
use super::money::MoneyError;
use super::string_canonical::Canonical;
use crate::error::PaftError;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Currency {
    /// US Dollar
    #[default]
    USD,
    /// Euro
    EUR,
    /// British Pound Sterling
    GBP,
    /// Japanese Yen
    JPY,
    /// Canadian Dollar
    CAD,
    /// Australian Dollar
    AUD,
    /// Swiss Franc
    CHF,
    /// Chinese Yuan
    CNY,
    /// Hong Kong Dollar
    HKD,
    /// Singapore Dollar
    SGD,
    /// Indian Rupee
    INR,
    /// Brazilian Real
    BRL,
    /// Mexican Peso
    MXN,
    /// South Korean Won
    KRW,
    /// New Zealand Dollar
    NZD,
    /// Norwegian Krone
    NOK,
    /// Swedish Krona
    SEK,
    /// Danish Krone
    DKK,
    /// Polish Zloty
    PLN,
    /// Czech Koruna
    CZK,
    /// Hungarian Forint
    HUF,
    /// Russian Ruble
    RUB,
    /// Turkish Lira
    TRY,
    /// South African Rand
    ZAR,
    /// Israeli Shekel
    ILS,
    /// Thai Baht
    THB,
    /// Malaysian Ringgit
    MYR,
    /// Philippine Peso
    PHP,
    /// Indonesian Rupiah
    IDR,
    /// Vietnamese Dong
    VND,
    /// Bitcoin
    BTC,
    /// Ethereum
    ETH,
    /// Monero
    XMR,
    /// Unknown or provider-specific currency
    Other(Canonical),
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
            Self::USD | Self::EUR | Self::GBP | Self::JPY | Self::CHF
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
    ///
    /// assert_eq!(Currency::USD.decimal_places(), 2);
    /// assert_eq!(Currency::JPY.decimal_places(), 0);
    /// assert_eq!(Currency::BTC.decimal_places(), 8);
    /// assert_eq!(Currency::ETH.decimal_places(), 18);
    /// assert_eq!(Currency::try_from_str("XYZ").unwrap().decimal_places(), 2);
    /// ```
    #[must_use]
    pub fn decimal_places(&self) -> u8 {
        if let Self::Other(code) = self
            && let Some(decimals) = currency_minor_units(code.as_ref())
        {
            return decimals;
        }
        self.base_decimal_places()
    }

    const fn base_decimal_places(&self) -> u8 {
        match self {
            // Fiat currencies with 0 decimal places
            Self::JPY | Self::KRW | Self::VND => 0,

            // Most fiat currencies have 2 decimal places
            Self::USD
            | Self::EUR
            | Self::GBP
            | Self::CAD
            | Self::AUD
            | Self::CHF
            | Self::CNY
            | Self::HKD
            | Self::SGD
            | Self::INR
            | Self::BRL
            | Self::MXN
            | Self::NZD
            | Self::NOK
            | Self::SEK
            | Self::DKK
            | Self::PLN
            | Self::CZK
            | Self::RUB
            | Self::TRY
            | Self::ZAR
            | Self::ILS
            | Self::THB
            | Self::MYR
            | Self::PHP
            | Self::IDR
            | Self::HUF
            | Self::Other(_) => 2,
            // Cryptocurrencies with specific decimal places
            Self::ETH => 18, // Ethereum uses 18 decimal places (wei)
            Self::XMR => 12, // Monero uses 12 decimal places
            Self::BTC => 8,  // Most cryptocurrencies use 8 decimal places
        }
    }

    /// Returns the human-readable name for this currency.
    #[must_use]
    pub fn full_name(&self) -> &str {
        match self {
            Self::USD => "US Dollar",
            Self::EUR => "Euro",
            Self::GBP => "Pound Sterling",
            Self::JPY => "Japanese Yen",
            Self::CAD => "Canadian Dollar",
            Self::AUD => "Australian Dollar",
            Self::CHF => "Swiss Franc",
            Self::CNY => "Chinese Yuan",
            Self::HKD => "Hong Kong Dollar",
            Self::SGD => "Singapore Dollar",
            Self::INR => "Indian Rupee",
            Self::BRL => "Brazilian Real",
            Self::MXN => "Mexican Peso",
            Self::KRW => "South Korean Won",
            Self::NZD => "New Zealand Dollar",
            Self::NOK => "Norwegian Krone",
            Self::SEK => "Swedish Krona",
            Self::DKK => "Danish Krone",
            Self::PLN => "Polish Zloty",
            Self::CZK => "Czech Koruna",
            Self::HUF => "Hungarian Forint",
            Self::RUB => "Russian Ruble",
            Self::TRY => "Turkish Lira",
            Self::ZAR => "South African Rand",
            Self::ILS => "Israeli Shekel",
            Self::THB => "Thai Baht",
            Self::MYR => "Malaysian Ringgit",
            Self::PHP => "Philippine Peso",
            Self::IDR => "Indonesian Rupiah",
            Self::VND => "Vietnamese Dong",
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
    ///
    /// assert_eq!(Currency::USD.minor_unit_scale().unwrap(), 100);  // 10^2
    /// assert_eq!(Currency::JPY.minor_unit_scale().unwrap(), 1);    // 10^0
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

// Serde implemented via macro

// Implement code() and string impls via macro (open enum)
crate::string_enum_with_code!(
    Currency, Other, "Currency",
    {
        "USD" => Currency::USD,
        "EUR" => Currency::EUR,
        "GBP" => Currency::GBP,
        "JPY" => Currency::JPY,
        "CAD" => Currency::CAD,
        "AUD" => Currency::AUD,
        "CHF" => Currency::CHF,
        "CNY" => Currency::CNY,
        "HKD" => Currency::HKD,
        "SGD" => Currency::SGD,
        "INR" => Currency::INR,
        "BRL" => Currency::BRL,
        "MXN" => Currency::MXN,
        "KRW" => Currency::KRW,
        "NZD" => Currency::NZD,
        "NOK" => Currency::NOK,
        "SEK" => Currency::SEK,
        "DKK" => Currency::DKK,
        "PLN" => Currency::PLN,
        "CZK" => Currency::CZK,
        "HUF" => Currency::HUF,
        "RUB" => Currency::RUB,
        "TRY" => Currency::TRY,
        "ZAR" => Currency::ZAR,
        "ILS" => Currency::ILS,
        "THB" => Currency::THB,
        "MYR" => Currency::MYR,
        "PHP" => Currency::PHP,
        "IDR" => Currency::IDR,
        "VND" => Currency::VND,
        "BTC" => Currency::BTC,
        "ETH" => Currency::ETH,
        "XMR" => Currency::XMR
    },
    {
        // Aliases
        "US_DOLLAR" => Currency::USD,
        "USDOLLAR" => Currency::USD,
        "DOLLAR" => Currency::USD,
        "EURO" => Currency::EUR,
        "POUND" => Currency::GBP,
        "POUND_STERLING" => Currency::GBP,
        "RMB" => Currency::CNY,
        "CNH" => Currency::CNY,
        "XBT" => Currency::BTC,
        "BITCOIN" => Currency::BTC,
    }
);

crate::impl_display_via_code!(Currency);
