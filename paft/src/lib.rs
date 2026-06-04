//! Unified public API for the paft workspace.
//!
//! This facade crate aggregates the `paft` ecosystem into a single dependency
//! with coherent feature flags and a convenient `prelude` for common imports.
//!
//! Features
//! - `domain`, `market`, `fundamentals`, `aggregates`: opt into the areas you need
//! - `prediction`: prediction market data models (`Market`, `Token`)
//! - `bigdecimal`: change the money backend from `rust_decimal` to `bigdecimal`
//! - `dataframe`: enable `DataFrame` export via Polars helpers
//! - `panicking-money-ops`: opt‑in operator overloading for `Money` that panics on invalid input
//! - `money-formatting`: locale‑aware money formatting and parsing
//! - `tracing`: lightweight instrumentation; zero‑cost when disabled
//!
//! # Quickstart
//! ```rust
//! use paft::prelude::*;
//!
//! // Construct an instrument with identifiers
//! let aapl = Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity)
//!     .unwrap();
//!
//! // Build a validated history request
//! let req = HistoryRequest::try_from_range(Range::M1, Interval::D1).unwrap();
//! assert_eq!(req.interval(), Interval::D1);
//! ```
//!
//! See the crate README for installation instructions and feature details.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub use error::{Error, Result};

/// Namespaced access to `paft-core`.
pub mod core {
    pub use paft_core::PaftError;
    #[cfg(feature = "dataframe")]
    pub use paft_utils::dataframe;
}

/// Namespaced access to `paft-domain` (feature-gated).
#[cfg(feature = "domain")]
pub mod domain {
    pub use paft_domain::{
        AssetKind, CalendarPeriod, Canonical, CanonicalError, DomainError, Exchange, Figi, Horizon,
        Instrument, Isin, MarketState, OtherAssetKind, OtherExchange, OtherHorizon, OtherPeriod,
        PeriodDate, PeriodYear, QuarterOfYear, ReportingPeriod, StringCode, Symbol, canonicalize,
    };
    #[cfg(feature = "dataframe")]
    pub use paft_domain::{Decimal128Encode, ToDataFrame, ToDataFrameVec};
}

/// Namespaced access to `paft-market` (feature-gated).
#[cfg(feature = "market")]
pub mod market {
    pub use paft_market::error::{self, MarketError};
    pub use paft_market::market::{action, news, options, orderbook, quote};
    pub use paft_market::requests;
    pub use paft_market::responses;
    pub use paft_market::{
        Action, AdjustmentAnchor, AdjustmentMethod, BookLevel, Candle, CandleUpdate,
        CorporateActionAdjustmentCause, CorporateActionAdjustmentCauses, DownloadEntry,
        DownloadResponse, GenericBookLevel, GenericCandle, GenericCandleUpdate,
        GenericDownloadEntry, GenericDownloadResponse, GenericHistoryResponse, GenericNewsArticle,
        GenericOptionChain, GenericOptionContract, GenericOptionUpdate, GenericOrderBook,
        GenericQuote, GenericQuoteUpdate, GenericSearchResponse, GenericSearchResult, HistoryFlags,
        HistoryMeta, HistoryRequest, HistoryRequestBuilder, HistoryResponse, Interval, NewsArticle,
        NewsRequest, NewsTab, Ohlc, OhlcPriceBasis, OptionChain, OptionChainRequest,
        OptionContract, OptionContractKey, OptionExpirationsRequest, OptionExpirationsResponse,
        OptionGreeks, OptionSide, OptionUpdate, OrderBook, PriceBasis, Quote, QuoteUpdate, Range,
        SearchRequest, SearchRequestBuilder, SearchResponse, SearchResult, TimeSpec,
    };
}

/// Namespaced access to `paft-money` types.
pub mod money {
    #[cfg(feature = "money-formatting")]
    pub use paft_money::LocalizedMoney;
    pub use paft_money::{
        Currency, CurrencyMetadata, ExchangeRate, IsoCurrency, Locale, MAX_DECIMAL_PRECISION,
        MAX_MINOR_UNIT_DECIMALS, MinorUnitError, MonetaryAmount, Money, MoneyError,
        MoneyParseError, OtherCurrency, Price, PriceAmount, QuantityAmount,
        clear_currency_metadata, currency_metadata, override_currency_metadata,
        set_currency_metadata, try_normalize_currency_code,
    };
}

/// Direct access to decimal types.
pub use paft_decimal::{
    Decimal, DecimalConstraintError, NonNegativeDecimal, PositiveDecimal, Ratio, RoundingStrategy,
};

/// Top-level re-export of the dataframe runtime traits used by paft-owned
/// types. For deriving dataframe support on your own structs, depend on
/// `df-derive` directly.
#[cfg(feature = "dataframe")]
pub use paft_utils::dataframe;

/// Namespaced access to `paft-fundamentals` (feature-gated).
#[cfg(feature = "fundamentals")]
pub mod fundamentals {
    pub use paft_fundamentals::{
        Address, AnalysisSummary, BalanceSheetRow, Calendar, CashflowRow, CompanyProfile, Earnings,
        EarningsEstimate, EarningsQuarter, EarningsQuarterEps, EarningsTrendRow, EarningsYear,
        EpsRevisions, EpsTrend, EsgInvolvement, EsgScores, EsgSummary, FundKind, FundProfile,
        FundamentalsError, IncomeStatementRow, InsiderPosition, InsiderRosterHolder,
        InsiderTransaction, InstitutionalHolder, KeyStatistics, MajorHolder,
        NetSharePurchaseActivity, OtherFundKind, OtherInsiderPosition, OtherRecommendationAction,
        OtherRecommendationGrade, OtherTransactionType, PriceTarget, Profile, RecommendationAction,
        RecommendationGrade, RecommendationRow, RecommendationSummary, RevenueEstimate,
        RevisionPoint, ShareCount, TransactionType, TrendPoint, UpgradeDowngradeRow,
    };
    pub use paft_fundamentals::{analysis, esg, holders, profile, statements, statistics};
}

/// Namespaced access to `paft-aggregates` (feature-gated).
#[cfg(feature = "aggregates")]
pub mod aggregates {
    pub use paft_aggregates::{GenericSnapshot, Snapshot};
}

/// Namespaced access to `paft-prediction` (feature-gated).
#[cfg(feature = "prediction")]
pub mod prediction {
    pub use paft_prediction::{
        EventID, Market, OutcomeID, PredictionError, PredictionInstrument, Token,
    };
}

/// Frequently used types for convenient imports.
pub mod prelude;
