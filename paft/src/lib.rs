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
        AssetKind, Canonical, CanonicalError, DomainError, Exchange, Figi, IdentifierScheme,
        Instrument, Isin, MarketState, Period, SecurityId, StringCode, Symbol, canonicalize, PredictionID, EventID, OutcomeID
    };
    #[cfg(feature = "dataframe")]
    pub use paft_domain::{ToDataFrame, ToDataFrameVec};
}

/// Namespaced access to `paft-market` (feature-gated).
#[cfg(feature = "market")]
pub mod market {
    pub use paft_market::error::{self, MarketError};
    pub use paft_market::market::{action, news, options, orderbook, quote};
    pub use paft_market::requests;
    pub use paft_market::responses;
}

/// Namespaced access to `paft-money` types.
pub mod money {
    pub use paft_money::{
        Currency, ExchangeRate, IsoCurrency, MinorUnitError, Money, MoneyError,
        clear_currency_metadata, currency_metadata, set_currency_metadata,
        try_normalize_currency_code,
    };
    #[cfg(feature = "money-formatting")]
    pub use paft_money::{Locale, LocalizedMoney};
}

/// Namespaced access to `paft-fundamentals` (feature-gated).
#[cfg(feature = "fundamentals")]
pub mod fundamentals {
    pub use paft_fundamentals::{analysis, esg, holders, profile, statements};
}

/// Namespaced access to `paft-aggregates` (feature-gated).
#[cfg(feature = "aggregates")]
pub mod aggregates {
    pub use paft_aggregates::{FastInfo, Info};
}

/// Namespaced access to `paft-prediction` (feature-gated).
#[cfg(feature = "prediction")]
pub mod prediction {
    pub use paft_prediction::{Market, Token};
}

/// Frequently used types for convenient imports.
pub mod prelude;
