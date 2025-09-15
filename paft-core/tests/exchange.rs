//! Tests for exchange module

use paft_core::domain::Exchange;

#[test]
fn test_exchange_from_string() {
    assert_eq!(Exchange::from("NASDAQ".to_string()), Exchange::NASDAQ);
    assert_eq!(Exchange::from("nyse".to_string()), Exchange::NYSE);
    assert_eq!(Exchange::from("AMEX".to_string()), Exchange::AMEX);
    assert_eq!(Exchange::from("BATS".to_string()), Exchange::BATS);
    assert_eq!(Exchange::from("OTC".to_string()), Exchange::OTC);
    assert_eq!(
        Exchange::from("CRYPTO".to_string()),
        Exchange::Other("CRYPTO".to_string())
    );
}

#[test]
fn test_exchange_to_string() {
    assert_eq!(String::from(Exchange::NASDAQ), "NASDAQ");
    assert_eq!(String::from(Exchange::NYSE), "NYSE");
    assert_eq!(String::from(Exchange::AMEX), "AMEX");
    assert_eq!(String::from(Exchange::BATS), "BATS");
    assert_eq!(String::from(Exchange::OTC), "OTC");
    assert_eq!(
        String::from(Exchange::Other("CRYPTO".to_string())),
        "CRYPTO"
    );
}

#[test]
fn test_exchange_methods() {
    assert_eq!(Exchange::NASDAQ.code(), "NASDAQ");
    assert_eq!(Exchange::NYSE.code(), "NYSE");
    assert_eq!(Exchange::Other("CRYPTO".to_string()).code(), "CRYPTO");

    assert!(Exchange::NASDAQ.is_us_exchange());
    assert!(Exchange::NYSE.is_us_exchange());
    assert!(Exchange::AMEX.is_us_exchange());
    assert!(Exchange::BATS.is_us_exchange());
    assert!(Exchange::OTC.is_us_exchange());
    assert!(!Exchange::LSE.is_us_exchange());
    assert!(!Exchange::Other("CRYPTO".to_string()).is_us_exchange());

    assert!(Exchange::LSE.is_european_exchange());
    assert!(Exchange::Euronext.is_european_exchange());
    assert!(Exchange::XETRA.is_european_exchange());
    assert!(Exchange::SIX.is_european_exchange());
    assert!(!Exchange::NASDAQ.is_european_exchange());
    assert!(!Exchange::Other("CRYPTO".to_string()).is_european_exchange());
}

#[test]
fn test_exchange_case_normalization() {
    // Test that different case variations of the same string are normalized to uppercase
    let exchange1 = Exchange::from("crypto".to_string());
    let exchange2 = Exchange::from("CRYPTO".to_string());
    let exchange3 = Exchange::from("Crypto".to_string());
    let exchange4 = Exchange::from("cRyPtO".to_string());

    // All should be equal since they're normalized to uppercase
    assert_eq!(exchange1, exchange2);
    assert_eq!(exchange2, exchange3);
    assert_eq!(exchange3, exchange4);

    // All should store "CRYPTO" in uppercase
    assert_eq!(exchange1, Exchange::Other("CRYPTO".to_string()));
    assert_eq!(exchange2, Exchange::Other("CRYPTO".to_string()));
    assert_eq!(exchange3, Exchange::Other("CRYPTO".to_string()));
    assert_eq!(exchange4, Exchange::Other("CRYPTO".to_string()));

    // Test with a longer string
    let custom1 = Exchange::from("binance".to_string());
    let custom2 = Exchange::from("BINANCE".to_string());
    let custom3 = Exchange::from("Binance".to_string());

    assert_eq!(custom1, custom2);
    assert_eq!(custom2, custom3);
    assert_eq!(custom1, Exchange::Other("BINANCE".to_string()));
}
