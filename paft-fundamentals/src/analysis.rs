//! Analyst, recommendations, and earnings-related types under `paft_fundamentals::fundamentals::analysis`.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
#[cfg(feature = "dataframe")]
use paft_core::dataframe::ToDataFrame;
use paft_core::domain::{Money, Period};
use paft_core::error::PaftError;

use paft_core::domain::string_canonical::Canonical;

/// Analyst recommendation grades with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of recommendation grades while gracefully
/// handling unknown or provider-specific grades through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes using an escape prefix `~` as "~{s}" and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for all values, including `Other`, via the escape prefix
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
    Other(Canonical),
}

impl RecommendationGrade {
    /// Attempts to parse a recommendation grade, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `PaftError::InvalidEnumValue` when `input` is empty/whitespace.
    pub fn try_from_str(input: &str) -> Result<Self, PaftError> {
        Self::from_str(input)
    }
}

// serde via macro

// Implement code() and string impls via macro
paft_core::string_enum_with_code!(
    RecommendationGrade, Other, "RecommendationGrade",
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

/// Analyst recommendation actions with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of recommendation actions while gracefully
/// handling unknown or provider-specific actions through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes using an escape prefix `~` as "~{s}" and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for all values, including `Other`, via the escape prefix
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
    Other(Canonical),
}

impl RecommendationAction {
    /// Attempts to parse a recommendation action, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `PaftError::InvalidEnumValue` when `input` is empty/whitespace.
    pub fn try_from_str(input: &str) -> Result<Self, PaftError> {
        Self::from_str(input)
    }
}

// Implement code() and string impls via macro
paft_core::string_enum_with_code!(
    RecommendationAction, Other, "RecommendationAction",
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
    pub actual: Option<Money>,
    /// Estimated EPS.
    pub estimate: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Analyst price target summary.
pub struct PriceTarget {
    /// Mean price target.
    pub mean: Option<Money>,
    /// High price target.
    pub high: Option<Money>,
    /// Low price target.
    pub low: Option<Money>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
    pub mean: Option<f64>,
    /// Provider-specific text for the mean score (e.g., "Buy", "Overweight").
    pub mean_rating_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Broker action history for an instrument.
pub struct UpgradeDowngradeRow {
    /// Event timestamp.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ts: DateTime<Utc>,
    /// Research firm name.
    pub firm: Option<String>,
    /// Previous rating with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub from_grade: Option<RecommendationGrade>,
    /// New rating with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub to_grade: Option<RecommendationGrade>,
    /// Action description with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub action: Option<RecommendationAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Summary of key analysis metrics extracted from detailed analysis data.
pub struct AnalysisSummary {
    /// Analyst target mean price.
    pub target_mean_price: Option<Money>,
    /// Analyst target high price.
    pub target_high_price: Option<Money>,
    /// Analyst target low price.
    pub target_low_price: Option<Money>,
    /// Number of analyst opinions contributing to the recommendation.
    pub number_of_analyst_opinions: Option<u32>,
    /// Numeric recommendation score (provider-defined scale).
    pub recommendation_mean: Option<f64>,
    /// Categorical recommendation text (e.g., "Buy", "Overweight").
    pub recommendation_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Earnings estimate data with analyst consensus.
pub struct EarningsEstimate {
    /// Average earnings estimate.
    pub avg: Option<Money>,
    /// Low earnings estimate.
    pub low: Option<Money>,
    /// High earnings estimate.
    pub high: Option<Money>,
    /// Earnings per share from a year ago.
    pub year_ago_eps: Option<Money>,
    /// Number of analysts providing earnings estimates.
    pub num_analysts: Option<u32>,
    /// Estimated earnings growth.
    pub growth: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
    pub growth: Option<f64>,
}

/// A flexible data point for time-series trend data.
///
/// This struct allows any provider to represent trend data for any time period,
/// making the system provider-agnostic instead of tied to specific hardcoded buckets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct TrendPoint {
    /// The period this data point represents (e.g., "7d", "1mo", "3mo").
    /// This allows providers to use their own time period conventions.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// The value for this time period.
    pub value: Money,
}

impl TrendPoint {
    /// Creates a new trend point with the specified period and value.
    #[must_use]
    pub const fn new(period: Period, value: Money) -> Self {
        Self { period, value }
    }

    /// Creates a new trend point from a period string.
    ///
    /// # Errors
    /// Returns an error if the period string cannot be parsed.
    pub fn try_new_str(period: &str, value: Money) -> Result<Self, PaftError> {
        Ok(Self {
            period: period.parse()?,
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// EPS trend changes over different time periods.
///
/// This struct now uses a flexible collection of trend points instead of
/// hardcoded time buckets, making it provider-agnostic.
pub struct EpsTrend {
    /// Current EPS trend.
    pub current: Option<Money>,
    /// Historical EPS trend data points with flexible time periods.
    /// Each provider can populate this with their available time periods
    /// (e.g., a generic provider might use "7d", "30d", "60d", "90d" while another
    /// provider might use "1mo", "3mo", "6mo").
    pub historical: Vec<TrendPoint>,
}

impl EpsTrend {
    /// Creates a new EPS trend with the specified current value and historical data.
    #[must_use]
    pub const fn new(current: Option<Money>, historical: Vec<TrendPoint>) -> Self {
        Self {
            current,
            historical,
        }
    }

    /// Finds a trend point by period.
    #[must_use]
    pub fn find_by_period(&self, period: &Period) -> Option<&TrendPoint> {
        self.historical.iter().find(|point| &point.period == period)
    }

    /// Finds a trend point by period string.
    ///
    /// Parses `period` using `Period`'s string parser and performs the lookup.
    ///
    /// # Errors
    /// Returns `PaftError` if the provided `period` string cannot be parsed.
    pub fn find_by_period_str(&self, period: &str) -> Result<Option<&TrendPoint>, PaftError> {
        let parsed: Period = period.parse()?;
        Ok(self.find_by_period(&parsed))
    }

    /// Returns all available periods in the historical data.
    #[must_use]
    pub fn available_periods(&self) -> Vec<Period> {
        self.historical
            .iter()
            .map(|point| point.period.clone())
            .collect()
    }
}

/// A flexible data point for revision counts over different time periods.
///
/// This struct allows any provider to represent revision data for any time period,
/// making the system provider-agnostic instead of tied to specific hardcoded buckets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct RevisionPoint {
    /// The period this data point represents (e.g., "7d", "1mo", "3mo").
    /// This allows providers to use their own time period conventions.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// Number of upward revisions in this period.
    pub up_count: u32,
    /// Number of downward revisions in this period.
    pub down_count: u32,
}

impl RevisionPoint {
    /// Creates a new revision point with the specified period and counts.
    #[must_use]
    pub const fn new(period: Period, up_count: u32, down_count: u32) -> Self {
        Self {
            period,
            up_count,
            down_count,
        }
    }

    /// Creates a new revision point from a period string.
    ///
    /// # Errors
    /// Returns an error if the period string cannot be parsed.
    pub fn try_new_str(period: &str, up: u32, down: u32) -> Result<Self, PaftError> {
        Ok(Self {
            period: period.parse()?,
            up_count: up,
            down_count: down,
        })
    }

    /// Returns the total number of revisions (up + down) in this period.
    #[must_use]
    pub const fn total_revisions(&self) -> u32 {
        self.up_count + self.down_count
    }

    /// Returns the net revision count (up - down) in this period.
    /// Positive values indicate more upward revisions, negative values indicate more downward revisions.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn net_revisions(&self) -> i32 {
        self.up_count as i32 - self.down_count as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// EPS revisions tracking upward and downward changes.
///
/// This struct now uses a flexible collection of revision points instead of
/// hardcoded time buckets, making it provider-agnostic.
pub struct EpsRevisions {
    /// Historical EPS revision data points with flexible time periods.
    /// Each provider can populate this with their available time periods
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

    /// Finds a revision point by period.
    #[must_use]
    pub fn find_by_period(&self, period: &Period) -> Option<&RevisionPoint> {
        self.historical.iter().find(|point| &point.period == period)
    }

    /// Finds a revision point by period string.
    ///
    /// Parses `period` using `Period`'s string parser and performs the lookup.
    ///
    /// # Errors
    /// Returns `PaftError` if the provided `period` string cannot be parsed.
    pub fn find_by_period_str(&self, period: &str) -> Result<Option<&RevisionPoint>, PaftError> {
        let parsed: Period = period.parse()?;
        Ok(self.find_by_period(&parsed))
    }

    /// Returns all available periods in the historical data.
    #[must_use]
    pub fn available_periods(&self) -> Vec<Period> {
        self.historical
            .iter()
            .map(|point| point.period.clone())
            .collect()
    }

    /// Returns the total number of upward revisions across all periods.
    #[must_use]
    pub fn total_up_revisions(&self) -> u32 {
        self.historical.iter().map(|point| point.up_count).sum()
    }

    /// Returns the total number of downward revisions across all periods.
    #[must_use]
    pub fn total_down_revisions(&self) -> u32 {
        self.historical.iter().map(|point| point.down_count).sum()
    }

    /// Returns the net revision count across all periods (total up - total down).
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn net_revisions(&self) -> i32 {
        self.total_up_revisions() as i32 - self.total_down_revisions() as i32
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Represents a single row of earnings trend data for a specific period.
pub struct EarningsTrendRow {
    /// The period the trend data applies to with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// The growth rate.
    pub growth: Option<f64>,
    /// Earnings estimate data with analyst consensus.
    pub earnings_estimate: EarningsEstimate,
    /// Revenue estimate data with analyst consensus.
    pub revenue_estimate: RevenueEstimate,
    /// EPS trend changes over different time periods.
    pub eps_trend: EpsTrend,
    /// EPS revisions tracking upward and downward changes.
    pub eps_revisions: EpsRevisions,
}
