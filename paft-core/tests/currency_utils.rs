use paft_core::domain::{
    Currency, clear_currency_minor_units, currency_minor_units, describe_currency,
    is_common_currency, normalize_currency_code, set_currency_minor_units,
};

#[test]
fn test_currency_normalization() {
    assert_eq!(normalize_currency_code("DOLLAR"), Currency::USD);
    assert_eq!(normalize_currency_code("BITCOIN"), Currency::BTC);
    assert_eq!(
        normalize_currency_code("UNKNOWN"),
        Currency::Other("UNKNOWN".to_string())
    );
}

#[test]
fn test_currency_normalization_trims_whitespace() {
    assert_eq!(normalize_currency_code(" usd \t"), Currency::USD);
    assert_eq!(
        normalize_currency_code("  custom "),
        Currency::Other("CUSTOM".to_string())
    );
}

#[test]
fn test_common_currency_detection() {
    assert!(is_common_currency(&Currency::USD));
    assert!(is_common_currency(&Currency::BTC));
    assert!(is_common_currency(&Currency::Other("BTC".to_string())));
    assert!(!is_common_currency(&Currency::Other("UNKNOWN".to_string())));
}

#[test]
fn test_currency_description() {
    assert_eq!(describe_currency(&Currency::USD), "US Dollar");
    assert_eq!(describe_currency(&Currency::BTC), "BTC");
    assert_eq!(
        describe_currency(&Currency::Other("UNKNOWN".to_string())),
        "Unknown currency (UNKNOWN)"
    );
}

#[test]
fn test_builtin_currency_minor_units_override() {
    let xrp = normalize_currency_code("xrp");
    assert_eq!(xrp.decimal_places(), 6);
    assert_eq!(currency_minor_units("xrp"), Some(6));
}

#[test]
fn test_custom_currency_minor_units_override() {
    let code = "custom_token";

    // Ensure no prior override
    clear_currency_minor_units(code);

    assert_eq!(currency_minor_units(code), None);

    set_currency_minor_units(code, 9);
    let currency = normalize_currency_code(code);
    assert_eq!(currency.decimal_places(), 9);
    assert_eq!(currency_minor_units(code), Some(9));

    // Clearing should revert to default 2 decimal places
    clear_currency_minor_units(code);
    assert_eq!(currency_minor_units(code), None);
    assert_eq!(currency.decimal_places(), 2);
}
