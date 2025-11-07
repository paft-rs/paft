//! Order book types under the `paft_market::market::orderbook` namespace.

use paft_money::{Decimal, Money}; // Use Money for price, Decimal for size
use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

/// A single entry (bid or ask) in an order book.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct OrderBookEntry {
    /// The price point for this entry.
    pub price: Money,

    /// The total quantity (size) available at this price point.
    pub size: Decimal,
}

/// A snapshot of the order book for a specific instrument.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct OrderBook {
    /// A vector of ask (sell) entries, typically sorted by price ascending.
    pub asks: Vec<OrderBookEntry>,

    /// A vector of bid (buy) entries, typically sorted by price descending.
    pub bids: Vec<OrderBookEntry>,
}
