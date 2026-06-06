use chrono::{DateTime, NaiveDate};
use chrono_tz::Tz;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Instrument};
use paft_market::market::action::Action;
use paft_market::{
    AdjustmentAnchor, AdjustmentMethod, Candle, CandleUpdate, CorporateActionAdjustmentCause,
    CorporateActionAdjustmentCauses, GenericCandle, GenericHistoryResponse, HistoryMeta,
    HistoryResponse, HistoryValidationError, Interval, Ohlc, OhlcPriceBasis, PriceBasis,
};
use paft_money::{Currency, IsoCurrency, Price, PriceAmount, QuantityAmount};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::str::FromStr;

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: &str) -> PriceAmount {
    PriceAmount::new(Decimal::from_str(value).unwrap())
}

fn quantity(value: &str) -> QuantityAmount {
    QuantityAmount::from_decimal(Decimal::from_str(value).unwrap()).unwrap()
}

const fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn ohlc(open: &str, high: &str, low: &str, close: &str) -> Ohlc {
    Ohlc::new(amount(open), amount(high), amount(low), amount(close))
}

fn candle_at(unix_seconds: i64) -> Candle {
    Candle::new(
        DateTime::from_timestamp(unix_seconds, 0).unwrap(),
        usd(),
        ohlc("100.0", "110.0", "95.0", "105.0"),
    )
}

#[test]
fn candle_serialization() {
    let candle = Candle {
        ts: DateTime::from_timestamp(1_640_995_200, 321_000_000).unwrap(),
        currency: usd(),
        ohlc: ohlc("100.0", "110.0", "95.0", "105.0"),
        close_unadj: None,
        volume: Some(quantity("1000000.125")),

        provider: (),
    };

    let json = serde_json::to_string(&candle).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["ts"], serde_json::json!(1_640_995_200_321_i64));
    assert_eq!(value["currency"], serde_json::json!("USD"));
    assert_eq!(value["open"], serde_json::json!("100"));
    assert_eq!(value["volume"], serde_json::json!("1000000.125"));

    let deserialized: Candle = serde_json::from_str(&json).unwrap();
    assert_eq!(candle, deserialized);
}

#[test]
fn candle_with_none_volume() {
    let candle = Candle {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        currency: usd(),
        ohlc: ohlc("100.0", "110.0", "95.0", "105.0"),
        close_unadj: None,
        volume: None,

        provider: (),
    };

    let json = serde_json::to_string(&candle).unwrap();
    let deserialized: Candle = serde_json::from_str(&json).unwrap();
    assert_eq!(candle, deserialized);
}

#[test]
fn action_dividend_serialization() {
    let action = Action::Dividend {
        date: date(2022, 1, 1),
        amount: Price::new(Decimal::from_str("0.5").unwrap(), usd()),
    };

    let json = serde_json::to_string(&action).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["kind"], serde_json::json!("dividend"));
    assert_eq!(value["date"], serde_json::json!("2022-01-01"));
    assert_eq!(value["amount"]["amount"], serde_json::json!("0.5"));
    assert_eq!(value["amount"]["currency"], serde_json::json!("USD"));

    let deserialized: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn action_split_serialization_uses_new_shares_over_old_shares() {
    let action = Action::Split {
        date: date(2022, 1, 1),
        numerator: NonZeroU32::new(2).unwrap(),
        denominator: NonZeroU32::new(1).unwrap(),
    };

    let json = serde_json::to_string(&action).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "kind": "split",
            "date": "2022-01-01",
            "numerator": 2,
            "denominator": 1,
        })
    );

    let deserialized: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn action_split_rejects_zero_ratios() {
    for (numerator, denominator) in [(0, 1), (2, 0)] {
        let value = serde_json::json!({
            "kind": "split",
            "date": "2022-01-01",
            "numerator": numerator,
            "denominator": denominator,
        });

        let err = serde_json::from_value::<Action>(value).unwrap_err();
        assert!(err.to_string().contains("nonzero"));
    }
}

#[test]
fn action_ignores_unknown_fields() {
    let value = serde_json::json!({
        "kind": "dividend",
        "date": "2022-01-01",
        "amount": {
            "amount": "0.5",
            "currency": "USD",
        },
        "provider_field": true,
    });

    assert_eq!(
        serde_json::from_value::<Action>(value).unwrap(),
        Action::Dividend {
            date: date(2022, 1, 1),
            amount: Price::new(Decimal::from_str("0.5").unwrap(), usd()),
        }
    );
}

#[test]
fn action_capital_gain_serialization() {
    let action = Action::CapitalGain {
        date: date(2022, 1, 1),
        gain: Price::new(Decimal::from_str("1.0").unwrap(), usd()),
    };

    let json = serde_json::to_string(&action).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["kind"], serde_json::json!("capital_gain"));
    assert_eq!(value["date"], serde_json::json!("2022-01-01"));
    assert_eq!(value["gain"]["amount"], serde_json::json!("1"));
    assert_eq!(value["gain"]["currency"], serde_json::json!("USD"));

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
fn history_meta_deserialization_unknown_field_rejected() {
    let value = serde_json::json!({
        "timezone": "America/New_York",
        "utc_offset_seconds": -18_000,
        "utc_offset": "-05:00"
    });

    assert!(serde_json::from_value::<HistoryMeta>(value).is_err());
}

#[test]
fn ohlc_price_basis_helpers_and_serialization() {
    let raw = OhlcPriceBasis::raw();
    assert_eq!(
        raw.fields(),
        (
            &PriceBasis::Raw,
            &PriceBasis::Raw,
            &PriceBasis::Raw,
            &PriceBasis::Raw
        )
    );
    assert_eq!(
        serde_json::to_string(&raw).unwrap(),
        r#"{"kind":"uniform","basis":{"kind":"raw"}}"#
    );

    let provider_adjusted = PriceBasis::provider_latest_adjusted();
    assert_eq!(
        provider_adjusted,
        PriceBasis::ProviderAdjusted {
            anchor: AdjustmentAnchor::ProviderLatestBasis,
        }
    );
    assert_eq!(
        PriceBasis::provider_adjusted(AdjustmentAnchor::LastReturnedObservation),
        PriceBasis::ProviderAdjusted {
            anchor: AdjustmentAnchor::LastReturnedObservation,
        }
    );

    let split_adjusted = PriceBasis::split_adjusted_latest();
    assert_eq!(
        split_adjusted,
        PriceBasis::CorporateActionAdjusted {
            anchor: AdjustmentAnchor::ProviderLatestBasis,
            causes: CorporateActionAdjustmentCauses::splits(),
        }
    );
    assert_eq!(
        serde_json::to_string(&split_adjusted).unwrap(),
        r#"{"kind":"corporate_action_adjusted","anchor":{"kind":"provider_latest_basis"},"causes":["split"]}"#
    );

    let split_and_dividend_causes = CorporateActionAdjustmentCauses::splits()
        .union(CorporateActionAdjustmentCauses::dividends());
    assert!(split_and_dividend_causes.contains(CorporateActionAdjustmentCause::Split));
    assert!(split_and_dividend_causes.contains(CorporateActionAdjustmentCause::Dividend));
    assert!(!split_and_dividend_causes.contains(CorporateActionAdjustmentCause::CapitalGain));
    assert_eq!(
        serde_json::to_string(&split_and_dividend_causes).unwrap(),
        r#"["split","dividend"]"#
    );
    assert!(
        serde_json::from_str::<CorporateActionAdjustmentCauses>("[]").is_err(),
        "empty corporate-action cause sets should be rejected",
    );
    assert!(
        serde_json::from_str::<CorporateActionAdjustmentCauses>(r#"["split","split"]"#).is_err(),
        "duplicate corporate-action causes should be rejected",
    );

    let per_field = OhlcPriceBasis::per_field(
        PriceBasis::raw(),
        split_adjusted,
        PriceBasis::corporate_action_adjusted(
            AdjustmentAnchor::FirstReturnedObservation,
            split_and_dividend_causes,
        ),
        PriceBasis::ContractRollAdjusted {
            anchor: AdjustmentAnchor::Date(chrono::NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()),
            method: AdjustmentMethod::Multiplicative,
        },
    );

    let json = serde_json::to_string(&per_field).unwrap();
    let roundtrip: OhlcPriceBasis = serde_json::from_str(&json).unwrap();
    assert_eq!(per_field, roundtrip);
}

#[test]
fn price_basis_rejects_unknown_semantic_fields() {
    let price_basis = serde_json::json!({
        "kind": "provider_adjusted",
        "anchor": {
            "kind": "provider_latest_basis"
        },
        "adjustment_factor": "0.95",
    });
    let err = serde_json::from_value::<PriceBasis>(price_basis).unwrap_err();
    assert!(err.to_string().contains("unknown field"));

    let ohlc_basis = serde_json::json!({
        "kind": "uniform",
        "basis": {
            "kind": "raw"
        },
        "provider_basis": "close",
    });
    let err = serde_json::from_value::<OhlcPriceBasis>(ohlc_basis).unwrap_err();
    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn responses_smoke() {
    let candles = vec![Candle {
        ts: chrono::DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        currency: usd(),
        ohlc: ohlc("100.0", "110.0", "95.0", "105.0"),
        close_unadj: None,
        volume: Some(quantity("1000000")),

        provider: (),
    }];

    let meta = HistoryMeta {
        timezone: Some("America/New_York".parse::<Tz>().unwrap()),
        utc_offset_seconds: Some(-5 * 3600),
    };

    let response = HistoryResponse {
        candles,
        actions: vec![],
        price_basis: OhlcPriceBasis::raw(),
        meta: Some(meta),
        provider: (),
    };

    assert_eq!(response.candles.len(), 1);
}

#[test]
fn history_response_chronological_order_validation_allows_duplicate_timestamps() {
    let response = |candles| HistoryResponse {
        candles,
        actions: vec![],
        price_basis: OhlcPriceBasis::raw(),
        meta: None,
        provider: (),
    };

    assert!(response(vec![]).is_chronologically_ordered());
    assert!(response(vec![candle_at(1), candle_at(1), candle_at(2)]).is_chronologically_ordered());

    let out_of_order = response(vec![candle_at(2), candle_at(1)]);
    assert!(!out_of_order.is_chronologically_ordered());
    assert_eq!(
        out_of_order.validate(),
        Err(HistoryValidationError::CandlesNotChronological {
            previous_index: 0,
            previous_ts_millis: 2_000,
            current_index: 1,
            current_ts_millis: 1_000,
        })
    );

    let chronological = out_of_order.into_chronological();
    assert!(chronological.is_chronologically_ordered());
    assert_eq!(
        chronological
            .candles
            .iter()
            .map(|candle| candle.ts.timestamp())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
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
            currency: usd(),
            ohlc: ohlc("100.0", "110.0", "95.0", "105.0"),
            close_unadj: None,
            volume: Some(quantity("1000000")),
            provider: (),
        }],
        actions: vec![],
        price_basis: OhlcPriceBasis::raw(),
        provider: (),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: HistoryResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(response, deserialized);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ResponseMeta {
    request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct CandleMeta {
    sequence: u64,
}

#[test]
fn history_response_metadata_layers_can_use_different_types() {
    let response: GenericHistoryResponse<ResponseMeta, CandleMeta> = GenericHistoryResponse {
        meta: None,
        candles: vec![GenericCandle {
            ts: chrono::DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            currency: usd(),
            ohlc: ohlc("100.0", "110.0", "95.0", "105.0"),
            close_unadj: None,
            volume: Some(quantity("1000000")),
            provider: CandleMeta { sequence: 42 },
        }],
        actions: vec![],
        price_basis: OhlcPriceBasis::raw(),
        provider: ResponseMeta {
            request_id: "req-123".to_string(),
        },
    };

    let value = serde_json::to_value(&response).unwrap();
    assert_eq!(value["request_id"], serde_json::json!("req-123"));
    assert_eq!(value["candles"][0]["sequence"], serde_json::json!(42));

    let roundtrip: GenericHistoryResponse<ResponseMeta, CandleMeta> =
        serde_json::from_value(value).unwrap();
    assert_eq!(response, roundtrip);
}

#[test]
fn candle_update_serialization() {
    let instrument = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
    let candle = Candle {
        ts: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        currency: usd(),
        ohlc: ohlc("150.0", "155.0", "148.0", "152.0"),
        close_unadj: None,
        volume: Some(quantity("2500000")),

        provider: (),
    };
    let update = CandleUpdate {
        instrument,
        interval: Interval::I1m,
        candle,
        is_final: true,

        provider: (),
    };

    let json = serde_json::to_string(&update).unwrap();
    let roundtrip: CandleUpdate = serde_json::from_str(&json).unwrap();
    assert_eq!(update, roundtrip);
}
