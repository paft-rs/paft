use paft_decimal::{self as decimal, Decimal};
use serde_json::json;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Payload {
    #[serde(with = "decimal::serde::canonical_str")]
    value: Decimal,
    #[serde(default, with = "decimal::serde::option_canonical_str")]
    optional: Option<Decimal>,
}

#[test]
fn parsing_preserves_all_significant_digits_or_rejects_the_value() {
    for literal in [
        "1.00000000000000000000000000001",
        "-1.00000000000000000000000000001",
        "0.00000000000000000000000000001",
        "-0.00000000000000000000000000001",
        "1.00000000000000000000000000009",
        "7.9228162514264337593543950336",
        "7922816251426433759354395033.51",
        "79228162514264337593543950335.1",
        "79228162514264337593543950336",
        "1000000000000000000000000000000",
    ] {
        let padded = if literal.contains('.') {
            format!("{literal}000")
        } else {
            format!("{literal}.000")
        };
        for input in [literal, &padded] {
            let parsed = decimal::parse_decimal(input);
            if decimal::MAX_DECIMAL_PRECISION == 28 {
                assert!(parsed.is_none(), "{input} was accepted as {parsed:?}");
            } else {
                assert_eq!(decimal::to_canonical_string(&parsed.unwrap()), literal);
            }
        }
    }
}

#[test]
fn parsing_accepts_exact_boundaries_with_insignificant_trailing_zeros() {
    for canonical in [
        "0",
        "1.23",
        "-1.23",
        "1000",
        "0.0000000000000000000000000001",
        "-0.0000000000000000000000000001",
        "7.9228162514264337593543950335",
        "79228162514264337593543950335",
        "-79228162514264337593543950335",
    ] {
        let zeros = "0".repeat(64);
        let padded = if canonical.contains('.') {
            format!("{canonical}{zeros}")
        } else {
            format!("{canonical}.{zeros}")
        };
        for input in [canonical, &padded] {
            let parsed = decimal::parse_decimal(input)
                .unwrap_or_else(|| panic!("{input} should be representable exactly"));
            assert_eq!(decimal::to_canonical_string(&parsed), canonical);
        }
    }
}

#[test]
fn parsing_preserves_representable_scale() {
    let value = decimal::parse_decimal("1.2300").unwrap();
    assert_eq!(decimal::fractional_digit_count(&value), 4);
}

#[test]
fn canonical_serde_preserves_significant_digits_or_rejects_the_value() {
    let literal = "1.00000000000000000000000000001";
    for wire in [
        json!({ "value": literal, "optional": null }),
        json!({ "value": "1", "optional": literal }),
    ] {
        let parsed = serde_json::from_value::<Payload>(wire.clone());
        if decimal::MAX_DECIMAL_PRECISION == 28 {
            assert!(parsed.is_err(), "{wire} was accepted as {parsed:?}");
        } else {
            assert_eq!(serde_json::to_value(parsed.unwrap()).unwrap(), wire);
        }
    }
}

#[test]
fn canonical_serde_accepts_insignificant_zeros_without_relaxing_the_grammar() {
    let zeros = "0".repeat(64);
    let wire = json!({ "value": format!("  +001.23{zeros} "), "optional": format!("-.{zeros}") });
    let parsed = serde_json::from_value::<Payload>(wire).unwrap();
    assert_eq!(
        serde_json::to_value(parsed).unwrap(),
        json!({ "value": "1.23", "optional": "0" })
    );

    for literal in [
        format!("1.{zeros}x"),
        format!("1.{zeros}.0"),
        format!("1_000.{zeros}"),
        format!("1.{zeros}e0"),
        format!("+-0.{zeros}"),
    ] {
        assert!(decimal::parse_decimal(&literal).is_none());
        assert!(serde_json::from_value::<Payload>(json!({ "value": literal })).is_err());
    }
}

#[test]
fn constrained_decimals_validate_the_original_value() {
    assert!(
        serde_json::from_value::<decimal::Ratio>(json!("1.00000000000000000000000000001")).is_err()
    );
    assert!(
        serde_json::from_value::<decimal::NonNegativeDecimal>(json!(
            "-0.00000000000000000000000000001"
        ))
        .is_err()
    );
}
