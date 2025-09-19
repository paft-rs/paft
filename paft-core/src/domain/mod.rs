//! Core domain types used throughout the library.

pub mod currency;
pub mod currency_utils;
pub mod exchange;
pub mod instrument;
pub mod market_state;
pub mod money;
pub mod period;
pub mod string_canonical;

pub use crate::{
    impl_display_via_code, string_enum, string_enum_closed, string_enum_closed_with_code,
    string_enum_with_code,
};
pub use currency::Currency;
pub use currency_utils::{
    MinorUnitError, clear_currency_minor_units, currency_minor_units, set_currency_minor_units,
    try_normalize_currency_code,
};
pub use exchange::Exchange;
pub use instrument::{AssetKind, Instrument};
pub use market_state::MarketState;
pub use money::{ExchangeRate, Money, MoneyError};
pub use period::Period;
pub use string_canonical::{Canonical, canonicalize};
