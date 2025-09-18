//! Currency enumeration with major currencies and extensible fallback.
//!
//! This module provides type-safe handling of currency codes while gracefully
//! handling unknown or provider-specific currencies through the `Other` variant.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString};

use super::currency_utils::currency_minor_units;

/// Currency enumeration with major currencies and extensible fallback.
///
/// This enum provides type-safe handling of currency codes while gracefully
/// handling unknown or provider-specific currencies through the `Other` variant.
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
pub enum Currency {
    /// US Dollar
    #[strum(serialize = "USD")]
    #[default]
    USD,
    /// Euro
    #[strum(serialize = "EUR")]
    EUR,
    /// British Pound Sterling
    #[strum(serialize = "GBP")]
    GBP,
    /// Japanese Yen
    #[strum(serialize = "JPY")]
    JPY,
    /// Canadian Dollar
    #[strum(serialize = "CAD")]
    CAD,
    /// Australian Dollar
    #[strum(serialize = "AUD")]
    AUD,
    /// Swiss Franc
    #[strum(serialize = "CHF")]
    CHF,
    /// Chinese Yuan
    #[strum(serialize = "CNY")]
    CNY,
    /// Hong Kong Dollar
    #[strum(serialize = "HKD")]
    HKD,
    /// Singapore Dollar
    #[strum(serialize = "SGD")]
    SGD,
    /// Indian Rupee
    #[strum(serialize = "INR")]
    INR,
    /// Brazilian Real
    #[strum(serialize = "BRL")]
    BRL,
    /// Mexican Peso
    #[strum(serialize = "MXN")]
    MXN,
    /// South Korean Won
    #[strum(serialize = "KRW")]
    KRW,
    /// New Zealand Dollar
    #[strum(serialize = "NZD")]
    NZD,
    /// Norwegian Krone
    #[strum(serialize = "NOK")]
    NOK,
    /// Swedish Krona
    #[strum(serialize = "SEK")]
    SEK,
    /// Danish Krone
    #[strum(serialize = "DKK")]
    DKK,
    /// Polish Zloty
    #[strum(serialize = "PLN")]
    PLN,
    /// Czech Koruna
    #[strum(serialize = "CZK")]
    CZK,
    /// Hungarian Forint
    #[strum(serialize = "HUF")]
    HUF,
    /// Russian Ruble
    #[strum(serialize = "RUB")]
    RUB,
    /// Turkish Lira
    #[strum(serialize = "TRY")]
    TRY,
    /// South African Rand
    #[strum(serialize = "ZAR")]
    ZAR,
    /// Israeli Shekel
    #[strum(serialize = "ILS")]
    ILS,
    /// Thai Baht
    #[strum(serialize = "THB")]
    THB,
    /// Malaysian Ringgit
    #[strum(serialize = "MYR")]
    MYR,
    /// Philippine Peso
    #[strum(serialize = "PHP")]
    PHP,
    /// Indonesian Rupiah
    #[strum(serialize = "IDR")]
    IDR,
    /// Vietnamese Dong
    #[strum(serialize = "VND")]
    VND,
    /// Bitcoin
    #[strum(serialize = "BTC")]
    BTC,
    /// Ethereum
    #[strum(serialize = "ETH")]
    ETH,
    /// Monero
    #[strum(serialize = "XMR")]
    XMR,
    /// Unknown or provider-specific currency
    Other(String),
}

impl From<String> for Currency {
    fn from(s: String) -> Self {
        // Trim and uppercase once to handle noisy provider strings.
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Self::Other(trimmed.to_string());
        }

        // Try to parse the trimmed string as-is (this retains canonical casing
        // such as "usd" -> USD via `strum`'s ascii_case_insensitive option).
        if let Ok(parsed) = Self::from_str(trimmed) {
            return parsed;
        }

        // Apply additional alias normalization for common provider spellings
        // before falling back to `Other`.
        match trimmed.to_uppercase().as_str() {
            "US_DOLLAR" | "US DOLLAR" | "USDOLLAR" | "DOLLAR" => Self::USD,
            "EURO" => Self::EUR,
            "POUND" | "POUND STERLING" => Self::GBP,
            code => Self::Other(code.to_string()),
        }
    }
}

impl From<Currency> for String {
    fn from(currency: Currency) -> Self {
        match currency {
            Currency::Other(s) => s,
            _ => currency.to_string(),
        }
    }
}

impl Currency {
    /// Returns true if this is a major reserve currency
    #[must_use]
    pub const fn is_reserve_currency(&self) -> bool {
        matches!(
            self,
            Self::USD | Self::EUR | Self::GBP | Self::JPY | Self::CHF
        )
    }

    /// Returns the string code for this enum variant
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Other(s) => s,
            _ => self.as_ref(),
        }
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
    /// assert_eq!(Currency::Other("XYZ".to_string()).decimal_places(), 2);
    /// ```
    #[must_use]
    pub fn decimal_places(&self) -> u32 {
        if let Self::Other(code) = self
            && let Some(decimals) = currency_minor_units(code)
        {
            return decimals;
        }
        self.base_decimal_places()
    }

    const fn base_decimal_places(&self) -> u32 {
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

    /// Returns the scaling factor for converting between major and minor units.
    /// This is `10^decimal_places`.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::Currency;
    ///
    /// assert_eq!(Currency::USD.minor_unit_scale(), 100);  // 10^2
    /// assert_eq!(Currency::JPY.minor_unit_scale(), 1);    // 10^0
    /// ```
    #[must_use]
    pub fn minor_unit_scale(&self) -> i64 {
        10_i64.pow(self.decimal_places())
    }
}
