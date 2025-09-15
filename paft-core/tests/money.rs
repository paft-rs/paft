use paft_core::domain::{Currency, ExchangeRate, Money, MoneyError};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_same_currency_arithmetic() {
    let usd_100 = Money::new(Decimal::from(100), Currency::USD);
    let usd_50 = Money::new(Decimal::from(50), Currency::USD);

    // Addition should work
    let total = usd_100.add(&usd_50).unwrap();
    assert_eq!(total.amount(), Decimal::from(150));
    assert_eq!(total.currency(), &Currency::USD);

    // Subtraction should work
    let diff = usd_100.sub(&usd_50).unwrap();
    assert_eq!(diff.amount(), Decimal::from(50));
    assert_eq!(diff.currency(), &Currency::USD);
}

#[test]
fn test_different_currency_addition_returns_error() {
    let usd_100 = Money::new(Decimal::from(100), Currency::USD);
    let eur_100 = Money::new(Decimal::from(100), Currency::EUR);

    // Addition should return error
    let result = usd_100.add(&eur_100);
    assert!(result.is_err());
    match result {
        Err(MoneyError::CurrencyMismatch { expected, found }) => {
            assert_eq!(expected, Currency::USD);
            assert_eq!(found, Currency::EUR);
        }
        _ => panic!("Expected CurrencyMismatch error"),
    }
}

#[test]
fn test_money_scalar_operations() {
    let usd_100 = Money::new(Decimal::from(100), Currency::USD);

    // Multiplication
    let doubled = usd_100.mul(Decimal::from(2));
    assert_eq!(doubled.amount(), Decimal::from(200));
    assert_eq!(doubled.currency(), &Currency::USD);

    // Division
    let halved = usd_100.div(Decimal::from(2)).unwrap();
    assert_eq!(halved.amount(), Decimal::from(50));
    assert_eq!(halved.currency(), &Currency::USD);
}

#[test]
fn test_as_minor_units_basic() {
    let usd_123_45 = Money::new(Decimal::from_str("123.45").unwrap(), Currency::USD);
    assert_eq!(usd_123_45.as_minor_units(), Some(12345));
}

#[test]
fn test_exchange_rate_creation() {
    // Valid exchange rate should work
    let rate = ExchangeRate::new(Currency::USD, Currency::EUR, Decimal::new(85, 2)).unwrap();
    assert_eq!(rate.from(), &Currency::USD);
    assert_eq!(rate.to(), &Currency::EUR);
    assert_eq!(rate.rate(), Decimal::new(85, 2));
}
