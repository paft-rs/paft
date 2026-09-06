use paft::money::{PriceAmount, QuantityAmount};
use paft::{Decimal, NonNegativeDecimal, PositiveDecimal, Ratio};
use serde_json::json;

const fn extract(value: NonNegativeDecimal) -> Decimal {
    value.into_inner()
}

const fn extract_positive(value: PositiveDecimal) -> Decimal {
    value.into_inner()
}

const fn extract_ratio(value: Ratio) -> Decimal {
    value.into_inner()
}

const fn extract_quantity(value: QuantityAmount) -> NonNegativeDecimal {
    value.into_inner()
}

const PRICE: Decimal = PriceAmount::new(paft::decimal::one()).into_inner();
const COPIED: Decimal = paft::decimal::clone_decimal(&PRICE);

#[test]
fn decimal_identity_and_capabilities_are_unconditional() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<Decimal>();
    assert_copy::<NonNegativeDecimal>();
    assert_copy::<PositiveDecimal>();
    assert_copy::<Ratio>();
    let native: rust_decimal::Decimal = COPIED;
    let canonical: Decimal = native;
    let non_negative = NonNegativeDecimal::new(canonical).unwrap();
    assert_eq!(extract(non_negative), Decimal::ONE);
    assert_eq!(
        extract_positive(PositiveDecimal::new(canonical).unwrap()),
        Decimal::ONE
    );
    assert_eq!(extract_ratio(Ratio::new(canonical).unwrap()), Decimal::ONE);
    assert_eq!(
        extract_quantity(QuantityAmount::new(non_negative)),
        non_negative
    );
    assert_eq!(paft::decimal::MAX_DECIMAL_PRECISION, 28);
}

#[test]
fn constrained_serde_rejects_unrepresentable_values_before_validation() {
    let tiny = json!("0.00000000000000000000000000001");
    assert!(serde_json::from_value::<PositiveDecimal>(tiny).is_err());
    assert!(serde_json::from_value::<Ratio>(json!("1.00000000000000000000000000001")).is_err());
    assert!(
        serde_json::from_value::<NonNegativeDecimal>(json!("-0.00000000000000000000000000001"))
            .is_err()
    );
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<Ratio>(json!("0.2500")).unwrap()).unwrap(),
        json!("0.25")
    );
}

#[cfg(any(feature = "market", feature = "fundamentals"))]
fn check_optional_fields<T: serde::de::DeserializeOwned + serde::Serialize>(fields: &[&str]) {
    let empty = serde_json::from_value::<T>(json!({})).unwrap();
    let empty_wire = serde_json::to_value(empty).unwrap();
    for field in fields {
        assert!(empty_wire[*field].is_null());
        let mut wire = json!({});
        wire[*field] = json!("1.2300");
        let value = serde_json::from_value::<T>(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap()[*field], "1.23");
        wire[*field] = json!("1.00000000000000000000000000001");
        let error = serde_json::from_value::<T>(wire).err().unwrap();
        assert!(
            error
                .to_string()
                .contains("not exactly representable by PAFT"),
            "{field}: {error}"
        );
    }
}

#[cfg(feature = "market")]
#[test]
fn market_optional_decimals_use_canonical_serde() {
    check_optional_fields::<paft::market::OptionGreeks>(&[
        "delta", "gamma", "theta", "vega", "rho",
    ]);
}

#[cfg(feature = "fundamentals")]
#[test]
fn fundamentals_optional_decimals_use_canonical_serde() {
    use paft::fundamentals::{
        AnalysisSummary, EarningsEstimate, EsgScores, KeyStatistics, RecommendationSummary,
        RevenueEstimate,
    };
    check_optional_fields::<EsgScores>(&["environmental", "social", "governance"]);
    check_optional_fields::<KeyStatistics>(&[
        "pe_trailing_twelve_months",
        "dividend_yield_trailing",
        "dividend_yield_forward",
        "beta",
    ]);
    check_optional_fields::<RecommendationSummary>(&["mean"]);
    check_optional_fields::<AnalysisSummary>(&["recommendation_mean"]);
    check_optional_fields::<EarningsEstimate>(&["growth"]);
    check_optional_fields::<RevenueEstimate>(&["growth"]);
}

#[cfg(feature = "prediction")]
#[test]
fn prediction_numeric_bounds_use_canonical_serde() {
    use paft::prediction::NumericBound;
    for kind in ["included", "excluded"] {
        let valid = serde_json::from_value::<NumericBound>(json!({kind: "1.2300"})).unwrap();
        assert_eq!(serde_json::to_value(valid).unwrap(), json!({kind: "1.23"}));
        let error = serde_json::from_value::<NumericBound>(
            json!({kind: "1.00000000000000000000000000001"}),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("not exactly representable by PAFT")
        );
    }
}

#[cfg(feature = "dataframe")]
#[test]
fn dataframe_encoding_uses_the_same_decimal_types() {
    use paft::dataframe::Decimal128Encode;
    let value = paft::decimal::parse_decimal("1.25").unwrap();
    assert_eq!(value.try_to_i128_mantissa(1), Some(12));
    assert_eq!(
        NonNegativeDecimal::new(value)
            .unwrap()
            .try_to_i128_mantissa(1),
        Some(12)
    );
    assert_eq!(Decimal::ONE.try_to_i128_mantissa(38), None);
}
