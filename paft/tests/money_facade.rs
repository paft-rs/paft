use paft::money::{
    CurrencyMetadata, Locale, clear_currency_metadata, currency_metadata, set_currency_metadata,
};
use paft::prelude::{
    CurrencyMetadata as PreludeCurrencyMetadata, Locale as PreludeLocale,
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
