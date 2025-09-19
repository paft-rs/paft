//! Profile-related types under `paft_fundamentals::fundamentals::profile`.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
#[cfg(feature = "dataframe")]
use paft_core::dataframe::ToDataFrame;
use paft_core::error::PaftError;

use paft_core::domain::string_canonical::Canonical;

/// Fund types with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of fund types while gracefully
/// handling unknown or provider-specific fund types through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes using an escape prefix `~` as "~{s}" and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for all values, including `Other`, via the escape prefix
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum FundKind {
    /// Exchange-Traded Fund
    #[default]
    Etf,
    /// Mutual Fund
    MutualFund,
    /// Index Fund
    IndexFund,
    /// Closed-End Fund
    ClosedEndFund,
    /// Money Market Fund
    MoneyMarketFund,
    /// Hedge Fund
    HedgeFund,
    /// Real Estate Investment Trust
    Reit,
    /// Unit Investment Trust
    UnitInvestmentTrust,
    /// Unknown or provider-specific fund type
    Other(Canonical),
}

impl FundKind {
    /// Attempts to parse a fund kind, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `PaftError::InvalidEnumValue` when `input` is empty/whitespace.
    pub fn try_from_str(input: &str) -> Result<Self, PaftError> {
        Self::from_str(input)
    }
}

// Centralized string impls via macro
paft_core::string_enum_with_code!(
    FundKind, Other, "FundKind",
    {
        "ETF" => FundKind::Etf,
        "MUTUAL_FUND" => FundKind::MutualFund,
        "INDEX_FUND" => FundKind::IndexFund,
        "CLOSED_END_FUND" => FundKind::ClosedEndFund,
        "MONEY_MARKET_FUND" => FundKind::MoneyMarketFund,
        "HEDGE_FUND" => FundKind::HedgeFund,
        "REIT" => FundKind::Reit,
        "UIT" => FundKind::UnitInvestmentTrust
    },
    {
        "EXCHANGE_TRADED_FUND" => FundKind::Etf,
        "MUTUAL" => FundKind::MutualFund,
        "INDEX" => FundKind::IndexFund,
        "CEF" => FundKind::ClosedEndFund,
        "MMF" => FundKind::MoneyMarketFund,
        "REAL_ESTATE_INVESTMENT_TRUST" => FundKind::Reit,
        "UNIT_INVESTMENT_TRUST" => FundKind::UnitInvestmentTrust
    }
);

paft_core::impl_display_via_code!(FundKind);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Postal address details.
pub struct Address {
    /// First address line.
    pub street1: Option<String>,
    /// Second address line.
    pub street2: Option<String>,
    /// City or locality.
    pub city: Option<String>,
    /// State or region.
    pub state: Option<String>,
    /// Country.
    pub country: Option<String>,
    /// Postal or ZIP code.
    pub zip: Option<String>,
}

/// Company profile details (provider-agnostic; maps well to common models).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct CompanyProfile {
    /// Company display name.
    pub name: String,
    /// Sector classification.
    pub sector: Option<String>,
    /// Industry classification.
    pub industry: Option<String>,
    /// Company website.
    pub website: Option<String>,
    /// Registered address.
    pub address: Option<Address>,
    /// Business summary.
    pub summary: Option<String>,
    /// International Securities Identification Number.
    pub isin: Option<String>,
}

/// Fund profile details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct FundProfile {
    /// Fund name.
    pub name: String,
    /// Fund family (e.g., Vanguard, iShares).
    pub family: Option<String>,
    /// Fund type with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub kind: FundKind,
    /// International Securities Identification Number.
    pub isin: Option<String>,
}

/// Union of supported profile kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    /// Company profile.
    Company(CompanyProfile),
    /// Fund profile.
    Fund(FundProfile),
}

impl Profile {
    /// Returns the ISIN for the company or fund, if available.
    #[must_use]
    pub fn isin(&self) -> Option<&str> {
        match self {
            Self::Company(c) => c.isin.as_deref(),
            Self::Fund(f) => f.isin.as_deref(),
        }
    }
}

/// Represents a single data point in a time series of shares outstanding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct ShareCount {
    /// The timestamp for the data point.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: DateTime<Utc>,
    /// The number of shares outstanding.
    pub shares: u64,
}
