use paft_core::domain::{AssetKind, Exchange, Instrument};

// -----------------
// AssetKind tests
// -----------------

#[test]
fn asset_kind_variants() {
    // Test all variants exist and are distinct
    let variants = [
        AssetKind::Equity,
        AssetKind::Crypto,
        AssetKind::Fund,
        AssetKind::Index,
        AssetKind::Forex,
        AssetKind::Bond,
        AssetKind::Commodity,
        AssetKind::Option,
        AssetKind::Future,
        AssetKind::REIT,
        AssetKind::Warrant,
        AssetKind::Convertible,
    ];

    // Test that all variants are different
    for (i, variant1) in variants.iter().enumerate() {
        for (j, variant2) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(variant1, variant2);
            }
        }
    }
}

#[test]
fn asset_kind_debug_formatting() {
    let equity = AssetKind::Equity;
    let debug_str = format!("{equity:?}");
    assert_eq!(debug_str, "Equity");

    let crypto = AssetKind::Crypto;
    let debug_str = format!("{crypto:?}");
    assert_eq!(debug_str, "Crypto");
}

#[test]
fn asset_kind_full_name_is_human_friendly() {
    assert_eq!(AssetKind::Equity.full_name(), "Equity");
    assert_eq!(AssetKind::PerpetualFuture.full_name(), "Perpetual Future");
    assert_eq!(AssetKind::LST.full_name(), "Liquid Staking Token");
}

#[test]
fn asset_kind_clone() {
    let original = AssetKind::Equity;
    let cloned = original.clone();
    let copied = original.clone(); // Clone trait (no longer Copy)

    assert_eq!(original, cloned);
    assert_eq!(original, copied);
    assert_eq!(cloned, copied);
}

#[test]
fn asset_kind_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(AssetKind::Equity, "stock");
    map.insert(AssetKind::Crypto, "cryptocurrency");

    assert_eq!(map.get(&AssetKind::Equity), Some(&"stock"));
    assert_eq!(map.get(&AssetKind::Crypto), Some(&"cryptocurrency"));
    assert_eq!(map.get(&AssetKind::Forex), None);
}

#[test]
fn asset_kind_serialization() {
    let asset_kind = AssetKind::Equity;
    let json = serde_json::to_string(&asset_kind).unwrap();
    let deserialized: AssetKind = serde_json::from_str(&json).unwrap();
    assert_eq!(asset_kind, deserialized);
}

#[test]
fn asset_kind_all_variants_serialization() {
    let variants = [
        AssetKind::Equity,
        AssetKind::Crypto,
        AssetKind::Fund,
        AssetKind::Index,
        AssetKind::Forex,
        AssetKind::Bond,
        AssetKind::Commodity,
        AssetKind::Option,
        AssetKind::Future,
        AssetKind::REIT,
        AssetKind::Warrant,
        AssetKind::Convertible,
        AssetKind::NFT,
        AssetKind::PerpetualFuture,
        AssetKind::LeveragedToken,
        AssetKind::LPToken,
        AssetKind::LST,
        AssetKind::RWA,
    ];

    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: AssetKind = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, deserialized);
    }
}

// -----------------
// Instrument tests
// -----------------

#[test]
fn instrument_construction() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    assert_eq!(instrument.figi(), Some("BBG000B9XRY4"));
    assert_eq!(instrument.symbol(), "AAPL");
    assert_eq!(instrument.exchange(), Some(&Exchange::NASDAQ));
    assert_eq!(instrument.kind(), &AssetKind::Equity);
}

#[test]
fn instrument_crypto_construction() {
    let instrument = Instrument::new(
        "BTC-USD",
        AssetKind::Crypto,
        Some("BBG000B9XRY5".to_string()),
        None,
        Some(Exchange::try_from_str("COINBASE").unwrap()),
    );
    assert_eq!(instrument.figi(), Some("BBG000B9XRY5"));
    assert_eq!(instrument.symbol(), "BTC-USD");
    assert_eq!(
        instrument.exchange(),
        Some(&Exchange::try_from_str("COINBASE").unwrap())
    );
    assert_eq!(instrument.kind(), &AssetKind::Crypto);
}

#[test]
fn instrument_string_conversion() {
    let symbol_str = "AAPL".to_string();
    let instrument = Instrument::new(
        symbol_str.clone(),
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    assert_eq!(instrument.symbol(), &symbol_str);
}

#[test]
fn instrument_clone() {
    let original = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    let cloned = original.clone();

    assert_eq!(original.figi(), cloned.figi());
    assert_eq!(original.symbol(), cloned.symbol());
    assert_eq!(original.exchange(), cloned.exchange());
    assert_eq!(original.kind(), cloned.kind());
    assert_eq!(original, cloned);
}

#[test]
fn instrument_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let instrument1 = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    let instrument2 = Instrument::new(
        "BTC-USD",
        AssetKind::Crypto,
        Some("BBG000B9XRY5".to_string()),
        None,
        Some(Exchange::try_from_str("COINBASE").unwrap()),
    );

    map.insert(instrument1.clone(), "Apple");
    map.insert(instrument2.clone(), "Bitcoin");

    assert_eq!(map.get(&instrument1), Some(&"Apple"));
    assert_eq!(map.get(&instrument2), Some(&"Bitcoin"));
}

#[test]
fn instrument_debug_formatting() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    let debug_str = format!("{instrument:?}");
    assert!(debug_str.contains("BBG000B9XRY4"));
    assert!(debug_str.contains("AAPL"));
    assert!(debug_str.contains("NASDAQ"));
    assert!(debug_str.contains("Equity"));
}

#[test]
fn instrument_unique_key() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    assert_eq!(instrument.unique_key(), "BBG000B9XRY4");
}

#[test]
fn instrument_getters() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    assert_eq!(instrument.figi(), Some("BBG000B9XRY4"));
    assert_eq!(instrument.symbol(), "AAPL");
    assert_eq!(instrument.exchange(), Some(&Exchange::NASDAQ));
    assert_eq!(instrument.kind(), &AssetKind::Equity);
}

#[test]
fn instrument_equality_with_different_figi() {
    let instrument1 = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    let instrument2 = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY5".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    assert_ne!(instrument1, instrument2);
}

#[test]
fn instrument_equality_with_same_figi() {
    let instrument1 = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    let instrument2 = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    assert_eq!(instrument1, instrument2);
}

#[test]
fn instrument_equality_with_different_exchange() {
    let instrument1 = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    let instrument2 = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NYSE),
    );
    assert_ne!(instrument1, instrument2);
}

#[test]
fn instrument_serialization() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    let json = serde_json::to_string(&instrument).unwrap();
    let deserialized: Instrument = serde_json::from_str(&json).unwrap();
    assert_eq!(instrument, deserialized);
}

#[test]
fn instrument_serialization_with_other_exchange() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::try_from_str("CUSTOM").unwrap()),
    );
    let json = serde_json::to_string(&instrument).unwrap();
    let deserialized: Instrument = serde_json::from_str(&json).unwrap();
    assert_eq!(instrument, deserialized);
}

// Tests for hierarchical identifier functionality

#[test]
fn instrument_unique_key_with_figi() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        Some("US0378331005".to_string()),
        Some(Exchange::NASDAQ),
    );
    assert_eq!(instrument.unique_key(), "BBG000B9XRY4");
}

#[test]
fn instrument_unique_key_with_isin_only() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        None,
        Some("US0378331005".to_string()),
        Some(Exchange::NASDAQ),
    );
    assert_eq!(instrument.unique_key(), "US0378331005");
}

#[test]
fn instrument_unique_key_with_symbol_and_exchange() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        None,
        None,
        Some(Exchange::NASDAQ),
    );
    assert_eq!(instrument.unique_key(), "AAPL@NASDAQ");
}

#[test]
fn instrument_unique_key_symbol_only() {
    let instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
    assert_eq!(instrument.unique_key(), "AAPL");
}

#[test]
fn instrument_is_globally_identified() {
    let instrument_with_figi = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        None,
        Some(Exchange::NASDAQ),
    );
    assert!(instrument_with_figi.is_globally_identified());

    let instrument_with_isin = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        None,
        Some("US0378331005".to_string()),
        Some(Exchange::NASDAQ),
    );
    assert!(instrument_with_isin.is_globally_identified());

    let instrument_symbol_only = Instrument::from_symbol("AAPL", AssetKind::Equity);
    assert!(!instrument_symbol_only.is_globally_identified());
}

#[test]
fn instrument_backward_compatibility_constructors() {
    // Test from_symbol constructor
    let symbol_only = Instrument::from_symbol("AAPL", AssetKind::Equity);
    assert_eq!(symbol_only.symbol(), "AAPL");
    assert_eq!(symbol_only.kind(), &AssetKind::Equity);
    assert!(symbol_only.figi().is_none());
    assert!(symbol_only.exchange().is_none());

    // Test from_symbol_and_exchange constructor
    let symbol_exchange =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity);
    assert_eq!(symbol_exchange.symbol(), "AAPL");
    assert_eq!(symbol_exchange.exchange(), Some(&Exchange::NASDAQ));
    assert_eq!(symbol_exchange.kind(), &AssetKind::Equity);
    assert!(symbol_exchange.figi().is_none());
}

#[test]
fn instrument_has_methods() {
    let instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()),
        Some("US0378331005".to_string()),
        Some(Exchange::NASDAQ),
    );

    assert!(instrument.has_figi());
    assert!(instrument.has_isin());
    assert!(instrument.has_exchange());

    let minimal_instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
    assert!(!minimal_instrument.has_figi());
    assert!(!minimal_instrument.has_isin());
    assert!(!minimal_instrument.has_exchange());
}

#[test]
fn instrument_with_unicode_symbol() {
    let instrument = Instrument::new(
        "测试符号",
        AssetKind::Equity,
        Some("BBG000B9XRY6".to_string()),
        None,
        Some(Exchange::try_from_str("SHANGHAI").unwrap()),
    );
    let json = serde_json::to_string(&instrument).unwrap();
    let deserialized: Instrument = serde_json::from_str(&json).unwrap();
    assert_eq!(instrument, deserialized);
}
