//! Unified public API for the paft workspace.
#![warn(missing_docs)]

/// Namespaced access to `paft-core`.
pub mod core {
    pub use paft_core::PaftError;
    pub use paft_core::domain;
    pub use paft_core::error;
    #[cfg(feature = "dataframe")]
    pub use paft_utils::dataframe;
}

/// Namespaced access to `paft-market` (feature-gated).
#[cfg(feature = "market")]
pub mod market {
    pub use paft_market::error::{self, MarketError};
    pub use paft_market::market::{action, news, options, quote};
    pub use paft_market::requests;
    pub use paft_market::responses;
}

/// Namespaced access to `paft-money` types.
pub mod money {
    pub use paft_money::{
        Currency, ExchangeRate, MinorUnitError, Money, MoneyError, clear_currency_minor_units,
        currency_minor_units, set_currency_minor_units, try_normalize_currency_code,
    };

    /// Re-export `iso_currency::Currency` for convenience.
    pub use iso_currency::Currency as IsoCurrency;
}

/// Namespaced access to `paft-fundamentals` (feature-gated).
#[cfg(feature = "fundamentals")]
pub mod fundamentals {
    pub use paft_fundamentals::{analysis, esg, holders, profile, statements};
}

/// Frequently used types for convenient imports.
pub mod prelude;
