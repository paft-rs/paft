use paft_domain::{AssetKind, Exchange, Figi, Instrument, Isin};

#[test]
fn unique_key_distinguishes_asset_kind_for_symbol_identity() {
    let crypto = Instrument::from_symbol("BTC", AssetKind::Crypto).unwrap();
    let equity = Instrument::from_symbol("BTC", AssetKind::Equity).unwrap();

    assert_ne!(crypto.unique_key(), equity.unique_key());
    assert_eq!(crypto.unique_key(), "CRYPTO|SYMBOL|3:BTC");
    assert_eq!(equity.unique_key(), "EQUITY|SYMBOL|3:BTC");
}

#[test]
fn unique_key_distinguishes_asset_kind_for_symbol_exchange_identity() {
    let crypto =
        Instrument::from_symbol_and_exchange("BTC", Exchange::NASDAQ, AssetKind::Crypto).unwrap();
    let equity =
        Instrument::from_symbol_and_exchange("BTC", Exchange::NASDAQ, AssetKind::Equity).unwrap();

    assert_ne!(crypto.unique_key(), equity.unique_key());
    assert_eq!(crypto.unique_key(), "CRYPTO|SYMBOL|3:BTC|EXCHANGE|NASDAQ");
    assert_eq!(equity.unique_key(), "EQUITY|SYMBOL|3:BTC|EXCHANGE|NASDAQ");
}

#[test]
fn unique_key_does_not_collapse_symbol_with_embedded_exchange_separator() {
    let symbol_only = Instrument::from_symbol("BTC@NASDAQ", AssetKind::Crypto).unwrap();
    let exchange_scoped =
        Instrument::from_symbol_and_exchange("BTC", Exchange::NASDAQ, AssetKind::Crypto).unwrap();

    assert_ne!(symbol_only.unique_key(), exchange_scoped.unique_key());
    assert_eq!(symbol_only.unique_key(), "CRYPTO|SYMBOL|10:BTC@NASDAQ");
}

#[test]
fn unique_key_namespaces_global_identifiers() {
    let mut with_figi = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
    with_figi.figi = Some(Figi::new("BBG000B9XRY4").unwrap());

    let mut with_isin = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
    with_isin.isin = Some(Isin::new("US0378331005").unwrap());

    assert_eq!(with_figi.unique_key(), "EQUITY|FIGI|BBG000B9XRY4");
    assert_eq!(with_isin.unique_key(), "EQUITY|ISIN|US0378331005");
}

#[test]
fn display_key_keeps_compact_identifier_format() {
    let instrument =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity).unwrap();

    assert_eq!(instrument.display_key().as_ref(), "AAPL@NASDAQ");
    assert_eq!(instrument.to_string(), "AAPL@NASDAQ");
}
