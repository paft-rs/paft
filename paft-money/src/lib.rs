//! Currency and money primitives for the paft ecosystem.
//!
//! Policy for ISO currencies without a minor-unit exponent (ISO-None):
//! - If ISO 4217 defines a minor unit for an ISO currency, that exponent is used.
//! - If ISO is silent (for example `XAU`, `XDR`), the crate consults the metadata
//!   registry by ISO code. If metadata is present, its `minor_units` is used.
//! - If no metadata is registered, operations that require a scale return
//!   `MoneyError::MetadataNotFound` with the offending currency.
//!
//! Registering metadata overlays:
//! Use [`set_currency_metadata`] to register a human-friendly name and scale:
//! ```rust
//! # use paft_money::set_currency_metadata;
//! set_currency_metadata("XAU", "Gold", 3).unwrap();
//! set_currency_metadata("XDR", "SDR", 6).unwrap();
//! ```
//!
//! Using metals/funds (recommended defaults):
//! - Gold `XAU`: 3 or 6 decimal places are common; choose per domain needs.
//! - Silver `XAG`: similar; often 3.
//! - Platinum `XPT`: often 3.
//! - Special Drawing Rights `XDR`: 6 is common. These are recommendations; the
//!   appropriate scale is domain-driven. Always register the scale you need.
//!
//! # Decimal backend
//!
//! The crate exposes a backend-agnostic [`Decimal`] type alongside
//! [`RoundingStrategy`]. By default the optional `rust-decimal` feature is
//! enabled, providing 28 fractional digits of precision with a fast fixed-size
//! representation. Alternatively enabling the `bigdecimal` feature (and
//! disabling the default) switches the backend to
//! [`bigdecimal`](https://docs.rs/bigdecimal) for effectively unbounded
//! precision backed by big integers. Enabling both features simultaneously is a
//! compile-time error, and at least one backend must be selected.
//!
//! The public API, serde representation (amounts encoded as strings, currencies
//! as ISO codes), and `DataFrame` integration remain stable across backends. The
//! primary trade-offs are performance (the `bigdecimal` backend may allocate
//! more often) and precision (see [`MAX_DECIMAL_PRECISION`]). Minor-unit scaling
//! always uses 64-bit integers and therefore remains capped at 18 decimal
//! places so that `10^scale` fits inside an `i128` when performing
//! conversions.

#![warn(missing_docs)]

pub mod currency;
pub mod currency_utils;
/// Decimal abstraction toggled by feature flags.
pub mod decimal;
/// Error types shared across the money crate.
pub mod error;
pub mod money;

pub use currency::Currency;
pub use currency_utils::{
    MAX_DECIMAL_PRECISION, MAX_MINOR_UNIT_DECIMALS, MinorUnitError, clear_currency_metadata,
    currency_metadata, set_currency_metadata, try_normalize_currency_code,
};
pub use decimal::{Decimal, RoundingStrategy};
pub use error::MoneyParseError;
pub use money::{ExchangeRate, Money, MoneyError};

/// Re-export `iso_currency::Currency` for convenience.
pub use iso_currency::Currency as IsoCurrency;
