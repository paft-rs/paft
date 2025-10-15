use chrono::NaiveDate;
use paft_domain::Symbol;
use paft_market::requests::options::{OptionChainRequest, OptionExpirationsRequest};
use paft_market::responses::options::OptionExpirationsResponse;

#[test]
fn option_expirations_request_roundtrip() {
    let req = OptionExpirationsRequest {
        symbol: Symbol::new("AAPL").unwrap(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let de: OptionExpirationsRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, de);
}

#[test]
fn option_chain_request_roundtrip() {
    let req = OptionChainRequest {
        symbol: Symbol::new("AAPL").unwrap(),
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
