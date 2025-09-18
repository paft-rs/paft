//! Tests for currency module

use paft_core::domain::Currency;

#[test]
fn test_currency_from_string() {
    assert_eq!(Currency::from("USD".to_string()), Currency::USD);
    assert_eq!(Currency::from("eur".to_string()), Currency::EUR);
    assert_eq!(Currency::from("GBP".to_string()), Currency::GBP);
    assert_eq!(Currency::from("JPY".to_string()), Currency::JPY);
    assert_eq!(Currency::from("CAD".to_string()), Currency::CAD);
    assert_eq!(Currency::from("BTC".to_string()), Currency::BTC);

    // Whitespace is trimmed before parsing
    assert_eq!(Currency::from("  usd  ".to_string()), Currency::USD);

    // Common aliases map to canonical variants
    assert_eq!(Currency::from("US_DOLLAR".to_string()), Currency::USD);
    assert_eq!(Currency::from("Pound Sterling".to_string()), Currency::GBP);

    // Unknown strings default to uppercase Other variant
    assert_eq!(
        Currency::from("custom coin".to_string()),
        Currency::Other("CUSTOM COIN".to_string())
    );
}

#[test]
fn test_currency_to_string() {
    assert_eq!(String::from(Currency::USD), "USD");
    assert_eq!(String::from(Currency::EUR), "EUR");
    assert_eq!(String::from(Currency::GBP), "GBP");
    assert_eq!(String::from(Currency::JPY), "JPY");
    assert_eq!(String::from(Currency::CAD), "CAD");
    assert_eq!(String::from(Currency::BTC), "BTC");
}

#[test]
fn test_currency_methods() {
    assert_eq!(Currency::USD.code(), "USD");
    assert_eq!(Currency::EUR.code(), "EUR");
    assert_eq!(Currency::Other("BTC".to_string()).code(), "BTC");

    assert!(Currency::USD.is_reserve_currency());
    assert!(Currency::EUR.is_reserve_currency());
    assert!(Currency::GBP.is_reserve_currency());
    assert!(Currency::JPY.is_reserve_currency());
    assert!(Currency::CHF.is_reserve_currency());
    assert!(!Currency::CAD.is_reserve_currency());
    assert!(!Currency::Other("BTC".to_string()).is_reserve_currency());
}

#[test]
fn test_currency_case_normalization() {
    // Test that different case variations of the same string are normalized to uppercase
    let currency1 = Currency::from("btc".to_string());
    let currency2 = Currency::from("BTC".to_string());
    let currency3 = Currency::from("Btc".to_string());
    let currency4 = Currency::from("bTc".to_string());

    // All should be equal since they're normalized to uppercase
    assert_eq!(currency1, currency2);
    assert_eq!(currency2, currency3);
    assert_eq!(currency3, currency4);

    // All should be BTC variant
    assert_eq!(currency1, Currency::BTC);
    assert_eq!(currency2, Currency::BTC);
    assert_eq!(currency3, Currency::BTC);
    assert_eq!(currency4, Currency::BTC);

    // Test with ETH (which is now a proper variant)
    let crypto1 = Currency::from("eth".to_string());
    let crypto2 = Currency::from("ETH".to_string());
    let crypto3 = Currency::from("Eth".to_string());

    assert_eq!(crypto1, crypto2);
    assert_eq!(crypto2, crypto3);
    assert_eq!(crypto1, Currency::ETH);

    // Test with a truly unknown currency
    let unknown1 = Currency::from("ethereum".to_string());
    let unknown2 = Currency::from("ETHEREUM".to_string());
    let unknown3 = Currency::from("Ethereum".to_string());

    assert_eq!(unknown1, unknown2);
    assert_eq!(unknown2, unknown3);
    assert_eq!(unknown1, Currency::Other("ETHEREUM".to_string()));
}

#[test]
fn test_cryptocurrency_decimal_places() {
    // Test that cryptocurrencies have correct decimal places
    assert_eq!(Currency::BTC.decimal_places(), 8);
    assert_eq!(Currency::ETH.decimal_places(), 18);
    assert_eq!(Currency::XMR.decimal_places(), 12);

    // Test minor unit scale
    assert_eq!(Currency::BTC.minor_unit_scale(), 100_000_000); // 10^8
    assert_eq!(Currency::ETH.minor_unit_scale(), 1_000_000_000_000_000_000); // 10^18
    assert_eq!(Currency::XMR.minor_unit_scale(), 1_000_000_000_000); // 10^12
}

#[test]
fn test_decimal_places_for_fiat_outliers() {
    assert_eq!(Currency::JPY.decimal_places(), 0);
    assert_eq!(Currency::KRW.decimal_places(), 0);
    assert_eq!(Currency::IDR.decimal_places(), 2);
    assert_eq!(Currency::HUF.decimal_places(), 2);
}
