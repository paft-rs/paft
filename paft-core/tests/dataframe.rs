#![cfg(feature = "dataframe")]
use paft_core::dataframe::{ToDataFrame, ToDataFrameVec};
use paft_core::domain::{Currency, ExchangeRate, Money};
use rust_decimal::Decimal;

#[test]
fn money_dataframe_roundtrip() {
    let m = Money::new(Decimal::new(12345, 2), Currency::USD);
    let df = m.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    assert!(df.get_column_names().iter().any(|c| c.as_str() == "amount"));
    assert!(
        df.get_column_names()
            .iter()
            .any(|c| c.as_str() == "currency")
    );
}

#[test]
fn vec_money_to_dataframe() {
    let v = [
        Money::new(Decimal::new(100, 0), Currency::USD),
        Money::new(Decimal::new(200, 0), Currency::EUR),
    ];
    let df = v.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
}

#[test]
fn exchange_rate_dataframe_schema() {
    let _rate = ExchangeRate::new(Currency::USD, Currency::EUR, Decimal::new(85, 2)).unwrap();
    let schema = ExchangeRate::schema().unwrap();
    assert_eq!(schema.len(), 3);
    let names: Vec<&str> = schema.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"from"));
    assert!(names.contains(&"to"));
    assert!(names.contains(&"rate"));
}
