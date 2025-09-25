//! Core domain types for the paft ecosystem.

#![warn(missing_docs)]

pub mod error;
pub mod exchange;
pub mod instrument;
pub mod market_state;
pub mod period;

pub use error::DomainError;
pub use exchange::Exchange;
pub use instrument::{AssetKind, Instrument};
pub use market_state::MarketState;
pub use period::Period;

#[cfg(feature = "dataframe")]
pub use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

pub use paft_utils::{Canonical, CanonicalError, StringCode, canonicalize};

pub use paft_core::{
    impl_display_via_code, string_enum_closed, string_enum_closed_with_code, string_enum_with_code,
};
