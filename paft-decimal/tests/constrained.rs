use std::str::FromStr;

use paft_decimal::{Decimal, NonNegativeDecimal, PositiveDecimal, Ratio};

fn dec(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap()
}

#[cfg(not(feature = "bigdecimal"))]
const fn decimal_for_reuse(value: &Decimal) -> Decimal {
    *value
}

#[cfg(feature = "bigdecimal")]
fn decimal_for_reuse(value: &Decimal) -> Decimal {
    value.clone()
}

#[test]
fn non_negative_decimal_accepts_zero_and_positive_values() {
    assert_eq!(
        NonNegativeDecimal::new(dec("0")).unwrap().as_decimal(),
        &dec("0")
    );
    assert_eq!(
        NonNegativeDecimal::new(dec("1.25")).unwrap().as_decimal(),
        &dec("1.25")
    );
    assert!(NonNegativeDecimal::new(dec("-0.01")).is_err());
}

#[test]
fn positive_decimal_rejects_zero_and_negative_values() {
    assert_eq!(
        PositiveDecimal::new(dec("1")).unwrap().as_decimal(),
        &dec("1")
    );
    assert!(PositiveDecimal::new(dec("0")).is_err());
    assert!(PositiveDecimal::new(dec("-1")).is_err());
}

#[test]
fn ratio_accepts_only_inclusive_unit_interval() {
    assert!(Ratio::new(dec("0")).is_ok());
    assert!(Ratio::new(dec("0.5")).is_ok());
    assert!(Ratio::new(dec("1")).is_ok());
    assert!(Ratio::new(dec("-0.01")).is_err());
    assert!(Ratio::new(dec("1.01")).is_err());
}

#[test]
fn constrained_decimal_display_uses_canonical_decimal() {
    assert_eq!(
        NonNegativeDecimal::new(dec("0.2500")).unwrap().to_string(),
        "0.25"
    );
    assert_eq!(
        PositiveDecimal::new(dec("1.2300")).unwrap().to_string(),
        "1.23"
    );
    assert_eq!(Ratio::new(dec("1.0000")).unwrap().to_string(), "1");
}

#[test]
fn constrained_decimals_serialize_as_canonical_strings() {
    let decimal = dec("0.2500");
    let ratio_for_serialization = Ratio::new(decimal_for_reuse(&decimal)).unwrap();
    let ratio_for_comparison = Ratio::new(decimal_for_reuse(&decimal)).unwrap();

    assert_eq!(
        serde_json::to_value(ratio_for_serialization).unwrap(),
        serde_json::json!("0.25")
    );
    assert_eq!(
        serde_json::from_value::<Ratio>(serde_json::json!("0.2500")).unwrap(),
        ratio_for_comparison
    );
}

#[test]
fn constrained_decimal_deserialization_rejects_invalid_values() {
    assert!(serde_json::from_value::<NonNegativeDecimal>(serde_json::json!(-1)).is_err());
    assert!(serde_json::from_value::<PositiveDecimal>(serde_json::json!(0)).is_err());
    assert!(serde_json::from_value::<Ratio>(serde_json::json!(1.01)).is_err());
}
