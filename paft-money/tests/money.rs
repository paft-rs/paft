use iso_currency::Currency as IsoCurrency;
use paft_money::{Currency, Decimal, ExchangeRate, Money, RoundingStrategy};
use std::str::FromStr;

#[cfg(feature = "dataframe")]
use paft_money::decimal;
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;
#[cfg(feature = "dataframe")]
use std::convert::TryFrom;

#[cfg(feature = "panicking-money-ops")]
mod panicking_ops_tests {
    use super::*;

    #[test]
    fn test_same_currency_arithmetic() {
        let usd_100 = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        let usd_50 = Money::new(Decimal::from(50), Currency::Iso(IsoCurrency::USD)).unwrap();

        // Addition should work
        let total = &usd_100 + &usd_50;
        assert_eq!(total.amount(), Decimal::from(150));
        assert_eq!(total.currency(), &Currency::Iso(IsoCurrency::USD));

        // Subtraction should work
        let diff = &usd_100 - &usd_50;
        assert_eq!(diff.amount(), Decimal::from(50));
        assert_eq!(diff.currency(), &Currency::Iso(IsoCurrency::USD));
    }

    #[test]
    #[should_panic(expected = "currency mismatch")]
    fn test_different_currency_addition_panics() {
        let usd_100 = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        let eur_100 = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::EUR)).unwrap();

        // Addition should panic on currency mismatch
        let _ = &usd_100 + &eur_100;
    }

    #[test]
    fn test_money_scalar_operations() {
        let usd_100 = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();

        // Multiplication
        let doubled = usd_100.try_mul(Decimal::from(2)).unwrap();
        assert_eq!(doubled.amount(), Decimal::from(200));
        assert_eq!(doubled.currency(), &Currency::Iso(IsoCurrency::USD));

        let try_tripled = usd_100.try_mul(Decimal::from(3)).unwrap();
        assert_eq!(try_tripled.amount(), Decimal::from(300));
        assert_eq!(try_tripled.currency(), &Currency::Iso(IsoCurrency::USD));

        // Division
        let halved = &usd_100 / Decimal::from(2);
        assert_eq!(halved.amount(), Decimal::from(50));
        assert_eq!(halved.currency(), &Currency::Iso(IsoCurrency::USD));
    }

    #[test]
    fn test_money_reference_arithmetic_is_ergonomic() {
        let lhs = Money::new(Decimal::from(125), Currency::Iso(IsoCurrency::USD)).unwrap();
        let rhs = Money::new(Decimal::from(75), Currency::Iso(IsoCurrency::USD)).unwrap();

        // Addition and subtraction should work on references without cloning
        let sum = &lhs + &rhs;
        assert_eq!(sum.amount(), Decimal::from(200));
        assert_eq!(sum.currency(), &Currency::Iso(IsoCurrency::USD));

        let diff = &lhs - &rhs;
        assert_eq!(diff.amount(), Decimal::from(50));
        assert_eq!(diff.currency(), &Currency::Iso(IsoCurrency::USD));

        // The original values should remain accessible afterwards (not moved)
        assert_eq!(lhs.amount(), Decimal::from(125));
        assert_eq!(rhs.amount(), Decimal::from(75));
    }

    #[test]
    fn test_money_owned_arithmetic_is_ergonomic() {
        let a = Money::new(Decimal::from(125), Currency::Iso(IsoCurrency::USD)).unwrap();
        let b = Money::new(Decimal::from(75), Currency::Iso(IsoCurrency::USD)).unwrap();

        let sum = a + b;
        assert_eq!(sum.amount(), Decimal::from(200));
        assert_eq!(sum.currency(), &Currency::Iso(IsoCurrency::USD));

        let c = Money::new(Decimal::from(125), Currency::Iso(IsoCurrency::USD)).unwrap();
        let d = Money::new(Decimal::from(75), Currency::Iso(IsoCurrency::USD)).unwrap();

        let diff = c - d;
        assert_eq!(diff.amount(), Decimal::from(50));
        assert_eq!(diff.currency(), &Currency::Iso(IsoCurrency::USD));
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_division_by_zero_panics() {
        let usd_100 = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        let _ = &usd_100 / Decimal::from(0);
    }
}

#[cfg(not(feature = "panicking-money-ops"))]
mod non_panicking_default_tests {
    use super::*;

    #[test]
    fn test_non_panicking_division_uses_try_div() {
        let usd_100 = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        assert!(usd_100.try_div(Decimal::from(0)).is_err());
        let ok = usd_100.try_div(Decimal::from(2)).unwrap();
        assert_eq!(ok.amount(), Decimal::from(50));
        assert_eq!(ok.currency(), &Currency::Iso(IsoCurrency::USD));
    }

    #[test]
    fn test_try_add_try_sub_work_without_ops() {
        let usd_100 = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        let usd_50 = Money::new(Decimal::from(50), Currency::Iso(IsoCurrency::USD)).unwrap();
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
    let usd_123_45 = Money::new(
        Decimal::from_str("123.45").unwrap(),
        Currency::Iso(IsoCurrency::USD),
    )
    .unwrap();
    assert_eq!(usd_123_45.as_minor_units().unwrap(), 12345i128);
}

#[test]
fn test_exchange_rate_creation() {
    // Valid exchange rate should work
    let rate = ExchangeRate::new(
        Currency::Iso(IsoCurrency::USD),
        Currency::Iso(IsoCurrency::EUR),
        Decimal::from_str("0.85").unwrap(),
    )
    .unwrap();
    assert_eq!(rate.from(), &Currency::Iso(IsoCurrency::USD));
    assert_eq!(rate.to(), &Currency::Iso(IsoCurrency::EUR));
    assert_eq!(rate.rate(), Decimal::from_str("0.85").unwrap());
}

#[test]
fn test_try_convert_respects_target_precision() {
    let jpy = Money::new(Decimal::from(1000), Currency::Iso(IsoCurrency::JPY)).unwrap();
    let rate = ExchangeRate::new(
        Currency::Iso(IsoCurrency::JPY),
        Currency::Iso(IsoCurrency::USD),
        Decimal::from_str("0.0089").unwrap(),
    )
    .unwrap();

    let usd = jpy.try_convert(&rate).unwrap();

    assert_eq!(usd.currency(), &Currency::Iso(IsoCurrency::USD));
    assert_eq!(usd.amount(), Decimal::from_str("8.90").unwrap());
    assert_eq!(usd.as_minor_units().unwrap(), 890i128);

    let eth_ten = Money::new(Decimal::from(10), Currency::ETH).unwrap();
    assert_eq!(
        eth_ten.as_minor_units().unwrap(),
        10_000_000_000_000_000_000i128
    );
}

#[test]
fn test_try_convert_with_custom_rounding_strategy() {
    let eur = Money::new(
        Decimal::from_str("1.00").unwrap(),
        Currency::Iso(IsoCurrency::EUR),
    )
    .unwrap();
    let rate = ExchangeRate::new(
        Currency::Iso(IsoCurrency::EUR),
        Currency::Iso(IsoCurrency::USD),
        Decimal::from_str("1.015").unwrap(),
    )
    .unwrap();

    let default = eur.try_convert(&rate).unwrap();
    assert_eq!(default.amount(), Decimal::from_str("1.02").unwrap());

    let toward_zero = eur
        .try_convert_with(&rate, RoundingStrategy::MidpointTowardZero)
        .unwrap();
    assert_eq!(toward_zero.amount(), Decimal::from_str("1.01").unwrap());
}

#[test]
fn test_from_minor_units_large_precision() {
    let wei: i128 = 1_234_567_890_123_456_789;
    let eth = Money::from_minor_units(wei, Currency::ETH).unwrap();

    assert_eq!(eth.currency(), &Currency::ETH);
    assert_eq!(eth.as_minor_units().unwrap(), wei);
    assert_eq!(
        eth.amount(),
        Decimal::from_str("1.234567890123456789").unwrap()
    );
}

#[test]
fn test_money_minor_units_boundary_precisions() {
    // 0 decimals (JPY)
    let jpy_minor = 42i128;
    let jpy = Money::from_minor_units(jpy_minor, Currency::Iso(IsoCurrency::JPY)).unwrap();
    assert_eq!(jpy.currency(), &Currency::Iso(IsoCurrency::JPY));
    assert_eq!(jpy.as_minor_units().unwrap(), jpy_minor);
    assert_eq!(jpy.amount(), Decimal::from(jpy_minor));

    // 8 decimals (BTC)
    let sats = 12_345_678i128;
    let btc = Money::from_minor_units(sats, Currency::BTC).unwrap();
    assert_eq!(btc.currency(), &Currency::BTC);
    assert_eq!(btc.as_minor_units().unwrap(), sats);
    assert_eq!(btc.amount(), Decimal::from_str("0.12345678").unwrap());

    // 12 decimals (XMR)
    let atomic_units = 1_234_567_890_123i128;
    let xmr = Money::from_minor_units(atomic_units, Currency::XMR).unwrap();
    assert_eq!(xmr.currency(), &Currency::XMR);
    assert_eq!(xmr.as_minor_units().unwrap(), atomic_units);
    assert_eq!(xmr.amount(), Decimal::from_str("1.234567890123").unwrap());

    // 18 decimals (ETH)
    let wei = 1_000_000_000_000_000_000i128;
    let eth = Money::from_minor_units(wei, Currency::ETH).unwrap();
    assert_eq!(eth.currency(), &Currency::ETH);
    assert_eq!(eth.as_minor_units().unwrap(), wei);
    assert_eq!(eth.amount(), Decimal::from(1));
}

#[test]
fn test_money_respects_builtin_usdc_override() {
    let usdc = Currency::try_from_str("USDC").unwrap();
    let microscopic = 1_500_000i128; // 1.5 USDC with 6 decimal places
    let money = Money::from_minor_units(microscopic, usdc.clone()).unwrap();

    assert_eq!(money.currency(), &usdc);
    assert_eq!(money.as_minor_units().unwrap(), microscopic);
    assert_eq!(money.amount(), Decimal::from_str("1.5").unwrap());
}

#[test]
fn test_money_display() {
    let usd = Money::new(
        Decimal::from_str("123.45").unwrap(),
        Currency::Iso(IsoCurrency::USD),
    )
    .unwrap();
    assert_eq!(format!("{usd}"), "123.45 USD");

    let eur = Money::new(
        Decimal::from_str("99.99").unwrap(),
        Currency::Iso(IsoCurrency::EUR),
    )
    .unwrap();
    assert_eq!(format!("{eur}"), "99.99 EUR");

    // Test with trailing zeros removed
    let jpy = Money::new(
        Decimal::from_str("100.00").unwrap(),
        Currency::Iso(IsoCurrency::JPY),
    )
    .unwrap();
    assert_eq!(format!("{jpy}"), "100 JPY");
}

#[cfg(all(feature = "dataframe", not(feature = "bigdecimal")))]
#[test]
fn test_money_dataframe_rust_decimal_backend() {
    use polars::prelude::AnyValue;

    let usd = Money::new(
        Decimal::from_str("123.45").unwrap(),
        Currency::Iso(IsoCurrency::USD),
    )
    .unwrap();

    let df = usd.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);

    let amount_value = df.column("amount").unwrap().get(0).unwrap();
    match amount_value {
        AnyValue::Decimal(value, scale) => {
            assert_eq!(scale, 10);
            let df_amount =
                decimal::from_minor_units(value, u32::try_from(scale).expect("scale fits in u32"));
            assert_eq!(df_amount, usd.amount());
        }
        other => panic!("expected decimal value, got {other:?}"),
    }

    let currency_value = df.column("currency").unwrap().get(0).unwrap();
    match currency_value {
        AnyValue::String(s) => assert_eq!(s, "USD"),
        AnyValue::StringOwned(s) => assert_eq!(s.as_str(), "USD"),
        other => panic!("expected string value, got {other:?}"),
    }
}

#[cfg(all(feature = "dataframe", feature = "bigdecimal"))]
#[test]
fn test_money_dataframe_bigdecimal_backend() {
    use polars::prelude::AnyValue;

    let eth_amount = Decimal::from_str("1.234567890123456789012345").unwrap();
    let eth = Money::new(eth_amount, Currency::ETH).unwrap();

    let df = eth.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);

    let amount_value = df.column("amount").unwrap().get(0).unwrap();
    match amount_value {
        AnyValue::Decimal(value, _precision, scale) => {
            assert_eq!(scale, 10);
            let df_amount =
                decimal::from_minor_units(value, u32::try_from(scale).expect("scale fits in u32"));
            let expected = decimal::round_dp_with_strategy(
                &eth.amount(),
                u32::try_from(scale).expect("scale fits in u32"),
                RoundingStrategy::ToZero,
            );
            assert_eq!(df_amount, expected);
        }
        other => panic!("expected decimal value, got {other:?}"),
    }

    let currency_value = df.column("currency").unwrap().get(0).unwrap();
    match currency_value {
        AnyValue::String(s) => assert_eq!(s, "ETH"),
        AnyValue::StringOwned(s) => assert_eq!(s.as_str(), "ETH"),
        other => panic!("expected string value, got {other:?}"),
    }
}
