use chrono::{DateTime, NaiveDate};
use paft_domain::{AssetKind, Instrument};
use paft_market::market::options::OptionUpdate;
use paft_market::requests::options::{OptionChainRequest, OptionExpirationsRequest};
use paft_market::responses::options::OptionExpirationsResponse;

#[test]
fn option_expirations_request_roundtrip() {
    let req = OptionExpirationsRequest {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let de: OptionExpirationsRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, de);
}

#[test]
fn option_chain_request_roundtrip() {
    let req = OptionChainRequest {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
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
fn option_update_ts_serde_uses_unix_seconds() {
    let update = OptionUpdate::new(
        Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    );

    let value = serde_json::to_value(&update).unwrap();
    assert_eq!(value.get("ts"), Some(&serde_json::json!(1_640_995_200)));

    let deserialized: OptionUpdate = serde_json::from_value(value).unwrap();
    assert_eq!(update, deserialized);
}
