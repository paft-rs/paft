use chrono::DateTime;
use chrono_tz::Tz;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Instrument};
use paft_market::market::action::Action;
use paft_market::{
    AdjustmentAnchor, AdjustmentMethod, Candle, CandleUpdate, CorporateActionAdjustmentCause,
    CorporateActionAdjustmentCauses, HistoryMeta, HistoryResponse, Interval, Ohlc, OhlcPriceBasis,
    PriceBasis,
};
use paft_money::{Currency, IsoCurrency, Price, PriceAmount};
use std::num::NonZeroU32;
use std::str::FromStr;

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: &str) -> PriceAmount {
    PriceAmount::new(Decimal::from_str(value).unwrap())
}

fn ohlc(open: &str, high: &str, low: &str, close: &str) -> Ohlc {
    Ohlc::new(amount(open), amount(high), amount(low), amount(close))
}

#[test]
fn candle_serialization() {
    let candle = Candle {
        ts: DateTime::from_timestamp(1_640_995_200, 321_000_000).unwrap(),
        currency: usd(),
        ohlc: ohlc("100.0", "110.0", "95.0", "105.0"),
        close_unadj: None,
        volume: Some(1_000_000),

        provider: (),
    };

    let json = serde_json::to_string(&candle).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["ts"], serde_json::json!(1_640_995_200_321_i64));
    assert_eq!(value["currency"], serde_json::json!("USD"));
    assert_eq!(value["open"], serde_json::json!("100.0"));

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
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        amount: Price::new(Decimal::from_str("0.5").unwrap(), usd()),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn action_split_serialization() {
    let action = Action::Split {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        numerator: NonZeroU32::new(2).unwrap(),
        denominator: NonZeroU32::new(1).unwrap(),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn action_split_rejects_zero_ratios() {
    for (numerator, denominator) in [(0, 1), (2, 0)] {
        let value = serde_json::json!({
            "Split": {
                "ts": 1_640_995_200_000_i64,
                "numerator": numerator,
                "denominator": denominator,
            }
        });

        let err = serde_json::from_value::<Action>(value).unwrap_err();
        assert!(err.to_string().contains("nonzero"));
    }
}

#[test]
fn action_capital_gain_serialization() {
    let action = Action::CapitalGain {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        gain: Price::new(Decimal::from_str("1.0").unwrap(), usd()),
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
fn responses_smoke() {
    let candles = vec![Candle {
        ts: chrono::DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        currency: usd(),
        ohlc: ohlc("100.0", "110.0", "95.0", "105.0"),
        close_unadj: None,
        volume: Some(1_000_000),

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
            volume: Some(1_000_000),
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

#[test]
fn candle_update_serialization() {
    let instrument = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
    let candle = Candle {
        ts: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        currency: usd(),
        ohlc: ohlc("150.0", "155.0", "148.0", "152.0"),
        close_unadj: None,
        volume: Some(2_500_000),

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
