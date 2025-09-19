//! Unified public API for the paft workspace.
#![warn(missing_docs)]

/// Namespaced access to `paft-core`.
pub mod core {
    pub use paft_core::PaftError;
    #[cfg(feature = "dataframe")]
    pub use paft_core::dataframe;
    pub use paft_core::domain;
    pub use paft_core::error;
}

/// Namespaced access to `paft-market` (feature-gated).
#[cfg(feature = "market")]
pub mod market {
    pub use paft_market::error::{self, MarketError};
    pub use paft_market::market::{action, news, options, quote};
    pub use paft_market::requests;
    pub use paft_market::responses;
}

/// Namespaced access to `paft-fundamentals` (feature-gated).
#[cfg(feature = "fundamentals")]
pub mod fundamentals {
    pub use paft_fundamentals::{analysis, esg, holders, profile, statements};
}

/// Frequently used types for convenient imports.
pub mod prelude;
