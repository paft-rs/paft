#![cfg(feature = "dataframe")]
use iso_currency::Currency as IsoCurrency;
use paft_money::{Currency, Decimal, ExchangeRate, Money};
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
