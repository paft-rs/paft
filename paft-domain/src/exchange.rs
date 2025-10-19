//! Exchange enumeration with major exchanges and extensible fallback.
//!
//! This module provides type-safe handling of exchange identifiers while gracefully
//! handling unknown or provider-specific exchanges through the `Other` variant.

use crate::error::DomainError;
// no module-level serde imports needed here
use paft_utils::Canonical;
use std::str::FromStr;

/// Exchange enumeration with major exchanges and extensible fallback.
///
/// This enum provides type-safe handling of exchange identifiers while gracefully
/// handling unknown or provider-specific exchanges through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix) and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Exchange {
    /// NASDAQ Stock Market
    #[default]
    NASDAQ,
    /// New York Stock Exchange
    NYSE,
    /// American Stock Exchange
    AMEX,
    /// BATS Global Markets
    BATS,
    /// Over-the-Counter Markets
    OTC,
    /// London Stock Exchange
    LSE,
    /// Tokyo Stock Exchange
    TSE,
    /// Hong Kong Stock Exchange
    HKEX,
    /// Shanghai Stock Exchange
    SSE,
    /// Shenzhen Stock Exchange
    SZSE,
    /// Toronto Stock Exchange
    TSX,
    /// Australian Securities Exchange
    ASX,
    /// Euronext
    Euronext,
    /// Deutsche Börse (XETRA)
    XETRA,
    /// Swiss Exchange
    SIX,
    /// Borsa Italiana
    BIT,
    /// Bolsa de Madrid
    BME,
    /// Euronext Amsterdam
    AEX,
    /// Euronext Brussels
    BRU,
    /// Euronext Lisbon
    LIS,
    /// Euronext Paris
    EPA,
    /// Oslo Børs
    OSL,
    /// Stockholm Stock Exchange
    STO,
    /// Copenhagen Stock Exchange
    CPH,
    /// Warsaw Stock Exchange
    WSE,
    /// Prague Stock Exchange
    #[allow(non_camel_case_types)]
    PSE_CZ,
    /// Budapest Stock Exchange
    #[allow(non_camel_case_types)]
    BSE_HU,
    /// Moscow Exchange
    MOEX,
    /// Istanbul Stock Exchange
    BIST,
    /// Johannesburg Stock Exchange
    JSE,
    /// Tel Aviv Stock Exchange
    TASE,
    /// Bombay Stock Exchange
    BSE,
    /// National Stock Exchange of India
    NSE,
    /// Korea Exchange
    KRX,
    /// Singapore Exchange
    SGX,
    /// Thailand Stock Exchange
    SET,
    /// Bursa Malaysia
    KLSE,
    /// Philippine Stock Exchange
    PSE,
    /// Indonesia Stock Exchange
    IDX,
    /// Ho Chi Minh Stock Exchange
    HOSE,
    /// Unknown or provider-specific exchange
    Other(Canonical),
}

impl Exchange {
    /// Attempts to parse an exchange identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is empty or contains only whitespace.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn try_from_str(input: &str) -> Result<Self, DomainError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidExchangeValue {
                value: input.to_string(),
            });
        }

        Self::from_str(trimmed).map_err(|_| DomainError::InvalidExchangeValue {
            value: input.to_string(),
        })
    }

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
                | Self::PSE_CZ
                | Self::BSE_HU
        )
    }

    /// Returns the human-readable name for this exchange.
    #[must_use]
    pub fn full_name(&self) -> &str {
        match self {
            Self::NASDAQ => "Nasdaq",
            Self::NYSE => "NYSE",
            Self::AMEX => "AMEX",
            Self::BATS => "BATS",
            Self::OTC => "OTC",
            Self::LSE => "London Stock Exchange",
            Self::TSE => "Tokyo Stock Exchange",
            Self::HKEX => "Hong Kong Stock Exchange",
            Self::SSE => "Shanghai Stock Exchange",
            Self::SZSE => "Shenzhen Stock Exchange",
            Self::TSX => "Toronto Stock Exchange",
            Self::ASX => "Australian Securities Exchange",
            Self::Euronext => "Euronext",
            Self::XETRA => "Xetra",
            Self::SIX => "Swiss Exchange",
            Self::BIT => "Borsa Italiana",
            Self::BME => "Bolsa de Madrid",
            Self::AEX => "Euronext Amsterdam",
            Self::BRU => "Euronext Brussels",
            Self::LIS => "Euronext Lisbon",
            Self::EPA => "Euronext Paris",
            Self::OSL => "Oslo Børs",
            Self::STO => "Stockholm Stock Exchange",
            Self::CPH => "Copenhagen Stock Exchange",
            Self::WSE => "Warsaw Stock Exchange",
            Self::PSE_CZ => "Prague Stock Exchange",
            Self::BSE_HU => "Budapest Stock Exchange",
            Self::MOEX => "Moscow Exchange",
            Self::BIST => "Istanbul Stock Exchange",
            Self::JSE => "Johannesburg Stock Exchange",
            Self::TASE => "Tel Aviv Stock Exchange",
            Self::BSE => "Bombay Stock Exchange",
            Self::NSE => "National Stock Exchange of India",
            Self::KRX => "Korea Exchange",
            Self::SGX => "Singapore Exchange",
            Self::SET => "Stock Exchange of Thailand",
            Self::KLSE => "Bursa Malaysia",
            Self::PSE => "Philippine Stock Exchange",
            Self::IDX => "Indonesia Stock Exchange",
            Self::HOSE => "Ho Chi Minh Stock Exchange",
            Self::Other(code) => code.as_ref(),
        }
    }
}

// Implement code() and string impls via macro (open enum)
crate::string_enum_with_code!(
    Exchange, Other, "Exchange",
    {
        "NASDAQ" => Exchange::NASDAQ,
        "NYSE" => Exchange::NYSE,
        "AMEX" => Exchange::AMEX,
        "BATS" => Exchange::BATS,
        "OTC" => Exchange::OTC,
        "LSE" => Exchange::LSE,
        "TSE" => Exchange::TSE,
        "HKEX" => Exchange::HKEX,
        "SSE" => Exchange::SSE,
        "SZSE" => Exchange::SZSE,
        "TSX" => Exchange::TSX,
        "ASX" => Exchange::ASX,
        "EURONEXT" => Exchange::Euronext,
        "XETRA" => Exchange::XETRA,
        "SIX" => Exchange::SIX,
        "BIT" => Exchange::BIT,
        "BME" => Exchange::BME,
        "AEX" => Exchange::AEX,
        "BRU" => Exchange::BRU,
        "LIS" => Exchange::LIS,
        "EPA" => Exchange::EPA,
        "OSL" => Exchange::OSL,
        "STO" => Exchange::STO,
        "CPH" => Exchange::CPH,
        "WSE" => Exchange::WSE,
        "PSE_CZ" => Exchange::PSE_CZ,
        "BSE_HU" => Exchange::BSE_HU,
        "MOEX" => Exchange::MOEX,
        "BIST" => Exchange::BIST,
        "JSE" => Exchange::JSE,
        "TASE" => Exchange::TASE,
        "BSE" => Exchange::BSE,
        "NSE" => Exchange::NSE,
        "KRX" => Exchange::KRX,
        "SGX" => Exchange::SGX,
        "SET" => Exchange::SET,
        "KLSE" => Exchange::KLSE,
        "PSE" => Exchange::PSE,
        "IDX" => Exchange::IDX,
        "HOSE" => Exchange::HOSE
    },
    {
        // Provider aliases
        "EURONEXT_PARIS" => Exchange::EPA,
        "BOMBAY" => Exchange::BSE,
        "BSE_INDIA" => Exchange::BSE
    }
);

crate::impl_display_via_code!(Exchange);
