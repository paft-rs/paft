//! Money type for representing financial values with currency.

use crate::decimal::{self, Decimal, RoundingStrategy, ToPrimitive};
use crate::error::MoneyError;
use serde::{Deserialize, Deserializer, Serialize};
use std::hash::{Hash, Hasher};
#[cfg(feature = "panicking-money-ops")]
use std::ops::{Add, Div, Mul, Sub};

#[cfg(feature = "money-formatting")]
use std::fmt;

#[cfg(feature = "money-formatting")]
use std::borrow::Cow;

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

use crate::currency::Currency;
#[cfg(feature = "money-formatting")]
use crate::currency_utils::MAX_DECIMAL_PRECISION;
use crate::currency_utils::MAX_MINOR_UNIT_DECIMALS;
use crate::exact::{
    CurrencyAmount, canonical_amount_format, checked_add_decimal, checked_div_decimal,
    checked_mul_decimal, checked_sub_decimal, copy_decimal, decimal_from_scaled_units,
    decimal_scale, parse_canonical_decimal,
};
#[cfg(feature = "money-formatting")]
use crate::format::{FormatItem, Formatter, Params};
#[cfg(feature = "money-formatting")]
use crate::locale::Locale;
#[cfg(feature = "money-formatting")]
use crate::parser;

/// Represents an exchange rate between two currencies.
///
/// Construct via [`ExchangeRate::new`] or by deserialization. Both paths
/// enforce the same invariants — the `Deserialize` impl funnels through
/// [`ExchangeRate::new`] so a stray JSON document like
/// `{"from":"USD","to":"USD","rate":"-1"}` is rejected, not silently
/// accepted as a structurally valid rate that downstream code would have
/// to re-validate.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct ExchangeRate {
    /// The source currency.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    from: Currency,
    /// The target currency.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    to: Currency,
    /// The exchange rate (how many units of 'to' currency per 1 unit of 'from' currency).
    #[serde(with = "paft_decimal::serde::canonical_str")]
    rate: Decimal,
}

impl ExchangeRate {
    /// Creates a new `ExchangeRate` instance with validation.
    ///
    /// Identity rates (`from == to`) are accepted only when the rate is
    /// exactly `1` — anything else describes a non-existent currency
    /// translation. Negative or zero rates are always rejected.
    ///
    /// # Errors
    /// Returns `MoneyError::InvalidExchangeRate` when `rate` is not strictly
    /// positive, or when `from == to` and `rate != 1`.
    pub fn new(from: Currency, to: Currency, rate: Decimal) -> Result<Self, MoneyError> {
        if rate <= decimal::zero() {
            return Err(MoneyError::InvalidExchangeRate { rate });
        }
        if from == to && rate != decimal::one() {
            return Err(MoneyError::InvalidExchangeRate { rate });
        }
        Ok(Self { from, to, rate })
    }

    /// Returns the source currency.
    #[must_use]
    pub const fn from(&self) -> &Currency {
        &self.from
    }

    /// Returns the target currency.
    #[must_use]
    pub const fn to(&self) -> &Currency {
        &self.to
    }

    /// Returns the exchange rate.
    ///
    /// The value is cloned from the active decimal backend.
    #[must_use]
    pub fn rate(&self) -> Decimal {
        copy_decimal(&self.rate)
    }

    /// Creates the inverse exchange rate (swaps from/to and inverts the rate).
    ///
    /// Use [`ExchangeRate::try_inverse`] when the input rate is not already
    /// known to be representable after inversion.
    ///
    /// # Panics
    /// Panics when the inverted rate overflows the active decimal backend.
    /// This is only possible under the fixed-width `rust_decimal` backend.
    #[must_use]
    pub fn inverse(&self) -> Self {
        self.try_inverse()
            .expect("inverse exchange rate overflows decimal backend")
    }

    /// Tries to create the inverse exchange rate.
    ///
    /// This swaps `from` and `to`, then computes `1 / rate` using checked
    /// division so very small fixed-width decimal rates return an error instead
    /// of panicking.
    ///
    /// # Errors
    /// Returns [`MoneyError::ConversionError`] when the inverted rate cannot be
    /// represented by the active decimal backend. This is only possible under
    /// the fixed-width `rust_decimal` backend.
    pub fn try_inverse(&self) -> Result<Self, MoneyError> {
        let one = decimal::one();
        let rate = checked_div_decimal(&one, &self.rate)?;

        Ok(Self {
            from: self.to.clone(),
            to: self.from.clone(),
            rate,
        })
    }

    /// Checks if this exchange rate can be used to convert the given money.
    #[must_use]
    pub fn is_compatible(&self, money: &Money) -> bool {
        money.currency == self.from
    }
}

/// Shadow type used for deserializing [`ExchangeRate`].
///
/// Captures the on-the-wire shape and then routes through
/// [`ExchangeRate::new`] so validation cannot be skipped. Any field added to
/// `ExchangeRate` must be reflected here too.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExchangeRateShadow {
    from: Currency,
    to: Currency,
    #[serde(with = "paft_decimal::serde::canonical_str")]
    rate: Decimal,
}

impl<'de> Deserialize<'de> for ExchangeRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let shadow = ExchangeRateShadow::deserialize(deserializer)?;
        Self::new(shadow.from, shadow.to, shadow.rate).map_err(serde::de::Error::custom)
    }
}

/// Represents a financial value with its currency, enforcing safe operations.
///
/// Construct via [`Money::new`] (rounds to the currency's exponent),
/// [`Money::new_exact`] (rejects over-precise input), or
/// [`Money::from_canonical_str`] (which delegates to `new_exact`).
/// Deserialization carries and validates the captured `minor_units` scale, so
/// JSON cannot silently reinterpret the settlement scale through whatever
/// metadata registry happens to exist in the receiving process.
///
/// The resolved minor-unit scale is captured at construction. This keeps an
/// existing `Money` value's arithmetic and minor-unit conversion stable even
/// if the process-local metadata registry is later changed or cleared.
///
/// `Hash` and `PartialEq` use a canonical string representation of the
/// numeric value plus the captured minor-unit scale, so two `Money` values
/// that differ only in trailing zero-scale digits compare equal and hash to
/// the same bucket, while values created under incompatible scale metadata do
/// not collapse into one identity.
///
/// ```
/// # use paft_money::IsoCurrency;
/// # use paft_money::{Currency, Money};
/// let usd = Money::from_canonical_str("12.34", Currency::Iso(IsoCurrency::USD)).unwrap();
/// let json = serde_json::to_string(&usd).unwrap();
/// assert_eq!(json, "{\"amount\":\"12.34\",\"currency\":\"USD\",\"minor_units\":2}");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Money {
    /// The numeric value.
    #[serde(with = "paft_decimal::serde::canonical_str")]
    amount: Decimal,
    /// The currency.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    currency: Currency,
    /// Minor-unit scale resolved when the value was created.
    #[cfg_attr(feature = "dataframe", df_derive(skip))]
    minor_units: u8,
}

impl Hash for Money {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash currency directly
        self.currency.hash(state);
        // Scale is part of the captured settlement semantics for this value.
        self.minor_units.hash(state);
        // Hash a canonical numeric representation so equivalent scales collide
        decimal::to_canonical_string(&self.amount).hash(state);
    }
}

impl Money {
    /// Creates a new `Money` instance, **rounding** the amount to the
    /// currency's minor units using
    /// [`RoundingStrategy::MidpointAwayFromZero`].
    ///
    /// Use this when callers explicitly want lossy quantization (e.g. a UI
    /// computation, an unrounded fraction from a calculation pipeline). When
    /// the input scale must be preserved exactly — including for
    /// deserialization or string parsing — use [`Money::new_exact`] instead.
    ///
    /// # Errors
    /// Returns `MoneyError::MetadataNotFound` when metadata is not registered for a custom currency.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(amount: Decimal, currency: Currency) -> Result<Self, MoneyError> {
        let (minor_units, scale) = Self::scale_for_currency(&currency)?;
        let rounded = Self::round_amount_to_scale(&amount, scale);
        Ok(Self {
            amount: rounded,
            currency,
            minor_units,
        })
    }

    /// Creates a new `Money` instance, **rejecting** any amount whose
    /// fractional precision exceeds the currency's `decimal_places()`.
    ///
    /// Trailing zeros do not count as precision (so `1.230` is a valid USD
    /// amount because rounding to two places leaves `1.23` numerically
    /// unchanged). The accepted amount is canonicalized to the currency's
    /// exact scale, which guarantees that two `Money` values built from
    /// equal numbers — for example one constructed via the API and one
    /// arriving over the wire — share the same internal representation
    /// regardless of how their string form was written.
    ///
    /// This is the constructor used by [`Money::from_canonical_str`]. Serde
    /// deserialization applies the same exact-scale validation to the
    /// serialized `minor_units` field.
    ///
    /// # Errors
    /// - Returns `MoneyError::MetadataNotFound` when metadata is not registered.
    /// - Returns `MoneyError::PrecisionExceeded` when the supplied amount has
    ///   more fractional digits than the currency's exponent permits.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    // We take `amount` by value to mirror `Money::new` and avoid forcing
    // callers (notably the deserialize path) to clone before construction.
    // The body uses `&amount` for validation and only consumes the canonical
    // value; keep the signature consistent across backends and deserialize paths.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new_exact(amount: Decimal, currency: Currency) -> Result<Self, MoneyError> {
        let (minor_units, scale) = Self::scale_for_currency(&currency)?;
        let canonical = Self::canonicalize_exact_amount(&amount, &currency, scale)?;
        Ok(Self {
            amount: canonical,
            currency,
            minor_units,
        })
    }

    /// Creates a new `Money` instance with zero amount and the specified currency.
    ///
    /// # Errors
    /// Returns `MoneyError::MetadataNotFound` when metadata is not registered for a custom currency.
    pub fn zero(currency: Currency) -> Result<Self, MoneyError> {
        Self::new(decimal::zero(), currency)
    }

    /// Returns the amount as a [`Decimal`].
    ///
    /// The value is cloned from the internal representation. Cloning is a
    /// cheap copy with the default backend, but incurs an
    /// allocation proportional to the number of digits when the `bigdecimal`
    /// feature is enabled.
    #[must_use]
    pub fn amount(&self) -> Decimal {
        copy_decimal(&self.amount)
    }

    /// Returns the `Currency`.
    #[must_use]
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Returns the minor-unit scale captured when this value was constructed.
    #[must_use]
    pub const fn minor_units(&self) -> u8 {
        self.minor_units
    }

    /// Returns the amount as the smallest currency unit (minor units).
    ///
    /// Uses checked multiplication so a value that would overflow the
    /// fixed-width `rust_decimal` backend surfaces as
    /// `MoneyError::ConversionError` instead of panicking.
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` when conversion cannot be performed.
    pub fn as_minor_units(&self) -> Result<i128, MoneyError> {
        let scale = Self::ensure_scale_within_limits(self.minor_units)?;

        // The cap on `scale` is enforced by `ensure_scale_within_limits`
        // (currently 18 dp) so `10^scale` always fits inside `i64`.
        let multiplier = Decimal::from(10_i64.pow(scale));
        let scaled = checked_mul_decimal(&self.amount, &multiplier)?;
        let integral = decimal::round_dp_with_strategy(&scaled, 0, RoundingStrategy::ToZero);
        if integral != scaled {
            return Err(MoneyError::ConversionError);
        }
        scaled.to_i128().ok_or(MoneyError::ConversionError)
    }

    /// Creates a new `Money` instance from a canonical decimal string and currency.
    ///
    /// Delegates to [`Money::new_exact`], so a string that carries more
    /// fractional precision than the currency exponent allows is rejected
    /// rather than silently rounded. Use [`Money::new`] explicitly if you
    /// want rounding behaviour from a previously parsed `Decimal`.
    ///
    /// # Errors
    /// - Returns `MoneyError::InvalidDecimal` when the string cannot be
    ///   parsed as a decimal.
    /// - Returns `MoneyError::PrecisionExceeded` when the parsed amount has
    ///   more fractional digits than the currency exponent permits.
    ///
    /// Leading and trailing whitespace is ignored and an optional leading `+`
    /// sign is supported. Scientific notation is rejected so that behaviour is
    /// consistent across decimal backends.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn from_canonical_str(amount: &str, currency: Currency) -> Result<Self, MoneyError> {
        let amount = parse_canonical_decimal(amount)?;
        Self::new_exact(amount, currency)
    }

    /// Creates a new Money instance from an integer amount in the currency's minor units.
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` when the currency precision exceeds supported limits
    /// (currently 18 decimal places to keep `10^scale` within `i128`) or the scaled value cannot
    /// be represented by the active decimal backend.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn from_minor_units(minor_units: i128, currency: Currency) -> Result<Self, MoneyError> {
        let (currency_minor_units, scale) = Self::scale_for_currency(&currency)?;
        let amount = decimal_from_scaled_units(minor_units, scale)?;
        Ok(Self {
            amount,
            currency,
            minor_units: currency_minor_units,
        })
    }

    /// Returns the amount as a canonical string with currency code (`"<amount> <CODE>"`).
    #[must_use]
    pub fn format(&self) -> String {
        canonical_amount_format(self)
    }

    /// Parses a human-formatted string using an explicit locale (strict grouping/decimal rules).
    ///
    /// The input must respect the separators and grouping pattern for `locale` and
    /// already match the currency exponent. No implicit rounding is performed.
    ///
    /// # Errors
    /// Returns [`MoneyError`] when the input fails validation or exceeds the currency scale.
    #[cfg(feature = "money-formatting")]
    pub fn from_str_locale(
        amount: &str,
        currency: Currency,
        locale: Locale,
    ) -> Result<Self, MoneyError> {
        let decimal = parser::parse_localized_str(amount, &currency, Some(locale), true)?;
        Self::new_exact(decimal, currency)
    }

    /// Parses a human-formatted string using the currency's metadata-defined default locale.
    ///
    /// # Errors
    /// Returns [`MoneyError`] when the input cannot be parsed or violates the currency scale.
    #[cfg(feature = "money-formatting")]
    pub fn from_default_locale_str(amount: &str, currency: Currency) -> Result<Self, MoneyError> {
        let default_locale = currency.default_locale();
        Self::from_str_locale(amount, currency, default_locale)
    }

    /// Formats the amount using the currency's default locale (symbol first when appropriate).
    ///
    /// # Errors
    /// Propagates [`MoneyError::InvalidAmountFormat`] when rounding for display fails.
    #[cfg(feature = "money-formatting")]
    pub fn to_localized_string(&self) -> Result<String, MoneyError> {
        self.localized(self.currency.default_locale()).into_string()
    }

    /// Formats the amount using an explicit locale (symbol included, code omitted).
    ///
    /// For Display integration use [`Money::localized`].
    ///
    /// # Errors
    /// Returns [`MoneyError::InvalidAmountFormat`] when rounding for display fails.
    #[cfg(feature = "money-formatting")]
    pub fn format_with_locale(&self, locale: Locale) -> Result<String, MoneyError> {
        self.localized(locale).into_string()
    }

    /// Returns a builder that renders this money value with the provided locale.
    #[must_use]
    #[cfg(feature = "money-formatting")]
    pub const fn localized(&self, locale: Locale) -> LocalizedMoney<'_> {
        LocalizedMoney::new(self, locale)
    }

    /// Renders the numeric portion with custom fraction digits (no symbol or code).
    ///
    /// The `fraction_digits` parameter pads with zeros or rounds as needed.
    /// Requests above [`MAX_DECIMAL_PRECISION`] are rejected to keep display
    /// formatting bounded.
    ///
    /// # Errors
    /// Returns [`MoneyError::FormatPrecisionExceeded`] when `fraction_digits`
    /// exceeds [`MAX_DECIMAL_PRECISION`], or
    /// [`MoneyError::InvalidAmountFormat`] when rounding fails (rare).
    #[cfg(feature = "money-formatting")]
    pub fn amount_string_with_locale(
        &self,
        locale: Locale,
        fraction_digits: u32,
    ) -> Result<String, MoneyError> {
        self.render_with_locale(locale, false, false, None, fraction_digits)
    }

    /// Addition that returns an error for currency mismatch.
    ///
    /// # Errors
    /// - Returns `MoneyError::CurrencyMismatch` when the operands use
    ///   different currencies.
    /// - Returns `MoneyError::MinorUnitMismatch` when the operands use the
    ///   same currency code but carry different captured minor-unit scales.
    /// - Returns `MoneyError::ConversionError` when the sum overflows the
    ///   active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_add(&self, rhs: &Self) -> Result<Self, MoneyError> {
        self.ensure_compatible_money(rhs)?;
        let sum = checked_add_decimal(&self.amount, &rhs.amount)?;
        Ok(Self::from_rounded_parts(
            &sum,
            self.currency.clone(),
            self.minor_units,
        ))
    }

    /// Subtraction that returns an error for currency mismatch.
    ///
    /// # Errors
    /// - Returns `MoneyError::CurrencyMismatch` when the operands use
    ///   different currencies.
    /// - Returns `MoneyError::MinorUnitMismatch` when the operands use the
    ///   same currency code but carry different captured minor-unit scales.
    /// - Returns `MoneyError::ConversionError` when the difference overflows
    ///   the active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_sub(&self, rhs: &Self) -> Result<Self, MoneyError> {
        self.ensure_compatible_money(rhs)?;
        let diff = checked_sub_decimal(&self.amount, &rhs.amount)?;
        Ok(Self::from_rounded_parts(
            &diff,
            self.currency.clone(),
            self.minor_units,
        ))
    }

    /// Multiplication that preserves the currency.
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` when the product overflows the
    ///   active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_mul(&self, rhs: &Decimal) -> Result<Self, MoneyError> {
        let product = checked_mul_decimal(&self.amount, rhs)?;
        Ok(Self::from_rounded_parts(
            &product,
            self.currency.clone(),
            self.minor_units,
        ))
    }

    /// Division that returns an error for division by zero.
    ///
    /// # Errors
    /// - Returns `MoneyError::DivisionByZero` when `rhs` is zero.
    /// - Returns `MoneyError::ConversionError` when the quotient overflows
    ///   the active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_div(&self, rhs: &Decimal) -> Result<Self, MoneyError> {
        let quotient = checked_div_decimal(&self.amount, rhs)?;
        Ok(Self::from_rounded_parts(
            &quotient,
            self.currency.clone(),
            self.minor_units,
        ))
    }

    /// Divides two `Money` values of the same currency, returning the unitless ratio.
    ///
    /// Use this for finance ratios such as "how many shares fit in a budget"
    /// (`budget.try_div_money(&price)`) or P/E-style quotients. The result is a
    /// pure [`Decimal`] and is **not** rounded to any currency precision.
    ///
    /// # Errors
    /// - [`MoneyError::CurrencyMismatch`] when the operands use different currencies.
    /// - [`MoneyError::MinorUnitMismatch`] when the operands use the same
    ///   currency code but carry different captured minor-unit scales.
    /// - [`MoneyError::DivisionByZero`] when `rhs` has a zero amount.
    /// - [`MoneyError::ConversionError`] when the quotient overflows the
    ///   active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_div_money(&self, rhs: &Self) -> Result<Decimal, MoneyError> {
        self.ensure_compatible_money(rhs)?;
        checked_div_decimal(&self.amount, &rhs.amount)
    }

    /// Converts this money to another currency using the provided exchange rate and rounding strategy.
    ///
    /// Identity conversions (`from == to`, rate `1`) bypass arithmetic and
    /// rounding, returning a clone of `self`. The conversion uses
    /// `checked_mul`, so a multiplication that would overflow the
    /// fixed-width `rust_decimal` backend surfaces as
    /// `MoneyError::ConversionError` instead of panicking.
    ///
    /// # Errors
    /// - Returns `MoneyError::IncompatibleExchangeRate` when the exchange rate does not match the money's currency.
    /// - Returns `MoneyError::ConversionError` when the rate-scaled amount overflows the active decimal backend.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rate), err)
    )]
    pub fn try_convert_with(
        &self,
        rate: &ExchangeRate,
        rounding: RoundingStrategy,
    ) -> Result<Self, MoneyError> {
        if !rate.is_compatible(self) {
            return Err(MoneyError::IncompatibleExchangeRate {
                from: rate.from.clone(),
                to: rate.to.clone(),
                money_currency: self.currency.clone(),
            });
        }

        // Identity rate fast path. By construction (`ExchangeRate::new`),
        // `from == to` implies `rate == 1`, so the conversion is a no-op
        // and we can avoid both the multiplication and the re-rounding
        // step that might pull a value off-scale.
        if rate.from == rate.to {
            return Ok(self.clone());
        }

        let (minor_units, scale) = Self::scale_for_currency(&rate.to)?;
        let product = checked_mul_decimal(&self.amount, &rate.rate)?;
        let converted_amount = decimal::round_dp_with_strategy(&product, scale, rounding);
        Ok(Self {
            amount: converted_amount,
            currency: rate.to.clone(),
            minor_units,
        })
    }

    /// Converts this money to another currency using the provided exchange rate.
    ///
    /// This method rounds using `RoundingStrategy::MidpointAwayFromZero` to match the
    /// target currency precision. Use [`Money::try_convert_with`] to customize
    /// the rounding behavior.
    ///
    /// # Errors
    /// Returns `MoneyError::IncompatibleExchangeRate` when the exchange rate does not match the money's currency.
    pub fn try_convert(&self, rate: &ExchangeRate) -> Result<Self, MoneyError> {
        self.try_convert_with(rate, RoundingStrategy::MidpointAwayFromZero)
    }

    fn ensure_scale_within_limits(decimals: u8) -> Result<u32, MoneyError> {
        if decimals > decimal::max_decimal_precision() {
            return Err(MoneyError::ConversionError);
        }
        if decimals > MAX_MINOR_UNIT_DECIMALS {
            return Err(MoneyError::ConversionError);
        }
        Ok(u32::from(decimals))
    }

    fn decimals_for_currency(currency: &Currency) -> Result<u8, MoneyError> {
        currency.decimal_places()
    }

    fn scale_for_currency(currency: &Currency) -> Result<(u8, u32), MoneyError> {
        let minor_units = Self::decimals_for_currency(currency)?;
        let scale = Self::ensure_scale_within_limits(minor_units)?;
        Ok((minor_units, scale))
    }

    fn round_amount_to_scale(amount: &Decimal, scale: u32) -> Decimal {
        decimal::round_dp_with_strategy(amount, scale, RoundingStrategy::MidpointAwayFromZero)
    }

    fn canonicalize_exact_amount(
        amount: &Decimal,
        currency: &Currency,
        scale: u32,
    ) -> Result<Decimal, MoneyError> {
        // Round toward zero so any rounding "decision" turns into a pure
        // truncation: if even one fractional digit would have been dropped,
        // the truncated value differs from the original.
        let canonical = decimal::round_dp_with_strategy(amount, scale, RoundingStrategy::ToZero);
        if canonical != *amount {
            let actual_scale = u32::try_from(decimal_scale(amount)).unwrap_or(u32::MAX);
            return Err(MoneyError::PrecisionExceeded {
                currency_code: currency.code().to_string(),
                max_scale: scale,
                actual_scale,
            });
        }
        Ok(canonical)
    }

    fn from_rounded_parts(amount: &Decimal, currency: Currency, minor_units: u8) -> Self {
        let scale = Self::ensure_scale_within_limits(minor_units)
            .expect("stored minor-unit scale was validated at construction");
        Self {
            amount: Self::round_amount_to_scale(amount, scale),
            currency,
            minor_units,
        }
    }

    fn ensure_compatible_money(&self, rhs: &Self) -> Result<(), MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                found: rhs.currency.clone(),
            });
        }
        if self.minor_units != rhs.minor_units {
            return Err(MoneyError::MinorUnitMismatch {
                currency: self.currency.clone(),
                expected_scale: self.minor_units,
                found_scale: rhs.minor_units,
            });
        }
        Ok(())
    }

    fn from_serialized_parts(
        amount: &Decimal,
        currency: Currency,
        minor_units: u8,
    ) -> Result<Self, MoneyError> {
        let scale = Self::ensure_scale_within_limits(minor_units)?;
        let amount = Self::canonicalize_exact_amount(amount, &currency, scale)?;
        Self::ensure_serialized_scale_matches_metadata(&currency, minor_units)?;

        Ok(Self {
            amount,
            currency,
            minor_units,
        })
    }

    fn ensure_serialized_scale_matches_metadata(
        currency: &Currency,
        minor_units: u8,
    ) -> Result<(), MoneyError> {
        match Self::decimals_for_currency(currency) {
            Ok(expected) if expected != minor_units => Err(MoneyError::MinorUnitMismatch {
                currency: currency.clone(),
                expected_scale: expected,
                found_scale: minor_units,
            }),
            Ok(_) | Err(MoneyError::MetadataNotFound { .. }) => Ok(()),
            Err(err) => Err(err),
        }
    }

    #[cfg(feature = "panicking-money-ops")]
    fn assert_compatible_money(&self, rhs: &Self) {
        if let Err(err) = self.ensure_compatible_money(rhs) {
            panic!("{err}");
        }
    }

    #[cfg(feature = "money-formatting")]
    fn render_with_locale(
        &self,
        locale: Locale,
        include_symbol: bool,
        include_code: bool,
        symbol_first_override: Option<bool>,
        rounding_digits: u32,
    ) -> Result<String, MoneyError> {
        let max_fraction_digits = u32::from(MAX_DECIMAL_PRECISION);
        if rounding_digits > max_fraction_digits {
            return Err(MoneyError::FormatPrecisionExceeded {
                actual_fraction_digits: rounding_digits,
                max_fraction_digits,
            });
        }

        let mut symbol = if include_symbol {
            self.currency.symbol().filter(|s| !s.as_ref().is_empty())
        } else {
            None
        };

        if include_code
            && symbol
                .as_ref()
                .is_some_and(|sym| sym.as_ref().eq_ignore_ascii_case(self.currency.code()))
        {
            symbol = None;
        }

        let symbol_first = symbol_first_override.unwrap_or_else(|| self.currency.symbol_first());
        let symbol_spacing = symbol.as_ref().is_some_and(|s| s.chars().count() > 1);

        let code = if include_code {
            match &self.currency {
                Currency::Other(_) => Some(Cow::Owned(self.currency.code().to_string())),
                _ => Some(Cow::Borrowed(self.currency.code())),
            }
        } else {
            None
        };

        let mut positions = Vec::new();
        positions.push(FormatItem::Sign);

        if symbol.is_some() {
            if symbol_first {
                positions.push(FormatItem::Symbol);
                if symbol_spacing {
                    positions.push(FormatItem::Space);
                }
                positions.push(FormatItem::Amount);
            } else {
                positions.push(FormatItem::Amount);
                if symbol_spacing {
                    positions.push(FormatItem::Space);
                }
                positions.push(FormatItem::Symbol);
            }
        } else {
            positions.push(FormatItem::Amount);
        }

        if include_code {
            positions.push(FormatItem::Space);
            positions.push(FormatItem::Code);
        }

        let mut params = Params::new(positions);
        params.rounding_digits = Some(rounding_digits);
        params.symbol = symbol;
        params.code = code;

        // clone the amount once (cheap in rust_decimal; explicit clone in bigdecimal)
        let amount = copy_decimal(&self.amount);

        Formatter::new(amount, locale, params).format()
    }
}

impl CurrencyAmount for Money {
    fn raw_amount(&self) -> &Decimal {
        &self.amount
    }

    fn raw_currency(&self) -> &Currency {
        &self.currency
    }
}

/// Shadow type used for deserializing [`Money`].
///
/// Matches the on-the-wire shape produced by the `Serialize` derive but
/// routes the deserialised values through `Money::from_serialized_parts` so
/// amount validation and scale compatibility cannot be skipped. The serialized
/// `minor_units` field is part of the value identity: if current metadata is
/// missing, it is enough to reconstruct the captured scale; if current metadata
/// exists but disagrees, deserialization rejects the payload instead of
/// silently reinterpreting it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoneyShadow {
    #[serde(with = "paft_decimal::serde::canonical_str")]
    amount: Decimal,
    currency: Currency,
    minor_units: u8,
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let shadow = MoneyShadow::deserialize(deserializer)?;
        Self::from_serialized_parts(&shadow.amount, shadow.currency, shadow.minor_units)
            .map_err(serde::de::Error::custom)
    }
}

/// Builder returned by [`Money::localized`] for configuring locale-aware rendering.
#[cfg(feature = "money-formatting")]
#[derive(Debug, Clone, Copy)]
pub struct LocalizedMoney<'a> {
    money: &'a Money,
    locale: Locale,
    include_symbol: bool,
    include_code: bool,
    symbol_first_override: Option<bool>,
    fraction_digits: Option<u32>,
}

#[cfg(feature = "money-formatting")]
impl<'a> LocalizedMoney<'a> {
    /// Creates a localized view; prefer [`Money::localized`] for external callers.
    #[must_use]
    pub const fn new(money: &'a Money, locale: Locale) -> Self {
        Self {
            money,
            locale,
            include_symbol: true,
            include_code: false,
            symbol_first_override: None,
            fraction_digits: None,
        }
    }

    /// Include the currency code (e.g. `USD`) in the rendered output.
    #[must_use]
    pub const fn with_code(mut self) -> Self {
        self.include_code = true;
        self
    }

    /// Omit the currency symbol when rendering.
    #[must_use]
    pub const fn without_symbol(mut self) -> Self {
        self.include_symbol = false;
        self
    }

    /// Override whether the symbol is rendered before (`true`) or after (`false`) the amount.
    #[must_use]
    pub const fn symbol_first(mut self, first: bool) -> Self {
        self.symbol_first_override = Some(first);
        self
    }

    /// Render using the provided number of fractional digits.
    #[must_use]
    pub const fn fraction_digits(mut self, digits: u32) -> Self {
        self.fraction_digits = Some(digits);
        self
    }

    /// Produce the localized string according to the configured options.
    ///
    /// # Errors
    /// Returns [`MoneyError::FormatPrecisionExceeded`] when the requested
    /// fraction digits exceed [`crate::MAX_DECIMAL_PRECISION`], or
    /// [`MoneyError::InvalidAmountFormat`] when the number cannot be
    /// represented with the requested fraction digits.
    pub fn into_string(self) -> Result<String, MoneyError> {
        self.format_internal()
    }

    fn format_internal(&self) -> Result<String, MoneyError> {
        let digits = self
            .fraction_digits
            .unwrap_or_else(|| u32::from(self.money.minor_units()));

        self.money.render_with_locale(
            self.locale,
            self.include_symbol,
            self.include_code,
            self.symbol_first_override,
            digits,
        )
    }
}

#[cfg(feature = "money-formatting")]
impl fmt::Display for LocalizedMoney<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format_internal() {
            Ok(output) => f.write_str(&output),
            Err(_) => f.write_str(&canonical_amount_format(self.money)),
        }
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&canonical_amount_format(self))
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.assert_compatible_money(&rhs);
        let sum = checked_add_decimal(&self.amount, &rhs.amount).expect("money addition overflow");
        Self::from_rounded_parts(&sum, self.currency, self.minor_units)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl<'b> Add<&'b Money> for &Money {
    type Output = Money;

    fn add(self, rhs: &'b Money) -> Self::Output {
        self.assert_compatible_money(rhs);
        let sum = checked_add_decimal(&self.amount, &rhs.amount).expect("money addition overflow");
        Money::from_rounded_parts(&sum, self.currency.clone(), self.minor_units)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Sub for Money {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.assert_compatible_money(&rhs);
        let diff =
            checked_sub_decimal(&self.amount, &rhs.amount).expect("money subtraction overflow");
        Self::from_rounded_parts(&diff, self.currency, self.minor_units)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl<'b> Sub<&'b Money> for &Money {
    type Output = Money;

    fn sub(self, rhs: &'b Money) -> Self::Output {
        self.assert_compatible_money(rhs);
        let diff =
            checked_sub_decimal(&self.amount, &rhs.amount).expect("money subtraction overflow");
        Money::from_rounded_parts(&diff, self.currency.clone(), self.minor_units)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Mul<Decimal> for Money {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        let product =
            checked_mul_decimal(&self.amount, &rhs).expect("money multiplication overflow");
        Self::from_rounded_parts(&product, self.currency, self.minor_units)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Div<Decimal> for Money {
    type Output = Self;

    fn div(self, rhs: Decimal) -> Self::Output {
        assert!(rhs != decimal::zero(), "division by zero");
        let quotient = checked_div_decimal(&self.amount, &rhs).expect("money division overflow");
        Self::from_rounded_parts(&quotient, self.currency, self.minor_units)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Div<Decimal> for &Money {
    type Output = Money;

    fn div(self, rhs: Decimal) -> Self::Output {
        assert!(rhs != decimal::zero(), "division by zero");
        let quotient = checked_div_decimal(&self.amount, &rhs).expect("money division overflow");
        Money::from_rounded_parts(&quotient, self.currency.clone(), self.minor_units)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Div for Money {
    type Output = Decimal;

    fn div(self, rhs: Self) -> Self::Output {
        self.assert_compatible_money(&rhs);
        assert!(rhs.amount != decimal::zero(), "division by zero");

        checked_div_decimal(&self.amount, &rhs.amount).expect("money division overflow")
    }
}

#[cfg(feature = "panicking-money-ops")]
impl<'b> Div<&'b Money> for &Money {
    type Output = Decimal;

    fn div(self, rhs: &'b Money) -> Self::Output {
        self.assert_compatible_money(rhs);
        assert!(rhs.amount != decimal::zero(), "division by zero");

        checked_div_decimal(&self.amount, &rhs.amount).expect("money division overflow")
    }
}
