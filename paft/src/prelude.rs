//! Commonly used types for convenient glob import.
//!
//! Note: All enums expose a canonical `code()` string used by both `Display`
//! and serde to ensure round-trip stability across providers.

// Re-export core types via namespaced modules
#[cfg(feature = "domain")]
pub use crate::domain::{
    AssetKind, Canonical, Exchange, Figi, Instrument, Isin, MarketState, OtherAssetKind,
    OtherExchange, OtherPeriod, Period, PeriodDate, PeriodYear, QuarterOfYear, StringCode, Symbol,
    canonicalize,
};
#[cfg(feature = "money-formatting")]
pub use crate::money::LocalizedMoney;
pub use crate::money::{
    Currency, CurrencyMetadata, Locale, MAX_DECIMAL_PRECISION, MAX_MINOR_UNIT_DECIMALS,
    MonetaryAmount, OtherCurrency, Price, PriceAmount, QuantityAmount, clear_currency_metadata,
    currency_metadata, set_currency_metadata, try_normalize_currency_code,
};
pub use crate::money::{ExchangeRate, Money};
pub use crate::{
    Decimal, DecimalConstraintError, Error, NonNegativeDecimal, PositiveDecimal, Ratio, Result,
    RoundingStrategy,
};

// Re-export dataframe traits
#[cfg(feature = "dataframe")]
pub use paft_utils::dataframe::{Decimal128Encode, ToDataFrame, ToDataFrameVec};

// Re-export fundamentals types (flattened via namespace)
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::analysis::{
    AnalysisSummary, Earnings, EarningsEstimate, EarningsQuarter, EarningsQuarterEps,
    EarningsTrendRow, EarningsYear, EpsRevisions, EpsTrend, OtherRecommendationAction,
    OtherRecommendationGrade, PriceTarget, RecommendationAction, RecommendationGrade,
    RecommendationRow, RecommendationSummary, RevenueEstimate, RevisionPoint, TrendPoint,
    UpgradeDowngradeRow,
};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::esg::{EsgInvolvement, EsgScores, EsgSummary};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::holders::{
    InsiderPosition, InsiderRosterHolder, InsiderTransaction, InstitutionalHolder, MajorHolder,
    NetSharePurchaseActivity, OtherInsiderPosition, OtherTransactionType, TransactionType,
};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::profile::{
    Address, CompanyProfile, FundKind, FundProfile, OtherFundKind, Profile, ShareCount,
};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::statements::{
    BalanceSheetRow, Calendar, CashflowRow, IncomeStatementRow,
};
#[cfg(feature = "fundamentals")]
pub use crate::fundamentals::statistics::KeyStatistics;

// Re-export market types (flattened via namespace)
#[cfg(feature = "market")]
pub use crate::market::{
    Action, AdjustmentAnchor, AdjustmentMethod, BookLevel, Candle, CandleUpdate,
    CorporateActionAdjustmentCause, CorporateActionAdjustmentCauses, DownloadEntry,
    DownloadResponse, GenericBookLevel, GenericCandle, GenericCandleUpdate, GenericDownloadEntry,
    GenericDownloadResponse, GenericHistoryResponse, GenericNewsArticle, GenericOptionChain,
    GenericOptionContract, GenericOptionUpdate, GenericOrderBook, GenericQuote, GenericQuoteUpdate,
    GenericSearchResponse, GenericSearchResult, HistoryFlags, HistoryMeta, HistoryRequest,
    HistoryRequestBuilder, HistoryResponse, Interval, NewsArticle, NewsRequest, NewsTab, Ohlc,
    OhlcPriceBasis, OptionChain, OptionChainRequest, OptionContract, OptionContractKey,
    OptionExpirationsRequest, OptionExpirationsResponse, OptionGreeks, OptionSide, OptionUpdate,
    OrderBook, PriceBasis, Quote, QuoteUpdate, Range, SearchRequest, SearchRequestBuilder,
    SearchResponse, SearchResult, TimeSpec,
};

// Re-export aggregates snapshot types
#[cfg(feature = "aggregates")]
pub use crate::aggregates::{GenericSnapshot, Snapshot};

#[cfg(feature = "prediction")]
pub use crate::prediction::{EventID, Market, OutcomeID, PredictionInstrument, Token};
