use paft_decimal::{Decimal, parse_decimal};
use paft_money::{Currency, IsoCurrency, MonetaryAmount, Money, MoneyError, Price, QuantityAmount};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

#[test]
fn prices_and_amounts_reject_lossy_arithmetic() {
    let max_price = Price::new(Decimal::MAX, usd());
    let max_amount = MonetaryAmount::new(Decimal::MAX, usd());
    let tiny = parse_decimal("0.0000000000000000000000000001").unwrap();
    let tiny_price = Price::new(tiny, usd());
    let tiny_amount = MonetaryAmount::new(tiny, usd());

    assert_eq!(
        max_price.try_add(&tiny_price),
        Err(MoneyError::NotRepresentable)
    );
    assert_eq!(
        max_price.try_sub(&tiny_price),
        Err(MoneyError::NotRepresentable)
    );
    assert_eq!(
        max_amount.try_add(&tiny_amount),
        Err(MoneyError::NotRepresentable)
    );
    assert_eq!(
        max_amount.try_sub(&tiny_amount),
        Err(MoneyError::NotRepresentable)
    );
    assert_eq!(
        max_price.try_mul(&Decimal::TWO),
        Err(MoneyError::NotRepresentable)
    );
    assert_eq!(
        max_amount.try_mul(&Decimal::TWO),
        Err(MoneyError::NotRepresentable)
    );
    let tenth = parse_decimal("0.1").unwrap();
    assert_eq!(
        tiny_price.try_mul(&tenth),
        Err(MoneyError::NotRepresentable)
    );
    assert_eq!(
        tiny_amount.try_mul(&tenth),
        Err(MoneyError::NotRepresentable)
    );
    assert_eq!(
        tiny_price.try_total_decimal(&tenth),
        Err(MoneyError::NotRepresentable)
    );
    assert_eq!(
        tiny_price.try_total(&QuantityAmount::from_decimal(tenth).unwrap()),
        Err(MoneyError::NotRepresentable)
    );
}

#[test]
fn exact_division_is_distinct_from_settlement_division() {
    let price = Price::new(Decimal::ONE, usd());
    let amount = MonetaryAmount::new(Decimal::ONE, usd());
    let three = Decimal::from(3);
    assert_eq!(price.try_div(&three), Err(MoneyError::NotRepresentable));
    assert_eq!(amount.try_div(&three), Err(MoneyError::NotRepresentable));
    assert_eq!(
        price.try_div(&Decimal::ZERO),
        Err(MoneyError::DivisionByZero)
    );
    assert_eq!(
        amount.try_div(&Decimal::ZERO),
        Err(MoneyError::DivisionByZero)
    );
    assert_eq!(
        price.try_div(&Decimal::from(8)).unwrap().amount(),
        parse_decimal("0.125").unwrap()
    );
    assert_eq!(
        amount.try_div(&Decimal::from(8)).unwrap().amount(),
        parse_decimal("0.125").unwrap()
    );

    let money = Money::new_exact(Decimal::ONE, usd()).unwrap();
    assert_eq!(
        money.try_div(&three).unwrap().amount(),
        parse_decimal("0.33").unwrap()
    );
}

#[test]
fn exact_amount_results_can_reduce_precision_without_losing_value() {
    let lhs = Decimal::from_i128_with_scale(2_i128.pow(95), 28);
    let rhs = Decimal::from_i128_with_scale(5_i128.pow(28), 0);
    let expected = Decimal::from_i128_with_scale(2_i128.pow(67), 0);
    let price = Price::new(lhs, usd());
    let amount = MonetaryAmount::new(lhs, usd());
    assert_eq!(price.try_mul(&rhs).unwrap().amount(), expected);
    assert_eq!(amount.try_mul(&rhs).unwrap().amount(), expected);
    assert_eq!(price.try_total_decimal(&-rhs).unwrap().amount(), -expected);
    assert_eq!(
        price
            .try_total(&QuantityAmount::from_decimal(rhs).unwrap())
            .unwrap()
            .amount(),
        expected
    );
}

#[test]
fn currency_validation_precedes_exact_arithmetic() {
    let eur = Currency::Iso(IsoCurrency::EUR);
    assert!(matches!(
        Price::new(Decimal::MAX, usd()).try_add(&Price::new(Decimal::ONE, eur.clone())),
        Err(MoneyError::CurrencyMismatch { .. })
    ));
    assert!(matches!(
        MonetaryAmount::new(Decimal::MAX, usd()).try_sub(&MonetaryAmount::new(Decimal::MIN, eur)),
        Err(MoneyError::CurrencyMismatch { .. })
    ));
}
