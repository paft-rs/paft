use chrono::{NaiveDate, TimeZone, Utc};
use paft_aggregates::{FastInfo, Info};
use paft_domain::{Exchange, Isin, MarketState, Symbol};
use paft_money::IsoCurrency;
use paft_money::{Currency, Money};

#[test]
fn fast_info_roundtrip() {
    let usd = Currency::Iso(IsoCurrency::USD);
    let fast = FastInfo {
        symbol: Symbol::new("AAPL").unwrap(),
        name: Some("Apple Inc.".to_string()),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        currency: Some(usd.clone()),
        last: Some(Money::from_canonical_str("170.12", usd.clone()).unwrap()),
        previous_close: Some(Money::from_canonical_str("169.50", usd).unwrap()),
        volume: Some(12_345_678),
    };

    let json = serde_json::to_string(&fast).unwrap();
    let back: FastInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back, fast);
}

#[test]
fn info_roundtrip() {
    let usd = Currency::Iso(IsoCurrency::USD);
    let info = Info {
        symbol: Symbol::new("MSFT").unwrap(),
        name: Some("Microsoft Corporation".to_string()),
        isin: Some(Isin::new("US5949181045").unwrap()),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Pre),
        currency: Some(usd.clone()),
        last: Some(Money::from_canonical_str("430.01", usd.clone()).unwrap()),
        open: Some(Money::from_canonical_str("428.00", usd.clone()).unwrap()),
        high: Some(Money::from_canonical_str("432.22", usd.clone()).unwrap()),
        low: Some(Money::from_canonical_str("427.80", usd.clone()).unwrap()),
        previous_close: Some(Money::from_canonical_str("429.50", usd.clone()).unwrap()),
        day_range_low: Some(Money::from_canonical_str("427.80", usd.clone()).unwrap()),
        day_range_high: Some(Money::from_canonical_str("432.22", usd.clone()).unwrap()),
        fifty_two_week_low: Some(Money::from_canonical_str("310.00", usd.clone()).unwrap()),
        fifty_two_week_high: Some(Money::from_canonical_str("450.00", usd.clone()).unwrap()),
        volume: Some(25_000_000),
        average_volume: Some(23_000_000),
        market_cap: Some(Money::from_minor_units(3_000_000_000_000i128, usd.clone()).unwrap()),
        shares_outstanding: Some(7_500_000_000),
        eps_ttm: Some(Money::from_canonical_str("11.20", usd).unwrap()),
        pe_ttm: Some(38.4),
        dividend_yield: Some(0.008),
        ex_dividend_date: Some(NaiveDate::from_ymd_opt(2024, 11, 14).unwrap()),
        price_target: None,
        recommendation_summary: None,
        esg_scores: None,
        as_of: Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap()),
    };

    let json = serde_json::to_string(&info).unwrap();
    let back: Info = serde_json::from_str(&json).unwrap();
    assert_eq!(back, info);
}
