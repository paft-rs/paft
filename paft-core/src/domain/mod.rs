//! Core domain types used throughout the library.

pub mod exchange;
pub mod instrument;
pub mod market_state;
pub mod period;
pub use crate::{
    impl_display_via_code, string_enum_closed, string_enum_closed_with_code, string_enum_with_code,
};
pub use exchange::Exchange;
pub use instrument::{AssetKind, Instrument};
pub use market_state::MarketState;
pub use paft_utils::{Canonical, CanonicalError, StringCode, canonicalize};
pub use period::Period;
