//! Core types and utilities for the paft ecosystem.
#![warn(missing_docs)]

/// Core cross-cutting domain types (currency, exchange, period, money, market state, instrument).
pub mod domain;
/// Error definitions shared across crates.
pub mod error;
/// Private serde helper modules for custom serialization patterns.
pub mod serde_helpers;

#[cfg(feature = "dataframe")]
/// DataFrame conversion traits for paft
pub mod dataframe;

pub use error::PaftError;

#[cfg(feature = "dataframe")]
pub use dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
