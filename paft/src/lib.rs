//! Unified public API for the paft workspace.
#![warn(missing_docs)]

// Re-export core types unconditionally if the `core` feature is enabled.
// Since `core` is part of the new default, these will usually be available.
#[cfg(feature = "core")]
pub use paft_core::{
    domain::{
        AssetKind, Currency, Exchange, ExchangeRate, Instrument, MarketState, Money, MoneyError,
        Period,
    },
    error::PaftError,
};

// Re-export dataframe traits if the `dataframe` feature is enabled.
#[cfg(feature = "dataframe")]
pub use paft_core::dataframe::{ToDataFrame, ToDataFrameVec};

// Conditionally re-export the entire market crate as a module.
#[cfg(feature = "market")]
pub use paft_market as market;

// Conditionally re-export the entire fundamentals crate as a module.
#[cfg(feature = "fundamentals")]
pub use paft_fundamentals as fundamentals;

/// Frequently used types for convenient imports.
pub mod prelude;
