use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use iso_currency::Currency as IsoCurrency;
use serde_json::{Value, json};

use paft_decimal::{self as decimal, Decimal, RoundingStrategy};
use paft_money::{Currency, MonetaryAmount, Money, MoneyError, Price};

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
fn monetary_amount_construction_requires_currency() {
    let usd = usd();
    let from_decimal = MonetaryAmount::new(parse_decimal("42.5"), usd.clone());
    assert_eq!(from_decimal.amount(), parse_decimal("42.5"));
    assert_eq!(from_decimal.currency(), &usd);

    let from_str = MonetaryAmount::from_canonical_str("42.5", usd.clone()).unwrap();
    assert_eq!(from_str, from_decimal);

    assert_eq!(
        MonetaryAmount::from_canonical_str("bad", usd.clone()).unwrap_err(),
        MoneyError::InvalidDecimal
    );

    let from_units = MonetaryAmount::from_scaled_units(12345, 3, usd.clone()).unwrap();
    assert_eq!(from_units.amount(), parse_decimal("12.345"));
    assert_eq!(from_units.currency(), &usd);

    assert_eq!(MonetaryAmount::zero(usd).amount(), decimal::zero());
}

#[cfg(not(feature = "bigdecimal"))]
#[test]
fn monetary_amount_from_scaled_units_invalid_scale() {
    let err = MonetaryAmount::from_scaled_units(
        1,
        u32::from(paft_money::MAX_DECIMAL_PRECISION) + 1,
        usd(),
    )
    .unwrap_err();
    assert_eq!(err, MoneyError::ConversionError);
}

#[test]
fn monetary_amount_from_money_preserves_currency() {
    let usd = usd();
    let money = Money::from_canonical_str("10", usd.clone()).unwrap();
    let from_money: MonetaryAmount = money.clone().into();
    assert_eq!(from_money.amount(), money.amount());
    assert_eq!(from_money.currency(), &usd);
}

#[test]
fn monetary_amount_equality_and_hash_include_currency() {
    let usd_amount = MonetaryAmount::from_canonical_str("123.45", usd()).unwrap();
    let eur_amount = MonetaryAmount::from_canonical_str("123.45", eur()).unwrap();
    let usd_amount_again = MonetaryAmount::from_canonical_str("123.450", usd()).unwrap();

    assert_ne!(usd_amount, eur_amount);
    assert_eq!(usd_amount, usd_amount_again);

    let mut hasher_usd_a = DefaultHasher::new();
    usd_amount.hash(&mut hasher_usd_a);
    let hash_usd_a = hasher_usd_a.finish();

    let mut hasher_usd_b = DefaultHasher::new();
    usd_amount_again.hash(&mut hasher_usd_b);
    assert_eq!(hash_usd_a, hasher_usd_b.finish());

    let different = MonetaryAmount::from_canonical_str("999.99", usd()).unwrap();
    let mut hasher_diff = DefaultHasher::new();
    different.hash(&mut hasher_diff);
    assert_ne!(hash_usd_a, hasher_diff.finish());
}

#[test]
fn monetary_amount_arithmetic_checks_currency() {
    let usd_amount = MonetaryAmount::from_canonical_str("10.0", usd()).unwrap();
    let usd_other = MonetaryAmount::from_canonical_str("5.0", usd()).unwrap();
    let eur_amount = MonetaryAmount::from_canonical_str("5.0", eur()).unwrap();

    let sum = usd_amount.try_add(&usd_other).unwrap();
    assert_eq!(sum.amount(), parse_decimal("15.0"));
    assert_eq!(sum.currency(), &usd());

    let difference = sum.try_sub(&usd_other).unwrap();
    assert_eq!(difference.amount(), parse_decimal("10.0"));
    assert_eq!(difference.currency(), &usd());

    assert_eq!(
        usd_amount.try_add(&eur_amount).unwrap_err(),
        MoneyError::CurrencyMismatch {
            expected: usd(),
            found: eur(),
        }
    );

    let scaled = usd_amount.try_mul(parse_decimal("2")).unwrap();
    assert_eq!(scaled.amount(), parse_decimal("20.0"));
    assert_eq!(scaled.currency(), &usd());

    let halved = scaled.try_div(parse_decimal("4")).unwrap();
    assert_eq!(halved.amount(), parse_decimal("5"));
    assert_eq!(halved.currency(), &usd());

    assert_eq!(
        usd_amount.try_div(decimal::zero()).unwrap_err(),
        MoneyError::DivisionByZero
    );
}

#[test]
fn monetary_amount_serde_roundtrip() {
    let amount = MonetaryAmount::from_canonical_str("12.340", usd()).unwrap();
    let serialized_amount = serde_json::to_string(&amount).unwrap();

    let value_amount: Value = serde_json::to_value(&amount).unwrap();
    assert_eq!(
        value_amount,
        json!({
            "amount": "12.340",
            "currency": "USD",
        })
    );

    let decoded: MonetaryAmount = serde_json::from_str(&serialized_amount).unwrap();
    assert_eq!(decoded, amount);
    assert_eq!(decoded.currency(), &usd());
}

#[test]
fn monetary_amount_to_money_finalization_is_explicit() {
    let source = MonetaryAmount::from_canonical_str("12.345", usd()).unwrap();

    let money = source.to_money().unwrap();
    assert_eq!(money.currency(), &usd());
    assert_eq!(money.amount(), parse_decimal("12.35"));

    let customized = source
        .to_money_with(RoundingStrategy::MidpointAwayFromZero, Some(1))
        .unwrap();
    assert_eq!(customized.amount(), parse_decimal("12.3"));

    let precise = source
        .to_money_with(RoundingStrategy::ToZero, None)
        .unwrap();
    assert_eq!(precise.amount(), parse_decimal("12.34"));

    assert_eq!(
        source
            .to_money_with(RoundingStrategy::MidpointAwayFromZero, Some(3))
            .unwrap_err(),
        MoneyError::ConversionError
    );
}

#[test]
fn price_preserves_market_precision() {
    let price = Price::from_canonical_str("1.3578", usd()).unwrap();
    assert_eq!(price.amount(), parse_decimal("1.3578"));
    assert_eq!(price.format(), "1.3578 USD");

    assert_eq!(
        Money::from_canonical_str("1.3578", usd()).unwrap_err(),
        MoneyError::PrecisionExceeded {
            currency_code: "USD".to_string(),
            max_scale: 2,
            actual_scale: 4,
        }
    );

    let rounded_money = price.to_money().unwrap();
    assert_eq!(rounded_money.amount(), parse_decimal("1.36"));
}

#[test]
fn price_serde_accepts_over_minor_unit_precision() {
    let decoded: Price = serde_json::from_value(json!({
        "amount": "1.3578",
        "currency": "USD",
    }))
    .unwrap();

    assert_eq!(decoded.amount(), parse_decimal("1.3578"));
    assert_eq!(decoded.currency(), &usd());
}

#[test]
fn price_arithmetic_checks_currency() {
    let lhs = Price::from_canonical_str("10.125", usd()).unwrap();
    let rhs = Price::from_canonical_str("0.875", usd()).unwrap();
    let eur_rhs = Price::from_canonical_str("0.875", eur()).unwrap();

    assert_eq!(lhs.try_add(&rhs).unwrap().amount(), parse_decimal("11.000"));
    assert_eq!(lhs.try_sub(&rhs).unwrap().amount(), parse_decimal("9.250"));
    assert_eq!(
        lhs.try_add(&eur_rhs).unwrap_err(),
        MoneyError::CurrencyMismatch {
            expected: usd(),
            found: eur(),
        }
    );
    assert_eq!(
        lhs.try_mul(parse_decimal("2")).unwrap().amount(),
        parse_decimal("20.250")
    );
    assert_eq!(
        lhs.try_div(parse_decimal("2")).unwrap().amount(),
        parse_decimal("5.0625")
    );
}

#[test]
fn price_total_returns_monetary_amount() {
    let price = Price::from_canonical_str("182.345678", usd()).unwrap();
    let total = price.try_total(parse_decimal("4.91")).unwrap();

    assert_eq!(total.currency(), &usd());
    assert_eq!(total.amount(), parse_decimal("895.31727898"));
}
