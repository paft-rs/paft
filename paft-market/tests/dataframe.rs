#![cfg(feature = "dataframe")]
use chrono::DateTime;
use paft_core::dataframe::{ToDataFrame, ToDataFrameVec};
use paft_core::domain::{Currency, Exchange, MarketState, Money};
use paft_market::market::quote::{Quote, QuoteUpdate};
use rust_decimal::Decimal;

#[test]
fn quote_to_dataframe_smoke() {
    let q = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(Decimal::from(147), Currency::USD)),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };
    let df = q.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn vec_quote_to_dataframe_smoke() {
    let v = [Quote {
        symbol: "AAPL".to_string(),
        shortname: None,
        price: None,
        previous_close: None,
        exchange: None,
        market_state: None,
    }];
    let df = v.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn quote_update_to_dataframe_smoke() {
    let u = QuoteUpdate {
        symbol: "AAPL".to_string(),
        price: None,
        previous_close: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    };
    let df = u.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}
