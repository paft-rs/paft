use chrono::{DateTime, NaiveDate, Utc};
use paft_domain::{Exchange, Isin, MarketState};
use paft_money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FastInfo {
    pub symbol: String,
    pub name: Option<String>,
    pub exchange: Option<Exchange>,
    pub market_state: Option<MarketState>,
    pub currency: Option<Currency>,
    pub last: Option<Money>,
    pub previous_close: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Info {
    // Identity
    pub symbol: String,
    pub name: Option<String>,
    pub isin: Option<Isin>,
    pub exchange: Option<Exchange>,

    // Market snapshot
    pub market_state: Option<MarketState>,
    pub currency: Option<Currency>,
    pub last: Option<Money>,
    pub open: Option<Money>,
    pub high: Option<Money>,
    pub low: Option<Money>,
    pub previous_close: Option<Money>,

    // Ranges & volumes
    pub day_range_low: Option<Money>,
    pub day_range_high: Option<Money>,
    pub fifty_two_week_low: Option<Money>,
    pub fifty_two_week_high: Option<Money>,
    pub volume: Option<u64>,
    pub average_volume: Option<u64>,

    // Fundamentals (generic)
    pub market_cap: Option<Money>,
    pub shares_outstanding: Option<u64>,
    pub eps_ttm: Option<Money>,
    pub pe_ttm: Option<f64>,
    pub dividend_yield: Option<f64>, // 0.025 = 2.5%
    pub ex_dividend_date: Option<NaiveDate>,

    // Timestamp of snapshot
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub as_of: Option<DateTime<Utc>>,
}
