//! Key statistics: slow-moving valuation, dividend, and risk metrics
//! associated with an instrument.
//!
//! These fields are typically derived from a mix of market data (price-driven,
//! refreshes intraday) and fundamentals (earnings, share counts, dividend
//! history). They are not part of a snapshot quote — see
//! [`paft_market::market::quote::Quote`](https://docs.rs/paft-market) for
//! point-in-time price data — and they are not part of a statement row
//! either, because they aggregate across periods. They live here, alongside
//! the other instrument-attached fundamentals types.

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_decimal::Decimal;
use paft_money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Slow-moving valuation, dividend, and risk metrics for an instrument.
///
/// All fields are optional because providers expose different subsets and
/// some metrics are undefined for certain asset classes (e.g. dividend
/// yield on a non-dividend-paying stock).
pub struct KeyStatistics {
    /// Timestamp at which these statistics were observed. Useful when
    /// snapshotting price-driven values like `market_cap` that move
    /// intraday.
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub as_of: Option<DateTime<Utc>>,

    // ---- Valuation ----
    /// Market capitalisation (price × shares outstanding).
    pub market_cap: Option<Money>,
    /// Shares outstanding.
    pub shares_outstanding: Option<u64>,

    // ---- Earnings (trailing) ----
    /// Earnings per share over the trailing twelve months.
    pub eps_trailing_twelve_months: Option<Money>,
    /// Price-to-earnings ratio computed against trailing-twelve-month EPS.
    pub pe_trailing_twelve_months: Option<Decimal>,

    // ---- Dividends ----
    /// Forward (declared / expected) dividend per share.
    pub dividend_per_share_forward: Option<Money>,
    /// Trailing twelve-month dividend yield expressed as a fraction
    /// (e.g. 0.025 for 2.5%).
    pub dividend_yield_trailing: Option<Decimal>,
    /// Forward dividend yield expressed as a fraction.
    pub dividend_yield_forward: Option<Decimal>,
    /// Next or most recent ex-dividend date.
    #[serde(with = "paft_core::serde_helpers::ts_seconds_option")]
    pub ex_dividend_date: Option<DateTime<Utc>>,

    // ---- 52-week range ----
    /// 52-week high price.
    pub fifty_two_week_high: Option<Money>,
    /// 52-week low price.
    pub fifty_two_week_low: Option<Money>,

    // ---- Volume statistic ----
    /// Average daily traded volume over the last three months.
    pub average_daily_volume_3m: Option<u64>,

    // ---- Risk ----
    /// Market beta. The calculation period and frequency are not
    /// standardised across providers (Yahoo uses 5y monthly; Bloomberg
    /// is configurable); consumers comparing values across sources
    /// should account for this.
    pub beta: Option<Decimal>,
}
