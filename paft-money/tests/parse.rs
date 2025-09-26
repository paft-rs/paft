use paft_money::{Currency, Decimal, Money};
use std::str::FromStr;

fn usd() -> Currency {
    Currency::try_from_str("USD").unwrap()
}

#[test]
fn parse_plain_strings() {
    let m = Money::from_str("00123.4500", usd()).unwrap();
    assert_eq!(m.amount(), Decimal::from_str("123.45").unwrap());

    let negative = Money::from_str("-42.1", usd()).unwrap();
    assert_eq!(negative.amount(), Decimal::from_str("-42.1").unwrap());

    let positive = Money::from_str("+7.5", usd()).unwrap();
    assert_eq!(positive.amount(), Decimal::from_str("7.5").unwrap());

    let trimmed = Money::from_str(" 1.23 ", usd()).unwrap();
    assert_eq!(trimmed.amount(), Decimal::from_str("1.23").unwrap());
}

#[test]
fn parse_rejects_scientific_notation() {
    assert!(Money::from_str("1e3", usd()).is_err());
    assert!(Money::from_str("-2E-2", usd()).is_err());
}

#[test]
fn parse_rejects_invalid_tokens() {
    assert!(Money::from_str("abc", usd()).is_err());
    assert!(Money::from_str("--1.0", usd()).is_err());
}

#[cfg(feature = "bigdecimal")]
#[test]
fn parse_high_precision_bigdecimal_mode() {
    let m = Money::from_str(
        "12345678901234567890.123456789012345678",
        Currency::ETH,
    )
    .unwrap();
    assert_eq!(
        m.amount(),
        Decimal::from_str("12345678901234567890.123456789012345678").unwrap()
    );
}

#[cfg(not(feature = "bigdecimal"))]
#[test]
fn parse_high_precision_rust_decimal_mode() {
    let m = Money::from_str(
        "12345678901234567890.123456789012345678",
        Currency::ETH,
    )
    .unwrap();
    eprintln!("m: {:?}", m);
    eprintln!("m.amount(): {:?}", m.amount());
    eprintln!("Decimal::from_str(\"12345678901234567890.123456789012345678\"): {:?}", Decimal::from_str("12345678901234567890.123456789012345678"));
    assert_eq!(
        m.amount(),
        Decimal::from_str("12345678901234567890.123456789012345678").unwrap()
    );
}
