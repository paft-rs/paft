//! Currency and money primitives for the paft ecosystem.

#![warn(missing_docs)]

pub mod currency;
pub mod currency_utils;
/// Error types shared across the money crate.
pub mod error;
pub mod money;

pub use currency::Currency;
pub use currency_utils::{
    MAX_DECIMAL_PRECISION, MAX_MINOR_UNIT_DECIMALS, MinorUnitError, clear_currency_minor_units,
    currency_minor_units, set_currency_minor_units, try_normalize_currency_code,
};
pub use error::MoneyParseError;
pub use money::{ExchangeRate, Money, MoneyError};
