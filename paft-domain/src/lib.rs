//! Core domain types for the paft ecosystem.
//!
//! This crate defines strongly-typed primitives for instruments, exchanges,
//! market sessions, security identifiers (`Symbol`, `Figi`, `Isin`), financial
//! periods, and lookback horizons used across the paft ecosystem. Types are
//! designed to be:
//! - Canonical and stable in string form (for serde, display, and storage)
//! - Liberal in what they accept when parsing (aliases, case-insensitivity),
//!   strict and consistent in emission
//! - Extensible via `Other(...)` variants where providers use custom codes
//!
//! # Quickstart
//!
//! ```rust
//! use paft_domain::{
//!     AssetKind, CalendarPeriod, Exchange, Horizon, Instrument, ReportingPeriod, Symbol,
//! };
//!
//! let symbol = Symbol::new("AAPL").unwrap();
//! let aapl = Instrument::from_symbol_and_exchange(
//!     symbol.as_str(),
//!     Exchange::NASDAQ,
//!     AssetKind::Equity,
//! ).unwrap();
//! assert_eq!(aapl.symbol.as_str(), "AAPL");
//! assert!(aapl.exchange.is_some());
//!
//! let q4 = "2023-Q4".parse::<ReportingPeriod>().unwrap();
//! assert_eq!(q4.to_string(), "2023Q4");
//! let calendar_q4 = CalendarPeriod::quarterly(2023, 4).unwrap();
//! assert_eq!(calendar_q4.start_date().to_string(), "2023-10-01");
//!
//! let horizon = "3mo".parse::<Horizon>().unwrap();
//! assert_eq!(horizon.to_string(), "3mo");
//! ```
//!
//! # Serde
//! All domain types implement serde with stable string representations that match
//! their `Display` output. Unknown provider codes round-trip via `Other` where
//! applicable.
//!
//! # Feature flags
//! - `tracing`: enable lightweight instrumentation on constructors and validators
//! - `dataframe`: enable `paft-utils` `DataFrame` traits for convenient export

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod exchange;
pub mod horizon;
pub mod identifiers;
pub mod instrument;
pub mod market_state;
pub mod period;

pub use error::DomainError;
pub use exchange::{Exchange, OtherExchange};
pub use horizon::{Horizon, OtherHorizon};
pub use identifiers::{Figi, Isin, Symbol};
pub use instrument::{AssetKind, Instrument, OtherAssetKind};
pub use market_state::MarketState;
pub use period::{
    CalendarPeriod, OtherPeriod, PeriodDate, PeriodYear, QuarterOfYear, ReportingPeriod,
};

#[cfg(feature = "dataframe")]
pub use paft_utils::dataframe::{Decimal128Encode, ToDataFrame, ToDataFrameVec};

pub use paft_utils::{Canonical, CanonicalError, StringCode, canonicalize};

pub use paft_core::{impl_display_via_code, string_enum_closed_with_code, string_enum_with_code};
