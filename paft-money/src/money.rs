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
#[cfg(not(feature = "bigdecimal"))]
use crate::currency_utils::MAX_DECIMAL_PRECISION;
use crate::currency_utils::MAX_MINOR_UNIT_DECIMALS;
#[cfg(feature = "money-formatting")]
use crate::format::{FormatItem, Formatter, Params};
#[cfg(feature = "money-formatting")]
use crate::locale::Locale;
#[cfg(feature = "money-formatting")]
use crate::parser;

// `Decimal` is `Copy` under `rust_decimal` but not under `bigdecimal`. The two
// definitions below let the rest of the module say `copy_decimal(&value)`
// without sprinkling `cfg` on every call site, while still keeping the no-op
// path `const`-eligible under the default backend.
#[cfg(not(feature = "bigdecimal"))]
#[inline]
const fn copy_decimal(value: &Decimal) -> Decimal {
    *value
}

#[cfg(feature = "bigdecimal")]
#[inline]
fn copy_decimal(value: &Decimal) -> Decimal {
    value.clone()
}

// Backend-agnostic checked arithmetic. Under `rust_decimal` these can
// genuinely overflow because the type is fixed-width (96-bit mantissa);
// under `bigdecimal` the type is arbitrary precision so the helpers always
// return `Some(_)`. The `unnecessary_wraps` lint fires on the bigdecimal
// build because of that — silence it explicitly so the wrapper stays
// uniform across backends and call sites do not have to branch.

/// Backend-agnostic checked multiplication of two decimals.
///
/// Returns `None` on overflow. `rust_decimal` is fixed-width and can overflow
/// when the product exceeds 96 mantissa bits; `bigdecimal` is arbitrary
/// precision and never overflows, so the operation is unconditionally
/// successful there.
#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
fn checked_mul_decimal(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_mul(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs * rhs)
    }
}

/// Backend-agnostic checked division of two decimals.
#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
fn checked_div_decimal(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_div(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs / rhs)
    }
}

/// Backend-agnostic checked addition of two decimals.
#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
fn checked_add_decimal(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_add(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs + rhs)
    }
}

/// Backend-agnostic checked subtraction of two decimals.
#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
fn checked_sub_decimal(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_sub(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs - rhs)
    }
}

/// Number of fractional digits the underlying `Decimal` is currently
/// representing.
///
/// Both backends store an explicit scale, but expose it via different
/// methods. The returned value is widened to `i64` to match
/// `bigdecimal`'s native type — `rust_decimal` uses `u32`, which always
/// fits.
fn decimal_scale(value: &Decimal) -> i64 {
    #[cfg(not(feature = "bigdecimal"))]
    {
        i64::from(value.scale())
    }
    #[cfg(feature = "bigdecimal")]
    {
        value.fractional_digit_count()
    }
}

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
    /// `const`-qualified under `rust_decimal` (which is `Copy`); under
    /// `bigdecimal` the underlying clone must run at runtime.
    #[must_use]
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn rate(&self) -> Decimal {
        copy_decimal(&self.rate)
    }

    /// Returns the exchange rate.
    #[must_use]
    #[cfg(feature = "bigdecimal")]
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
        let rate = checked_div_decimal(&one, &self.rate).ok_or(MoneyError::ConversionError)?;

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
struct ExchangeRateShadow {
    from: Currency,
    to: Currency,
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
/// Deserialization also funnels through `new_exact`, so untrusted JSON cannot
/// produce a `Money` with a scale beyond the currency's `decimal_places()`.
///
/// `Hash` and `PartialEq` use a canonical string representation of the
/// numeric value, so two `Money` values that differ only in trailing
/// zero-scale digits compare equal and hash to the same bucket.
///
/// ```
/// # use paft_money::IsoCurrency;
/// # use paft_money::{Currency, Money};
/// let usd = Money::from_canonical_str("12.34", Currency::Iso(IsoCurrency::USD)).unwrap();
/// let json = serde_json::to_string(&usd).unwrap();
/// assert_eq!(json, "{\"amount\":\"12.34\",\"currency\":\"USD\"}");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Money {
    /// The numeric value.
    amount: Decimal,
    /// The currency.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    currency: Currency,
}

impl Hash for Money {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash currency directly
        self.currency.hash(state);
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
    pub fn new(amount: Decimal, currency: Currency) -> Result<Self, MoneyError> {
        let rounded = Self::round_amount(amount, &currency)?;
        Ok(Self {
            amount: rounded,
            currency,
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
    /// This is the constructor used by [`Money::from_canonical_str`] and by
    /// `serde::Deserialize`, so untrusted JSON cannot smuggle in an over-
    /// precise amount.
    ///
    /// # Errors
    /// - Returns `MoneyError::MetadataNotFound` when metadata is not registered.
    /// - Returns `MoneyError::PrecisionExceeded` when the supplied amount has
    ///   more fractional digits than the currency's exponent permits.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    // We take `amount` by value to mirror `Money::new` and avoid forcing
    // callers (notably the deserialize path) to clone before construction.
    // Under `bigdecimal` the body uses `&amount` for the round and
    // comparison, and only consumes the canonical value; the lint is
    // suppressed to keep the signatures consistent across backends.
    #[cfg_attr(feature = "bigdecimal", allow(clippy::needless_pass_by_value))]
    pub fn new_exact(amount: Decimal, currency: Currency) -> Result<Self, MoneyError> {
        let decimals = Self::decimals_for_currency(&currency)?;
        let scale = Self::ensure_scale_within_limits(decimals)?;
        // Round toward zero so any rounding "decision" turns into a pure
        // truncation: if even one fractional digit would have been dropped,
        // the truncated value differs from the original.
        let canonical = decimal::round_dp_with_strategy(&amount, scale, RoundingStrategy::ToZero);
        if canonical != amount {
            let actual_scale = u32::try_from(decimal_scale(&amount)).unwrap_or(u32::MAX);
            return Err(MoneyError::PrecisionExceeded {
                currency_code: currency.code().to_string(),
                max_scale: scale,
                actual_scale,
            });
        }
        Ok(Self {
            amount: canonical,
            currency,
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
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn amount(&self) -> Decimal {
        copy_decimal(&self.amount)
    }

    /// Returns the amount as a [`Decimal`].
    #[must_use]
    #[cfg(feature = "bigdecimal")]
    pub fn amount(&self) -> Decimal {
        copy_decimal(&self.amount)
    }

    /// Returns the `Currency`.
    #[must_use]
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Returns the amount as the smallest currency unit (minor units).
    ///
    /// Uses checked multiplication so a value that would overflow the
    /// fixed-width `rust_decimal` backend surfaces as
    /// `MoneyError::ConversionError` instead of panicking.
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` or `MoneyError::MetadataNotFound` when conversion cannot be performed.
    pub fn as_minor_units(&self) -> Result<i128, MoneyError> {
        let decimals = Self::decimals_for_currency(&self.currency)?;
        let scale = Self::ensure_scale_within_limits(decimals)?;

        // The cap on `scale` is enforced by `ensure_scale_within_limits`
        // (currently 18 dp) so `10^scale` always fits inside `i64`.
        let multiplier = Decimal::from(10_i64.pow(scale));
        let scaled =
            checked_mul_decimal(&self.amount, &multiplier).ok_or(MoneyError::ConversionError)?;
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
        let amount = decimal::parse_decimal(amount).ok_or(MoneyError::InvalidDecimal)?;
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
        let decimals = Self::decimals_for_currency(&currency)?;
        let scale = Self::ensure_scale_within_limits(decimals)?;
        let amount = decimal::try_from_scaled_units(minor_units, scale)
            .ok_or(MoneyError::ConversionError)?;
        Self::new(amount, currency)
    }

    /// Returns the amount as a canonical string with currency code (`"<amount> <CODE>"`).
    #[must_use]
    pub fn format(&self) -> String {
        self.canonical_format()
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
        Self::new(decimal, currency)
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
    /// The `fraction_digits` parameter allows any number of digits and will pad with zeros
    /// or round as needed. This is useful for UI sliders, CSV exports, and other cases
    /// where you need flexible display precision beyond the currency's natural scale.
    ///
    /// # Errors
    /// Returns [`MoneyError::InvalidAmountFormat`] when rounding fails (rare).
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
    /// - Returns `MoneyError::ConversionError` when the sum overflows the
    ///   active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_add(&self, rhs: &Self) -> Result<Self, MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                found: rhs.currency.clone(),
            });
        }
        let sum =
            checked_add_decimal(&self.amount, &rhs.amount).ok_or(MoneyError::ConversionError)?;
        Self::new(sum, self.currency.clone())
    }

    /// Subtraction that returns an error for currency mismatch.
    ///
    /// # Errors
    /// - Returns `MoneyError::CurrencyMismatch` when the operands use
    ///   different currencies.
    /// - Returns `MoneyError::ConversionError` when the difference overflows
    ///   the active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_sub(&self, rhs: &Self) -> Result<Self, MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                found: rhs.currency.clone(),
            });
        }
        let diff =
            checked_sub_decimal(&self.amount, &rhs.amount).ok_or(MoneyError::ConversionError)?;
        Self::new(diff, self.currency.clone())
    }

    /// Multiplication that preserves the currency.
    ///
    /// # Errors
    /// - Returns `MoneyError::MetadataNotFound` when metadata is missing for the currency.
    /// - Returns `MoneyError::ConversionError` when the product overflows the
    ///   active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    // The public signature takes `Decimal` by value to keep the existing
    // API stable. Under `bigdecimal`, `Decimal` is not `Copy`, so the
    // borrow checker would otherwise complain about a needless pass-by-
    // value; allowing this lint here preserves the API contract while
    // still avoiding an extra clone in the helper call below.
    #[allow(clippy::needless_pass_by_value)]
    pub fn try_mul(&self, rhs: Decimal) -> Result<Self, MoneyError> {
        let product = checked_mul_decimal(&self.amount, &rhs).ok_or(MoneyError::ConversionError)?;
        Self::new(product, self.currency.clone())
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
    pub fn try_div(&self, rhs: Decimal) -> Result<Self, MoneyError> {
        if rhs == decimal::zero() {
            return Err(MoneyError::DivisionByZero);
        }
        let quotient =
            checked_div_decimal(&self.amount, &rhs).ok_or(MoneyError::ConversionError)?;
        Self::new(quotient, self.currency.clone())
    }

    /// Divides two `Money` values of the same currency, returning the unitless ratio.
    ///
    /// Use this for finance ratios such as "how many shares fit in a budget"
    /// (`budget.try_div_money(&price)`) or P/E-style quotients. The result is a
    /// pure [`Decimal`] and is **not** rounded to any currency precision.
    ///
    /// # Errors
    /// - [`MoneyError::CurrencyMismatch`] when the operands use different currencies.
    /// - [`MoneyError::DivisionByZero`] when `rhs` has a zero amount.
    /// - [`MoneyError::ConversionError`] when the quotient overflows the
    ///   active decimal backend (only possible under `rust_decimal`).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_div_money(&self, rhs: &Self) -> Result<Decimal, MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                found: rhs.currency.clone(),
            });
        }
        if rhs.amount == decimal::zero() {
            return Err(MoneyError::DivisionByZero);
        }
        checked_div_decimal(&self.amount, &rhs.amount).ok_or(MoneyError::ConversionError)
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

        let decimals = rate.to.decimal_places()?;
        let scale = Self::ensure_scale_within_limits(decimals)?;
        let product =
            checked_mul_decimal(&self.amount, &rate.rate).ok_or(MoneyError::ConversionError)?;
        let converted_amount = decimal::round_dp_with_strategy(&product, scale, rounding);
        Self::new(converted_amount, rate.to.clone())
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
        #[cfg(not(feature = "bigdecimal"))]
        if decimals > MAX_DECIMAL_PRECISION {
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

    fn canonical_format(&self) -> String {
        format!(
            "{} {}",
            decimal::to_canonical_string(&self.amount),
            self.currency.code()
        )
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

    fn round_amount(mut amount: Decimal, currency: &Currency) -> Result<Decimal, MoneyError> {
        let decimals = Self::decimals_for_currency(currency)?;
        let scale = Self::ensure_scale_within_limits(decimals)?;
        amount =
            decimal::round_dp_with_strategy(&amount, scale, RoundingStrategy::MidpointAwayFromZero);
        Ok(amount)
    }
}

/// Shadow type used for deserializing [`Money`].
///
/// Matches the on-the-wire shape produced by the `Serialize` derive but
/// routes the deserialised values through [`Money::new_exact`] so that
/// validation, scale canonicalization, and metadata lookup happen on the
/// deserialised path. Without this, `#[derive(Deserialize)]` would build a
/// `Money` whose `amount` retained whatever scale was in the JSON,
/// breaking `Hash`/`Eq` consistency under `bigdecimal` and silently
/// admitting over-precise values.
#[derive(Deserialize)]
struct MoneyShadow {
    amount: Decimal,
    currency: Currency,
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let shadow = MoneyShadow::deserialize(deserializer)?;
        Self::new_exact(shadow.amount, shadow.currency).map_err(serde::de::Error::custom)
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
    /// Returns [`MoneyError::InvalidAmountFormat`] when the number cannot be represented with the requested fraction digits.
    pub fn into_string(self) -> Result<String, MoneyError> {
        self.format_internal()
    }

    fn format_internal(&self) -> Result<String, MoneyError> {
        let digits = match self.fraction_digits {
            Some(d) => d,
            None => u32::from(self.money.currency().decimal_places()?),
        };

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
            Err(_) => f.write_str(&self.money.canonical_format()),
        }
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.canonical_format())
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self {
            amount: lhs_amount,
            currency: lhs_currency,
        } = self;
        let Self {
            amount: rhs_amount,
            currency: rhs_currency,
        } = rhs;

        assert!(
            lhs_currency == rhs_currency,
            "currency mismatch: expected {lhs_currency}, found {rhs_currency}"
        );

        Self::new(lhs_amount + rhs_amount, lhs_currency)
            .expect("matching currencies share metadata")
    }
}

#[cfg(feature = "panicking-money-ops")]
impl<'b> Add<&'b Money> for &Money {
    type Output = Money;

    fn add(self, rhs: &'b Money) -> Self::Output {
        assert!(
            self.currency == rhs.currency,
            "currency mismatch: expected {expected}, found {found}",
            expected = self.currency,
            found = rhs.currency
        );

        Money::new(
            copy_decimal(&self.amount) + copy_decimal(&rhs.amount),
            self.currency.clone(),
        )
        .expect("matching currencies share metadata")
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Sub for Money {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let Self {
            amount: lhs_amount,
            currency: lhs_currency,
        } = self;
        let Self {
            amount: rhs_amount,
            currency: rhs_currency,
        } = rhs;

        assert!(
            lhs_currency == rhs_currency,
            "currency mismatch: expected {lhs_currency}, found {rhs_currency}"
        );

        Self::new(lhs_amount - rhs_amount, lhs_currency)
            .expect("matching currencies share metadata")
    }
}

#[cfg(feature = "panicking-money-ops")]
impl<'b> Sub<&'b Money> for &Money {
    type Output = Money;

    fn sub(self, rhs: &'b Money) -> Self::Output {
        assert!(
            self.currency == rhs.currency,
            "currency mismatch: expected {expected}, found {found}",
            expected = self.currency,
            found = rhs.currency
        );

        Money::new(
            copy_decimal(&self.amount) - copy_decimal(&rhs.amount),
            self.currency.clone(),
        )
        .expect("matching currencies share metadata")
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Mul<Decimal> for Money {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self::new(self.amount * rhs, self.currency).expect("currency metadata available")
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Div<Decimal> for Money {
    type Output = Self;

    fn div(self, rhs: Decimal) -> Self::Output {
        assert!(rhs != decimal::zero(), "division by zero");
        Self::new(self.amount / rhs, self.currency).expect("currency metadata available")
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Div<Decimal> for &Money {
    type Output = Money;

    fn div(self, rhs: Decimal) -> Self::Output {
        assert!(rhs != decimal::zero(), "division by zero");
        Money::new(copy_decimal(&self.amount) / rhs, self.currency.clone())
            .expect("currency metadata available")
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Div for Money {
    type Output = Decimal;

    fn div(self, rhs: Self) -> Self::Output {
        let Self {
            amount: lhs_amount,
            currency: lhs_currency,
        } = self;
        let Self {
            amount: rhs_amount,
            currency: rhs_currency,
        } = rhs;

        assert!(
            lhs_currency == rhs_currency,
            "currency mismatch: expected {lhs_currency}, found {rhs_currency}"
        );
        assert!(rhs_amount != decimal::zero(), "division by zero");

        lhs_amount / rhs_amount
    }
}

#[cfg(feature = "panicking-money-ops")]
impl<'b> Div<&'b Money> for &Money {
    type Output = Decimal;

    fn div(self, rhs: &'b Money) -> Self::Output {
        assert!(
            self.currency == rhs.currency,
            "currency mismatch: expected {expected}, found {found}",
            expected = self.currency,
            found = rhs.currency
        );
        assert!(rhs.amount != decimal::zero(), "division by zero");

        copy_decimal(&self.amount) / copy_decimal(&rhs.amount)
    }
}
