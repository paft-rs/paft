//! Core domain types for the paft ecosystem.
//!
//! This crate defines strongly-typed primitives for instruments, exchanges,
//! market sessions, identifiers, and financial periods used across the paft
//! ecosystem. Types are designed to be:
//! - Canonical and stable in string form (for serde, display, and storage)
//! - Liberal in what they accept when parsing (aliases, case-insensitivity),
//!   strict and consistent in emission
//! - Extensible via `Other(...)` variants where providers use custom codes
//!
//! # Quickstart
//!
//! ```rust
//! use paft_domain::{Instrument, AssetKind, Exchange, Period};
//!
//! // Create an instrument from a symbol and kind
//! let aapl = Instrument::from_symbol_and_exchange(
//!     "AAPL",
//!     Exchange::NASDAQ,
//!     AssetKind::Equity,
//! );
//! assert_eq!(aapl.symbol(), "AAPL");
//! assert!(aapl.exchange().is_some());
//!
//! // Parse financial periods from flexible inputs and get canonical form
//! let q4 = "2023-Q4".parse::<Period>().unwrap();
//! assert_eq!(q4.to_string(), "2023Q4");
//! let next = q4.next_quarter().unwrap();
//! assert_eq!(next.to_string(), "2024Q1");
//! ```
//!
//! # Serde
//! All domain types implement serde with stable string representations that match
//! their `Display` output. Unknown provider codes round-trip via `Other` where
//! applicable.
//!
//! # Feature flags
//! - `rust-decimal` (default): use `paft-money` with `rust-decimal`
//! - `bigdecimal`: use `paft-money` with `bigdecimal`
//! - `dataframe`: enable `paft-utils` dataframe traits for convenient export
//! - `isin-validate`: strict ISIN normalization/validation
//! - `figi-validate`: strict FIGI checksum validation

#![warn(missing_docs)]

pub mod error;
pub mod exchange;
pub mod identifiers;
pub mod instrument;
pub mod market_state;
pub mod period;

pub use error::DomainError;
pub use exchange::Exchange;
pub use identifiers::{Figi, Isin};
pub use instrument::{AssetKind, Instrument};
pub use market_state::MarketState;
pub use period::Period;

#[cfg(feature = "dataframe")]
pub use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

pub use paft_utils::{Canonical, CanonicalError, StringCode, canonicalize};

pub use paft_core::{impl_display_via_code, string_enum_closed_with_code, string_enum_with_code};
