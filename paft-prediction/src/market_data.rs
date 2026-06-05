//! Prediction-market quotes, order books, and trade-history payloads.

use crate::error::PredictionError;
use crate::instrument::{BinaryMarketKey, OutcomeInstrument};
use crate::price::{ContractQuantity, OutcomePrice, PriceGrid};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// YES/NO outcome side for binary prediction markets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOutcome {
    /// YES outcome.
    Yes,
    /// NO outcome.
    No,
}

impl BinaryOutcome {
    /// Returns the opposite binary outcome.
    #[must_use]
    pub const fn complement(self) -> Self {
        match self {
            Self::Yes => Self::No,
            Self::No => Self::Yes,
        }
    }
}

/// Bid/ask side of an order book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookSide {
    /// Bid side.
    Bid,
    /// Ask side.
    Ask,
}

/// Buy/sell trade or order action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeAction {
    /// Buy action.
    Buy,
    /// Sell action.
    Sell,
}

/// Explicit binary order direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BinaryOrderDirection {
    /// Buy/sell action.
    pub action: TradeAction,
    /// YES/NO outcome being bought or sold.
    pub outcome: BinaryOutcome,
}

impl BinaryOrderDirection {
    /// Buy YES.
    #[must_use]
    pub const fn buy_yes() -> Self {
        Self {
            action: TradeAction::Buy,
            outcome: BinaryOutcome::Yes,
        }
    }

    /// Sell YES.
    #[must_use]
    pub const fn sell_yes() -> Self {
        Self {
            action: TradeAction::Sell,
            outcome: BinaryOutcome::Yes,
        }
    }

    /// Buy NO.
    #[must_use]
    pub const fn buy_no() -> Self {
        Self {
            action: TradeAction::Buy,
            outcome: BinaryOutcome::No,
        }
    }

    /// Sell NO.
    #[must_use]
    pub const fn sell_no() -> Self {
        Self {
            action: TradeAction::Sell,
            outcome: BinaryOutcome::No,
        }
    }
}

/// A single prediction-market book level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredictionBookLevel {
    /// Outcome price at this level.
    pub price: OutcomePrice,
    /// Displayed quantity at this level.
    pub quantity: ContractQuantity,
    /// Optional number of orders aggregated at this price level.
    pub order_count: Option<NonZeroU32>,
}

impl PredictionBookLevel {
    /// Build a book level with no order count.
    #[must_use]
    pub const fn new(price: OutcomePrice, quantity: ContractQuantity) -> Self {
        Self {
            price,
            quantity,
            order_count: None,
        }
    }

    /// Return the same level with its price complemented around 1.0.
    #[must_use]
    pub const fn complement_price(self) -> Self {
        Self {
            price: self.price.complement(),
            quantity: self.quantity,
            order_count: self.order_count,
        }
    }
}

/// Top-of-book quote for a binary market in canonical YES view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericBinaryQuote<M = ()> {
    /// Venue-namespaced binary market key.
    pub market: BinaryMarketKey,
    /// Timestamp (UTC) when this quote snapshot was observed.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub as_of: Option<DateTime<Utc>>,
    /// Best YES bid, if known.
    pub yes_bid: Option<PredictionBookLevel>,
    /// Best YES ask, if known.
    pub yes_ask: Option<PredictionBookLevel>,
    /// Last traded YES-view price, if known.
    pub last_price: Option<OutcomePrice>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard binary quote with no provider metadata.
pub type BinaryQuote = GenericBinaryQuote<()>;

/// Canonical YES-view order book for an atomic binary market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericBinaryOrderBook<M = ()> {
    /// Venue-namespaced binary market key.
    pub market: BinaryMarketKey,
    /// Timestamp (UTC) when this book snapshot was observed.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub as_of: Option<DateTime<Utc>>,
    /// YES bids, normally sorted by price descending.
    pub yes_bids: Vec<PredictionBookLevel>,
    /// YES asks, normally sorted by price ascending.
    pub yes_asks: Vec<PredictionBookLevel>,
    /// Market-specific price grid, if known.
    pub price_grid: Option<PriceGrid>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericBinaryOrderBook<M> {
    /// Build an empty canonical YES-view binary order book.
    #[must_use]
    pub fn new(market: BinaryMarketKey) -> Self {
        Self {
            market,
            as_of: None,
            yes_bids: Vec::new(),
            yes_asks: Vec::new(),
            price_grid: None,
            provider: M::default(),
        }
    }
}

impl<M> GenericBinaryOrderBook<M> {
    /// Returns the best YES bid.
    #[must_use]
    pub fn best_yes_bid(&self) -> Option<&PredictionBookLevel> {
        self.yes_bids.first()
    }

    /// Returns the best YES ask.
    #[must_use]
    pub fn best_yes_ask(&self) -> Option<&PredictionBookLevel> {
        self.yes_asks.first()
    }

    /// Returns the derived best NO bid.
    #[must_use]
    pub fn best_no_bid(&self) -> Option<PredictionBookLevel> {
        self.best_yes_ask()
            .copied()
            .map(PredictionBookLevel::complement_price)
    }

    /// Returns the derived best NO ask.
    #[must_use]
    pub fn best_no_ask(&self) -> Option<PredictionBookLevel> {
        self.best_yes_bid()
            .copied()
            .map(PredictionBookLevel::complement_price)
    }

    /// Returns the YES-view midpoint when both top-of-book sides exist.
    #[must_use]
    pub fn yes_midpoint(&self) -> Option<OutcomePrice> {
        Some(
            self.best_yes_bid()?
                .price
                .midpoint(self.best_yes_ask()?.price),
        )
    }

    /// Returns the YES-view spread when both sides exist and the book is not crossed.
    #[must_use]
    pub fn yes_spread(&self) -> Option<OutcomePrice> {
        let bid = self.best_yes_bid()?.price.micros();
        let ask = self.best_yes_ask()?.price.micros();
        let spread = ask.checked_sub(bid)?;
        Some(OutcomePrice::from_valid_micros(spread))
    }

    /// Returns `true` when bids are descending, asks are ascending, and the
    /// best bid does not exceed the best ask.
    #[must_use]
    pub fn is_sorted(&self) -> bool {
        descending_prices(&self.yes_bids) && ascending_prices(&self.yes_asks) && !self.is_crossed()
    }

    /// Validate canonical ordering for the book.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::UnsortedOrderBook`] or
    /// [`PredictionError::CrossedOrderBook`] when the book violates canonical
    /// ordering.
    pub fn validate_sorted(&self) -> Result<(), PredictionError> {
        if !descending_prices(&self.yes_bids) || !ascending_prices(&self.yes_asks) {
            return Err(PredictionError::UnsortedOrderBook { book: "binary" });
        }

        if let (Some(bid), Some(ask)) = (self.best_yes_bid(), self.best_yes_ask())
            && bid.price > ask.price
        {
            return Err(PredictionError::CrossedOrderBook {
                book: "binary",
                bid_micros: bid.price.micros(),
                ask_micros: ask.price.micros(),
            });
        }

        Ok(())
    }

    /// Validate every book level against `price_grid`, when present.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidPriceGrid`] for an invalid grid or
    /// [`PredictionError::PriceOffGrid`] for the first off-grid level.
    pub fn validate_on_grid(&self) -> Result<(), PredictionError> {
        let Some(grid) = &self.price_grid else {
            return Ok(());
        };
        grid.validate()?;
        validate_levels_on_grid(grid, &self.yes_bids)?;
        validate_levels_on_grid(grid, &self.yes_asks)
    }

    fn is_crossed(&self) -> bool {
        matches!(
            (self.best_yes_bid(), self.best_yes_ask()),
            (Some(bid), Some(ask)) if bid.price > ask.price
        )
    }
}

/// Standard binary order book with no provider metadata.
pub type BinaryOrderBook = GenericBinaryOrderBook<()>;

/// Bid/ask book for one outcome instrument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericOutcomeOrderBook<M = ()> {
    /// Venue-namespaced outcome instrument.
    pub instrument: OutcomeInstrument,
    /// Timestamp (UTC) when this book snapshot was observed.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub as_of: Option<DateTime<Utc>>,
    /// Bids for this outcome instrument, normally sorted by price descending.
    pub bids: Vec<PredictionBookLevel>,
    /// Asks for this outcome instrument, normally sorted by price ascending.
    pub asks: Vec<PredictionBookLevel>,
    /// Market-specific price grid, if known.
    pub price_grid: Option<PriceGrid>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericOutcomeOrderBook<M> {
    /// Build an empty outcome-instrument order book.
    #[must_use]
    pub fn new(instrument: OutcomeInstrument) -> Self {
        Self {
            instrument,
            as_of: None,
            bids: Vec::new(),
            asks: Vec::new(),
            price_grid: None,
            provider: M::default(),
        }
    }
}

impl<M> GenericOutcomeOrderBook<M> {
    /// Returns the best bid.
    #[must_use]
    pub fn best_bid(&self) -> Option<&PredictionBookLevel> {
        self.bids.first()
    }

    /// Returns the best ask.
    #[must_use]
    pub fn best_ask(&self) -> Option<&PredictionBookLevel> {
        self.asks.first()
    }

    /// Returns `true` when bids are descending, asks are ascending, and the
    /// best bid does not exceed the best ask.
    #[must_use]
    pub fn is_sorted(&self) -> bool {
        descending_prices(&self.bids) && ascending_prices(&self.asks) && !self.is_crossed()
    }

    /// Validate canonical ordering for the book.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::UnsortedOrderBook`] or
    /// [`PredictionError::CrossedOrderBook`] when the book violates canonical
    /// ordering.
    pub fn validate_sorted(&self) -> Result<(), PredictionError> {
        if !descending_prices(&self.bids) || !ascending_prices(&self.asks) {
            return Err(PredictionError::UnsortedOrderBook { book: "outcome" });
        }

        if let (Some(bid), Some(ask)) = (self.best_bid(), self.best_ask())
            && bid.price > ask.price
        {
            return Err(PredictionError::CrossedOrderBook {
                book: "outcome",
                bid_micros: bid.price.micros(),
                ask_micros: ask.price.micros(),
            });
        }

        Ok(())
    }

    /// Validate every book level against `price_grid`, when present.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidPriceGrid`] for an invalid grid or
    /// [`PredictionError::PriceOffGrid`] for the first off-grid level.
    pub fn validate_on_grid(&self) -> Result<(), PredictionError> {
        let Some(grid) = &self.price_grid else {
            return Ok(());
        };
        grid.validate()?;
        validate_levels_on_grid(grid, &self.bids)?;
        validate_levels_on_grid(grid, &self.asks)
    }

    fn is_crossed(&self) -> bool {
        matches!(
            (self.best_bid(), self.best_ask()),
            (Some(bid), Some(ask)) if bid.price > ask.price
        )
    }
}

/// Standard outcome order book with no provider metadata.
pub type OutcomeOrderBook = GenericOutcomeOrderBook<()>;

/// Trade print for one outcome instrument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericPredictionTrade<M = ()> {
    /// Venue-namespaced outcome instrument.
    pub instrument: OutcomeInstrument,
    /// Trade price in outcome-price micros.
    pub price: OutcomePrice,
    /// Trade quantity in microcontracts.
    pub quantity: ContractQuantity,
    /// Optional aggressor/action side when known.
    pub action: Option<TradeAction>,
    /// Provider-native trade identifier, if available.
    pub trade_id: Option<String>,
    /// Event timestamp (UTC).
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub ts: DateTime<Utc>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard prediction trade with no provider metadata.
pub type PredictionTrade = GenericPredictionTrade<()>;

fn descending_prices(levels: &[PredictionBookLevel]) -> bool {
    levels.windows(2).all(|pair| pair[0].price >= pair[1].price)
}

fn ascending_prices(levels: &[PredictionBookLevel]) -> bool {
    levels.windows(2).all(|pair| pair[0].price <= pair[1].price)
}

fn validate_levels_on_grid(
    grid: &PriceGrid,
    levels: &[PredictionBookLevel],
) -> Result<(), PredictionError> {
    for level in levels {
        grid.validate_price(level.price)?;
    }

    Ok(())
}
