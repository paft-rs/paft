use paft::money::{
    Currency, CurrencyMetadata, Locale, MAX_DECIMAL_PRECISION, MAX_MINOR_UNIT_DECIMALS,
    MoneyParseError, PriceAmount, QuantityAmount, clear_currency_metadata, currency_metadata,
    set_currency_metadata,
};
use paft::prelude::{
    CurrencyMetadata as PreludeCurrencyMetadata, Locale as PreludeLocale,
    MAX_DECIMAL_PRECISION as PRELUDE_MAX_DECIMAL_PRECISION,
    MAX_MINOR_UNIT_DECIMALS as PRELUDE_MAX_MINOR_UNIT_DECIMALS, PriceAmount as PreludePriceAmount,
    QuantityAmount as PreludeQuantityAmount,
    set_currency_metadata as prelude_set_currency_metadata,
};

#[test]
fn facade_reexports_metadata_types_without_formatting() {
    let code = "facade_metadata";
    clear_currency_metadata(code);

    let previous: Option<CurrencyMetadata> =
        set_currency_metadata(code, "Facade Token", 4, "FT", true, Locale::EnUs)
            .expect("metadata registration should succeed");
    assert!(previous.is_none());

    let metadata: CurrencyMetadata = currency_metadata(code).expect("metadata should be present");
    assert_eq!(metadata.minor_units, 4);
    assert_eq!(metadata.default_locale, Locale::EnUs);

    let previous: Option<PreludeCurrencyMetadata> =
        prelude_set_currency_metadata(code, "Facade Token", 5, "FT", true, PreludeLocale::EnUs)
            .expect("metadata update should succeed");
    assert_eq!(
        previous
            .expect("previous metadata should be returned")
            .minor_units,
        4
    );

    clear_currency_metadata(code);
}

#[test]
fn facade_reexports_money_precision_limits() {
    assert_eq!(MAX_DECIMAL_PRECISION, PRELUDE_MAX_DECIMAL_PRECISION);
    assert_eq!(MAX_MINOR_UNIT_DECIMALS, PRELUDE_MAX_MINOR_UNIT_DECIMALS);
}

#[test]
fn facade_reexports_price_amount() {
    let amount: PriceAmount = PreludePriceAmount::new(paft::Decimal::from(123));
    assert_eq!(amount.as_decimal(), &paft::Decimal::from(123));
}

#[test]
fn facade_reexports_quantity_amount() {
    let amount: QuantityAmount =
        PreludeQuantityAmount::from_decimal(paft::Decimal::from(123)).unwrap();
    assert_eq!(amount.as_decimal(), &paft::Decimal::from(123));
}

#[test]
fn facade_reexports_currency_parse_error() {
    let err: MoneyParseError =
        Currency::try_from_str("").expect_err("empty currency should fail to parse");

    assert!(matches!(
        err,
        MoneyParseError::InvalidEnumValue {
            enum_name: "Currency",
            ..
        }
    ));
}

#[test]
fn facade_error_converts_minor_unit_errors() {
    fn configure_invalid_metadata() -> paft::Result<()> {
        set_currency_metadata("", "Invalid Token", 4, "IT", true, Locale::EnUs)?;
        Ok(())
    }

    let err = configure_invalid_metadata().expect_err("invalid metadata should fail");

    assert!(matches!(
        err,
        paft::Error::MinorUnit(paft::money::MinorUnitError::InvalidCurrencyCode { .. })
    ));
}
