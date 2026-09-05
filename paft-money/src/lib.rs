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
//! Once a scale is known for a code, [`set_currency_metadata`] refuses to
//! change `minor_units`; use [`override_currency_metadata`] when a scale change
//! is intentional. `Money` captures the resolved scale at construction, so
//! existing values are not reinterpreted by later registry changes.
//! Serialized `Money` values also carry this captured `minor_units` scale. On
//! deserialization, the serialized scale is validated against the amount and is
//! enough to reconstruct values whose custom metadata is currently absent. If
//! metadata is present but resolves to a different scale, deserialization fails
//! rather than silently changing equality, hashing, or arithmetic compatibility.
//! Metadata display fields are the source of truth for non-ISO currency names
//! and for localized formatting metadata. ISO currencies keep their ISO 4217
//! name and, when ISO defines an exponent, their ISO minor-unit scale.
//! Built-in non-ISO metadata uses documented native denominations, not venue
//! increments or display precision. `USDC`, `USDT`, `BNB`, and `AVAX` have no
//! default metadata because their scales depend on network or asset variant;
//! register the intended scale before constructing settlement `Money`.
//! The registry is keyed only by code. Use distinct application-defined codes
//! when different denominations must coexist in the same process.
//!
//! Using metals/funds (recommended defaults):
//! - Gold `XAU`: 3 or 6 decimal places are common; choose per domain needs.
//! - Silver `XAG`: similar; often 3.
//! - Platinum `XPT`: often 3.
//! - Special Drawing Rights `XDR`: 6 is common. These are recommendations; the
//!   appropriate scale is domain-driven. Always register the scale you need.
//!
//! # Decimal contract
//!
//! [`paft_decimal::Decimal`] always re-exports `rust_decimal::Decimal`: a 96-bit
//! coefficient with scale 0 through 28. Magnitude and fractional precision share
//! that coefficient budget. PAFT parsing and canonical serde preserve numeric
//! value exactly or reject it. Native decimal operations retain upstream
//! semantics, and constructors taking an existing decimal cannot detect prior
//! precision loss. Caller-defined provider metadata controls its own serde.
//!
//! [`Price`] and [`MonetaryAmount`] arithmetic is exact-or-error. [`Money`]
//! arithmetic uses upstream precision rounding, followed by settlement rounding
//! where applicable. Ratios and exchange-rate inverses can round at decimal
//! precision. `DataFrame` scale reduction uses half-even rounding.
//!
//! Currency minor-unit scales remain capped at 18 decimal places
//! ([`MAX_MINOR_UNIT_DECIMALS`]). [`Money::as_minor_units`] scales the coefficient
//! exactly in `i128`, so the count may exceed the decimal coefficient range.
//! Scaled-unit constructors accept numeric representability, removing trailing
//! zeros when necessary while preserving the supplied value exactly.
//!
//! # Currency value types
//!
//! The ecosystem exposes complementary concrete types for different financial
//! meanings:
//! - [`paft_decimal`](https://docs.rs/paft-decimal): fixed-width decimal helpers
//!   such as [`paft_decimal::parse_decimal`], [`paft_decimal::from_minor_units`],
//!   [`paft_decimal::zero`], and [`paft_decimal::one`].
//! - [`Money`]: settled or payable amounts that enforce currency exponents and
//!   settlement-ready rounding.
//! - [`Price`]: full-precision per-unit/per-share quoted values.
//! - [`MonetaryAmount`]: full-precision currency-denominated totals and
//!   intermediate values before final settlement rounding.
//! - [`QuantityAmount`]: full-precision non-negative market quantities whose
//!   unit is supplied by surrounding context.
//!
//! ```rust
//! # use paft_money::IsoCurrency;
//! # use paft_decimal::{self as decimal, RoundingStrategy};
//! # use paft_money::{Currency, MonetaryAmount, MoneyError, Price, QuantityAmount};
//! # fn run() -> Result<(), MoneyError> {
//! let usd = Currency::Iso(IsoCurrency::USD);
//!
//! // Quotes preserve provider precision beyond settlement minor units.
//! let quote = Price::from_canonical_str("1.3578", usd.clone())?;
//! let quantity = QuantityAmount::from_decimal(decimal::from_minor_units(250, 2)).unwrap();
//! let exact_total = quote.try_total(&quantity)?;
//!
//! // Intermediate totals stay exact until settlement.
//! let adjustment = MonetaryAmount::from_canonical_str("0.0049", usd)?;
//! let subtotal = exact_total.try_add(&adjustment)?;
//!
//! let settled = subtotal.to_money_with(
//!     RoundingStrategy::MidpointAwayFromZero,
//!     None,
//! )?;
//! assert_eq!(settled.format(), "3.4 USD");
//! # Ok(()) }
//! # run().unwrap();
//! ```
//!
//! # Quickstart
//!
//! Create money in ISO currencies, add and subtract safely, serialize with
//! stable representations, and convert via explicit exchange rates.
//!
//! ```rust
//! # use paft_money::IsoCurrency;
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
//! # use paft_money::IsoCurrency;
//! # use paft_decimal::{Decimal, RoundingStrategy};
//! # use paft_money::{Currency, Money, ExchangeRate};
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
//! Amounts serialize as strings (to avoid exponent notation), currencies
//! serialize as their codes, and `Money` serializes the captured `minor_units`
//! scale that participates in equality, hashing, and arithmetic compatibility.
//! Example:
//!
//! ```rust
//! # use paft_money::IsoCurrency;
//! # use paft_money::{Currency, Money};
//! let usd = Money::from_canonical_str("12.34", Currency::Iso(IsoCurrency::USD)).unwrap();
//! let json = serde_json::to_string(&usd).unwrap();
//! assert_eq!(json, "{\"amount\":\"12.34\",\"currency\":\"USD\",\"minor_units\":2}");
//! ```
//!
//! Deserialization validates the amount against serialized `minor_units`. If
//! metadata for the currency is currently registered and resolves to a different
//! scale, the payload is rejected; if metadata is absent for a custom/ISO-None
//! currency, the serialized scale preserves the captured settlement semantics.
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
//! Existing `Money` values retain the scale resolved at construction. Updating
//! or clearing the process-local registry can affect future construction and
//! formatting metadata, but not minor-unit conversion for values that already
//! exist. Serialized `Money` values carry that retained scale as `minor_units`;
//! conflicting current metadata is rejected at deserialization instead of
//! silently changing the value's identity.
//! For modeled non-ISO currencies such as `BTC`, `ETH`, and `XMR`, metadata is
//! also the source of truth for `Currency::full_name()`. ISO currency names are
//! resolved from ISO 4217 even if metadata is registered for formatting.
//!
//! # Feature flags
//!
//! - `dataframe`: enables `serde`/`polars`/`df-derive-macros` integration for dataframes.
//! - `panicking-money-ops`: implements `Add`/`Sub`/`Mul`/`Div` for `Money` that
//!   assert on invalid operations. Prefer the `try_*` methods for fallible APIs.
//! - `money-formatting`: opt-in locale-aware formatting and strict parsing for [`Money`].
//!
//! When `money-formatting` is enabled you opt into localized rendering explicitly:
//! ```rust
//! # #[cfg(feature = "money-formatting")] {
//! # use paft_money::IsoCurrency;
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
//! Canonical serde and numeric types are independent of PAFT features; see
//! [`MAX_DECIMAL_PRECISION`] and [`MAX_MINOR_UNIT_DECIMALS`] for limits that
//! affect scaling and minor-unit conversions.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod amount;
pub(crate) mod decimal;
mod exact;
#[cfg(feature = "money-formatting")]
mod format;
mod locale;
#[cfg(feature = "money-formatting")]
mod parser;

pub mod currency;
pub mod currency_utils;
/// Error types shared across the money crate.
pub mod error;
pub mod money;
mod price;
mod quantity;

pub use amount::MonetaryAmount;
pub use currency::{Currency, OtherCurrency};
pub use currency_utils::{
    CurrencyMetadata, MAX_DECIMAL_PRECISION, MAX_MINOR_UNIT_DECIMALS, MinorUnitError,
    clear_currency_metadata, currency_metadata, override_currency_metadata, set_currency_metadata,
    try_normalize_currency_code,
};
pub use error::{MoneyError, MoneyParseError};
pub use locale::Locale;
#[cfg(feature = "money-formatting")]
pub use money::LocalizedMoney;
pub use money::{ExchangeRate, Money};
pub use price::{Price, PriceAmount};
pub use quantity::QuantityAmount;

/// Re-export `iso_currency::Currency` for convenience.
pub use iso_currency::Currency as IsoCurrency;
