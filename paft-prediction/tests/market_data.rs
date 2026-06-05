use paft_prediction::{
    BinaryMarketKey, BinaryOrderBook, ContractQuantity, OutcomePrice, PredictionBookLevel,
    PriceBand, PriceGrid, PriceTick,
};

fn level(micros: u32, qty: u64) -> PredictionBookLevel {
    PredictionBookLevel::new(
        OutcomePrice::from_micros(micros).unwrap(),
        ContractQuantity::from_microcontracts(qty),
    )
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
