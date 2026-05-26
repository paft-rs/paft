use iso_currency::Currency as IsoCurrency;
use paft_decimal::{Decimal, RoundingStrategy};
use paft_money::{Currency, ExchangeRate, Money};
use std::str::FromStr;

#[cfg(feature = "dataframe")]
use paft_decimal as decimal;
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
    fn test_money_div_money_yields_decimal_ratio() {
        let budget = Money::new(Decimal::from(1000), Currency::Iso(IsoCurrency::USD)).unwrap();
        let price = Money::new(Decimal::from(40), Currency::Iso(IsoCurrency::USD)).unwrap();

        let ratio_ref = &budget / &price;
        assert_eq!(ratio_ref, Decimal::from(25));

        let ratio_owned = budget / price;
        assert_eq!(ratio_owned, Decimal::from(25));
    }

    #[test]
    #[should_panic(expected = "currency mismatch")]
    fn test_money_div_money_currency_mismatch_panics() {
        let usd = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        let eur = Money::new(Decimal::from(50), Currency::Iso(IsoCurrency::EUR)).unwrap();
        let _ = &usd / &eur;
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_money_div_money_zero_panics() {
        let usd = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        let zero = Money::zero(Currency::Iso(IsoCurrency::USD)).unwrap();
        let _ = &usd / &zero;
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
    fn test_try_div_money_returns_decimal_ratio() {
        let budget = Money::new(Decimal::from(1000), Currency::Iso(IsoCurrency::USD)).unwrap();
        let price = Money::new(Decimal::from(40), Currency::Iso(IsoCurrency::USD)).unwrap();

        let ratio = budget.try_div_money(&price).unwrap();
        assert_eq!(ratio, Decimal::from(25));
    }

    #[test]
    fn test_try_div_money_rejects_currency_mismatch() {
        let usd = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        let eur = Money::new(Decimal::from(50), Currency::Iso(IsoCurrency::EUR)).unwrap();

        assert!(matches!(
            usd.try_div_money(&eur),
            Err(paft_money::MoneyError::CurrencyMismatch { .. })
        ));
    }

    #[test]
    fn test_try_div_money_rejects_zero_divisor() {
        let usd = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
        let zero = Money::zero(Currency::Iso(IsoCurrency::USD)).unwrap();

        assert!(matches!(
            usd.try_div_money(&zero),
            Err(paft_money::MoneyError::DivisionByZero)
        ));
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

#[cfg(not(feature = "bigdecimal"))]
#[test]
fn money_from_minor_units_returns_error_on_decimal_overflow() {
    let err = Money::from_minor_units(i128::MAX, Currency::Iso(IsoCurrency::USD)).unwrap_err();
    assert!(matches!(err, paft_money::MoneyError::ConversionError));
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
        AnyValue::Decimal(value, _, scale) => {
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
        AnyValue::Decimal(value, _, scale) => {
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

#[test]
fn money_new_exact_rejects_overprecise_input() {
    let amount = Decimal::from_str("1.234").unwrap();
    let err = Money::new_exact(amount, Currency::Iso(IsoCurrency::USD)).unwrap_err();
    match err {
        paft_money::MoneyError::PrecisionExceeded {
            currency_code,
            max_scale,
            actual_scale,
        } => {
            assert_eq!(currency_code, "USD");
            assert_eq!(max_scale, 2);
            assert_eq!(actual_scale, 3);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn money_new_exact_accepts_trailing_zeros() {
    let amount = Decimal::from_str("1.230").unwrap();
    let money = Money::new_exact(amount, Currency::Iso(IsoCurrency::USD)).unwrap();
    let canonical = Money::from_canonical_str("1.23", Currency::Iso(IsoCurrency::USD)).unwrap();
    assert_eq!(money, canonical);
}

#[test]
fn money_new_still_rounds() {
    let rounded = Money::new(
        Decimal::from_str("1.234").unwrap(),
        Currency::Iso(IsoCurrency::USD),
    )
    .unwrap();
    assert_eq!(rounded.amount(), Decimal::from_str("1.23").unwrap());

    let rounded_up = Money::new(
        Decimal::from_str("1.235").unwrap(),
        Currency::Iso(IsoCurrency::USD),
    )
    .unwrap();
    assert_eq!(rounded_up.amount(), Decimal::from_str("1.24").unwrap());
}

#[test]
fn money_from_canonical_str_defers_to_new_exact() {
    let ok = Money::from_canonical_str("1.23", Currency::Iso(IsoCurrency::USD)).unwrap();
    assert_eq!(ok.amount(), Decimal::from_str("1.23").unwrap());

    let err = Money::from_canonical_str("1.234", Currency::Iso(IsoCurrency::USD)).unwrap_err();
    assert!(matches!(
        err,
        paft_money::MoneyError::PrecisionExceeded { .. }
    ));
}

#[test]
fn money_hash_eq_consistency_across_scales() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let trailing = Money::from_canonical_str("1.230", Currency::Iso(IsoCurrency::USD)).unwrap();
    let canonical = Money::from_canonical_str("1.23", Currency::Iso(IsoCurrency::USD)).unwrap();

    assert_eq!(trailing, canonical);

    let mut hasher_a = DefaultHasher::new();
    trailing.hash(&mut hasher_a);
    let mut hasher_b = DefaultHasher::new();
    canonical.hash(&mut hasher_b);
    assert_eq!(hasher_a.finish(), hasher_b.finish());
}

#[test]
fn money_serde_rejects_overprecise_amount() {
    // Bypass the Money struct's own constructors and feed serde a value
    // whose scale exceeds USD's exponent. Without the custom Deserialize
    // impl this would silently produce a malformed Money; with it, serde
    // returns an error.
    let raw = "{\"amount\":\"1.234\",\"currency\":\"USD\"}";
    let err = serde_json::from_str::<Money>(raw).unwrap_err();
    assert!(
        err.to_string().contains("precision exceeded"),
        "unexpected error: {err}"
    );
}

#[test]
fn money_serde_accepts_trailing_zero_amount() {
    let raw = "{\"amount\":\"1.230\",\"currency\":\"USD\"}";
    let money: Money = serde_json::from_str(raw).unwrap();
    let expected = Money::from_canonical_str("1.23", Currency::Iso(IsoCurrency::USD)).unwrap();
    assert_eq!(money, expected);
}

#[test]
fn exchange_rate_new_accepts_identity_with_one() {
    let rate = ExchangeRate::new(
        Currency::Iso(IsoCurrency::USD),
        Currency::Iso(IsoCurrency::USD),
        Decimal::from(1),
    )
    .unwrap();
    assert_eq!(rate.rate(), Decimal::from(1));

    let usd = Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap();
    let converted = usd.try_convert(&rate).unwrap();
    assert_eq!(converted, usd);
}

#[test]
fn exchange_rate_new_rejects_identity_with_non_one() {
    let err = ExchangeRate::new(
        Currency::Iso(IsoCurrency::USD),
        Currency::Iso(IsoCurrency::USD),
        Decimal::from_str("1.5").unwrap(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        paft_money::MoneyError::InvalidExchangeRate { .. }
    ));
}

#[test]
fn exchange_rate_new_rejects_negative_or_zero_rate() {
    assert!(matches!(
        ExchangeRate::new(
            Currency::Iso(IsoCurrency::USD),
            Currency::Iso(IsoCurrency::EUR),
            Decimal::from(0),
        )
        .unwrap_err(),
        paft_money::MoneyError::InvalidExchangeRate { .. }
    ));
    assert!(matches!(
        ExchangeRate::new(
            Currency::Iso(IsoCurrency::USD),
            Currency::Iso(IsoCurrency::EUR),
            Decimal::from(-1),
        )
        .unwrap_err(),
        paft_money::MoneyError::InvalidExchangeRate { .. }
    ));
}

#[test]
fn exchange_rate_serde_rejects_invalid_payload() {
    // Negative rate would have been accepted by `#[derive(Deserialize)]`
    // before; the custom Deserialize impl now routes through
    // `ExchangeRate::new` and surfaces the validation error.
    let raw = "{\"from\":\"USD\",\"to\":\"EUR\",\"rate\":\"-1\"}";
    assert!(serde_json::from_str::<ExchangeRate>(raw).is_err());

    // Identity with non-1 rate is also rejected.
    let raw_identity = "{\"from\":\"USD\",\"to\":\"USD\",\"rate\":\"2\"}";
    assert!(serde_json::from_str::<ExchangeRate>(raw_identity).is_err());

    // Zero rate is rejected.
    let raw_zero = "{\"from\":\"USD\",\"to\":\"EUR\",\"rate\":\"0\"}";
    assert!(serde_json::from_str::<ExchangeRate>(raw_zero).is_err());
}

#[test]
fn exchange_rate_serde_accepts_valid_payload() {
    let raw = "{\"from\":\"USD\",\"to\":\"EUR\",\"rate\":\"0.9\"}";
    let parsed: ExchangeRate = serde_json::from_str(raw).unwrap();
    assert_eq!(parsed.from(), &Currency::Iso(IsoCurrency::USD));
    assert_eq!(parsed.to(), &Currency::Iso(IsoCurrency::EUR));
    assert_eq!(parsed.rate(), Decimal::from_str("0.9").unwrap());
}

#[cfg(not(feature = "bigdecimal"))]
#[test]
fn money_try_div_returns_error_on_decimal_overflow() {
    let currency = Currency::Iso(IsoCurrency::USD);
    let money = Money::new_exact(Decimal::MAX, currency.clone()).unwrap();
    let divisor = Decimal::from_str("0.1").unwrap();

    let err = money.try_div(divisor).unwrap_err();
    assert!(matches!(err, paft_money::MoneyError::ConversionError));

    let divisor_money = Money::new_exact(Decimal::from_str("0.1").unwrap(), currency).unwrap();
    let err = money.try_div_money(&divisor_money).unwrap_err();
    assert!(matches!(err, paft_money::MoneyError::ConversionError));
}

#[cfg(not(feature = "bigdecimal"))]
#[test]
fn money_as_minor_units_returns_error_on_overflow() {
    use std::str::FromStr;

    // ETH has 18 decimal places, so the multiplier is 10^18. A value just
    // shy of `i128::MAX / 10^18` already pushes the conversion close to
    // the i128 bound; the constant below is well past `i128::MAX` once
    // multiplied. With unchecked `*`, this used to panic; now it surfaces
    // as ConversionError.
    let huge = Decimal::from_str("99999999999999999999.123456789012345678").unwrap();
    let money = Money::new(huge, Currency::ETH).unwrap();
    let err = money.as_minor_units().unwrap_err();
    assert!(matches!(err, paft_money::MoneyError::ConversionError));
}

#[cfg(feature = "bigdecimal")]
#[test]
fn money_as_minor_units_returns_error_on_i128_overflow_under_bigdecimal() {
    use std::str::FromStr;

    // BigDecimal has unbounded precision, so the *multiplication* never
    // overflows. The conversion to `i128`, however, can still fail when
    // the scaled value exceeds the `i128` range — and that path must
    // still surface as ConversionError, not a panic.
    let huge =
        Decimal::from_str("999999999999999999999999999999999999999999999999.123456789012345678")
            .unwrap();
    let money = Money::new(huge, Currency::ETH).unwrap();
    let err = money.as_minor_units().unwrap_err();
    assert!(matches!(err, paft_money::MoneyError::ConversionError));
}
