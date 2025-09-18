//! Commonly used types for convenient glob import.

// Re-export core types via namespaced modules
pub use crate::core::PaftError;
pub use crate::core::domain::{
    AssetKind, Currency, Exchange, ExchangeRate, Instrument, MarketState, Money, MoneyError,
    Period, clear_currency_minor_units, currency_minor_units, describe_currency,
    is_common_currency, normalize_currency_code, set_currency_minor_units,
};

// Re-export dataframe traits
#[cfg(feature = "dataframe")]
pub use crate::core::dataframe::{ToDataFrame, ToDataFrameVec};

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
pub use crate::market::quote::{Quote, QuoteUpdate};
#[cfg(feature = "market")]
pub use crate::market::requests::history::{
    HistoryRequest, HistoryRequestBuilder, Interval, Range,
};
#[cfg(feature = "market")]
pub use crate::market::requests::search::SearchRequest;
#[cfg(feature = "market")]
pub use crate::market::responses::download::DownloadResponse;
#[cfg(feature = "market")]
pub use crate::market::responses::history::{Candle, HistoryMeta, HistoryResponse};
#[cfg(feature = "market")]
pub use crate::market::responses::search::{SearchResponse, SearchResult};
