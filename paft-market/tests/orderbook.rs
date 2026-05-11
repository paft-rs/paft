use iso_currency::Currency as IsoCurrency;
use paft_decimal::Decimal;
use paft_market::market::orderbook::{BookLevel, OrderBook};
use paft_money::{Currency, Money};

fn usd(amount: i64) -> Money {
    Money::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD)).unwrap()
}

#[test]
fn book_level_constructor_with_size() {
    let level = BookLevel::new(usd(100), Some(Decimal::from(500)));
    assert_eq!(level.price, usd(100));
    assert_eq!(level.size, Some(Decimal::from(500)));
}

#[test]
fn book_level_constructor_without_size() {
    let level = BookLevel::new(usd(100), None);
    assert_eq!(level.price, usd(100));
    assert!(level.size.is_none());
}

#[test]
fn book_level_serde_roundtrip_with_size() {
    let level = BookLevel {
        price: usd(100),
        size: Some(Decimal::from(500)),
        provider: (),
    };
    let json = serde_json::to_string(&level).unwrap();
    let decoded: BookLevel = serde_json::from_str(&json).unwrap();
    assert_eq!(level, decoded);
}

#[test]
fn book_level_serde_roundtrip_no_size() {
    let level = BookLevel {
        price: usd(100),
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
        asks: vec![
            BookLevel::new(usd(101), Some(Decimal::from(200))),
            BookLevel::new(usd(102), None),
        ],
        bids: vec![
            BookLevel::new(usd(99), Some(Decimal::from(300))),
            BookLevel::new(usd(98), None),
        ],
        provider: (),
    };

    let json = serde_json::to_string(&book).unwrap();
    let decoded: OrderBook = serde_json::from_str(&json).unwrap();
    assert_eq!(book, decoded);
    assert_eq!(decoded.asks.len(), 2);
    assert_eq!(decoded.bids.len(), 2);
    assert_eq!(decoded.asks[0].size, Some(Decimal::from(200)));
    assert!(decoded.asks[1].size.is_none());
    assert!(decoded.bids[1].size.is_none());
}
