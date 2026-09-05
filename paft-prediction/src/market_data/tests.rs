use super::*;

fn level(micros: u32, quantity: u64) -> PredictionBookLevel {
    PredictionBookLevel {
        price: OutcomePrice::from_micros(micros).unwrap(),
        quantity: NonZeroContractQuantity::from_microcontracts(quantity).unwrap(),
        order_count: NonZeroU32::new(2),
    }
}

fn binary_book() -> BinaryOrderBook {
    BinaryOrderBook::new(BinaryMarketKey::new("POLYMARKET", "condition-1").unwrap())
}

fn outcome_book() -> OutcomeOrderBook {
    OutcomeOrderBook::new(OutcomeInstrument::new("POLYMARKET", "condition-1", "yes-token").unwrap())
}

#[test]
fn book_queries_are_independent_of_level_order() {
    // Exercise every permutation of each side, without a random dependency.
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let bids = [level(200_000, 1), level(800_000, 2), level(600_000, 3)];
    let mut binary = binary_book();
    let mut outcome = outcome_book();

    for (ask_micros, midpoint, spread, crossed) in [
        (700_000, 750_000, None, true),
        (800_000, 800_000, Some(0), false),
        (850_000, 825_000, Some(50_000), false),
    ] {
        let asks = [level(900_000, 4), level(ask_micros, 5), level(950_000, 6)];
        for bid_order in orders {
            for ask_order in orders {
                let stored_bids = bid_order.map(|index| bids[index]);
                let stored_asks = ask_order.map(|index| asks[index]);
                binary.yes_bids = stored_bids.to_vec();
                binary.yes_asks = stored_asks.to_vec();
                outcome.bids = stored_bids.to_vec();
                outcome.asks = stored_asks.to_vec();

                assert_eq!(binary.best_yes_bid(), Some(&bids[1]));
                assert_eq!(binary.best_yes_ask(), Some(&asks[1]));
                assert_eq!(binary.best_no_bid(), Some(level(1_000_000 - ask_micros, 5)));
                assert_eq!(binary.best_no_ask(), Some(level(200_000, 2)));
                assert_eq!(binary.yes_midpoint().unwrap().micros(), midpoint);
                assert_eq!(binary.yes_spread().map(OutcomePrice::micros), spread);
                assert_eq!(binary.is_crossed(), crossed);
                assert_eq!(outcome.best_bid(), Some(&bids[1]));
                assert_eq!(outcome.best_ask(), Some(&asks[1]));
                assert_eq!(outcome.is_crossed(), crossed);

                assert_eq!(binary.yes_bids, stored_bids);
                assert_eq!(binary.yes_asks, stored_asks);
                assert_eq!(outcome.bids, stored_bids);
                assert_eq!(outcome.asks, stored_asks);
            }
        }
    }
}

#[test]
fn equal_best_prices_select_the_first_stored_level_before_and_after_sorting() {
    let best_bid = level(800_000, 2);
    let best_ask = level(900_000, 5);
    let later_bid = PredictionBookLevel {
        order_count: NonZeroU32::new(3),
        ..level(800_000, 3)
    };
    let later_ask = PredictionBookLevel {
        order_count: NonZeroU32::new(4),
        ..level(900_000, 6)
    };
    let mut binary = binary_book();
    binary.yes_bids = vec![level(200_000, 1), best_bid, later_bid];
    binary.yes_asks = vec![level(950_000, 4), best_ask, later_ask];
    let mut outcome = outcome_book();
    outcome.bids = binary.yes_bids.clone();
    outcome.asks = binary.yes_asks.clone();

    for sorted in [false, true] {
        if sorted {
            binary.sort_levels();
            outcome.sort_levels();
            assert!(binary.validate_sorted().is_ok());
            assert!(outcome.validate_sorted().is_ok());
        }
        assert_eq!(binary.best_yes_bid(), Some(&best_bid));
        assert_eq!(binary.best_yes_ask(), Some(&best_ask));
        assert_eq!(binary.best_no_bid(), Some(level(100_000, 5)));
        assert_eq!(binary.best_no_ask(), Some(level(200_000, 2)));
        assert_eq!(outcome.best_bid(), Some(&best_bid));
        assert_eq!(outcome.best_ask(), Some(&best_ask));
    }
}

#[test]
fn empty_and_one_sided_books_keep_missing_quotes() {
    let mut binary = binary_book();
    let mut outcome = outcome_book();
    for (bids, asks, best_bid, best_ask) in [
        (vec![], vec![], None, None),
        (
            vec![level(0, 1), level(1_000_000, 2)],
            vec![],
            Some(level(1_000_000, 2)),
            None,
        ),
        (
            vec![],
            vec![level(1_000_000, 3), level(0, 4)],
            None,
            Some(level(0, 4)),
        ),
    ] {
        binary.yes_bids.clone_from(&bids);
        binary.yes_asks.clone_from(&asks);
        outcome.bids = bids;
        outcome.asks = asks;

        assert_eq!(binary.best_yes_bid().copied(), best_bid);
        assert_eq!(binary.best_yes_ask().copied(), best_ask);
        assert_eq!(
            binary.best_no_bid(),
            best_ask.map(PredictionBookLevel::complement_price)
        );
        assert_eq!(
            binary.best_no_ask(),
            best_bid.map(PredictionBookLevel::complement_price)
        );
        assert_eq!(binary.yes_midpoint(), None);
        assert_eq!(binary.yes_spread(), None);
        assert!(!binary.is_crossed());
        assert_eq!(outcome.best_bid().copied(), best_bid);
        assert_eq!(outcome.best_ask().copied(), best_ask);
        assert!(!outcome.is_crossed());
    }
}
