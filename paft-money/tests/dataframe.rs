#![cfg(feature = "dataframe")]
use iso_currency::Currency as IsoCurrency;
use paft_decimal::Decimal;
use paft_money::{
    Currency, ExchangeRate, MonetaryAmount, Money, Price, PriceAmount, QuantityAmount,
};
use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
use polars::prelude::{AnyValue, DataFrame, DataType};

fn usd(amount: i64) -> Money {
    Money::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD)).unwrap()
}

#[test]
fn exchange_rate_to_dataframe() {
    let rate = ExchangeRate::new(
        Currency::Iso(IsoCurrency::USD),
        Currency::Iso(IsoCurrency::EUR),
        Decimal::from(9) / Decimal::from(10),
    )
    .unwrap();

    let df = rate.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn money_to_dataframe() {
    let money = usd(123);

    let df = money.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    assert_eq!(
        Money::schema().unwrap(),
        vec![
            ("amount".into(), DataType::Decimal(38, 10)),
            ("currency".into(), DataType::String),
            ("minor_units".into(), DataType::UInt8),
        ]
    );
    assert_eq!(
        df.column("amount").unwrap().get(0).unwrap(),
        AnyValue::Decimal(1_230_000_000_000, 38, 10)
    );
    assert_eq!(
        df.column("currency").unwrap().str().unwrap().get(0),
        Some("USD")
    );
    assert_eq!(
        df.column("minor_units").unwrap().u8().unwrap().get(0),
        Some(2)
    );
}

#[derive(df_derive_macros::ToDataFrame)]
struct SettlementRow {
    value: Option<Money>,
}

fn assert_captured_scales(df: &DataFrame, prefix: &str) {
    let scales = df
        .column(&format!("{prefix}minor_units"))
        .unwrap()
        .u8()
        .unwrap();
    assert_eq!(df.height(), 2);
    assert_eq!([scales.get(0), scales.get(1)], [Some(2), Some(3)]);
    for row in 0..2 {
        assert_eq!(
            df.column(&format!("{prefix}amount"))
                .unwrap()
                .get(row)
                .unwrap(),
            AnyValue::Decimal(10_000_000_000, 38, 10)
        );
        assert_eq!(
            df.column(&format!("{prefix}currency"))
                .unwrap()
                .str()
                .unwrap()
                .get(row),
            Some("AUDITCOIN")
        );
    }
}

#[test]
fn money_vectors_and_nested_rows_preserve_distinct_settlement_scales() {
    // No registry entry: serde restores each value's captured specification.
    let values: Vec<Money> = serde_json::from_str(
        r#"[
        {"amount":"1","currency":"AUDITCOIN","minor_units":2},
        {"amount":"1","currency":"AUDITCOIN","minor_units":3}
    ]"#,
    )
    .unwrap();
    assert_ne!(values[0], values[1]);
    assert_captured_scales(&values.to_dataframe().unwrap(), "");
    assert_captured_scales(
        &Money::columnar_from_refs(&[&values[0], &values[1]]).unwrap(),
        "",
    );
    let rows: Vec<_> = values
        .into_iter()
        .map(|value| SettlementRow { value: Some(value) })
        .collect();
    assert_captured_scales(&rows.to_dataframe().unwrap(), "value.");
    assert_eq!(
        rows[1]
            .to_dataframe()
            .unwrap()
            .column("value.minor_units")
            .unwrap()
            .u8()
            .unwrap()
            .get(0),
        Some(3)
    );
    let absent = SettlementRow { value: None }.to_dataframe().unwrap();
    assert_eq!(absent.column("value.minor_units").unwrap().null_count(), 1);
    assert_eq!(
        Money::empty_dataframe()
            .unwrap()
            .schema()
            .get("minor_units"),
        Some(&DataType::UInt8)
    );
}

#[test]
fn price_to_dataframe() {
    let price = Price::new(Decimal::from(123), Currency::Iso(IsoCurrency::USD));

    let df = price.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn price_amount_to_dataframe() {
    let amount = PriceAmount::new(Decimal::from(123));

    let df = amount.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn quantity_amount_to_dataframe() {
    let amount = QuantityAmount::from_decimal(Decimal::from(123)).unwrap();

    let df = amount.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn monetary_amount_to_dataframe() {
    let amount = MonetaryAmount::new(Decimal::from(123), Currency::Iso(IsoCurrency::USD));

    let df = amount.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}
