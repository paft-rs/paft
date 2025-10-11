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
//! # #[cfg(feature = "money-formatting")]
//! # {
//! # use paft_money::Locale;
//! set_currency_metadata("XAU", "Gold", 3, "XAU", true, Locale::EnUs).unwrap();
//! set_currency_metadata("XDR", "SDR", 6, "XDR", true, Locale::EnUs).unwrap();
//! # }
//! # #[cfg(not(feature = "money-formatting"))]
//! # {
//! use paft_money::Locale;
//! set_currency_metadata("XAU", "Gold", 3, "XAU", true, Locale::EnUs).unwrap();
//! set_currency_metadata("XDR", "SDR", 6, "XDR", true, Locale::EnUs).unwrap();
//! # }
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
//! [`RoundingStrategy`]. By default it uses
//! [`rust_decimal`](https://docs.rs/rust_decimal) providing 28 fractional
//! digits of precision with a fast fixed-size representation. Alternatively,
//! enabling the `bigdecimal` feature switches the backend to
//! [`bigdecimal`](https://docs.rs/bigdecimal) for effectively unbounded
//! precision backed by big integers.
//!
//! The public API, serde representation (amounts encoded as strings, currencies
//! as ISO codes), and `DataFrame` integration remain stable across backends. The
//! primary trade-offs are performance (the `bigdecimal` backend may allocate
//! more often) and precision (see [`MAX_DECIMAL_PRECISION`]). Minor-unit scaling
//! always uses 64-bit integers and therefore remains capped at 18 decimal
//! places so that `10^scale` fits inside an `i128` when performing
//! conversions.
//!
//! # Quickstart
//!
//! Create money in ISO currencies, add and subtract safely, serialize with
//! stable representations, and convert via explicit exchange rates.
//!
//! ```rust
//! # use iso_currency::Currency as IsoCurrency;
//! # use paft_money::{Currency, Money};
//! # fn run() -> Result<(), paft_money::MoneyError> {
//! let price = Money::from_canonical_str("12.34", Currency::Iso(IsoCurrency::USD))?;
//! let tax   = Money::from_canonical_str("1.23",  Currency::Iso(IsoCurrency::USD))?;
//! let total = price.try_add(&tax)?;
//! assert_eq!(total.format(), "13.57 USD");
//!
//! // Cross-currency addition is rejected
//! let eur = Money::from_canonical_str("5", Currency::Iso(IsoCurrency::EUR))?;
//! assert!(price.try_add(&eur).is_err());
//! # Ok(()) } run().unwrap();
//! ```
//!
//! # Currency conversion
//!
//! Use an [`ExchangeRate`] to convert with explicit rounding.
//!
//! ```rust
//! # use iso_currency::Currency as IsoCurrency;
//! # use paft_money::{Currency, Money, ExchangeRate, Decimal, RoundingStrategy};
//! # fn run() -> Result<(), paft_money::MoneyError> {
//! let usd = Money::from_canonical_str("10.00", Currency::Iso(IsoCurrency::USD))?;
//! let rate = ExchangeRate::new(
//!     Currency::Iso(IsoCurrency::USD),
//!     Currency::Iso(IsoCurrency::EUR),
//!     Decimal::from(9) / Decimal::from(10), // 1 USD = 0.9 EUR
//! )?;
//! let eur = usd.try_convert_with(&rate, RoundingStrategy::MidpointAwayFromZero)?;
//! assert_eq!(eur.currency().code(), "EUR");
//! # Ok(()) } run().unwrap();
//! ```
//!
//! # Serde
//!
//! Amounts serialize as strings (to avoid exponent notation); currencies serialize
//! as their codes. Example:
//!
//! ```rust
//! # use iso_currency::Currency as IsoCurrency;
//! # use paft_money::{Currency, Money};
//! let usd = Money::from_canonical_str("12.34", Currency::Iso(IsoCurrency::USD)).unwrap();
//! let json = serde_json::to_string(&usd).unwrap();
//! assert_eq!(json, "{\"amount\":\"12.34\",\"currency\":\"USD\"}");
//! ```
//!
//! # Currency metadata overlays
//!
//! For ISO codes without a prescribed minor-unit exponent (e.g., `XAU`, `XDR`),
//! register a scale so that rounding and minor-unit conversions are well-defined:
//!
//! ```rust
//! # use paft_money::set_currency_metadata;
//! # #[cfg(feature = "money-formatting")]
//! # {
//! # use paft_money::Locale;
//! set_currency_metadata("XAU", "Gold", 3, "XAU", true, Locale::EnUs).unwrap();
//! set_currency_metadata("XDR", "SDR", 6, "XDR", true, Locale::EnUs).unwrap();
//! # }
//! # #[cfg(not(feature = "money-formatting"))]
//! # {
//! use paft_money::Locale;
//! set_currency_metadata("XAU", "Gold", 3, "XAU", true, Locale::EnUs).unwrap();
//! set_currency_metadata("XDR", "SDR", 6, "XDR", true, Locale::EnUs).unwrap();
//! # }
//! ```
//!
//! # Feature flags
//!
//! - `bigdecimal`: switch to arbitrary precision decimals (slower, allocates for large values).
//! - `dataframe`: enables `serde`/`polars`/`df-derive` integration for dataframes.
//! - `panicking-money-ops`: implements `Add`/`Sub`/`Mul`/`Div` for `Money` that
//!   assert on invalid operations. Prefer the `try_*` methods for fallible APIs.
//! - `money-formatting`: opt-in locale-aware formatting and strict parsing for [`Money`].
//!
//! When `money-formatting` is enabled you opt into localized rendering explicitly:
//! ```rust
//! # #[cfg(feature = "money-formatting")] {
//! # use iso_currency::Currency as IsoCurrency;
//! # use paft_money::{Currency, Locale, Money};
//! let eur = Money::from_canonical_str("1234.56", Currency::Iso(IsoCurrency::EUR)).unwrap();
//! assert_eq!(format!("{eur}"), "1234.56 EUR"); // canonical display stays locale-neutral
//! assert_eq!(eur.format_with_locale(Locale::EnEu).unwrap(), "€1.234,56");
//! assert_eq!(
//!     Money::from_str_locale("€1.234,56", Currency::Iso(IsoCurrency::EUR), Locale::EnEu)
//!         .unwrap()
//!         .format(),
//!     "1234.56 EUR"
//! );
//! assert_eq!(
//!     format!("{}", eur.localized(Locale::EnEu).with_code()),
//!     "€1.234,56 EUR"
//! );
//! # }
//! ```
//!
//! Regardless of backend, serde and the high-level API remain stable; see
//! [`MAX_DECIMAL_PRECISION`] and [`MAX_MINOR_UNIT_DECIMALS`] for limits that
//! affect scaling and minor-unit conversions.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::cargo_common_metadata)]

#[cfg(feature = "money-formatting")]
mod format;
mod locale;
#[cfg(feature = "money-formatting")]
mod parser;

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
pub use error::{MoneyError, MoneyParseError};
pub use locale::Locale;
#[cfg(feature = "money-formatting")]
pub use money::LocalizedMoney;
pub use money::{ExchangeRate, Money};

/// Re-export `iso_currency::Currency` for convenience.
pub use iso_currency::Currency as IsoCurrency;
