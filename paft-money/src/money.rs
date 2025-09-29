//! Money type for representing financial values with currency.

use crate::decimal::{self, Decimal, RoundingStrategy, ToPrimitive};
use serde::{Deserialize, Serialize};
#[cfg(feature = "panicking-money-ops")]
use std::ops::{Add, Div, Mul, Sub};
use thiserror::Error;

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

use crate::currency::Currency;
#[cfg(not(feature = "bigdecimal"))]
use crate::currency_utils::MAX_DECIMAL_PRECISION;
use crate::currency_utils::MAX_MINOR_UNIT_DECIMALS;
use crate::error::MoneyParseError;

#[inline]
#[allow(clippy::missing_const_for_fn)]
fn copy_decimal(value: &Decimal) -> Decimal {
    #[cfg(feature = "rust-decimal")]
    {
        *value
    }
    #[cfg(feature = "bigdecimal")]
    {
        value.clone()
    }
}

#[inline]
fn decimal_to_string(value: &Decimal) -> String {
    let mut repr = value.to_string();
    if let Some(dot) = repr.find('.') {
        let mut end = repr.len();
        while end > dot + 1 && repr.as_bytes()[end - 1] == b'0' {
            end -= 1;
        }
        if end == dot + 1 {
            end -= 1;
        }
        repr.truncate(end);
    }
    repr
}

/// Errors that can occur when performing operations on Money values.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// Occurs when attempting to perform arithmetic operations on Money values with different currencies.
    #[error("currency mismatch: expected {expected}, found {found}")]
    CurrencyMismatch {
        /// The expected currency.
        expected: Currency,
        /// The actual currency found.
        found: Currency,
    },
    /// Occurs when converting a Money amount to cents fails due to overflow or precision issues.
    #[error("could not convert amount to cents")]
    ConversionError,
    /// Occurs when attempting to divide by zero.
    #[error("division by zero")]
    DivisionByZero,
    /// Occurs when an exchange rate is invalid (e.g., negative or zero rate).
    #[error("invalid exchange rate: {rate}")]
    InvalidExchangeRate {
        /// The invalid rate value.
        rate: Decimal,
    },
    /// Occurs when attempting to convert using an incompatible exchange rate.
    #[error(
        "incompatible exchange rate: from {from} to {to}, but money currency is {money_currency}"
    )]
    IncompatibleExchangeRate {
        /// The source currency of the exchange rate.
        from: Currency,
        /// The target currency of the exchange rate.
        to: Currency,
        /// The currency of the money being converted.
        money_currency: Currency,
    },
    /// Occurs when attempting to use a currency without registered metadata.
    #[error("metadata not registered for currency {currency}")]
    MetadataNotFound {
        /// The currency missing metadata.
        currency: Currency,
    },
    /// Occurs when parsing a decimal amount fails.
    #[error("invalid decimal")]
    InvalidDecimal,
    /// Occurs when parsing a currency fails.
    #[error("invalid currency: {source}")]
    InvalidCurrency {
        /// Underlying currency parsing error.
        source: MoneyParseError,
    },
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
/// let usd = Money::from_str("12.34", Currency::Iso(IsoCurrency::USD)).unwrap();
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
    /// cheap copy with the default `rust-decimal` backend, but incurs an
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

    /// Creates a new Money instance from a string amount and currency.
    ///
    /// # Errors
    /// Returns an error when the string cannot be parsed as a decimal.
    /// Leading and trailing whitespace is ignored and an optional leading `+`
    /// sign is supported. Scientific notation is rejected so that behaviour is
    /// consistent across decimal backends.
    pub fn from_str(amount: &str, currency: Currency) -> Result<Self, MoneyError> {
        let amount = decimal::parse_decimal(amount).ok_or(MoneyError::InvalidDecimal)?;
        Self::new(amount, currency)
    }

    /// Creates a new Money instance from an integer amount in the currency's minor units.
    ///
    /// # Errors
    /// Returns `MoneyError::ConversionError` when the currency precision exceeds supported limits
    /// (currently 18 decimal places to keep `10^scale` within `i128`).
    pub fn from_minor_units(minor_units: i128, currency: Currency) -> Result<Self, MoneyError> {
        let decimals = Self::decimals_for_currency(&currency)?;
        let scale = Self::ensure_scale_within_limits(decimals)?;
        let amount = decimal::from_minor_units(minor_units, scale);
        Self::new(amount, currency)
    }

    /// Returns the amount as a formatted string with currency code.
    ///
    /// The numeric portion is emitted without exponent notation so the output
    /// remains human-readable and stable across decimal backends.
    #[must_use]
    pub fn format(&self) -> String {
        format!(
            "{} {}",
            decimal_to_string(&self.amount),
            self.currency.code()
        )
    }

    /// Addition that returns an error for currency mismatch.
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` when the operands use different currencies.
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
    pub fn try_mul(&self, rhs: Decimal) -> Result<Self, MoneyError> {
        Self::new(copy_decimal(&self.amount) * rhs, self.currency.clone())
    }

    /// Division that returns an error for division by zero.
    ///
    /// # Errors
    /// Returns `MoneyError::DivisionByZero` when `rhs` is zero.
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

    fn round_amount(mut amount: Decimal, currency: &Currency) -> Result<Decimal, MoneyError> {
        let decimals = Self::decimals_for_currency(currency)?;
        let scale = Self::ensure_scale_within_limits(decimals)?;
        amount =
            decimal::round_dp_with_strategy(&amount, scale, RoundingStrategy::MidpointAwayFromZero);
        Ok(amount)
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", decimal_to_string(&self.amount), self.currency)
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
