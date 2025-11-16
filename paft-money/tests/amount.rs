use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use iso_currency::Currency as IsoCurrency;
use serde_json::Value;

use paft_money::{Currency, Decimal, Money, MoneyAmount, MoneyError, RoundingStrategy, decimal};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

const fn eur() -> Currency {
    Currency::Iso(IsoCurrency::EUR)
}

fn parse_decimal(input: &str) -> Decimal {
    decimal::parse_decimal(input).expect("valid decimal")
}

#[test]
fn decimal_facade_behaviour() {
    let parsed = decimal::parse_decimal("  +123.450 ").unwrap();
    assert_eq!(parsed, parse_decimal("123.45"));

    assert_eq!(decimal::parse_decimal("1e2"), None);
    assert_eq!(decimal::parse_decimal("   "), None);

    let from_units = decimal::from_minor_units(12345, 3);
    assert_eq!(from_units, parse_decimal("12.345"));

    assert_eq!(decimal::zero(), parse_decimal("0"));
    assert_eq!(decimal::one(), parse_decimal("1"));
}

#[test]
fn money_amount_construction() {
    let decimal = parse_decimal("42.5");
    let from_decimal = MoneyAmount::new(decimal.clone());
    assert_eq!(from_decimal.amount(), decimal);
    assert!(from_decimal.currency_hint().is_none());

    let from_str = MoneyAmount::from_str("42.5").unwrap();
    assert_eq!(from_str, from_decimal);

    assert_eq!(
        MoneyAmount::from_str("bad").unwrap_err(),
        MoneyError::InvalidDecimal
    );

    let from_units = MoneyAmount::from_minor_units(12345, 3).unwrap();
    assert_eq!(from_units.amount(), parse_decimal("12.345"));
    assert!(from_units.currency_hint().is_none());

    assert_eq!(MoneyAmount::zero().amount(), decimal::zero());
}

#[cfg(not(feature = "bigdecimal"))]
#[test]
fn money_amount_from_minor_units_invalid_scale() {
    let err = MoneyAmount::from_minor_units(1, u32::from(paft_money::MAX_DECIMAL_PRECISION) + 1)
        .unwrap_err();
    assert_eq!(err, MoneyError::ConversionError);
}

#[test]
fn money_amount_currency_hint_behaviour() {
    let amount = MoneyAmount::from_str("10").unwrap();
    assert!(amount.currency_hint().is_none());

    let usd = usd();
    let hinted = amount.with_currency_hint(usd.clone());
    assert_eq!(hinted.currency_hint(), Some(&usd));
    assert_eq!(hinted.amount(), amount.amount());
    assert!(
        amount.currency_hint().is_none(),
        "original should remain unchanged"
    );

    let money = Money::from_canonical_str("10", usd).unwrap();
    let from_money: MoneyAmount = money.clone().into();
    assert_eq!(from_money.amount(), money.amount());
    assert_eq!(from_money.currency_hint(), Some(money.currency()));
}

#[test]
fn money_amount_equality_and_hash() {
    let base = MoneyAmount::from_str("123.45").unwrap();
    let usd_amount = base.with_currency_hint(usd());
    let eur_amount = base.with_currency_hint(eur());

    assert_eq!(usd_amount, eur_amount);

    let mut hasher1 = DefaultHasher::new();
    usd_amount.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    eur_amount.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);

    let different = MoneyAmount::from_str("999.99").unwrap();
    assert_ne!(usd_amount, different);

    let mut hasher3 = DefaultHasher::new();
    different.hash(&mut hasher3);
    assert_ne!(hash1, hasher3.finish());
}

#[test]
fn money_amount_arithmetic_and_hints() {
    let base = MoneyAmount::from_str("10.0").unwrap();
    let usd = usd();

    let usd_amount = base.with_currency_hint(usd.clone());
    let eur_amount = base.with_currency_hint(eur());
    let plain = MoneyAmount::from_str("5.0").unwrap();

    let sum = usd_amount.add(&plain);
    assert_eq!(sum.amount(), parse_decimal("15.0"));
    assert_eq!(sum.currency_hint(), Some(&usd));

    let sum_same_hint = usd_amount.add(&usd_amount);
    assert_eq!(sum_same_hint.currency_hint(), Some(&usd));

    let sum_conflict = usd_amount.add(&eur_amount);
    assert!(sum_conflict.currency_hint().is_none());

    let difference = sum.sub(&plain);
    assert_eq!(difference.amount(), parse_decimal("10.0"));
    assert_eq!(difference.currency_hint(), Some(&usd));

    let scaled = usd_amount.mul(parse_decimal("2"));
    assert_eq!(scaled.amount(), parse_decimal("20.0"));
    assert_eq!(scaled.currency_hint(), Some(&usd));

    let halved = scaled.div(parse_decimal("4")).unwrap();
    assert_eq!(halved.amount(), parse_decimal("5"));
    assert_eq!(halved.currency_hint(), Some(&usd));

    assert_eq!(
        usd_amount.div(decimal::zero()).unwrap_err(),
        MoneyError::DivisionByZero
    );
}

#[test]
fn money_amount_serde_roundtrip() {
    let usd = usd();
    let amount = MoneyAmount::from_str("12.340")
        .unwrap()
        .with_currency_hint(usd);
    let serialized_amount = serde_json::to_string(&amount).unwrap();

    let decimal = parse_decimal("12.340");
    let serialized_decimal = serde_json::to_string(&decimal).unwrap();
    assert_eq!(serialized_amount, serialized_decimal);

    let value_amount: Value = serde_json::to_value(&amount).unwrap();
    let value_decimal: Value = serde_json::to_value(&decimal).unwrap();
    assert_eq!(value_amount, value_decimal);

    let decoded: MoneyAmount = serde_json::from_str(&serialized_amount).unwrap();
    assert_eq!(decoded.amount(), decimal);
    assert!(decoded.currency_hint().is_none());
}

#[test]
fn money_amount_to_money_finalization() {
    let usd = usd();
    let source = MoneyAmount::from_str("12.345").unwrap();

    let money = source.to_money(usd.clone()).unwrap();
    assert_eq!(money.currency(), &usd);
    assert_eq!(money.amount(), parse_decimal("12.35"));

    let customized = source
        .to_money_with(usd.clone(), RoundingStrategy::MidpointAwayFromZero, Some(1))
        .unwrap();
    assert_eq!(customized.amount(), parse_decimal("12.3"));

    let precise = source
        .to_money_with(usd.clone(), RoundingStrategy::ToZero, None)
        .unwrap();
    assert_eq!(precise.amount(), parse_decimal("12.34"));

    assert_eq!(
        source
            .to_money_with(usd, RoundingStrategy::MidpointAwayFromZero, Some(3))
            .unwrap_err(),
        MoneyError::ConversionError
    );
}
