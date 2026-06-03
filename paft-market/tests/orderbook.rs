use paft_decimal::Decimal;
use paft_domain::{AssetKind, Instrument};
use paft_market::market::orderbook::{BookLevel, OrderBook};
use paft_money::{Currency, IsoCurrency, PriceAmount, QuantityAmount};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: i64) -> PriceAmount {
    PriceAmount::new(Decimal::from(value))
}

fn aapl() -> Instrument {
    Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap()
}

fn size(amount: i64) -> QuantityAmount {
    QuantityAmount::from_decimal(Decimal::from(amount)).unwrap()
}

#[test]
fn book_level_constructor_with_size() {
    let level = BookLevel::new(amount(100), Some(size(500)));
    assert_eq!(level.price, amount(100));
    assert_eq!(level.size, Some(size(500)));
}

#[test]
fn book_level_constructor_without_size() {
    let level = BookLevel::new(amount(100), None);
    assert_eq!(level.price, amount(100));
    assert!(level.size.is_none());
}

#[test]
fn book_level_serde_roundtrip_with_size() {
    let level = BookLevel {
        price: amount(100),
        size: Some(size(500)),
        provider: (),
    };
    let json = serde_json::to_string(&level).unwrap();
    let decoded: BookLevel = serde_json::from_str(&json).unwrap();
    assert_eq!(level, decoded);
}

#[test]
fn book_level_rejects_negative_size() {
    let level = BookLevel {
        price: amount(100),
        size: Some(size(1)),
        provider: (),
    };
    let mut value = serde_json::to_value(level).unwrap();
    value["size"] = serde_json::json!(-1);

    assert!(serde_json::from_value::<BookLevel>(value).is_err());
}

#[test]
fn book_level_serde_roundtrip_no_size() {
    let level = BookLevel {
        price: amount(100),
        size: None,
        provider: (),
    };
    let json = serde_json::to_string(&level).unwrap();
    let decoded: BookLevel = serde_json::from_str(&json).unwrap();
    assert_eq!(level, decoded);
    assert!(decoded.size.is_none());
}

#[test]
fn order_book_with_mixed_size_availability() {
    let book = OrderBook {
        instrument: aapl(),
        as_of: chrono::DateTime::from_timestamp(1_700_000_000, 456_000_000),
        currency: usd(),
        asks: vec![
            BookLevel::new(amount(101), Some(size(200))),
            BookLevel::new(amount(102), None),
        ],
        bids: vec![
            BookLevel::new(amount(99), Some(size(300))),
            BookLevel::new(amount(98), None),
        ],
        provider: (),
    };

    let json = serde_json::to_string(&book).unwrap();
    assert!(json.contains(r#""as_of":1700000000456"#));

    let decoded: OrderBook = serde_json::from_str(&json).unwrap();
    assert_eq!(book, decoded);
    assert_eq!(decoded.asks.len(), 2);
    assert_eq!(decoded.bids.len(), 2);
    assert_eq!(decoded.asks[0].size, Some(size(200)));
    assert!(decoded.asks[1].size.is_none());
    assert!(decoded.bids[1].size.is_none());
    assert_eq!(
        decoded.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL"
    );
    assert_eq!(decoded.as_of.unwrap().timestamp_millis(), 1_700_000_000_456);
}

#[test]
fn order_book_constructor_sets_required_context() {
    let book = OrderBook::new(aapl(), usd());

    assert_eq!(
        book.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL"
    );
    assert!(book.as_of.is_none());
    assert_eq!(book.currency, usd());
    assert!(book.asks.is_empty());
    assert!(book.bids.is_empty());
}

#[test]
fn order_book_deserializes_missing_as_of_as_none() {
    let json = r#"{
        "instrument": { "symbol": "AAPL", "kind": "equity" },
        "currency": "USD",
        "asks": [],
        "bids": []
    }"#;

    let book: OrderBook = serde_json::from_str(json).unwrap();

    assert_eq!(
        book.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL"
    );
    assert!(book.as_of.is_none());
    assert_eq!(book.currency, usd());
}
