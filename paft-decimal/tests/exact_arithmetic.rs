use paft_decimal::{
    Decimal, checked_add_exact, checked_div_exact, checked_mul_exact, checked_sub_exact,
    parse_decimal,
};

fn dec(value: &str) -> Decimal {
    parse_decimal(value).unwrap()
}

#[test]
fn addition_and_subtraction_preserve_digits_or_fail() {
    for (lhs, rhs, sum, difference) in [
        ("1.2300", "0.0045", "1.2345", "1.2255"),
        ("-1.2300", "0.0045", "-1.2255", "-1.2345"),
        (
            "10000000000000000000000000000",
            "-10000000000000000000000000000",
            "0",
            "20000000000000000000000000000",
        ),
    ] {
        assert_eq!(checked_add_exact(&dec(lhs), &dec(rhs)), Some(dec(sum)));
        assert_eq!(
            checked_sub_exact(&dec(lhs), &dec(rhs)),
            Some(dec(difference))
        );
    }

    for delta in [
        Decimal::ONE,
        dec("0.1"),
        dec("0.0000000000000000000000000001"),
    ] {
        assert_eq!(checked_add_exact(&Decimal::MAX, &delta), None);
        assert_eq!(checked_sub_exact(&Decimal::MIN, &delta), None);
    }
    assert_eq!(checked_sub_exact(&Decimal::MAX, &dec("0.1")), None);
    assert_eq!(checked_add_exact(&Decimal::MIN, &dec("0.1")), None);
    assert_eq!(
        checked_sub_exact(&Decimal::MAX, &Decimal::MAX),
        Some(Decimal::ZERO)
    );
}

#[test]
fn addition_can_reduce_an_exact_result_to_fit() {
    let lhs = dec("7.9228162514264337593543950335");
    let rhs = dec("0.0000000000000000000000000005");
    let expected = dec("7.922816251426433759354395034");
    assert_eq!(checked_add_exact(&lhs, &rhs), Some(expected));
    assert_eq!(checked_sub_exact(&lhs, &-rhs), Some(expected));
}

#[test]
fn multiplication_preserves_values_or_rejects_precision_loss() {
    for (lhs, rhs, result) in [
        ("182.345678", "4.91", "895.31727898"),
        ("1.2500", "0.8", "1"),
        ("-1.25", "0.8", "-1"),
        ("-1.25", "-0.8", "1"),
        (
            "0.0000000000000000000000000001",
            "10",
            "0.000000000000000000000000001",
        ),
    ] {
        assert_eq!(checked_mul_exact(&dec(lhs), &dec(rhs)), Some(dec(result)));
    }
    for (lhs, rhs) in [
        (dec("0.0000000000000000000000000001"), dec("0.1")),
        (dec("1.0000000000000000000000000001"), dec("1.1")),
        (Decimal::MAX, Decimal::TWO),
        (Decimal::MAX, Decimal::MAX),
    ] {
        assert_eq!(checked_mul_exact(&lhs, &rhs), None);
        assert_eq!(checked_mul_exact(&-lhs, &rhs), None);
    }
    assert_eq!(
        checked_mul_exact(&Decimal::MAX, &dec("-0.0000")),
        Some(Decimal::ZERO)
    );
}

#[test]
fn multiplication_cancels_split_factors_before_intermediate_overflow() {
    // The unreduced product needs 161 bits. Removing 28 factors of ten
    // produces exactly 2^67, which fits the decimal coefficient.
    let lhs = Decimal::from_i128_with_scale(2_i128.pow(95), 28);
    let rhs = Decimal::from_i128_with_scale(5_i128.pow(28), 0);
    let expected = Decimal::from_i128_with_scale(2_i128.pow(67), 0);
    assert!(lhs.mantissa().checked_mul(rhs.mantissa()).is_none());
    assert_eq!(checked_mul_exact(&lhs, &rhs), Some(expected));
    assert_eq!(checked_mul_exact(&rhs, &lhs), Some(expected));
    assert_eq!(checked_mul_exact(&-lhs, &rhs), Some(-expected));
    assert_eq!(checked_mul_exact(&-lhs, &-rhs), Some(expected));
}

#[test]
fn division_requires_an_exact_representable_quotient() {
    for (lhs, rhs, result) in [
        ("1", "8", "0.125"),
        ("10", "4", "2.5"),
        ("-10", "4", "-2.5"),
        ("2.5", "0.05", "50"),
        ("0", "0.0000000000000000000000000001", "0"),
    ] {
        assert_eq!(checked_div_exact(&dec(lhs), &dec(rhs)), Some(dec(result)));
    }
    for divisor in ["3", "6", "7", "9", "0"] {
        assert_eq!(checked_div_exact(&Decimal::ONE, &dec(divisor)), None);
    }
    assert_eq!(checked_div_exact(&Decimal::ZERO, &Decimal::ZERO), None);
    assert_eq!(
        checked_div_exact(&dec("0.0000000000000000000000000001"), &Decimal::TWO),
        None
    );
    assert_eq!(checked_div_exact(&Decimal::ONE, &Decimal::MAX), None);
    assert_eq!(checked_div_exact(&Decimal::MAX, &dec("0.1")), None);
    // Rounded reverse multiplication can equal MAX even though MAX / 11
    // has no exact representable quotient. Verify with exact multiplication.
    assert_eq!(checked_div_exact(&Decimal::MAX, &Decimal::from(11)), None);
    assert_eq!(
        checked_div_exact(&Decimal::MAX, &Decimal::MAX),
        Some(Decimal::ONE)
    );
}
