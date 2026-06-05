use paft_prediction::{
    BinaryMarketKey, BinaryOrderBook, BinaryQuote, NonZeroContractQuantity, OutcomeInstrument,
    OutcomeOrderBook, OutcomePrice, PredictionBookLevel, PredictionQuoteLevel, PredictionTrade,
    PriceBand, PriceGrid, PriceTick,
};
use std::mem::size_of;

fn level(micros: u32, qty: u64) -> PredictionBookLevel {
    PredictionBookLevel::new(
        OutcomePrice::from_micros(micros).unwrap(),
        NonZeroContractQuantity::from_microcontracts(qty).unwrap(),
    )
}

#[test]
fn prediction_book_level_size_is_compact() {
    assert_eq!(size_of::<PredictionBookLevel>(), 16);
}

#[test]
fn binary_quote_levels_allow_missing_quantity() {
    let quote = BinaryQuote {
        market: BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap(),
        as_of: None,
        yes_bid: Some(PredictionQuoteLevel::new(
            OutcomePrice::from_micros(410_000).unwrap(),
            None,
        )),
        yes_ask: Some(PredictionQuoteLevel::new(
            OutcomePrice::from_micros(430_000).unwrap(),
            Some(NonZeroContractQuantity::from_microcontracts(2_000_000).unwrap()),
        )),
        last_price: None,
        provider: (),
    };

    assert_eq!(quote.yes_bid.unwrap().quantity, None);
    assert_eq!(
        quote.yes_ask.unwrap().quantity.unwrap().microcontracts(),
        2_000_000
    );
}

#[test]
fn binary_quote_deserializes_missing_level_quantity() {
    let json = r#"{
        "market": {
            "venue": "POLYMARKET",
            "market_id": "condition-1"
        },
        "as_of": null,
        "yes_bid": {
            "price": 410000
        },
        "yes_ask": {
            "price": 430000,
            "quantity": 2000000
        },
        "last_price": null
    }"#;

    let quote: BinaryQuote = serde_json::from_str(json).unwrap();

    assert_eq!(quote.yes_bid.unwrap().quantity, None);
    assert_eq!(
        quote.yes_ask.unwrap().quantity.unwrap().microcontracts(),
        2_000_000
    );
}

#[test]
fn binary_quote_deserialization_rejects_zero_level_quantity() {
    let json = r#"{
        "market": {
            "venue": "POLYMARKET",
            "market_id": "condition-1"
        },
        "as_of": null,
        "yes_bid": {
            "price": 410000,
            "quantity": 0
        },
        "yes_ask": null,
        "last_price": null
    }"#;

    assert!(serde_json::from_str::<BinaryQuote>(json).is_err());
}

#[test]
fn order_book_deserialization_rejects_zero_level_quantity() {
    let json = r#"{
        "market": {
            "venue": "POLYMARKET",
            "market_id": "condition-1"
        },
        "as_of": null,
        "yes_bids": [
            { "price": 410000, "quantity": 0, "order_count": null }
        ],
        "yes_asks": [],
        "price_grid": null
    }"#;

    assert!(serde_json::from_str::<BinaryOrderBook>(json).is_err());
}

#[test]
fn prediction_trade_deserialization_rejects_zero_quantity() {
    let json = r#"{
        "instrument": {
            "venue": "POLYMARKET",
            "market_id": "condition-1",
            "outcome_id": "yes-token"
        },
        "price": 410000,
        "quantity": 0,
        "action": null,
        "trade_id": null,
        "ts": 0
    }"#;

    assert!(serde_json::from_str::<PredictionTrade>(json).is_err());
}

#[test]
fn binary_order_book_derives_no_side_from_yes_view() {
    let mut book =
        BinaryOrderBook::new(BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap());
    book.yes_bids = vec![level(410_000, 2_000_000), level(400_000, 1_000_000)];
    book.yes_asks = vec![level(430_000, 3_000_000), level(440_000, 1_000_000)];

    assert!(book.is_sorted());
    assert_eq!(book.best_yes_bid().unwrap().price.micros(), 410_000);
    assert_eq!(book.best_yes_ask().unwrap().price.micros(), 430_000);
    assert_eq!(book.best_no_bid().unwrap().price.micros(), 570_000);
    assert_eq!(book.best_no_ask().unwrap().price.micros(), 590_000);
    assert_eq!(book.yes_midpoint().unwrap().micros(), 420_000);
    assert_eq!(book.yes_spread().unwrap().micros(), 20_000);
}

#[test]
fn binary_order_book_validation_rejects_unsorted_and_crossed_books() {
    let mut unsorted =
        BinaryOrderBook::new(BinaryMarketKey::new("POLYMARKET", "condition-1").unwrap());
    unsorted.yes_bids = vec![level(400_000, 1), level(410_000, 1)];
    assert!(unsorted.validate_sorted().is_err());

    let mut crossed =
        BinaryOrderBook::new(BinaryMarketKey::new("POLYMARKET", "condition-2").unwrap());
    crossed.yes_bids = vec![level(600_000, 1)];
    crossed.yes_asks = vec![level(590_000, 1)];
    assert!(crossed.validate_sorted().is_err());
    assert!(crossed.yes_spread().is_none());
}

#[test]
fn binary_order_book_sort_levels_canonicalizes_sides_stably() {
    let mut book = BinaryOrderBook::new(BinaryMarketKey::new("POLYMARKET", "condition-1").unwrap());
    book.yes_bids = vec![level(400_000, 1), level(410_000, 2), level(410_000, 3)];
    book.yes_asks = vec![level(430_000, 4), level(420_000, 5), level(420_000, 6)];

    assert!(!book.is_sorted());
    book.sort_levels();

    assert!(book.validate_sorted().is_ok());
    assert_eq!(book.yes_bids[0].quantity.microcontracts(), 2);
    assert_eq!(book.yes_bids[1].quantity.microcontracts(), 3);
    assert_eq!(book.yes_bids[2].quantity.microcontracts(), 1);
    assert_eq!(book.yes_asks[0].quantity.microcontracts(), 5);
    assert_eq!(book.yes_asks[1].quantity.microcontracts(), 6);
    assert_eq!(book.yes_asks[2].quantity.microcontracts(), 4);
}

#[test]
fn outcome_order_book_sort_levels_canonicalizes_sides_stably() {
    let mut book = OutcomeOrderBook::new(
        OutcomeInstrument::new("POLYMARKET", "condition-1", "yes-token").unwrap(),
    );
    book.bids = vec![level(400_000, 1), level(410_000, 2), level(410_000, 3)];
    book.asks = vec![level(430_000, 4), level(420_000, 5), level(420_000, 6)];

    assert!(!book.is_sorted());
    book.sort_levels();

    assert!(book.validate_sorted().is_ok());
    assert_eq!(book.bids[0].quantity.microcontracts(), 2);
    assert_eq!(book.bids[1].quantity.microcontracts(), 3);
    assert_eq!(book.bids[2].quantity.microcontracts(), 1);
    assert_eq!(book.asks[0].quantity.microcontracts(), 5);
    assert_eq!(book.asks[1].quantity.microcontracts(), 6);
    assert_eq!(book.asks[2].quantity.microcontracts(), 4);
}

#[test]
fn binary_order_book_validates_prices_on_grid() {
    let grid = PriceGrid::new(vec![PriceBand {
        start: OutcomePrice::ZERO,
        end: OutcomePrice::ONE,
        tick: PriceTick::from_micros(10_000).unwrap(),
    }])
    .unwrap();

    let mut book =
        BinaryOrderBook::new(BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap());
    book.price_grid = Some(grid);
    book.yes_bids = vec![level(410_000, 1)];
    book.yes_asks = vec![level(430_001, 1)];

    assert!(book.validate_on_grid().is_err());
}
