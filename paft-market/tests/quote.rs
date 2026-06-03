use chrono::DateTime;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Exchange, Instrument, MarketState};
use paft_market::market::orderbook::BookLevel;
use paft_market::market::quote::{Quote, QuoteUpdate};
use paft_money::{Currency, IsoCurrency, PriceAmount, QuantityAmount};
use std::str::FromStr;

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: impl Into<Decimal>) -> PriceAmount {
    PriceAmount::new(value.into())
}

fn quantity(value: impl Into<Decimal>) -> QuantityAmount {
    QuantityAmount::from_decimal(value.into()).unwrap()
}

fn aapl() -> Instrument {
    Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap()
}

fn aapl_nasdaq() -> Instrument {
    Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity).unwrap()
}

#[test]
fn quote_construction() {
    let quote = Quote {
        instrument: aapl_nasdaq(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        day_volume: Some(quantity(Decimal::from_str("12345.678").unwrap())),
        market_state: Some(MarketState::Regular),
        as_of: Some(DateTime::from_timestamp(1_640_995_200, 123_000_000).unwrap()),
        bid: None,
        ask: None,
        provider: (),
    };

    assert_eq!(
        quote.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL|EXCHANGE|NASDAQ"
    );
    assert_eq!(quote.name, Some("Apple Inc.".to_string()));
    assert_eq!(quote.currency, usd());
    assert_eq!(quote.price, Some(amount(150)));
    assert_eq!(
        quote.previous_close,
        Some(amount(Decimal::from(1475) / Decimal::from(10)))
    );
    assert_eq!(quote.instrument.exchange, Some(Exchange::NASDAQ));
    assert_eq!(
        quote.day_volume.as_ref().unwrap().as_decimal(),
        &Decimal::from_str("12345.678").unwrap()
    );
    assert_eq!(quote.market_state, Some(MarketState::Regular));
    assert_eq!(quote.as_of.unwrap().timestamp_millis(), 1_640_995_200_123);
}

#[test]
fn quote_minimal_construction_still_requires_currency() {
    let quote = Quote {
        instrument: aapl(),
        name: None,
        currency: usd(),
        price: None,
        previous_close: None,
        day_volume: None,
        market_state: None,
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };
    assert_eq!(
        quote.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL"
    );
    assert_eq!(quote.currency, usd());
    assert!(quote.name.is_none());
    assert!(quote.price.is_none());
    assert!(quote.previous_close.is_none());
    assert!(quote.instrument.exchange.is_none());
    assert!(quote.market_state.is_none());
    assert!(quote.as_of.is_none());
}

#[test]
fn quote_clone() {
    let original = Quote {
        instrument: aapl_nasdaq(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
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
        instrument: aapl_nasdaq(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    let debug_str = format!("{quote:?}");
    assert!(debug_str.contains("AAPL"));
    assert!(debug_str.contains("150"));
}

#[test]
fn quote_currency_is_record_context() {
    let quote = Quote {
        instrument: aapl_nasdaq(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    assert_eq!(quote.currency, usd());
    assert_eq!(
        quote
            .price
            .as_ref()
            .unwrap()
            .with_currency(quote.currency.clone()),
        paft_money::Price::new(Decimal::from(150), usd())
    );
}

#[test]
fn quote_without_prices_still_has_currency() {
    let quote = Quote {
        instrument: aapl_nasdaq(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: None,
        previous_close: None,
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    assert_eq!(quote.currency, usd());
    assert!(quote.price.is_none());
    assert!(quote.previous_close.is_none());
}

#[test]
fn quote_price_amount_fields() {
    let quote = Quote {
        instrument: aapl(),
        name: None,
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(147)),
        day_volume: None,
        market_state: None,
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    assert_eq!(
        quote.price.as_ref().unwrap().as_decimal(),
        &Decimal::from(150)
    );
    assert_eq!(
        quote.previous_close.as_ref().unwrap().as_decimal(),
        &Decimal::from(147)
    );

    let quote_no_prices = Quote {
        instrument: aapl(),
        name: None,
        currency: usd(),
        price: None,
        previous_close: None,
        day_volume: None,
        market_state: None,
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    assert!(quote_no_prices.price.is_none());
    assert!(quote_no_prices.previous_close.is_none());
}

#[test]
fn quote_update_construction() {
    let update = QuoteUpdate {
        instrument: aapl(),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        volume_delta: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        provider: (),
    };

    assert_eq!(
        update.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL"
    );
    assert_eq!(update.currency, usd());
    assert_eq!(update.price, Some(amount(150)));
    assert_eq!(
        update.previous_close,
        Some(amount(Decimal::from(1475) / Decimal::from(10)))
    );
    assert_eq!(update.ts.timestamp(), 1_640_995_200);
}

#[test]
fn quote_update_partial_fields() {
    let update = QuoteUpdate {
        instrument: aapl(),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: None,
        volume_delta: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        provider: (),
    };

    assert_eq!(
        update.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL"
    );
    assert_eq!(update.price, Some(amount(150)));
    assert_eq!(update.previous_close, None);
    assert_eq!(update.ts.timestamp(), 1_640_995_200);
}

#[test]
fn quote_update_clone() {
    let original = QuoteUpdate {
        instrument: aapl(),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        volume_delta: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        provider: (),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn quote_update_debug_formatting() {
    let update = QuoteUpdate {
        instrument: aapl(),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        volume_delta: None,
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        provider: (),
    };

    let debug_str = format!("{update:?}");
    assert!(debug_str.contains("AAPL"));
    assert!(debug_str.contains("150"));
}

#[test]
fn quote_serialization() {
    let quote = Quote {
        instrument: aapl(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        day_volume: Some(quantity(Decimal::from_str("12345.678").unwrap())),
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    let json = serde_json::to_string(&quote).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["currency"], serde_json::json!("USD"));
    assert_eq!(value["price"], serde_json::json!("150"));
    assert_eq!(value["day_volume"], serde_json::json!("12345.678"));

    let deserialized: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, deserialized);
}

#[test]
fn quote_as_of_serializes_as_unix_milliseconds() {
    let mut quote = Quote::new(aapl(), usd());
    quote.as_of = Some(DateTime::from_timestamp(1_640_995_200, 654_000_000).unwrap());

    let json = serde_json::to_string(&quote).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["as_of"], serde_json::json!(1_640_995_200_654_i64));

    let deserialized: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, deserialized);
}

#[test]
fn quote_with_none_fields() {
    let quote = Quote {
        instrument: aapl(),
        name: None,
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(147)),
        day_volume: None,
        market_state: None,
        as_of: None,
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
        instrument: aapl_nasdaq(),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        volume_delta: None,
        ts: DateTime::from_timestamp(1_640_995_200, 654_000_000).unwrap(),
        provider: (),
    };

    let json = serde_json::to_string(&update).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["ts"], serde_json::json!(1_640_995_200_654_i64));
    assert_eq!(value["currency"], serde_json::json!("USD"));

    let deserialized: QuoteUpdate = serde_json::from_str(&json).unwrap();
    assert_eq!(update, deserialized);
}

#[test]
fn quote_update_with_none_fields() {
    let update = QuoteUpdate {
        instrument: aapl(),
        currency: usd(),
        price: None,
        previous_close: None,
        volume_delta: None,
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
        instrument: aapl(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(Decimal::from_str("150.123456789").unwrap())),
        previous_close: Some(amount(Decimal::from_str("147.135802469").unwrap())),
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    let json = serde_json::to_string(&quote).unwrap();
    let deserialized: Quote = serde_json::from_str(&json).unwrap();

    assert_eq!(quote.price, deserialized.price);
    assert_eq!(quote.previous_close, deserialized.previous_close);
    assert_eq!(quote, deserialized);
}

#[test]
fn quote_with_bid_and_ask_roundtrips() {
    let bid = BookLevel {
        price: amount(149),
        size: Some(quantity(200)),
        provider: (),
    };
    let ask = BookLevel {
        price: amount(151),
        size: None,
        provider: (),
    };
    let quote = Quote {
        instrument: aapl(),
        name: None,
        currency: usd(),
        price: Some(amount(150)),
        bid: Some(bid),
        ask: Some(ask),
        previous_close: None,
        day_volume: None,
        market_state: None,
        as_of: None,
        provider: (),
    };
    let json = serde_json::to_string(&quote).unwrap();
    let decoded: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, decoded);
    let decoded_ask = decoded.ask.as_ref().unwrap();
    assert!(decoded_ask.size.is_none(), "ask size None preserved");
    let decoded_bid = decoded.bid.as_ref().unwrap();
    assert_eq!(decoded_bid.size, Some(quantity(200)));
}

#[test]
fn quote_new_initialises_bid_and_ask_to_none() {
    let quote = Quote::new(aapl(), usd());
    assert_eq!(quote.currency, usd());
    assert!(quote.bid.is_none());
    assert!(quote.ask.is_none());
    assert!(quote.as_of.is_none());
}

#[test]
fn deserialization_handles_missing_optional_fields() {
    let json = r#"{
        "instrument": { "symbol": "AAPL", "kind": "equity" },
        "currency": "USD"
    }"#;

    let deserialized: Quote = serde_json::from_str(json).unwrap();

    assert_eq!(
        deserialized.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL"
    );
    assert_eq!(deserialized.currency, usd());
    assert!(deserialized.as_of.is_none());
    assert!(deserialized.price.is_none());
    assert!(deserialized.bid.is_none());
    assert!(deserialized.ask.is_none());
}
