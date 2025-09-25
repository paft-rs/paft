#![cfg(feature = "dataframe")]
use iso_currency::Currency as IsoCurrency;
use paft_money::{Currency, ExchangeRate, Money};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};
use rust_decimal::Decimal;

#[test]
fn money_dataframe_roundtrip() {
    let m = Money::new(Decimal::new(12345, 2), Currency::Iso(IsoCurrency::USD));
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
        Money::new(Decimal::new(100, 0), Currency::Iso(IsoCurrency::USD)),
        Money::new(Decimal::new(200, 0), Currency::Iso(IsoCurrency::EUR)),
    ];
    let df = v.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
}

#[test]
fn exchange_rate_dataframe_schema() {
    let _rate = ExchangeRate::new(
        Currency::Iso(IsoCurrency::USD),
        Currency::Iso(IsoCurrency::EUR),
        Decimal::new(85, 2),
    )
    .unwrap();
    let schema = ExchangeRate::schema().unwrap();
    assert_eq!(schema.len(), 3);
    let names: Vec<&str> = schema.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"from"));
    assert!(names.contains(&"to"));
    assert!(names.contains(&"rate"));
}
