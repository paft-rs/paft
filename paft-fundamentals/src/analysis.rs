//! Analyst, recommendations, and earnings-related types under `paft_fundamentals::analysis`.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_decimal::Decimal;
use paft_domain::{DomainError, Horizon, Period};
use paft_money::{Money, Price};

use crate::FundamentalsError;

paft_core::other_string_code_type!(
    /// Provider-specific recommendation grade not modeled by [`RecommendationGrade`].
    pub struct OtherRecommendationGrade for RecommendationGrade;
    type Error = FundamentalsError;
    parse(input) => RecommendationGrade::from_str(input);
    invalid(input) => FundamentalsError::InvalidEnumValue {
        enum_name: "RecommendationGrade",
        value: input.to_string(),
    };
);

/// Analyst recommendation grades with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of recommendation grades while gracefully
/// handling unknown or provider-specific grades through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix)
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecommendationGrade {
    /// Strong buy recommendation
    StrongBuy,
    /// Buy recommendation
    Buy,
    /// Hold recommendation
    Hold,
    /// Sell recommendation
    Sell,
    /// Strong sell recommendation
    StrongSell,
    /// Outperform recommendation
    Outperform,
    /// Underperform recommendation
    Underperform,
    /// Unknown or provider-specific grade
    Other(OtherRecommendationGrade),
}

impl RecommendationGrade {
    /// Attempts to parse a recommendation grade, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `FundamentalsError::InvalidEnumValue` when `input` is empty/whitespace.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn try_from_str(input: &str) -> Result<Self, FundamentalsError> {
        Self::from_str(input)
    }

    /// Builds an unknown recommendation grade, rejecting modeled grades and aliases.
    ///
    /// # Errors
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`RecommendationGrade`] variant.
    pub fn other(input: &str) -> Result<Self, FundamentalsError> {
        OtherRecommendationGrade::new(input).map(Self::Other)
    }
}

// serde via macro

// Implement code() and string impls via macro
paft_core::string_enum_with_code!(
    RecommendationGrade, Other(OtherRecommendationGrade), "RecommendationGrade",
    type Error = FundamentalsError;
    invalid(input) => FundamentalsError::InvalidEnumValue {
        enum_name: "RecommendationGrade",
        value: input.to_string(),
    };
    {
        "STRONG_BUY" => RecommendationGrade::StrongBuy,
        "BUY" => RecommendationGrade::Buy,
        "HOLD" => RecommendationGrade::Hold,
        "SELL" => RecommendationGrade::Sell,
        "STRONG_SELL" => RecommendationGrade::StrongSell,
        "OUTPERFORM" => RecommendationGrade::Outperform,
        "UNDERPERFORM" => RecommendationGrade::Underperform
    },
    {
        // Aliases
        "NEUTRAL" => RecommendationGrade::Hold,
        "MARKET_PERFORM" => RecommendationGrade::Hold,
        "OVERWEIGHT" => RecommendationGrade::Outperform,
        "UNDERWEIGHT" => RecommendationGrade::Underperform
    }
);

// Display should match code for these enums
paft_core::impl_display_via_code!(RecommendationGrade);

paft_core::other_string_code_type!(
    /// Provider-specific recommendation action not modeled by [`RecommendationAction`].
    pub struct OtherRecommendationAction for RecommendationAction;
    type Error = FundamentalsError;
    parse(input) => RecommendationAction::from_str(input);
    invalid(input) => FundamentalsError::InvalidEnumValue {
        enum_name: "RecommendationAction",
        value: input.to_string(),
    };
);

/// Analyst recommendation actions with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of recommendation actions while gracefully
/// handling unknown or provider-specific actions through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix)
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecommendationAction {
    /// Upgrade action
    Upgrade,
    /// Downgrade action
    Downgrade,
    /// Initiate coverage
    Initiate,
    /// Maintain or reiterate recommendation
    Maintain,
    /// Resume coverage
    Resume,
    /// Suspend coverage
    Suspend,
    /// Unknown or provider-specific action
    Other(OtherRecommendationAction),
}

impl RecommendationAction {
    /// Attempts to parse a recommendation action, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `FundamentalsError::InvalidEnumValue` when `input` is empty/whitespace.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn try_from_str(input: &str) -> Result<Self, FundamentalsError> {
        Self::from_str(input)
    }

    /// Builds an unknown recommendation action, rejecting modeled actions and aliases.
    ///
    /// # Errors
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`RecommendationAction`] variant.
    pub fn other(input: &str) -> Result<Self, FundamentalsError> {
        OtherRecommendationAction::new(input).map(Self::Other)
    }
}

// Implement code() and string impls via macro
paft_core::string_enum_with_code!(
    RecommendationAction, Other(OtherRecommendationAction), "RecommendationAction",
    type Error = FundamentalsError;
    invalid(input) => FundamentalsError::InvalidEnumValue {
        enum_name: "RecommendationAction",
        value: input.to_string(),
    };
    {
        "UPGRADE" => RecommendationAction::Upgrade,
        "DOWNGRADE" => RecommendationAction::Downgrade,
        "INIT" => RecommendationAction::Initiate,
        "MAINTAIN" => RecommendationAction::Maintain,
        "RESUME" => RecommendationAction::Resume,
        "SUSPEND" => RecommendationAction::Suspend
    },
    {
        // Aliases
        "UP" => RecommendationAction::Upgrade,
        "DOWN" => RecommendationAction::Downgrade,
        "INITIATED" => RecommendationAction::Initiate,
        "INITIATE" => RecommendationAction::Initiate,
        "REITERATE" => RecommendationAction::Maintain
    }
);

paft_core::impl_display_via_code!(RecommendationAction);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Earnings datasets: yearly summaries and quarterly breakdowns.
pub struct Earnings {
    /// Annual earnings summary rows.
    pub yearly: Vec<EarningsYear>,
    /// Quarterly earnings summary rows.
    pub quarterly: Vec<EarningsQuarter>,
    /// Quarterly EPS actual vs estimate rows.
    pub quarterly_eps: Vec<EarningsQuarterEps>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Yearly earnings summary.
pub struct EarningsYear {
    /// Fiscal year.
    pub year: i32,
    /// Revenue for the year.
    pub revenue: Option<Money>,
    /// Earnings for the year.
    pub earnings: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Quarterly earnings summary for a period key (e.g., 2023Q4 or 2023-10-01).
pub struct EarningsQuarter {
    /// Period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// Revenue for the period.
    pub revenue: Option<Money>,
    /// Earnings for the period.
    pub earnings: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Quarterly EPS actual vs estimate for a period key.
pub struct EarningsQuarterEps {
    /// Period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// Actual EPS.
    pub actual: Option<Price>,
    /// Estimated EPS.
    pub estimate: Option<Price>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Analyst price target summary.
pub struct PriceTarget {
    /// Mean price target.
    pub mean: Option<Price>,
    /// High price target.
    pub high: Option<Price>,
    /// Low price target.
    pub low: Option<Price>,
    /// Number of contributing analysts.
    pub number_of_analysts: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Distribution of analyst recommendations for a period.
pub struct RecommendationRow {
    /// Period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// Count of "strong buy" recommendations.
    pub strong_buy: Option<u32>,
    /// Count of "buy" recommendations.
    pub buy: Option<u32>,
    /// Count of "hold" recommendations.
    pub hold: Option<u32>,
    /// Count of "sell" recommendations.
    pub sell: Option<u32>,
    /// Count of "strong sell" recommendations.
    pub strong_sell: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Summary of analyst recommendations and mean scoring.
pub struct RecommendationSummary {
    /// Most recent period of the summary.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub latest_period: Option<Period>,
    /// Count of "strong buy" recommendations.
    pub strong_buy: Option<u32>,
    /// Count of "buy" recommendations.
    pub buy: Option<u32>,
    /// Count of "hold" recommendations.
    pub hold: Option<u32>,
    /// Count of "sell" recommendations.
    pub sell: Option<u32>,
    /// Count of "strong sell" recommendations.
    pub strong_sell: Option<u32>,
    /// Mean recommendation score.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub mean: Option<Decimal>,
    /// Provider-specific text for the mean score (e.g., "Buy", "Overweight").
    pub mean_rating_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Broker action history for an instrument.
pub struct UpgradeDowngradeRow {
    /// Event timestamp.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub ts: DateTime<Utc>,
    /// Research firm name.
    pub firm: Option<String>,
    /// Previous rating with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub from_grade: Option<RecommendationGrade>,
    /// New rating with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub to_grade: Option<RecommendationGrade>,
    /// Action description with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub action: Option<RecommendationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Summary of key analysis metrics extracted from detailed analysis data.
pub struct AnalysisSummary {
    /// Analyst target mean price.
    pub target_mean_price: Option<Price>,
    /// Analyst target high price.
    pub target_high_price: Option<Price>,
    /// Analyst target low price.
    pub target_low_price: Option<Price>,
    /// Number of analyst opinions contributing to the recommendation.
    pub number_of_analyst_opinions: Option<u32>,
    /// Numeric recommendation score (provider-defined scale).
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub recommendation_mean: Option<Decimal>,
    /// Categorical recommendation text (e.g., "Buy", "Overweight").
    pub recommendation_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Earnings estimate data with analyst consensus.
pub struct EarningsEstimate {
    /// Average earnings estimate.
    pub avg: Option<Price>,
    /// Low earnings estimate.
    pub low: Option<Price>,
    /// High earnings estimate.
    pub high: Option<Price>,
    /// Earnings per share from a year ago.
    pub year_ago_eps: Option<Price>,
    /// Number of analysts providing earnings estimates.
    pub num_analysts: Option<u32>,
    /// Estimated earnings growth.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub growth: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Revenue estimate data with analyst consensus.
pub struct RevenueEstimate {
    /// Average revenue estimate.
    pub avg: Option<Money>,
    /// Low revenue estimate.
    pub low: Option<Money>,
    /// High revenue estimate.
    pub high: Option<Money>,
    /// Revenue from a year ago.
    pub year_ago_revenue: Option<Money>,
    /// Number of analysts providing revenue estimates.
    pub num_analysts: Option<u32>,
    /// Estimated revenue growth.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub growth: Option<Decimal>,
}

/// A flexible data point for time-series trend data.
///
/// This struct allows any provider to represent trend data for any lookback
/// horizon, making the system provider-agnostic instead of tied to specific
/// hardcoded buckets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct TrendPoint {
    /// The lookback horizon this data point represents (e.g., "7d", "1mo", "3mo").
    /// This allows providers to use their own horizon conventions.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub horizon: Horizon,
    /// The value for this horizon.
    pub value: Price,
}

impl TrendPoint {
    /// Creates a new trend point with the specified horizon and value.
    #[must_use]
    pub const fn new(horizon: Horizon, value: Price) -> Self {
        Self { horizon, value }
    }

    /// Creates a new trend point from a horizon string.
    ///
    /// # Errors
    /// Returns an error if the horizon string cannot be parsed.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn try_new_str(horizon: &str, value: Price) -> Result<Self, DomainError> {
        Ok(Self {
            horizon: horizon.parse()?,
            value,
        })
    }
}

/// EPS trend changes over different lookback horizons.
///
/// This struct now uses a flexible collection of trend points instead of
/// hardcoded time buckets, making it provider-agnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct EpsTrend {
    /// Current EPS trend.
    pub current: Option<Price>,
    /// Historical EPS trend data points with flexible lookback horizons.
    /// Each provider can populate this with their available horizons
    /// (e.g., a generic provider might use "7d", "30d", "60d", "90d" while another
    /// provider might use "1mo", "3mo", "6mo").
    pub historical: Vec<TrendPoint>,
}

impl EpsTrend {
    /// Creates a new EPS trend with the specified current value and historical data.
    #[must_use]
    pub const fn new(current: Option<Price>, historical: Vec<TrendPoint>) -> Self {
        Self {
            current,
            historical,
        }
    }

    /// Finds a trend point by horizon.
    #[must_use]
    pub fn find_by_horizon(&self, horizon: &Horizon) -> Option<&TrendPoint> {
        self.historical
            .iter()
            .find(|point| &point.horizon == horizon)
    }

    /// Finds a trend point by horizon string.
    ///
    /// Parses `horizon` using [`Horizon`]'s string parser and performs the lookup.
    ///
    /// # Errors
    /// Returns `DomainError` if the provided `horizon` string cannot be parsed.
    pub fn find_by_horizon_str(&self, horizon: &str) -> Result<Option<&TrendPoint>, DomainError> {
        let parsed: Horizon = horizon.parse()?;
        Ok(self.find_by_horizon(&parsed))
    }

    /// Returns all available horizons in the historical data.
    #[must_use]
    pub fn available_horizons(&self) -> Vec<Horizon> {
        self.historical
            .iter()
            .map(|point| point.horizon.clone())
            .collect()
    }
}

/// A flexible data point for revision counts over different lookback horizons.
///
/// This struct allows any provider to represent revision data for any lookback
/// horizon, making the system provider-agnostic instead of tied to specific
/// hardcoded buckets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct RevisionPoint {
    /// The lookback horizon this data point represents (e.g., "7d", "1mo", "3mo").
    /// This allows providers to use their own horizon conventions.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub horizon: Horizon,
    /// Number of upward revisions in this horizon.
    pub up_count: u32,
    /// Number of downward revisions in this horizon.
    pub down_count: u32,
}

impl RevisionPoint {
    /// Creates a new revision point with the specified horizon and counts.
    #[must_use]
    pub const fn new(horizon: Horizon, up_count: u32, down_count: u32) -> Self {
        Self {
            horizon,
            up_count,
            down_count,
        }
    }

    /// Creates a new revision point from a horizon string.
    ///
    /// # Errors
    /// Returns an error if the horizon string cannot be parsed.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn try_new_str(horizon: &str, up: u32, down: u32) -> Result<Self, DomainError> {
        Ok(Self {
            horizon: horizon.parse()?,
            up_count: up,
            down_count: down,
        })
    }

    /// Returns the total number of revisions (up + down) in this horizon.
    #[must_use]
    pub fn total_revisions(&self) -> u64 {
        u64::from(self.up_count) + u64::from(self.down_count)
    }

    /// Returns the net revision count (up - down) in this horizon.
    /// Positive values indicate more upward revisions, negative values indicate more downward revisions.
    #[must_use]
    pub fn net_revisions(&self) -> i64 {
        i64::from(self.up_count) - i64::from(self.down_count)
    }
}

/// EPS revisions tracking upward and downward changes.
///
/// This struct now uses a flexible collection of revision points instead of
/// hardcoded time buckets, making it provider-agnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct EpsRevisions {
    /// Historical EPS revision data points with flexible lookback horizons.
    /// Each provider can populate this with their available horizons
    /// (e.g., a generic provider might use "7d", "30d" while another provider might
    /// use "1mo", "3mo", "6mo").
    pub historical: Vec<RevisionPoint>,
}

impl EpsRevisions {
    /// Creates a new EPS revisions struct with the specified historical data.
    #[must_use]
    pub const fn new(historical: Vec<RevisionPoint>) -> Self {
        Self { historical }
    }

    /// Finds a revision point by horizon.
    #[must_use]
    pub fn find_by_horizon(&self, horizon: &Horizon) -> Option<&RevisionPoint> {
        self.historical
            .iter()
            .find(|point| &point.horizon == horizon)
    }

    /// Finds a revision point by horizon string.
    ///
    /// Parses `horizon` using [`Horizon`]'s string parser and performs the lookup.
    ///
    /// # Errors
    /// Returns `DomainError` if the provided `horizon` string cannot be parsed.
    pub fn find_by_horizon_str(
        &self,
        horizon: &str,
    ) -> Result<Option<&RevisionPoint>, DomainError> {
        let parsed: Horizon = horizon.parse()?;
        Ok(self.find_by_horizon(&parsed))
    }

    /// Returns all available horizons in the historical data.
    #[must_use]
    pub fn available_horizons(&self) -> Vec<Horizon> {
        self.historical
            .iter()
            .map(|point| point.horizon.clone())
            .collect()
    }

    /// Returns the total number of upward revisions across all horizons.
    #[must_use]
    pub fn total_up_revisions(&self) -> u64 {
        self.historical
            .iter()
            .map(|point| u64::from(point.up_count))
            .sum()
    }

    /// Returns the total number of downward revisions across all horizons.
    #[must_use]
    pub fn total_down_revisions(&self) -> u64 {
        self.historical
            .iter()
            .map(|point| u64::from(point.down_count))
            .sum()
    }

    /// Returns the net revision count across all horizons (total up - total down).
    #[must_use]
    pub fn net_revisions(&self) -> i64 {
        self.historical
            .iter()
            .map(RevisionPoint::net_revisions)
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Represents a single row of earnings trend data for a specific period.
pub struct EarningsTrendRow {
    /// The period the trend data applies to with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// The growth rate.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub growth: Option<Decimal>,
    /// Earnings estimate data with analyst consensus.
    pub earnings_estimate: EarningsEstimate,
    /// Revenue estimate data with analyst consensus.
    pub revenue_estimate: RevenueEstimate,
    /// EPS trend changes over different lookback horizons.
    pub eps_trend: EpsTrend,
    /// EPS revisions tracking upward and downward changes by lookback horizon.
    pub eps_revisions: EpsRevisions,
}
