use chrono::DateTime;
use chrono_tz::Tz;
use paft_core::domain::{Currency, Money};
use paft_market::market::action::Action;
use paft_market::responses::history::{Candle, HistoryMeta, HistoryResponse};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn candle_serialization() {
    let candle = Candle {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        open: Money::new(Decimal::from_str("100.0").unwrap(), Currency::USD),
        high: Money::new(Decimal::from_str("110.0").unwrap(), Currency::USD),
        low: Money::new(Decimal::from_str("95.0").unwrap(), Currency::USD),
        close: Money::new(Decimal::from_str("105.0").unwrap(), Currency::USD),
        volume: Some(1_000_000),
    };

    let json = serde_json::to_string(&candle).unwrap();
    let deserialized: Candle = serde_json::from_str(&json).unwrap();
    assert_eq!(candle, deserialized);
}

#[test]
fn candle_with_none_volume() {
    let candle = Candle {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        open: Money::new(Decimal::from_str("100.0").unwrap(), Currency::USD),
        high: Money::new(Decimal::from_str("110.0").unwrap(), Currency::USD),
        low: Money::new(Decimal::from_str("95.0").unwrap(), Currency::USD),
        close: Money::new(Decimal::from_str("105.0").unwrap(), Currency::USD),
        volume: None,
    };

    let json = serde_json::to_string(&candle).unwrap();
    let deserialized: Candle = serde_json::from_str(&json).unwrap();
    assert_eq!(candle, deserialized);
}

#[test]
fn action_dividend_serialization() {
    let action = Action::Dividend {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        amount: Money::new(Decimal::from_str("0.5").unwrap(), Currency::USD),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn action_split_serialization() {
    let action = Action::Split {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        numerator: 2,
        denominator: 1,
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn action_capital_gain_serialization() {
    let action = Action::CapitalGain {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        gain: Money::new(Decimal::from_str("1.0").unwrap(), Currency::USD),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn history_meta_serialization() {
    let meta = HistoryMeta {
        timezone: Some("America/New_York".parse::<Tz>().unwrap()),
        utc_offset_seconds: Some(-18_000),
    };

    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: HistoryMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(meta, deserialized);
}

#[test]
fn history_meta_with_none_fields() {
    let meta = HistoryMeta {
        timezone: None,
        utc_offset_seconds: None,
    };

    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: HistoryMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(meta, deserialized);
}

#[test]
fn complex_nested_serialization() {
    let history_response = HistoryResponse {
        candles: vec![Candle {
            ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            open: Money::new(Decimal::from_str("100.0").unwrap(), Currency::USD),
            high: Money::new(Decimal::from_str("110.0").unwrap(), Currency::USD),
            low: Money::new(Decimal::from_str("95.0").unwrap(), Currency::USD),
            close: Money::new(Decimal::from_str("105.0").unwrap(), Currency::USD),
            volume: Some(1_000_000),
        }],
        actions: vec![Action::Dividend {
            ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            amount: Money::new(Decimal::from_str("0.5").unwrap(), Currency::USD),
        }],
        adjusted: true,
        meta: Some(HistoryMeta {
            timezone: Some("America/New_York".parse::<Tz>().unwrap()),
            utc_offset_seconds: Some(-18_000),
        }),
        unadjusted_close: Some(vec![Money::new(
            Decimal::from_str("105.0").unwrap(),
            Currency::USD,
        )]),
    };

    let json = serde_json::to_string(&history_response).unwrap();
    let deserialized: HistoryResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(history_response, deserialized);
}
