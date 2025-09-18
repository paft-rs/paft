use paft_core::domain::{Currency, ExchangeRate, Money};
use rust_decimal::Decimal;
use std::str::FromStr;

#[cfg(feature = "panicking-money-ops")]
mod panicking_ops_tests {
    use super::*;

    #[test]
    fn test_same_currency_arithmetic() {
        let usd_100 = Money::new(Decimal::from(100), Currency::USD);
        let usd_50 = Money::new(Decimal::from(50), Currency::USD);

        // Addition should work
        let total = &usd_100 + &usd_50;
        assert_eq!(total.amount(), Decimal::from(150));
        assert_eq!(total.currency(), &Currency::USD);

        // Subtraction should work
        let diff = &usd_100 - &usd_50;
        assert_eq!(diff.amount(), Decimal::from(50));
        assert_eq!(diff.currency(), &Currency::USD);
    }

    #[test]
    #[should_panic(expected = "currency mismatch")]
    fn test_different_currency_addition_panics() {
        let usd_100 = Money::new(Decimal::from(100), Currency::USD);
        let eur_100 = Money::new(Decimal::from(100), Currency::EUR);

        // Addition should panic on currency mismatch
        let _ = &usd_100 + &eur_100;
    }

    #[test]
    fn test_money_scalar_operations() {
        let usd_100 = Money::new(Decimal::from(100), Currency::USD);

        // Multiplication
        let doubled = usd_100.mul(Decimal::from(2));
        assert_eq!(doubled.amount(), Decimal::from(200));
        assert_eq!(doubled.currency(), &Currency::USD);

        // Division
        let halved = &usd_100 / Decimal::from(2);
        assert_eq!(halved.amount(), Decimal::from(50));
        assert_eq!(halved.currency(), &Currency::USD);
    }

    #[test]
    fn test_money_reference_arithmetic_is_ergonomic() {
        let lhs = Money::new(Decimal::from(125), Currency::USD);
        let rhs = Money::new(Decimal::from(75), Currency::USD);

        // Addition and subtraction should work on references without cloning
        let sum = &lhs + &rhs;
        assert_eq!(sum.amount(), Decimal::from(200));
        assert_eq!(sum.currency(), &Currency::USD);

        let diff = &lhs - &rhs;
        assert_eq!(diff.amount(), Decimal::from(50));
        assert_eq!(diff.currency(), &Currency::USD);

        // The original values should remain accessible afterwards (not moved)
        assert_eq!(lhs.amount(), Decimal::from(125));
        assert_eq!(rhs.amount(), Decimal::from(75));
    }

    #[test]
    fn test_money_owned_arithmetic_is_ergonomic() {
        let a = Money::new(Decimal::from(125), Currency::USD);
        let b = Money::new(Decimal::from(75), Currency::USD);

        let sum = a + b;
        assert_eq!(sum.amount(), Decimal::from(200));
        assert_eq!(sum.currency(), &Currency::USD);

        let c = Money::new(Decimal::from(125), Currency::USD);
        let d = Money::new(Decimal::from(75), Currency::USD);

        let diff = c - d;
        assert_eq!(diff.amount(), Decimal::from(50));
        assert_eq!(diff.currency(), &Currency::USD);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_division_by_zero_panics() {
        let usd_100 = Money::new(Decimal::from(100), Currency::USD);
        let _ = &usd_100 / Decimal::from(0);
    }
}

#[cfg(not(feature = "panicking-money-ops"))]
mod non_panicking_default_tests {
    use super::*;

    #[test]
    fn test_non_panicking_division_uses_try_div() {
        let usd_100 = Money::new(Decimal::from(100), Currency::USD);
        assert!(usd_100.try_div(Decimal::ZERO).is_err());
        let ok = usd_100.try_div(Decimal::from(2)).unwrap();
        assert_eq!(ok.amount(), Decimal::from(50));
        assert_eq!(ok.currency(), &Currency::USD);
    }

    #[test]
    fn test_try_add_try_sub_work_without_ops() {
        let usd_100 = Money::new(Decimal::from(100), Currency::USD);
        let usd_50 = Money::new(Decimal::from(50), Currency::USD);
        assert_eq!(
            usd_100.try_add(&usd_50).unwrap().amount(),
            Decimal::from(150)
        );
        assert_eq!(
            usd_100.try_sub(&usd_50).unwrap().amount(),
            Decimal::from(50)
        );
    }
}

#[test]
fn test_as_minor_units_basic() {
    let usd_123_45 = Money::new(Decimal::from_str("123.45").unwrap(), Currency::USD);
    assert_eq!(usd_123_45.as_minor_units(), Some(12345i128));
}

#[test]
fn test_exchange_rate_creation() {
    // Valid exchange rate should work
    let rate = ExchangeRate::new(Currency::USD, Currency::EUR, Decimal::new(85, 2)).unwrap();
    assert_eq!(rate.from(), &Currency::USD);
    assert_eq!(rate.to(), &Currency::EUR);
    assert_eq!(rate.rate(), Decimal::new(85, 2));
}

#[test]
fn test_try_convert_respects_target_precision() {
    let jpy = Money::new(Decimal::from(1000), Currency::JPY);
    let rate = ExchangeRate::new(Currency::JPY, Currency::USD, Decimal::new(89, 4)).unwrap();

    let usd = jpy.try_convert(&rate).unwrap();

    assert_eq!(usd.currency(), &Currency::USD);
    assert_eq!(usd.amount(), Decimal::new(890, 2));
    assert_eq!(usd.as_minor_units(), Some(890i128));

    let eth_ten = Money::new(Decimal::from(10), Currency::ETH);
    assert_eq!(
        eth_ten.as_minor_units(),
        Some(10_000_000_000_000_000_000i128)
    );
}

#[test]
fn test_from_minor_units_large_precision() {
    let wei: i128 = 1_234_567_890_123_456_789;
    let eth = Money::from_minor_units(wei, Currency::ETH);

    assert_eq!(eth.currency(), &Currency::ETH);
    assert_eq!(eth.as_minor_units(), Some(wei));
    assert_eq!(
        eth.amount(),
        Decimal::from_str("1.234567890123456789").unwrap()
    );
}
