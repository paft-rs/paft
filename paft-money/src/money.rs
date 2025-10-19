//! Money type for representing financial values with currency.

use crate::decimal::{self, Decimal, RoundingStrategy, ToPrimitive};
use crate::error::MoneyError;
use serde::{Deserialize, Serialize};
#[cfg(feature = "panicking-money-ops")]
use std::ops::{Add, Div, Mul, Sub};

#[cfg(feature = "money-formatting")]
use std::fmt;

#[cfg(feature = "money-formatting")]
use std::borrow::Cow;

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

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

#[inline]
#[allow(clippy::missing_const_for_fn)]
fn copy_decimal(value: &Decimal) -> Decimal {
    #[cfg(not(feature = "bigdecimal"))]
    {
        *value
    }
    #[cfg(feature = "bigdecimal")]
    {
        value.clone()
    }
}

/// Represents an exchange rate between two currencies.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct ExchangeRate {
    /// The source currency.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub from: Currency,
    /// The target currency.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub to: Currency,
    /// The exchange rate (how many units of 'to' currency per 1 unit of 'from' currency).
    pub rate: Decimal,
}

impl ExchangeRate {
    /// Creates a new `ExchangeRate` instance with validation.
    ///
    /// # Errors
    /// Returns `MoneyError::InvalidExchangeRate` when `from == to` or `rate` is not strictly positive.
    pub fn new(from: Currency, to: Currency, rate: Decimal) -> Result<Self, MoneyError> {
        if from == to {
            return Err(MoneyError::InvalidExchangeRate { rate });
        }
        if rate <= decimal::zero() {
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
    #[must_use]
    pub fn rate(&self) -> Decimal {
        copy_decimal(&self.rate)
    }

    /// Creates the inverse exchange rate (swaps from/to and inverts the rate).
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self {
            from: self.to.clone(),
            to: self.from.clone(),
            rate: decimal::one() / copy_decimal(&self.rate),
        }
    }

    /// Checks if this exchange rate can be used to convert the given money.
    #[must_use]
    pub fn is_compatible(&self, money: &Money) -> bool {
        money.currency == self.from
    }
}

/// Represents a financial value with its currency, enforcing safe operations.
///
/// ```
/// # use iso_currency::Currency as IsoCurrency;
/// # use paft_money::{Currency, Money};
/// let usd = Money::from_canonical_str("12.34", Currency::Iso(IsoCurrency::USD)).unwrap();
/// let json = serde_json::to_string(&usd).unwrap();
/// assert_eq!(json, "{\"amount\":\"12.34\",\"currency\":\"USD\"}");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Money {
    /// The numeric value.
    amount: Decimal,
    /// The currency.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    currency: Currency,
}

impl Money {
    /// Creates a new `Money` instance rounded to the currency's minor units.
    ///
    /// The supplied amount is quantized using
    /// [`RoundingStrategy::MidpointAwayFromZero`], ensuring the resulting
    /// quantity can be settled precisely with the currency's minor units.
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

    /// Returns the amount as the smallest currency unit (minor units).
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` or `MoneyError::MetadataNotFound` when conversion cannot be performed.
    pub fn as_minor_units(&self) -> Result<i128, MoneyError> {
        let decimals = Self::decimals_for_currency(&self.currency)?;
        let scale = Self::ensure_scale_within_limits(decimals)?;

        let multiplier = Decimal::from(10_i64.pow(scale));
        (copy_decimal(&self.amount) * multiplier)
            .to_i128()
            .ok_or(MoneyError::ConversionError)
    }

    /// Creates a new `Money` instance from a canonical decimal string and currency.
    ///
    /// # Errors
    /// Returns an error when the string cannot be parsed as a decimal.
    /// Leading and trailing whitespace is ignored and an optional leading `+`
    /// sign is supported. Scientific notation is rejected so that behaviour is
    /// consistent across decimal backends.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn from_canonical_str(amount: &str, currency: Currency) -> Result<Self, MoneyError> {
        let amount = decimal::parse_decimal(amount).ok_or(MoneyError::InvalidDecimal)?;
        Self::new(amount, currency)
    }

    /// Creates a new Money instance from an integer amount in the currency's minor units.
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` when the currency precision exceeds supported limits
    /// (currently 18 decimal places to keep `10^scale` within `i128`).
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn from_minor_units(minor_units: i128, currency: Currency) -> Result<Self, MoneyError> {
        let decimals = Self::decimals_for_currency(&currency)?;
        let scale = Self::ensure_scale_within_limits(decimals)?;
        let amount = decimal::from_minor_units(minor_units, scale);
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
    /// Returns `MoneyError::CurrencyMismatch` when the operands use different currencies.
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
        Self::new(
            copy_decimal(&self.amount) + copy_decimal(&rhs.amount),
            self.currency.clone(),
        )
    }

    /// Subtraction that returns an error for currency mismatch.
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` when the operands use different currencies.
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
        Self::new(
            copy_decimal(&self.amount) - copy_decimal(&rhs.amount),
            self.currency.clone(),
        )
    }

    /// Multiplication that preserves the currency.
    ///
    /// # Errors
    /// Returns `MoneyError::MetadataNotFound` when metadata is missing for the currency.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_mul(&self, rhs: Decimal) -> Result<Self, MoneyError> {
        Self::new(copy_decimal(&self.amount) * rhs, self.currency.clone())
    }

    /// Division that returns an error for division by zero.
    ///
    /// # Errors
    /// Returns `MoneyError::DivisionByZero` when `rhs` is zero.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, rhs), err)
    )]
    pub fn try_div(&self, rhs: Decimal) -> Result<Self, MoneyError> {
        if rhs == decimal::zero() {
            return Err(MoneyError::DivisionByZero);
        }
        Self::new(copy_decimal(&self.amount) / rhs, self.currency.clone())
    }

    /// Converts this money to another currency using the provided exchange rate and rounding strategy.
    ///
    /// # Errors
    /// Returns `MoneyError::IncompatibleExchangeRate` when the exchange rate does not match the money's currency.
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

        let decimals = rate.to.decimal_places()?;
        let scale = Self::ensure_scale_within_limits(decimals)?;
        let product = copy_decimal(&self.amount) * rate.rate();
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

        // clone the amount once (cheap in rust-decimal; explicit clone in bigdecimal)
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
