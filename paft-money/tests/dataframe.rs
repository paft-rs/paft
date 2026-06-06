#![cfg(feature = "dataframe")]
use iso_currency::Currency as IsoCurrency;
use paft_decimal::Decimal;
use paft_money::{
    Currency, ExchangeRate, MonetaryAmount, Money, Price, PriceAmount, QuantityAmount,
};
use paft_utils::dataframe::ToDataFrame;

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
