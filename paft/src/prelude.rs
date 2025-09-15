//! Commonly used types for convenient glob import.

// Re-export core types that are available at the top level
#[cfg(feature = "core")]
pub use crate::{
    AssetKind, Currency, Exchange, ExchangeRate, Instrument, MarketState, Money, MoneyError,
    PaftError, Period,
};

// Re-export dataframe traits
#[cfg(feature = "dataframe")]
pub use crate::{ToDataFrame, ToDataFrameVec};

// Re-export fundamentals types
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::{
    FundKind, InsiderPosition, RecommendationAction, RecommendationGrade, TransactionType,
};

// Re-export market types
#[cfg(feature = "market")]
pub use crate::market::market::action::Action;
#[cfg(feature = "market")]
pub use crate::market::market::quote::{Quote, QuoteUpdate};
#[cfg(feature = "market")]
pub use crate::market::market::{NewsArticle, OptionChain, OptionContract};
#[cfg(feature = "market")]
pub use crate::market::requests::history::{
    HistoryRequest, HistoryRequestBuilder, Interval, Range,
};
#[cfg(feature = "market")]
pub use crate::market::requests::search::SearchRequest;
#[cfg(feature = "market")]
pub use crate::market::responses::history::{Candle, HistoryMeta, HistoryResponse};
#[cfg(feature = "market")]
pub use crate::market::responses::search::{SearchResponse, SearchResult};
