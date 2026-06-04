#[cfg(feature = "bigdecimal")]
use paft_money::Money;
use paft_money::currency_utils::CurrencyMetadata;
use paft_money::{
    Currency, MinorUnitError, clear_currency_metadata, currency_metadata,
    override_currency_metadata, set_currency_metadata,
};

use paft_money::Locale;
fn call_set_metadata(
    code: &str,
    name: &str,
    units: u8,
) -> Result<Option<CurrencyMetadata>, MinorUnitError> {
    set_currency_metadata(
        code,
        name.to_string(),
        units,
        code.to_string(),
        true,
        Locale::EnUs,
    )
}

fn call_override_metadata(
    code: &str,
    name: &str,
    units: u8,
) -> Result<Option<CurrencyMetadata>, MinorUnitError> {
    override_currency_metadata(
        code,
        name.to_string(),
        units,
        code.to_string(),
        true,
        Locale::EnUs,
    )
}

#[test]
fn test_currency_parsing_accepts_aliases_and_unknown() {
    assert_eq!(
        Currency::try_from_str("BITCOIN").unwrap(),
        Currency::other("BITCOIN").unwrap()
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
fn test_non_iso_full_name_routes_through_registry() {
    clear_currency_metadata("XMR");

    call_set_metadata("XMR", "Monero Overlay", 12).expect("same-scale display update");

    assert_eq!(Currency::XMR.full_name().as_ref(), "Monero Overlay");
    assert_eq!(Currency::XMR.decimal_places().unwrap(), 12);

    clear_currency_metadata("XMR");
}

#[test]
fn test_custom_currency_metadata_preserves_registered_scale() {
    let code = "custom_token";

    // Ensure no prior override
    clear_currency_metadata(code);

    assert!(currency_metadata(code).is_none());

    call_set_metadata(code, "Custom Token", 9).expect("valid override");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.decimal_places().unwrap(), 9);
    assert_eq!(currency_metadata(code).unwrap().minor_units, 9);

    // Updating presentation metadata with the same scale is still accepted.
    call_set_metadata(code, "Renamed Token", 9).expect("same-scale update");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.decimal_places().unwrap(), 9);
    assert_eq!(
        currency_metadata(code).unwrap().full_name.as_ref(),
        "Renamed Token"
    );

    let err = call_set_metadata(code, "Custom Token", 4).unwrap_err();
    assert!(matches!(
        err,
        MinorUnitError::MinorUnitsAlreadyRegistered {
            existing: 9,
            requested: 4,
            ..
        }
    ));

    clear_currency_metadata(code);
    assert!(currency_metadata(code).is_none());
}

#[test]
fn test_custom_currency_metadata_override_is_explicit() {
    let code = "custom_token_override";
    clear_currency_metadata(code);

    call_set_metadata(code, "Custom Token", 9).expect("valid registration");
    let previous = call_override_metadata(code, "Custom Token", 4).expect("explicit override");

    assert_eq!(previous.unwrap().minor_units, 9);
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.decimal_places().unwrap(), 4);
    assert_eq!(currency_metadata(code).unwrap().minor_units, 4);

    clear_currency_metadata(code);
}

#[test]
fn test_iso_currency_metadata_override_cannot_change_iso_scale() {
    use iso_currency::Currency as IsoCurrency;

    clear_currency_metadata("USD");

    let err = call_override_metadata("USD", "Display USD", 4).unwrap_err();
    assert!(matches!(
        err,
        MinorUnitError::MinorUnitsAlreadyRegistered {
            code,
            existing: 2,
            requested: 4,
        } if code == "USD"
    ));

    assert_eq!(currency_metadata("USD").unwrap().minor_units, 2);
    assert_eq!(Currency::Iso(IsoCurrency::USD).decimal_places().unwrap(), 2);

    call_override_metadata("USD", "Display USD", 2).expect("same-scale display override");
    let metadata = currency_metadata("USD").expect("metadata present");
    assert_eq!(metadata.full_name.as_ref(), "Display USD");
    assert_eq!(metadata.minor_units, 2);
    assert_eq!(Currency::Iso(IsoCurrency::USD).decimal_places().unwrap(), 2);

    clear_currency_metadata("USD");
}

#[test]
fn test_currency_metadata_rejects_overflowing_precision() {
    let err = call_set_metadata("overflow_minor", "Token", 19).unwrap_err();
    assert!(matches!(
        err,
        MinorUnitError::ExceedsMinorUnitScale { decimals } if decimals == 19
    ));

    let err = call_set_metadata("overflow_decimal", "Token", 29).unwrap_err();
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

#[test]
fn test_currency_metadata_rejects_empty_canonical_codes() {
    for code in ["", "   ", "!@#", "€—"] {
        let err = call_set_metadata(code, "Token", 2).unwrap_err();
        assert!(matches!(
            err,
            MinorUnitError::InvalidCurrencyCode { code: rejected } if rejected == code
        ));
        assert!(currency_metadata(code).is_none());
    }

    let err = call_set_metadata("", "Token", 19).unwrap_err();
    assert!(matches!(
        err,
        MinorUnitError::InvalidCurrencyCode { code } if code.is_empty()
    ));
}

#[cfg(feature = "bigdecimal")]
#[test]
fn test_bigdecimal_accepts_large_magnitudes() {
    call_set_metadata("HP18", "High Precision", 18).expect("metadata accepted");
    let currency = Currency::try_from_str("HP18").unwrap();

    let amount = "123456789012345678901234567890.123456789012345678";
    let money = Money::from_canonical_str(amount, currency).unwrap();
    assert_eq!(money.amount().to_string(), amount);

    clear_currency_metadata("HP18");
}

#[test]
fn test_currency_metadata_accepts_boundary_precision() {
    let code = "boundary_precision";
    clear_currency_metadata(code);

    // 18 is the highest supported minor-unit precision and should be accepted.
    call_set_metadata(code, "Boundary Token", 18).expect("18 decimals should be accepted");
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

    call_set_metadata(code, "Custom Token", 4).expect("metadata set");
    let currency = Currency::try_from_str(code).unwrap();
    assert_eq!(currency.full_name().as_ref(), "Custom Token");
    assert_eq!(currency.decimal_places().unwrap(), 4);

    // Updating metadata with the same scale should preserve the name.
    call_set_metadata(code, "Custom Token", 4).expect("same-scale update");
    let metadata = currency_metadata(code).expect("metadata present");
    assert_eq!(metadata.full_name.as_ref(), "Custom Token");
    assert_eq!(metadata.minor_units, 4);

    clear_currency_metadata(code);
}

#[test]
fn test_non_iso_decimal_places_route_through_registry() {
    // BTC, ETH, XMR, USDC, USDT no longer have hard-coded scales — the
    // values must come from `BUILTIN_CURRENCY_METADATA`. If a future
    // change drifts those entries, this test will catch the regression.
    assert_eq!(Currency::BTC.decimal_places().unwrap(), 8);
    assert_eq!(Currency::ETH.decimal_places().unwrap(), 18);
    assert_eq!(Currency::XMR.decimal_places().unwrap(), 12);
    assert_eq!(Currency::USDC.decimal_places().unwrap(), 6);
    assert_eq!(Currency::USDT.decimal_places().unwrap(), 6);

    // The values must match what's registered for the same code, with
    // no fallback to the (now removed) hardcoded arms.
    assert_eq!(currency_metadata("BTC").unwrap().minor_units, 8);
    assert_eq!(currency_metadata("ETH").unwrap().minor_units, 18);
    assert_eq!(currency_metadata("XMR").unwrap().minor_units, 12);
    assert_eq!(currency_metadata("USDC").unwrap().minor_units, 6);
    assert_eq!(currency_metadata("USDT").unwrap().minor_units, 6);
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

    call_set_metadata("XAU", "Gold", 3).unwrap();
    call_set_metadata("XDR", "SDR", 6).unwrap();

    assert_eq!(xau.decimal_places().unwrap(), 3);
    assert_eq!(xdr.decimal_places().unwrap(), 6);

    clear_currency_metadata("XAU");
    clear_currency_metadata("XDR");
}
