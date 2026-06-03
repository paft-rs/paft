//! Market data types, requests, and responses for paft.
//!
//! This crate provides strongly-typed market data models (quotes, options,
//! news), request builders (search, history), and response types that are
//! consistent across providers. It aims to:
//! - Offer validated builders to avoid invalid request states
//! - Encode canonical, serde-stable string forms for interop
//! - Integrate with `paft-domain` and `paft-money` for identifiers and values
//!
//! # Quickstart
//! ```rust
//! use paft_market::{HistoryRequest, Interval, Range, SearchRequest};
//!
//! // Build a history request for 1 month of daily candles
//! let req = HistoryRequest::try_from_range(Range::M1, Interval::D1).unwrap();
//! assert_eq!(req.interval(), Interval::D1);
//!
//! // Build a validated search request
//! let search = SearchRequest::new("AAPL").unwrap();
//! assert_eq!(search.query(), "AAPL");
//! ```
//!
//! # Feature flags
//! The default money backend is `rust_decimal`.
//!
//! - `bigdecimal`: switch `paft-money` to the `bigdecimal` backend
//! - `dataframe`: enable `polars`/`df-derive-macros` integration for dataframe export
//!
//! # Serde
//! All models serialize with stable, human-readable representations suitable for
//! storage and transport. Dataframe support emits string codes for enums.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod market;
pub mod requests;
pub mod responses;

pub use error::MarketError;

pub use market::{
    action::Action,
    news::{GenericNewsArticle, NewsArticle},
    options::{
        GenericOptionChain, GenericOptionContract, GenericOptionUpdate, OptionChain,
        OptionContract, OptionContractKey, OptionGreeks, OptionSide, OptionUpdate,
    },
    orderbook::{BookLevel, GenericBookLevel, GenericOrderBook, OrderBook},
    quote::{GenericQuote, GenericQuoteUpdate, Quote, QuoteUpdate},
};
pub use requests::history::{
    HistoryFlags, HistoryRequest, HistoryRequestBuilder, Interval, Range, TimeSpec,
};
pub use requests::news::{NewsRequest, NewsTab};
pub use requests::options::{OptionChainRequest, OptionExpirationsRequest};
pub use requests::search::{SearchRequest, SearchRequestBuilder};
pub use responses::download::{
    DownloadEntry, DownloadResponse, GenericDownloadEntry, GenericDownloadResponse,
};
pub use responses::history::{
    AdjustmentAnchor, AdjustmentMethod, Candle, CandleUpdate, CorporateActionAdjustmentCause,
    CorporateActionAdjustmentCauses, GenericCandle, GenericCandleUpdate, GenericHistoryResponse,
    HistoryMeta, HistoryResponse, Ohlc, OhlcPriceBasis, PriceBasis,
};
pub use responses::options::OptionExpirationsResponse;
pub use responses::search::{
    GenericSearchResponse, GenericSearchResult, SearchResponse, SearchResult,
};
