//! Exchange enumeration with major exchanges and extensible fallback.
//!
//! This module provides type-safe handling of exchange identifiers while gracefully
//! handling unknown or provider-specific exchanges through the `Other` variant.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString};

/// Exchange enumeration with major exchanges and extensible fallback.
///
/// This enum provides type-safe handling of exchange identifiers while gracefully
/// handling unknown or provider-specific exchanges through the `Other` variant.
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
pub enum Exchange {
    /// NASDAQ Stock Market
    #[strum(serialize = "NASDAQ")]
    #[default]
    NASDAQ,
    /// New York Stock Exchange
    #[strum(serialize = "NYSE")]
    NYSE,
    /// American Stock Exchange
    #[strum(serialize = "AMEX")]
    AMEX,
    /// BATS Global Markets
    #[strum(serialize = "BATS")]
    BATS,
    /// Over-the-Counter Markets
    #[strum(serialize = "OTC")]
    OTC,
    /// London Stock Exchange
    #[strum(serialize = "LSE")]
    LSE,
    /// Tokyo Stock Exchange
    #[strum(serialize = "TSE")]
    TSE,
    /// Hong Kong Stock Exchange
    #[strum(serialize = "HKEX")]
    HKEX,
    /// Shanghai Stock Exchange
    #[strum(serialize = "SSE")]
    SSE,
    /// Shenzhen Stock Exchange
    #[strum(serialize = "SZSE")]
    SZSE,
    /// Toronto Stock Exchange
    #[strum(serialize = "TSX")]
    TSX,
    /// Australian Securities Exchange
    #[strum(serialize = "ASX")]
    ASX,
    /// Euronext
    #[strum(to_string = "Euronext", serialize = "EURONEXT")]
    Euronext,
    /// Deutsche Börse (XETRA)
    #[strum(serialize = "XETRA")]
    XETRA,
    /// Swiss Exchange
    #[strum(serialize = "SIX")]
    SIX,
    /// Borsa Italiana
    #[strum(serialize = "BIT")]
    BIT,
    /// Bolsa de Madrid
    #[strum(serialize = "BME")]
    BME,
    /// Euronext Amsterdam
    #[strum(serialize = "AEX")]
    AEX,
    /// Euronext Brussels
    #[strum(serialize = "BRU")]
    BRU,
    /// Euronext Lisbon
    #[strum(serialize = "LIS")]
    LIS,
    /// Euronext Paris
    #[strum(serialize = "EPA")]
    EPA,
    /// Oslo Børs
    #[strum(serialize = "OSL")]
    OSL,
    /// Stockholm Stock Exchange
    #[strum(serialize = "STO")]
    STO,
    /// Copenhagen Stock Exchange
    #[strum(serialize = "CPH")]
    CPH,
    /// Warsaw Stock Exchange
    #[strum(serialize = "WSE")]
    WSE,
    /// Prague Stock Exchange
    #[strum(serialize = "PSE")]
    PSE,
    /// Budapest Stock Exchange
    #[strum(serialize = "BSE")]
    BSE,
    /// Moscow Exchange
    #[strum(serialize = "MOEX")]
    MOEX,
    /// Istanbul Stock Exchange
    #[strum(serialize = "BIST")]
    BIST,
    /// Johannesburg Stock Exchange
    #[strum(serialize = "JSE")]
    JSE,
    /// Tel Aviv Stock Exchange
    #[strum(serialize = "TASE")]
    TASE,
    /// Bombay Stock Exchange
    #[strum(to_string = "BOMBAY", serialize = "BSE_IND")]
    BseIndia,
    /// National Stock Exchange of India
    #[strum(serialize = "NSE")]
    NSE,
    /// Korea Exchange
    #[strum(serialize = "KRX")]
    KRX,
    /// Singapore Exchange
    #[strum(serialize = "SGX")]
    SGX,
    /// Thailand Stock Exchange
    #[strum(serialize = "SET")]
    SET,
    /// Bursa Malaysia
    #[strum(serialize = "KLSE")]
    KLSE,
    /// Philippine Stock Exchange
    #[strum(to_string = "PSEI", serialize = "PSE_PH")]
    PsePhil,
    /// Indonesia Stock Exchange
    #[strum(serialize = "IDX")]
    IDX,
    /// Ho Chi Minh Stock Exchange
    #[strum(serialize = "HOSE")]
    HOSE,
    /// Unknown or provider-specific exchange
    Other(String),
}

impl From<String> for Exchange {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        Self::from_str(&s).unwrap_or_else(|_| Self::Other(s.to_uppercase()))
    }
}

impl From<Exchange> for String {
    fn from(exchange: Exchange) -> Self {
        match exchange {
            Exchange::Other(s) => s,
            _ => exchange.to_string(),
        }
    }
}

impl Exchange {
    /// Returns true if this is a major US exchange
    #[must_use]
    pub const fn is_us_exchange(&self) -> bool {
        matches!(
            self,
            Self::NASDAQ | Self::NYSE | Self::AMEX | Self::BATS | Self::OTC
        )
    }

    /// Returns true if this is a European exchange
    #[must_use]
    pub const fn is_european_exchange(&self) -> bool {
        matches!(
            self,
            Self::LSE
                | Self::Euronext
                | Self::XETRA
                | Self::SIX
                | Self::BIT
                | Self::BME
                | Self::AEX
                | Self::BRU
                | Self::LIS
                | Self::EPA
                | Self::OSL
                | Self::STO
                | Self::CPH
                | Self::WSE
                | Self::PSE
                | Self::BSE
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
}
