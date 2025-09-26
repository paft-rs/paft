#[cfg(feature = "bigdecimal")]
use paft_money::Money;
use paft_money::{
    Currency, MinorUnitError, clear_currency_metadata, currency_metadata, set_currency_metadata,
};

#[test]
fn test_currency_parsing_accepts_aliases_and_unknown() {
    assert_eq!(
        Currency::try_from_str("BITCOIN").unwrap(),
        Currency::Other("BITCOIN".parse().unwrap())
    );
    let u = Currency::try_from_str("UNKNOWN").unwrap();
    assert_eq!(u.to_string(), "UNKNOWN");
}

#[test]
fn test_currency_normalization_trims_whitespace() {
    assert_eq!(
        Currency::try_from_str(" usd \t").unwrap().to_string(),
        "USD"
    );
    assert_eq!(
        Currency::try_from_str("  custom ").unwrap().to_string(),
        "CUSTOM"
    );

    assert!(Currency::try_from_str("   ").is_err());
}

#[test]
fn test_currency_full_name() {
    assert_eq!(
        Currency::Iso(iso_currency::Currency::USD)
            .full_name()
            .as_ref(),
        iso_currency::Currency::USD.name()
    );
    assert_eq!(Currency::BTC.full_name().as_ref(), "Bitcoin");
    let code = "UNKNOWN";
    clear_currency_metadata(code);
    let unknown = Currency::try_from_str(code).unwrap();
    assert_eq!(unknown.full_name().as_ref(), code);
}

#[test]
fn test_builtin_currency_metadata() {
    let usdc = Currency::try_from_str("usdc").unwrap();
    assert_eq!(usdc.decimal_places().unwrap(), 6);

    assert_eq!(
        currency_metadata("usdc").unwrap().full_name.as_ref(),
        "USD Coin"
    );

    let xrp = Currency::try_from_str("xrp").unwrap();
    assert_eq!(xrp.decimal_places().unwrap(), 6);
}

#[test]
fn test_custom_currency_metadata_updates() {
    let code = "custom_token";

    // Ensure no prior override
    clear_currency_metadata(code);

    assert!(currency_metadata(code).is_none());

    set_currency_metadata(code, "Custom Token", 9).expect("valid override");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.decimal_places().unwrap(), 9);
    assert_eq!(currency_metadata(code).unwrap().minor_units, 9);

    // Updating metadata should override the precision
    set_currency_metadata(code, "Custom Token", 4).expect("update override");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.decimal_places().unwrap(), 4);
    assert_eq!(currency_metadata(code).unwrap().minor_units, 4);

    clear_currency_metadata(code);
    assert!(currency_metadata(code).is_none());
}

#[test]
fn test_currency_metadata_rejects_overflowing_precision() {
    let err = set_currency_metadata("overflow_minor", "Token", 19).unwrap_err();
    assert!(matches!(
        err,
        MinorUnitError::ExceedsMinorUnitScale { decimals } if decimals == 19
    ));

    let err = set_currency_metadata("overflow_decimal", "Token", 29).unwrap_err();
    if cfg!(feature = "bigdecimal") {
        assert!(matches!(
            err,
            MinorUnitError::ExceedsMinorUnitScale { decimals } if decimals == 29
        ));
    } else {
        assert!(matches!(
            err,
            MinorUnitError::ExceedsDecimalPrecision { decimals } if decimals == 29
        ));
    }
}

#[cfg(feature = "bigdecimal")]
#[test]
fn test_bigdecimal_accepts_large_magnitudes() {
    set_currency_metadata("HP18", "High Precision", 18).expect("metadata accepted");
    let currency = Currency::try_from_str("HP18").unwrap();

    let amount = "123456789012345678901234567890.123456789012345678";
    let money = Money::from_str(amount, currency).unwrap();
    assert_eq!(money.amount().to_string(), amount);

    clear_currency_metadata("HP18");
}

#[test]
fn test_currency_metadata_accepts_boundary_precision() {
    let code = "boundary_precision";
    clear_currency_metadata(code);

    // 18 is the highest supported minor-unit precision and should be accepted.
    set_currency_metadata(code, "Boundary Token", 18).expect("18 decimals should be accepted");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.decimal_places().unwrap(), 18);
    assert_eq!(currency_metadata(code).unwrap().minor_units, 18);

    // Cleanup to avoid affecting other tests.
    clear_currency_metadata(code);
}

#[test]
fn test_custom_currency_metadata_required() {
    let code = "custom_meta";
    clear_currency_metadata(code);

    assert!(currency_metadata(code).is_none());

    set_currency_metadata(code, "Custom Token", 4).expect("metadata set");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.full_name().as_ref(), "Custom Token");
    assert_eq!(currency.decimal_places().unwrap(), 4);

    // Updating metadata should preserve the name.
    set_currency_metadata(code, "Custom Token", 5).expect("override precision");
    let metadata = currency_metadata(code).expect("metadata present");
    assert_eq!(metadata.full_name.as_ref(), "Custom Token");
    assert_eq!(metadata.minor_units, 5);

    clear_currency_metadata(code);
}

#[test]
fn test_iso_none_metadata_overlay_for_metals_and_funds() {
    use iso_currency::Currency as IsoCurrency;

    let xau = Currency::Iso(IsoCurrency::XAU);
    let xdr = Currency::Iso(IsoCurrency::XDR);

    clear_currency_metadata("XAU");
    clear_currency_metadata("XDR");

    let err_xau = xau.decimal_places().unwrap_err();
    assert!(matches!(
        err_xau,
        paft_money::MoneyError::MetadataNotFound { ref currency } if currency == &xau
    ));

    let err_xdr = xdr.decimal_places().unwrap_err();
    assert!(matches!(
        err_xdr,
        paft_money::MoneyError::MetadataNotFound { ref currency } if currency == &xdr
    ));

    set_currency_metadata("XAU", "Gold", 3).unwrap();
    set_currency_metadata("XDR", "SDR", 6).unwrap();

    assert_eq!(xau.decimal_places().unwrap(), 3);
    assert_eq!(xdr.decimal_places().unwrap(), 6);

    clear_currency_metadata("XAU");
    clear_currency_metadata("XDR");
}
