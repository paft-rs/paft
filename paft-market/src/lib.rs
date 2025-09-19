//! Market data types, requests, and responses for paft.
#![warn(missing_docs)]

pub mod error;
pub mod market;
pub mod requests;
pub mod responses;

pub use error::MarketError;

pub use market::{
    action::Action,
    news::NewsArticle,
    options::{OptionChain, OptionContract},
    quote::{Quote, QuoteUpdate},
};
pub use requests::history::{HistoryRequest, HistoryRequestBuilder, Interval, Range};
pub use requests::search::SearchRequest;
pub use responses::download::DownloadResponse;
pub use responses::history::{Candle, HistoryMeta, HistoryResponse};
pub use responses::search::{SearchResponse, SearchResult};
