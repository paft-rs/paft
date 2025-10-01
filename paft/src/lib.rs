//! Unified public API for the paft workspace.
#![warn(missing_docs)]

/// Namespaced access to `paft-core`.
pub mod core {
    pub use paft_core::PaftError;
    pub use paft_core::error;
    #[cfg(feature = "dataframe")]
    pub use paft_utils::dataframe;
}

/// Namespaced access to `paft-domain` (feature-gated).
#[cfg(feature = "domain")]
pub mod domain {
    pub use paft_domain::{
        AssetKind, Canonical, CanonicalError, DomainError, Exchange, Figi, Instrument, Isin,
        MarketState, Period, StringCode, canonicalize,
    };
    #[cfg(feature = "dataframe")]
    pub use paft_domain::{ToDataFrame, ToDataFrameVec};
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
        Currency, ExchangeRate, IsoCurrency, MinorUnitError, Money, MoneyError,
        clear_currency_metadata, currency_metadata, set_currency_metadata,
        try_normalize_currency_code,
    };
}

/// Namespaced access to `paft-fundamentals` (feature-gated).
#[cfg(feature = "fundamentals")]
pub mod fundamentals {
    pub use paft_fundamentals::{analysis, esg, holders, profile, statements};
}

/// Frequently used types for convenient imports.
pub mod prelude;
