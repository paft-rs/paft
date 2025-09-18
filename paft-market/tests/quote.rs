use chrono::DateTime;
use paft_core::domain::{Currency, Exchange, MarketState, Money};
use paft_market::market::quote::{Quote, QuoteUpdate};
use rust_decimal::Decimal;
use std::str::FromStr;

// -----------------
// Quote tests
// -----------------

#[test]
fn quote_construction() {
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };

    assert_eq!(quote.symbol, "AAPL");
    assert_eq!(quote.shortname, Some("Apple Inc.".to_string()));
    assert_eq!(
        quote.price,
        Some(Money::new(Decimal::from(150), Currency::USD))
    );
    assert_eq!(
        quote.previous_close,
        Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD
        ))
    );
    assert_eq!(quote.exchange, Some(Exchange::NASDAQ));
    assert_eq!(quote.market_state, Some(MarketState::Regular));
}

#[test]
fn quote_empty_construction() {
    let quote = Quote {
        symbol: String::new(),
        shortname: None,
        price: None,
        previous_close: None,
        exchange: None,
        market_state: None,
    };
    assert_eq!(quote.symbol, "");
    assert_eq!(quote.shortname, None);
    assert_eq!(quote.price, None);
    assert_eq!(quote.previous_close, None);
    assert_eq!(quote.exchange, None);
    assert_eq!(quote.market_state, None);
}

#[test]
fn quote_clone() {
    let original = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn quote_debug_formatting() {
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };

    let debug_str = format!("{quote:?}");
    assert!(debug_str.contains("AAPL"));
    assert!(debug_str.contains("150"));
}

#[test]
fn quote_currency_consistency() {
    // Test that currency is embedded in Money fields
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };

    // The currency should be accessible from the Money fields
    assert_eq!(quote.price.as_ref().unwrap().currency(), &Currency::USD);
    assert_eq!(
        quote.previous_close.as_ref().unwrap().currency(),
        &Currency::USD
    );
}

#[test]
fn quote_currency_none() {
    // Test that when no Money fields are present, currency access returns None
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: None, // No price fields
        previous_close: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };

    // Should return None when no Money fields are present
    assert!(quote.price.is_none());
    assert!(quote.previous_close.is_none());
}

#[test]
fn quote_money_fields() {
    // Test that Money fields work correctly
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: None,
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(Decimal::from(147), Currency::USD)),
        exchange: None,
        market_state: None,
    };

    // Test price
    let price_money = quote.price.as_ref().unwrap();
    assert_eq!(price_money.amount(), Decimal::from(150));
    assert_eq!(price_money.currency(), &Currency::USD);

    // Test previous_close
    let prev_close_money = quote.previous_close.as_ref().unwrap();
    assert_eq!(prev_close_money.amount(), Decimal::from(147));
    assert_eq!(prev_close_money.currency(), &Currency::USD);

    // Test with None prices
    let quote_no_prices = Quote {
        symbol: "AAPL".to_string(),
        shortname: None,
        price: None,
        previous_close: None,
        exchange: None,
        market_state: None,
    };

    // Should be None when prices are None
    assert!(quote_no_prices.price.is_none());
    assert!(quote_no_prices.previous_close.is_none());
}

// -----------------
// QuoteUpdate tests
// -----------------

#[test]
fn quote_update_construction() {
    let update = QuoteUpdate {
        symbol: "AAPL".to_string(),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    };

    assert_eq!(update.symbol, "AAPL");
    assert_eq!(
        update.price,
        Some(Money::new(Decimal::from(150), Currency::USD))
    );
    assert_eq!(
        update.previous_close,
        Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD
        ))
    );
    assert_eq!(update.ts.timestamp(), 1_640_995_200);
}

#[test]
fn quote_update_partial_fields() {
    let update = QuoteUpdate {
        symbol: "AAPL".to_string(),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    };

    assert_eq!(update.symbol, "AAPL");
    assert_eq!(
        update.price,
        Some(Money::new(Decimal::from(150), Currency::USD))
    );
    assert_eq!(update.previous_close, None);
    assert_eq!(update.ts.timestamp(), 1_640_995_200);
}

#[test]
fn quote_update_clone() {
    let original = QuoteUpdate {
        symbol: "AAPL".to_string(),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn quote_update_debug_formatting() {
    let update = QuoteUpdate {
        symbol: "AAPL".to_string(),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    };

    let debug_str = format!("{update:?}");
    assert!(debug_str.contains("AAPL"));
    assert!(debug_str.contains("150"));
}

// -----------------
// Serialization tests for Quote types
// -----------------

#[test]
fn quote_serialization() {
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };

    let json = serde_json::to_string(&quote).unwrap();
    let deserialized: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, deserialized);
}

#[test]
fn quote_with_none_fields() {
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: None,
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: None,
        exchange: None,
        market_state: None,
    };

    let json = serde_json::to_string(&quote).unwrap();
    let deserialized: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, deserialized);
}

#[test]
fn quote_update_serialization() {
    let update = QuoteUpdate {
        symbol: "AAPL".to_string(),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    };

    let json = serde_json::to_string(&update).unwrap();
    let deserialized: QuoteUpdate = serde_json::from_str(&json).unwrap();
    assert_eq!(update, deserialized);
}

#[test]
fn quote_update_with_none_fields() {
    let update = QuoteUpdate {
        symbol: "AAPL".to_string(),
        price: None,
        previous_close: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    };

    let json = serde_json::to_string(&update).unwrap();
    let deserialized: QuoteUpdate = serde_json::from_str(&json).unwrap();
    assert_eq!(update, deserialized);
}

#[test]
fn serialization_roundtrip_preserves_precision() {
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(
            Decimal::from_str("150.123456789").unwrap(),
            Currency::USD,
        )),
        previous_close: Some(Money::new(
            Decimal::from_str("147.135802469").unwrap(),
            Currency::USD,
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };

    let json = serde_json::to_string(&quote).unwrap();
    let deserialized: Quote = serde_json::from_str(&json).unwrap();

    // Check that floating point precision is preserved
    assert_eq!(quote.price, deserialized.price);
    assert_eq!(quote.previous_close, deserialized.previous_close);
    assert_eq!(quote, deserialized);
}

#[test]
fn deserialization_handles_missing_optional_fields() {
    // Test that missing optional fields are handled gracefully
    let json_without_optional_fields = r#"{
        "symbol": "AAPL",
        "price": {
            "amount": "150",
            "currency": "USD"
        }
    }"#;

    let deserialized: Quote = serde_json::from_str(json_without_optional_fields).unwrap();
    assert_eq!(deserialized.symbol, "AAPL");
    assert_eq!(
        deserialized.price,
        Some(Money::new(Decimal::from(150), Currency::USD))
    );
    assert_eq!(deserialized.shortname, None);
    assert_eq!(deserialized.previous_close, None);
    assert_eq!(deserialized.exchange, None);
    assert_eq!(deserialized.market_state, None);
}
