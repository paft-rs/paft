use chrono::DateTime;
use chrono_tz::Tz;
use iso_currency::Currency as IsoCurrency;
use paft_domain::{AssetKind, Instrument};
use paft_market::market::action::Action;
use paft_market::{Candle, CandleUpdate, HistoryMeta, HistoryResponse, Interval};
use paft_money::{Currency, Decimal, Money};
use std::str::FromStr;

#[test]
fn candle_serialization() {
    let candle = Candle {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        open: Money::new(
            Decimal::from_str("100.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        high: Money::new(
            Decimal::from_str("110.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        low: Money::new(
            Decimal::from_str("95.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        close: Money::new(
            Decimal::from_str("105.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        close_unadj: None,
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
        open: Money::new(
            Decimal::from_str("100.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        high: Money::new(
            Decimal::from_str("110.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        low: Money::new(
            Decimal::from_str("95.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        close: Money::new(
            Decimal::from_str("105.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        close_unadj: None,
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
        amount: Money::new(
            Decimal::from_str("0.5").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
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
        gain: Money::new(
            Decimal::from_str("1.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
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
fn responses_smoke() {
    let candles = vec![Candle {
        ts: chrono::DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        open: Money::new(
            Decimal::from_str("100.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        high: Money::new(
            Decimal::from_str("110.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        low: Money::new(
            Decimal::from_str("95.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        close: Money::new(
            Decimal::from_str("105.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        close_unadj: None,
        volume: Some(1_000_000),
    }];

    let meta = HistoryMeta {
        timezone: Some("America/New_York".parse::<Tz>().unwrap()),
        utc_offset_seconds: Some(-5 * 3600),
    };

    let response = HistoryResponse {
        candles,
        actions: vec![],
        adjusted: false,
        meta: Some(meta),
    };

    assert_eq!(response.candles.len(), 1);
}

#[test]
fn complex_nested_serialization() {
    let response = HistoryResponse {
        meta: Some(HistoryMeta {
            timezone: Some("America/New_York".parse::<Tz>().unwrap()),
            utc_offset_seconds: Some(-5 * 3600),
        }),
        candles: vec![Candle {
            ts: chrono::DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            open: Money::new(
                Decimal::from_str("100.0").unwrap(),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
            high: Money::new(
                Decimal::from_str("110.0").unwrap(),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
            low: Money::new(
                Decimal::from_str("95.0").unwrap(),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
            close: Money::new(
                Decimal::from_str("105.0").unwrap(),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
            close_unadj: None,
            volume: Some(1_000_000),
        }],
        actions: vec![],
        adjusted: false,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: HistoryResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(response, deserialized);
}

#[test]
fn candle_update_serialization() {
    let instrument = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
    let candle = Candle {
        ts: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        open: Money::new(
            Decimal::from_str("150.0").unwrap(),
            Currency::Iso(iso_currency::Currency::USD),
        )
        .unwrap(),
        high: Money::new(
            Decimal::from_str("155.0").unwrap(),
            Currency::Iso(iso_currency::Currency::USD),
        )
        .unwrap(),
        low: Money::new(
            Decimal::from_str("148.0").unwrap(),
            Currency::Iso(iso_currency::Currency::USD),
        )
        .unwrap(),
        close: Money::new(
            Decimal::from_str("152.0").unwrap(),
            Currency::Iso(iso_currency::Currency::USD),
        )
        .unwrap(),
        close_unadj: None,
        volume: Some(2_500_000),
    };
    let update = CandleUpdate {
        instrument,
        interval: Interval::I1m,
        candle,
        is_final: true,
    };

    let json = serde_json::to_string(&update).unwrap();
    let roundtrip: CandleUpdate = serde_json::from_str(&json).unwrap();
    assert_eq!(update, roundtrip);
}
