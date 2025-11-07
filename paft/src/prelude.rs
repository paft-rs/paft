//! Commonly used types for convenient glob import.
//!
//! Note: All enums expose a canonical `code()` string used by both `Display`
//! and serde to ensure round-trip stability across providers.

// Re-export core types via namespaced modules
#[cfg(feature = "domain")]
pub use crate::domain::{
    AssetKind, Canonical, Exchange, Figi, Instrument, Isin, MarketState, Period, StringCode,
    Symbol, canonicalize,
};
#[cfg(feature = "money-formatting")]
pub use crate::money::Locale;
#[cfg(feature = "money-formatting")]
pub use crate::money::LocalizedMoney;
pub use crate::money::{
    Currency, clear_currency_metadata, currency_metadata, set_currency_metadata,
    try_normalize_currency_code,
};
pub use crate::money::{ExchangeRate, Money};
pub use crate::{Error, Result};

// Re-export dataframe traits
#[cfg(feature = "dataframe")]
pub use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

// Re-export fundamentals types (flattened via namespace)
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::analysis::{
    AnalysisSummary, Earnings, EarningsQuarter, EarningsQuarterEps, EarningsTrendRow, EarningsYear,
    PriceTarget, RecommendationAction, RecommendationGrade, RecommendationRow,
    RecommendationSummary, UpgradeDowngradeRow,
};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::esg::{EsgInvolvement, EsgScores, EsgSummary};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::holders::{
    InsiderPosition, InsiderRosterHolder, InsiderTransaction, InstitutionalHolder, MajorHolder,
    NetSharePurchaseActivity, TransactionType,
};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::profile::{
    Address, CompanyProfile, FundKind, FundProfile, Profile, ShareCount,
};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::statements::{
    BalanceSheetRow, Calendar, CashflowRow, IncomeStatementRow,
};

// Re-export market types (flattened via namespace)
#[cfg(feature = "market")]
pub use crate::market::action::Action;
#[cfg(feature = "market")]
pub use crate::market::news::NewsArticle;
#[cfg(feature = "market")]
pub use crate::market::options::{OptionChain, OptionContract};
#[cfg(feature = "market")]
pub use crate::market::orderbook::{OrderBook, OrderBookEntry};
#[cfg(feature = "market")]
pub use crate::market::quote::{Quote, QuoteUpdate};
#[cfg(feature = "market")]
pub use crate::market::requests::history::{
    HistoryRequest, HistoryRequestBuilder, Interval, Range,
};
#[cfg(feature = "market")]
pub use crate::market::requests::options::{OptionChainRequest, OptionExpirationsRequest};
#[cfg(feature = "market")]
pub use crate::market::requests::search::SearchRequest;
#[cfg(feature = "market")]
pub use crate::market::responses::download::DownloadResponse;
#[cfg(feature = "market")]
pub use crate::market::responses::history::{Candle, HistoryMeta, HistoryResponse};
#[cfg(feature = "market")]
pub use crate::market::responses::search::{SearchResponse, SearchResult};

// Re-export aggregates snapshot types
#[cfg(feature = "aggregates")]
pub use crate::aggregates::{FastInfo, Info};
