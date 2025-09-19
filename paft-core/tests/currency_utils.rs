use paft_core::domain::currency_utils::MinorUnitError;
use paft_core::domain::{
    Currency, clear_currency_minor_units, currency_minor_units, set_currency_minor_units,
};

#[test]
fn test_currency_parsing_accepts_aliases_and_unknown() {
    assert_eq!(Currency::try_from_str("DOLLAR").unwrap(), Currency::USD);
    assert_eq!(Currency::try_from_str("BITCOIN").unwrap(), Currency::BTC);
    let u = Currency::try_from_str("UNKNOWN").unwrap();
    assert_eq!(u.to_string(), "UNKNOWN");
}

#[test]
fn test_currency_normalization_trims_whitespace() {
    assert_eq!(Currency::try_from_str(" usd \t").unwrap(), Currency::USD);
    assert_eq!(
        Currency::try_from_str("  custom ").unwrap().to_string(),
        "CUSTOM"
    );

    assert!(Currency::try_from_str("   ").is_err());
}

#[test]
fn test_currency_full_name() {
    assert_eq!(Currency::USD.full_name(), "US Dollar");
    assert_eq!(Currency::BTC.full_name(), "Bitcoin");
    let unknown = Currency::try_from_str("UNKNOWN").unwrap();
    assert_eq!(unknown.full_name(), "UNKNOWN");
}

#[test]
fn test_builtin_currency_minor_units_override() {
    let usdc = Currency::try_from_str("usdc").unwrap();
    assert_eq!(usdc.decimal_places(), 6);
    assert_eq!(currency_minor_units("usdc"), Some(6));

    let xrp = Currency::try_from_str("xrp").unwrap();
    assert_eq!(xrp.decimal_places(), 6);
    assert_eq!(currency_minor_units("xrp"), Some(6));
}

#[test]
fn test_custom_currency_minor_units_override() {
    let code = "custom_token";

    // Ensure no prior override
    clear_currency_minor_units(code);

    assert_eq!(currency_minor_units(code), None);

    set_currency_minor_units(code, 9).expect("valid override");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.decimal_places(), 9);
    assert_eq!(currency_minor_units(code), Some(9));

    // Clearing should revert to default 2 decimal places
    clear_currency_minor_units(code);
    assert_eq!(currency_minor_units(code), None);
    assert_eq!(currency.decimal_places(), 2);
}

#[test]
fn test_currency_minor_units_rejects_overflowing_precision() {
    let err = set_currency_minor_units("overflow_minor", 19).unwrap_err();
    assert!(matches!(
        err,
        MinorUnitError::ExceedsMinorUnitScale { decimals } if decimals == 19
    ));

    let err = set_currency_minor_units("overflow_decimal", 29).unwrap_err();
    assert!(matches!(
        err,
        MinorUnitError::ExceedsDecimalPrecision { decimals } if decimals == 29
    ));
}

#[test]
fn test_currency_minor_units_accepts_boundary_precision() {
    let code = "boundary_precision";
    clear_currency_minor_units(code);

    // 18 is the highest supported minor-unit precision and should be accepted.
    set_currency_minor_units(code, 18).expect("18 decimals should be accepted");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.decimal_places(), 18);
    assert_eq!(currency_minor_units(code), Some(18));

    // Cleanup to avoid affecting other tests.
    clear_currency_minor_units(code);
}
