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

use chrono::{DateTime, NaiveDate, Utc};
use paft_decimal::Decimal;
use paft_money::{Money, Price, QuantityAmount};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Slow-moving valuation, dividend, and risk metrics for an instrument.
///
/// All fields are optional because providers expose different subsets and
/// some metrics are undefined for certain asset classes (e.g. dividend
/// yield on a non-dividend-paying stock).
pub struct KeyStatistics {
    /// Timestamp at which these statistics were observed. Useful when
    /// snapshotting price-driven values like `market_cap` that move
    /// intraday.
    #[serde(default, with = "paft_core::serde_helpers::ts_milliseconds_option")]
    pub as_of: Option<DateTime<Utc>>,

    // ---- Valuation ----
    /// Market capitalisation (price × shares outstanding).
    pub market_cap: Option<Money>,
    /// Shares outstanding.
    pub shares_outstanding: Option<u64>,

    // ---- Earnings (trailing) ----
    /// Earnings per share over the trailing twelve months.
    pub eps_trailing_twelve_months: Option<Price>,
    /// Price-to-earnings ratio computed against trailing-twelve-month EPS.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub pe_trailing_twelve_months: Option<Decimal>,

    // ---- Dividends ----
    /// Forward (declared / expected) dividend per share.
    pub dividend_per_share_forward: Option<Price>,
    /// Trailing twelve-month dividend yield expressed as a fraction
    /// (e.g. 0.025 for 2.5%).
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub dividend_yield_trailing: Option<Decimal>,
    /// Forward dividend yield expressed as a fraction.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub dividend_yield_forward: Option<Decimal>,
    /// Next or most recent ex-dividend calendar date.
    #[serde(default)]
    pub ex_dividend_date: Option<NaiveDate>,

    // ---- 52-week range ----
    /// 52-week high price.
    pub fifty_two_week_high: Option<Price>,
    /// 52-week low price.
    pub fifty_two_week_low: Option<Price>,

    // ---- Volume statistic ----
    /// Arithmetic mean of daily traded quantity over the trailing three months,
    /// including zero-volume trading sessions. Fractional averages are retained.
    ///
    /// The unit is shares for equities. For other instruments, the surrounding
    /// context or provider metadata must identify the quantity unit (for example,
    /// contracts or base-asset units). `None` means unavailable; zero is a reported
    /// average of zero.
    pub average_daily_volume_3m: Option<QuantityAmount>,

    // ---- Risk ----
    /// Market beta. The calculation period and frequency are not
    /// standardised across providers (Yahoo uses 5y monthly; Bloomberg
    /// is configurable); consumers comparing values across sources
    /// should account for this.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub beta: Option<Decimal>,
}

#[cfg(feature = "dataframe")]
paft_utils::impl_checked_dataframe! {
    KeyStatistics {
        as_of: [Option<DateTime<Utc>>],
        market_cap: [Option<Money>],
        shares_outstanding: [Option<u64>],
        eps_trailing_twelve_months: [Option<Price>],
        pe_trailing_twelve_months: [Option<Decimal>],
        dividend_per_share_forward: [Option<Price>],
        dividend_yield_trailing: [Option<Decimal>],
        dividend_yield_forward: [Option<Decimal>],
        ex_dividend_date: [Option<NaiveDate>],
        fifty_two_week_high: [Option<Price>],
        fifty_two_week_low: [Option<Price>],
        average_daily_volume_3m: [Option<QuantityAmount>],
        beta: [Option<Decimal>],
    }
    validate |row| row.as_of.iter().all(|ts| paft_core::serde_helpers::timestamp_millis_exact(ts).is_some())
}
