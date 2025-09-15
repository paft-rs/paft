//! Profile-related types under `paft_fundamentals::fundamentals::profile`.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
#[cfg(feature = "dataframe")]
use paft_core::dataframe::ToDataFrame;

/// Fund types with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of fund types while gracefully
/// handling unknown or provider-specific fund types through the `Other` variant.
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
pub enum FundKind {
    /// Exchange-Traded Fund
    #[strum(to_string = "ETF", serialize = "EXCHANGE_TRADED_FUND")]
    #[default]
    Etf,
    /// Mutual Fund
    #[strum(to_string = "MUTUAL_FUND", serialize = "MUTUAL")]
    MutualFund,
    /// Index Fund
    #[strum(to_string = "INDEX_FUND", serialize = "INDEX")]
    IndexFund,
    /// Closed-End Fund
    #[strum(to_string = "CLOSED_END_FUND", serialize = "CEF")]
    ClosedEndFund,
    /// Money Market Fund
    #[strum(to_string = "MONEY_MARKET_FUND", serialize = "MMF")]
    MoneyMarketFund,
    /// Hedge Fund
    #[strum(to_string = "HEDGE_FUND")]
    HedgeFund,
    /// Real Estate Investment Trust
    #[strum(to_string = "REIT", serialize = "REAL_ESTATE_INVESTMENT_TRUST")]
    Reit,
    /// Unit Investment Trust
    #[strum(to_string = "UIT", serialize = "UNIT_INVESTMENT_TRUST")]
    UnitInvestmentTrust,
    /// Unknown or provider-specific fund type
    Other(String),
}

impl From<String> for FundKind {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        Self::from_str(&s).unwrap_or_else(|_| Self::Other(s.to_uppercase()))
    }
}

impl From<FundKind> for String {
    fn from(fund_kind: FundKind) -> Self {
        match fund_kind {
            FundKind::Other(s) => s,
            _ => fund_kind.to_string(),
        }
    }
}

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
