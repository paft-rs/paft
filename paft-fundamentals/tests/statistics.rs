use chrono::{NaiveDate, TimeZone, Utc};
use paft_decimal::Decimal;
use paft_fundamentals::statistics::KeyStatistics;
use paft_money::{Currency, IsoCurrency, Money, Price, QuantityAmount};
use serde_json::{from_str, json, to_string};
use std::str::FromStr;

fn usd(amount: i64) -> Money {
    Money::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD)).unwrap()
}

fn usd_price(amount: i64) -> Price {
    Price::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD))
}

fn dec(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap()
}

const fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
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
        eps_trailing_twelve_months: Some(usd_price(6)),
        pe_trailing_twelve_months: Some(dec("28.4")),
        dividend_per_share_forward: Some(usd_price(1)),
        dividend_yield_trailing: Some(dec("0.0050")),
        dividend_yield_forward: Some(dec("0.0055")),
        ex_dividend_date: Some(date(2023, 11, 15)),
        fifty_two_week_high: Some(usd_price(200)),
        fifty_two_week_low: Some(usd_price(120)),
        average_daily_volume_3m: Some(QuantityAmount::from_decimal(55_000_000.into()).unwrap()),
        beta: Some(dec("1.23")),
    };

    let encoded = to_string(&s).unwrap();
    let value: serde_json::Value = from_str(&encoded).unwrap();
    assert_eq!(value["dividend_yield_trailing"], json!("0.005"));
    assert_eq!(value["dividend_yield_forward"], json!("0.0055"));
    assert_eq!(value["ex_dividend_date"], json!("2023-11-15"));
    assert_eq!(value["average_daily_volume_3m"], json!("55000000"));
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

#[test]
fn fractional_average_daily_volume_preserves_quantity_semantics() {
    // 150 shares across 60 trading sessions average 2.5 shares per session.
    for amount in ["2.5", "0", "0.0000000000000000000000000001"] {
        let quantity = QuantityAmount::from_decimal(dec(amount)).unwrap();
        let row = KeyStatistics {
            average_daily_volume_3m: Some(quantity.clone()),
            ..KeyStatistics::default()
        };
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["average_daily_volume_3m"], amount);
        let decoded: KeyStatistics = serde_json::from_value(json).unwrap();
        assert_eq!(decoded.average_daily_volume_3m, Some(quantity));
    }
    for json in [json!({}), json!({"average_daily_volume_3m": null})] {
        let row: KeyStatistics = serde_json::from_value(json).unwrap();
        assert!(row.average_daily_volume_3m.is_none());
    }
    for value in [json!("-2.5"), json!(2.5), json!("1e-29")] {
        assert!(
            serde_json::from_value::<KeyStatistics>(json!({
                "average_daily_volume_3m": value
            }))
            .is_err()
        );
    }
}
