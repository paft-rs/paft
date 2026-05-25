use chrono::DateTime;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Exchange, Instrument, MarketState};
use paft_market::market::orderbook::BookLevel;
use paft_market::market::quote::{Quote, QuoteUpdate};
use paft_money::{Currency, IsoCurrency, Price};
use std::str::FromStr;

// -----------------
// Quote tests
// -----------------

#[test]
fn quote_construction() {
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: Some("Apple Inc.".to_string()),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        day_volume: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: (),
    };

    assert_eq!(quote.instrument.unique_key().as_ref(), "AAPL");
    assert_eq!(quote.name, Some("Apple Inc.".to_string()));
    assert_eq!(
        quote.price,
        Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD)
        ))
    );
    assert_eq!(
        quote.previous_close,
        Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        ))
    );
    assert_eq!(quote.exchange, Some(Exchange::NASDAQ));
    assert_eq!(quote.market_state, Some(MarketState::Regular));
}

#[test]
fn quote_minimal_construction() {
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: None,
        price: None,
        previous_close: None,
        day_volume: None,
        exchange: None,
        market_state: None,
        bid: None,
        ask: None,
        provider: (),
    };
    assert_eq!(quote.instrument.unique_key().as_ref(), "AAPL");
    assert!(quote.name.is_none());
    assert!(quote.price.is_none());
    assert!(quote.previous_close.is_none());
    assert!(quote.exchange.is_none());
    assert!(quote.market_state.is_none());
}

#[test]
fn quote_clone() {
    let original = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: Some("Apple Inc.".to_string()),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        day_volume: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: (),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn quote_debug_formatting() {
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: Some("Apple Inc.".to_string()),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        day_volume: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: (),
    };

    let debug_str = format!("{quote:?}");
    assert!(debug_str.contains("AAPL"));
    assert!(debug_str.contains("150"));
}

#[test]
fn quote_currency_consistency() {
    // Test that currency is embedded in Price fields
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: Some("Apple Inc.".to_string()),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        day_volume: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: (),
    };

    // The currency should be accessible from the Price fields
    assert_eq!(
        quote.price.as_ref().unwrap().currency(),
        &Currency::Iso(IsoCurrency::USD)
    );
    assert_eq!(
        quote.previous_close.as_ref().unwrap().currency(),
        &Currency::Iso(IsoCurrency::USD)
    );
}

#[test]
fn quote_currency_none() {
    // Test that when no Price fields are present, currency access returns None
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: Some("Apple Inc.".to_string()),
        price: None, // No price fields
        previous_close: None,
        day_volume: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: (),
    };

    // Should return None when no Price fields are present
    assert!(quote.price.is_none());
    assert!(quote.previous_close.is_none());
}

#[test]
fn quote_price_fields() {
    // Test that Price fields work correctly
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: None,
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(147),
            Currency::Iso(IsoCurrency::USD),
        )),
        day_volume: None,
        exchange: None,
        market_state: None,
        bid: None,
        ask: None,
        provider: (),
    };

    // Test price
    let price_value = quote.price.as_ref().unwrap();
    assert_eq!(price_value.amount(), Decimal::from(150));
    assert_eq!(price_value.currency(), &Currency::Iso(IsoCurrency::USD));

    // Test previous_close
    let prev_close_price = quote.previous_close.as_ref().unwrap();
    assert_eq!(prev_close_price.amount(), Decimal::from(147));
    assert_eq!(
        prev_close_price.currency(),
        &Currency::Iso(IsoCurrency::USD)
    );

    // Test with None prices
    let quote_no_prices = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: None,
        price: None,
        previous_close: None,
        day_volume: None,
        exchange: None,
        market_state: None,
        bid: None,
        ask: None,
        provider: (),
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
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        volume: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),

        provider: (),
    };

    assert_eq!(update.instrument.unique_key().as_ref(), "AAPL");
    assert_eq!(
        update.price,
        Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD)
        ),)
    );
    assert_eq!(
        update.previous_close,
        Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        ),)
    );
    assert_eq!(update.ts.timestamp(), 1_640_995_200);
}

#[test]
fn quote_update_partial_fields() {
    let update = QuoteUpdate {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: None,
        volume: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),

        provider: (),
    };

    assert_eq!(update.instrument.unique_key().as_ref(), "AAPL");
    assert_eq!(
        update.price,
        Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD)
        ),)
    );
    assert_eq!(update.previous_close, None);
    assert_eq!(update.ts.timestamp(), 1_640_995_200);
}

#[test]
fn quote_update_clone() {
    let original = QuoteUpdate {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        volume: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),

        provider: (),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn quote_update_debug_formatting() {
    let update = QuoteUpdate {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        volume: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),

        provider: (),
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
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: Some("Apple Inc.".to_string()),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        day_volume: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: (),
    };

    let json = serde_json::to_string(&quote).unwrap();
    let deserialized: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, deserialized);
}

#[test]
fn quote_with_none_fields() {
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: None,
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(147),
            Currency::Iso(IsoCurrency::USD),
        )),
        day_volume: None,
        exchange: None,
        market_state: None,
        bid: None,
        ask: None,
        provider: (),
    };

    let json = serde_json::to_string(&quote).unwrap();
    let deserialized: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, deserialized);
}

#[test]
fn quote_update_serialization() {
    let update = QuoteUpdate {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        volume: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),

        provider: (),
    };

    let json = serde_json::to_string(&update).unwrap();
    let deserialized: QuoteUpdate = serde_json::from_str(&json).unwrap();
    assert_eq!(update, deserialized);
}

#[test]
fn quote_update_with_none_fields() {
    let update = QuoteUpdate {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        price: None,
        previous_close: None,
        volume: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),

        provider: (),
    };

    let json = serde_json::to_string(&update).unwrap();
    let deserialized: QuoteUpdate = serde_json::from_str(&json).unwrap();
    assert_eq!(update, deserialized);
}

#[test]
fn serialization_roundtrip_preserves_precision() {
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: Some("Apple Inc.".to_string()),
        price: Some(Price::new(
            Decimal::from_str("150.123456789").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Price::new(
            Decimal::from_str("147.135802469").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )),
        day_volume: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: (),
    };

    let json = serde_json::to_string(&quote).unwrap();
    let deserialized: Quote = serde_json::from_str(&json).unwrap();

    // Check that floating point precision is preserved
    assert_eq!(quote.price, deserialized.price);
    assert_eq!(quote.previous_close, deserialized.previous_close);
    assert_eq!(quote, deserialized);
}

#[test]
fn quote_with_bid_and_ask_roundtrips() {
    let bid = BookLevel {
        price: Price::new(Decimal::from(149), Currency::Iso(IsoCurrency::USD)),
        size: Some(Decimal::from(200)),
        provider: (),
    };
    let ask = BookLevel {
        price: Price::new(Decimal::from(151), Currency::Iso(IsoCurrency::USD)),
        size: None,
        provider: (),
    };
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: None,
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        bid: Some(bid),
        ask: Some(ask),
        previous_close: None,
        day_volume: None,
        exchange: None,
        market_state: None,
        provider: (),
    };
    let json = serde_json::to_string(&quote).unwrap();
    let decoded: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, decoded);
    let decoded_ask = decoded.ask.as_ref().unwrap();
    assert!(decoded_ask.size.is_none(), "ask size None preserved");
    let decoded_bid = decoded.bid.as_ref().unwrap();
    assert_eq!(decoded_bid.size, Some(Decimal::from(200)));
}

#[test]
fn quote_new_initialises_bid_and_ask_to_none() {
    let quote = Quote::new(Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap());
    assert!(quote.bid.is_none());
    assert!(quote.ask.is_none());
}

#[test]
fn deserialization_handles_missing_optional_fields() {
    // Test that missing optional fields are handled gracefully via roundtrip
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: None,
        price: Some(Price::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: None,
        day_volume: None,
        exchange: None,
        market_state: None,
        bid: None,
        ask: None,
        provider: (),
    };
    let json = serde_json::to_string(&quote).unwrap();
    let deserialized: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, quote);
}
