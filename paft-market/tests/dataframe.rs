#![cfg(feature = "dataframe")]
use iso_currency::Currency as IsoCurrency;
use paft_domain::Exchange;
use paft_market::market::quote::Quote;
use paft_money::{Currency, Money};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};
use rust_decimal::Decimal;

#[test]
fn vec_quote_to_dataframe_smoke() {
    let quotes = [Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(Decimal::from(147), Currency::Iso(IsoCurrency::USD)).unwrap(),
        ),
        exchange: Some(Exchange::NASDAQ),
        market_state: None,
    }];

    let df = quotes.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "symbol"));
}

#[test]
fn quote_to_dataframe_smoke() {
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(Decimal::from(147), Currency::Iso(IsoCurrency::USD)).unwrap(),
        ),
        exchange: Some(Exchange::NASDAQ),
        market_state: None,
    };

    let df = quote.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "symbol"));
    assert_eq!(df.height(), 1);
}

#[test]
fn quote_update_to_dataframe_smoke() {
    use paft_market::market::quote::QuoteUpdate;
    let update = QuoteUpdate {
        symbol: "AAPL".to_string(),
        price: Some(Money::new(Decimal::from(150), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(Decimal::from(147), Currency::Iso(IsoCurrency::USD)).unwrap(),
        ),
        ts: chrono::DateTime::from_timestamp(0, 0).unwrap(),
    };

    let df = update.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "symbol"));
}
