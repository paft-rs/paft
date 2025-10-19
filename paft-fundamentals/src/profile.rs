//! Profile-related types under `paft_fundamentals::fundamentals::profile`.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_core::error::PaftError;
use paft_domain::{Canonical, Isin};
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};

/// Fund types with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of fund types while gracefully
/// handling unknown or provider-specific fund types through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix) and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
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
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
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
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub isin: Option<Isin>,
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
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub isin: Option<Isin>,
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
    pub const fn isin(&self) -> Option<&Isin> {
        match self {
            Self::Company(c) => c.isin.as_ref(),
            Self::Fund(f) => f.isin.as_ref(),
        }
    }
}

#[cfg(feature = "dataframe")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
struct ProfileRow {
    pub profile_type: String,
    pub name: String,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub address: Option<Address>,
    pub summary: Option<String>,
    pub family: Option<String>,
    pub fund_kind: Option<String>,
    pub isin: Option<String>,
}

#[cfg(feature = "dataframe")]
impl From<&Profile> for ProfileRow {
    fn from(profile: &Profile) -> Self {
        match profile {
            Profile::Company(company) => Self {
                profile_type: "Company".to_string(),
                name: company.name.clone(),
                sector: company.sector.clone(),
                industry: company.industry.clone(),
                website: company.website.clone(),
                address: company.address.clone(),
                summary: company.summary.clone(),
                family: None,
                fund_kind: None,
                isin: company.isin.as_ref().map(ToString::to_string),
            },
            Profile::Fund(fund) => Self {
                profile_type: "Fund".to_string(),
                name: fund.name.clone(),
                sector: None,
                industry: None,
                website: None,
                address: None,
                summary: None,
                family: fund.family.clone(),
                fund_kind: Some(fund.kind.to_string()),
                isin: fund.isin.as_ref().map(ToString::to_string),
            },
        }
    }
}

#[cfg(feature = "dataframe")]
impl ToDataFrame for Profile {
    fn to_dataframe(&self) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        ProfileRow::from(self).to_dataframe()
    }

    fn empty_dataframe() -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        ProfileRow::empty_dataframe()
    }

    fn schema() -> polars::prelude::PolarsResult<Vec<(&'static str, polars::datatypes::DataType)>> {
        ProfileRow::schema()
    }
}

#[cfg(feature = "dataframe")]
impl Columnar for Profile {
    fn columnar_to_dataframe(
        items: &[Self],
    ) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        let rows: Vec<ProfileRow> = items.iter().map(ProfileRow::from).collect();
        rows.as_slice().to_dataframe()
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
