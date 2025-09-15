//! Analyst, recommendations, and earnings-related types under `paft_fundamentals::fundamentals::analysis`.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
#[cfg(feature = "dataframe")]
use paft_core::dataframe::ToDataFrame;
use paft_core::domain::{Money, Period};

/// Analyst recommendation grades with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of recommendation grades while gracefully
/// handling unknown or provider-specific grades through the `Other` variant.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, AsRefStr, EnumString,
)]
#[strum(ascii_case_insensitive)]
#[serde(from = "String", into = "String")]
pub enum RecommendationGrade {
    /// Strong buy recommendation
    #[strum(to_string = "STRONG_BUY", serialize = "STRONG BUY", serialize = "BUY+")]
    StrongBuy,
    /// Buy recommendation
    #[strum(to_string = "BUY")]
    Buy,
    /// Hold recommendation
    #[strum(to_string = "HOLD", serialize = "NEUTRAL")]
    Hold,
    /// Sell recommendation
    #[strum(to_string = "SELL")]
    Sell,
    /// Strong sell recommendation
    #[strum(
        to_string = "STRONG_SELL",
        serialize = "STRONG SELL",
        serialize = "SELL-"
    )]
    StrongSell,
    /// Outperform recommendation
    #[strum(to_string = "OUTPERFORM", serialize = "OVERWEIGHT")]
    Outperform,
    /// Underperform recommendation
    #[strum(to_string = "UNDERPERFORM", serialize = "UNDERWEIGHT")]
    Underperform,
    /// Unknown or provider-specific grade
    Other(String),
}

impl From<String> for RecommendationGrade {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        Self::from_str(&s).unwrap_or_else(|_| Self::Other(s.to_uppercase()))
    }
}

impl From<RecommendationGrade> for String {
    fn from(grade: RecommendationGrade) -> Self {
        match grade {
            RecommendationGrade::Other(s) => s,
            _ => grade.to_string(),
        }
    }
}

/// Analyst recommendation actions with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of recommendation actions while gracefully
/// handling unknown or provider-specific actions through the `Other` variant.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, AsRefStr, EnumString,
)]
#[strum(ascii_case_insensitive)]
#[serde(from = "String", into = "String")]
pub enum RecommendationAction {
    /// Upgrade action
    #[strum(to_string = "UPGRADE", serialize = "UP")]
    Upgrade,
    /// Downgrade action
    #[strum(to_string = "DOWNGRADE", serialize = "DOWN")]
    Downgrade,
    /// Initiate coverage
    #[strum(to_string = "INIT", serialize = "INITIATED", serialize = "INITIATE")]
    Initiate,
    /// Maintain or reiterate recommendation
    #[strum(to_string = "MAINTAIN", serialize = "REITERATE")]
    Maintain,
    /// Resume coverage
    #[strum(to_string = "RESUME")]
    Resume,
    /// Suspend coverage
    #[strum(to_string = "SUSPEND")]
    Suspend,
    /// Unknown or provider-specific action
    Other(String),
}

impl From<String> for RecommendationAction {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        Self::from_str(&s).unwrap_or_else(|_| Self::Other(s.to_uppercase()))
    }
}

impl From<RecommendationAction> for String {
    fn from(action: RecommendationAction) -> Self {
        match action {
            RecommendationAction::Other(s) => s,
            _ => action.to_string(),
        }
    }
}

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
    pub period: String,
    /// The value for this time period.
    pub value: Money,
}

impl TrendPoint {
    /// Creates a new trend point with the specified period and value.
    pub fn new(period: impl Into<String>, value: Money) -> Self {
        Self {
            period: period.into(),
            value,
        }
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

    /// Finds a trend point by period string.
    #[must_use]
    pub fn find_by_period(&self, period: &str) -> Option<&TrendPoint> {
        self.historical.iter().find(|point| point.period == period)
    }

    /// Returns all available periods in the historical data.
    #[must_use]
    pub fn available_periods(&self) -> Vec<&str> {
        self.historical
            .iter()
            .map(|point| point.period.as_str())
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
    pub period: String,
    /// Number of upward revisions in this period.
    pub up_count: u32,
    /// Number of downward revisions in this period.
    pub down_count: u32,
}

impl RevisionPoint {
    /// Creates a new revision point with the specified period and counts.
    pub fn new(period: impl Into<String>, up_count: u32, down_count: u32) -> Self {
        Self {
            period: period.into(),
            up_count,
            down_count,
        }
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

    /// Finds a revision point by period string.
    #[must_use]
    pub fn find_by_period(&self, period: &str) -> Option<&RevisionPoint> {
        self.historical.iter().find(|point| point.period == period)
    }

    /// Returns all available periods in the historical data.
    #[must_use]
    pub fn available_periods(&self) -> Vec<&str> {
        self.historical
            .iter()
            .map(|point| point.period.as_str())
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
    #[serde(flatten)]
    pub earnings_estimate: EarningsEstimate,
    /// Revenue estimate data with analyst consensus.
    #[serde(flatten)]
    pub revenue_estimate: RevenueEstimate,
    /// EPS trend changes over different time periods.
    #[serde(flatten)]
    pub eps_trend: EpsTrend,
    /// EPS revisions tracking upward and downward changes.
    #[serde(flatten)]
    pub eps_revisions: EpsRevisions,
}
