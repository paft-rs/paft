use chrono::{TimeZone, Utc};
use iso_currency::Currency as IsoCurrency;
use paft_decimal::Decimal;
use paft_fundamentals::statistics::KeyStatistics;
use paft_money::{Currency, Money};
use serde_json::{from_str, to_string};
use std::str::FromStr;

fn usd(amount: i64) -> Money {
    Money::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD)).unwrap()
}

fn dec(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap()
}

#[test]
fn key_statistics_default_is_all_none() {
    let s = KeyStatistics::default();
    assert!(s.as_of.is_none());
    assert!(s.market_cap.is_none());
    assert!(s.shares_outstanding.is_none());
    assert!(s.eps_trailing_twelve_months.is_none());
    assert!(s.pe_trailing_twelve_months.is_none());
    assert!(s.dividend_per_share_forward.is_none());
    assert!(s.dividend_yield_trailing.is_none());
    assert!(s.dividend_yield_forward.is_none());
    assert!(s.ex_dividend_date.is_none());
    assert!(s.fifty_two_week_high.is_none());
    assert!(s.fifty_two_week_low.is_none());
    assert!(s.average_daily_volume_3m.is_none());
    assert!(s.beta.is_none());
}

#[test]
fn key_statistics_serde_roundtrip_populated() {
    let s = KeyStatistics {
        as_of: Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap()),
        market_cap: Some(usd(2_500_000_000_000)),
        shares_outstanding: Some(15_500_000_000),
        eps_trailing_twelve_months: Some(usd(6)),
        pe_trailing_twelve_months: Some(dec("28.4")),
        dividend_per_share_forward: Some(usd(1)),
        dividend_yield_trailing: Some(dec("0.0050")),
        dividend_yield_forward: Some(dec("0.0055")),
        ex_dividend_date: Some(Utc.timestamp_opt(1_700_086_400, 0).unwrap()),
        fifty_two_week_high: Some(usd(200)),
        fifty_two_week_low: Some(usd(120)),
        average_daily_volume_3m: Some(55_000_000),
        beta: Some(dec("1.23")),
    };

    let encoded = to_string(&s).unwrap();
    let decoded: KeyStatistics = from_str(&encoded).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn key_statistics_serde_roundtrip_empty() {
    let s = KeyStatistics::default();
    let encoded = to_string(&s).unwrap();
    let decoded: KeyStatistics = from_str(&encoded).unwrap();
    assert_eq!(s, decoded);
}
