use chrono::{DateTime, NaiveDate};
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Instrument};
use paft_market::market::OptionUpdate as MarketOptionUpdate;
use paft_market::{
    OptionChainRequest, OptionContract, OptionContractKey, OptionExpirationsRequest,
    OptionExpirationsResponse, OptionGreeks, OptionSide, OptionUpdate,
};
use paft_money::{Currency, IsoCurrency, Price};
use std::str::FromStr;

fn usd(amount: i64) -> Price {
    Price::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD))
}

fn dec(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap()
}

fn option_key() -> OptionContractKey {
    OptionContractKey::new(
        Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        OptionSide::Call,
        usd(150),
        NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
    )
}

#[test]
fn option_expirations_request_roundtrip() {
    let req = OptionExpirationsRequest {
        underlying: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let de: OptionExpirationsRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, de);
}

#[test]
fn option_chain_request_roundtrip() {
    let req = OptionChainRequest {
        underlying: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        expiration: NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let de: OptionChainRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, de);
}

#[test]
fn option_expirations_response_roundtrip() {
    let resp = OptionExpirationsResponse {
        dates: vec![
            NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
        ],
    };
    let json = serde_json::to_string(&resp).unwrap();
    let de: OptionExpirationsResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(resp, de);
}

#[test]
fn option_contract_in_the_money_distinguishes_unknown_from_false() {
    let unknown: OptionContract = OptionContract::new(option_key());
    assert_eq!(unknown.in_the_money, None);

    let mut value = serde_json::to_value(&unknown).unwrap();
    value
        .as_object_mut()
        .expect("option contract serializes as an object")
        .remove("in_the_money");

    let decoded_unknown: OptionContract = serde_json::from_value(value).unwrap();
    assert_eq!(decoded_unknown.in_the_money, None);

    let mut explicit_false = OptionContract::new(option_key());
    explicit_false.in_the_money = Some(false);

    let value = serde_json::to_value(&explicit_false).unwrap();
    assert_eq!(value.get("in_the_money"), Some(&serde_json::json!(false)));

    let decoded_false: OptionContract = serde_json::from_value(value).unwrap();
    assert_eq!(decoded_false.in_the_money, Some(false));
}

#[test]
fn option_greeks_decimal_serde_uses_canonical_strings() {
    let greeks = OptionGreeks {
        delta: Some(dec("0.5000")),
        gamma: Some(dec("0.0100")),
        ..OptionGreeks::default()
    };

    let value = serde_json::to_value(&greeks).unwrap();
    assert_eq!(value.get("delta"), Some(&serde_json::json!("0.5")));
    assert_eq!(value.get("gamma"), Some(&serde_json::json!("0.01")));

    let decoded: OptionGreeks = serde_json::from_value(serde_json::json!({
        "delta": "+0.5000"
    }))
    .unwrap();
    assert_eq!(decoded.delta, Some(dec("0.5000")));

    assert!(serde_json::from_value::<OptionGreeks>(serde_json::json!({ "delta": 0.5 })).is_err());
}

#[test]
fn option_update_ts_serde_uses_unix_milliseconds() {
    let update: MarketOptionUpdate = OptionUpdate::new(
        option_key(),
        DateTime::from_timestamp(1_640_995_200, 789_000_000).unwrap(),
    );

    let value = serde_json::to_value(&update).unwrap();
    assert_eq!(
        value.get("ts"),
        Some(&serde_json::json!(1_640_995_200_789_i64))
    );
    assert_eq!(value.get("side"), Some(&serde_json::json!("CALL")));
    assert!(value.get("underlying").is_some());

    let deserialized: OptionUpdate = serde_json::from_value(value).unwrap();
    assert_eq!(update, deserialized);
}
