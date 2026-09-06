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
    with_figi.figi = Some(Figi::new("BBG000B9Y5X2").unwrap());

    let mut with_isin = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
    with_isin.isin = Some(Isin::new("US0378331005").unwrap());

    assert_eq!(with_figi.unique_key(), "EQUITY|FIGI|BBG000B9Y5X2");
    assert_eq!(with_isin.unique_key(), "EQUITY|ISIN|US0378331005");
}

#[test]
fn display_key_keeps_compact_identifier_format() {
    let instrument =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity).unwrap();

    assert_eq!(instrument.display_key().as_ref(), "AAPL@NASDAQ");
    assert_eq!(instrument.to_string(), "AAPL@NASDAQ");
}

#[test]
fn security_and_listing_keys_separate_issues_from_venues() {
    let mut nasdaq =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity).unwrap();
    nasdaq.isin = Some(Isin::new("US0378331005").unwrap());
    let mut other_venue = nasdaq.clone();
    other_venue.exchange = Some(Exchange::other("VENUE2").unwrap());
    assert_eq!(
        nasdaq.security_key().as_deref(),
        Some("SECURITY|6:EQUITY|ISIN|US0378331005")
    );
    assert_eq!(nasdaq.security_key(), other_venue.security_key());
    assert_ne!(nasdaq.listing_key(), other_venue.listing_key());
    assert_eq!(
        nasdaq.listing_key().as_deref(),
        Some("LISTING|6:EQUITY|SYMBOL|4:AAPL|EXCHANGE|6:NASDAQ")
    );

    // Even a duplicated FIGI cannot erase supplied venue context.
    nasdaq.figi = Some(Figi::new("BBG000B9Y5X2").unwrap());
    other_venue.figi = nasdaq.figi.clone();
    assert_ne!(nasdaq.listing_key(), other_venue.listing_key());
    assert_eq!(nasdaq.security_key(), other_venue.security_key());
    other_venue.kind = AssetKind::Bond;
    assert_ne!(nasdaq.security_key(), other_venue.security_key());
}

#[test]
fn incomplete_context_does_not_claim_an_identity() {
    let mut instrument = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
    assert_eq!(instrument.security_key(), None);
    assert_eq!(instrument.listing_key(), None);
    instrument.figi = Some(Figi::new("BBG000B9Y5X2").unwrap());
    assert_eq!(instrument.security_key(), None);
    assert_eq!(instrument.listing_key(), None);
    instrument.exchange = Some(Exchange::NASDAQ);
    assert!(instrument.listing_key().is_some());
}

#[test]
fn same_issue_and_venue_can_have_distinct_symbol_listings() {
    let mut first =
        Instrument::from_symbol_and_exchange("LINE_A", Exchange::NASDAQ, AssetKind::Equity)
            .unwrap();
    first.isin = Some(Isin::new("US0378331005").unwrap());
    let mut second = first.clone();
    second.symbol = paft_domain::Symbol::new("LINE_B").unwrap();
    assert_eq!(first.security_key(), second.security_key());
    assert_ne!(first.listing_key(), second.listing_key());
}
