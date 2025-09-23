//! Money type for representing financial values with currency.

use super::currency_utils::{MAX_DECIMAL_PRECISION, MAX_MINOR_UNIT_DECIMALS};
use crate::domain::currency::Currency;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
#[cfg(feature = "panicking-money-ops")]
use std::ops::{Add, Div, Mul, Sub};
use thiserror::Error;

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

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
}

/// Represents an exchange rate between two currencies.
///
/// This struct provides type-safe currency conversion rates with validation
/// to ensure the rate is positive and the currencies are different.
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
    /// # Arguments
    /// * `from` - The source currency
    /// * `to` - The target currency  
    /// * `rate` - The exchange rate (must be positive)
    ///
    /// # Errors
    /// Returns `MoneyError::InvalidExchangeRate` if the rate is zero or negative,
    /// or if the source and target currencies are the same.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{ExchangeRate, Currency};
    /// use iso_currency::Currency as IsoCurrency;
    /// use rust_decimal::Decimal;
    ///
    /// let rate = ExchangeRate::new(Currency::Iso(IsoCurrency::USD), Currency::Iso(IsoCurrency::EUR), Decimal::new(85, 2)).unwrap();
    /// assert_eq!(rate.rate(), Decimal::new(85, 2)); // 0.85 EUR per 1 USD
    /// ```
    pub fn new(from: Currency, to: Currency, rate: Decimal) -> Result<Self, MoneyError> {
        if from == to {
            return Err(MoneyError::InvalidExchangeRate { rate });
        }
        if rate <= Decimal::ZERO {
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
    pub const fn rate(&self) -> Decimal {
        self.rate
    }

    /// Creates the inverse exchange rate (swaps from/to and inverts the rate).
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{ExchangeRate, Currency};
    /// use iso_currency::Currency as IsoCurrency;
    /// use rust_decimal::Decimal;
    ///
    /// let usd_to_eur = ExchangeRate::new(Currency::Iso(IsoCurrency::USD), Currency::Iso(IsoCurrency::EUR), Decimal::new(85, 2)).unwrap();
    /// let eur_to_usd = usd_to_eur.inverse();
    /// assert_eq!(eur_to_usd.from(), &Currency::Iso(IsoCurrency::EUR));
    /// assert_eq!(eur_to_usd.to(), &Currency::Iso(IsoCurrency::USD));
    /// ```
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self {
            from: self.to.clone(),
            to: self.from.clone(),
            rate: Decimal::ONE / self.rate,
        }
    }

    /// Checks if this exchange rate can be used to convert the given money.
    ///
    /// # Arguments
    /// * `money` - The money to check compatibility with
    ///
    /// # Returns
    /// `true` if the money's currency matches the 'from' currency of this rate.
    #[must_use]
    pub fn is_compatible(&self, money: &Money) -> bool {
        money.currency == self.from
    }
}

/// Represents a financial value with its currency, enforcing safe operations.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Money {
    /// The numeric value.
    pub amount: Decimal,
    /// The currency.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub currency: Currency,
}

impl Money {
    /// Creates a new `Money` instance.
    #[must_use]
    pub const fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Creates a new `Money` instance with zero amount and the specified currency.
    ///
    /// # Arguments
    /// * `currency` - The currency for the zero-value money
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency};
    ///
    /// let zero_usd = Money::zero(Currency::USD);
    /// assert_eq!(zero_usd.amount(), rust_decimal::Decimal::ZERO);
    /// assert_eq!(zero_usd.currency(), &Currency::USD);
    /// ```
    #[must_use]
    pub const fn zero(currency: Currency) -> Self {
        Self::new(Decimal::ZERO, currency)
    }

    /// Returns the amount as a `Decimal`.
    #[must_use]
    pub const fn amount(&self) -> Decimal {
        self.amount
    }

    /// Returns the `Currency`.
    #[must_use]
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Returns the amount as the smallest currency unit (minor units), if possible.
    ///
    /// This method respects the currency's decimal places:
    /// - USD: $1.23 -> 123 cents (2 decimal places)
    /// - JPY: ¥123 -> 123 yen (0 decimal places)
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency};
    /// use rust_decimal::Decimal;
    ///
    /// let usd = Money::new(Decimal::new(12345, 2), Currency::USD); // $123.45
    /// assert_eq!(usd.as_minor_units(), Some(12345i128));
    ///
    /// let jpy = Money::new(Decimal::new(123, 0), Currency::JPY); // ¥123
    /// assert_eq!(jpy.as_minor_units(), Some(123i128));
    /// ```
    #[must_use]
    pub fn as_minor_units(&self) -> Option<i128> {
        let decimals = self.currency.decimal_places();
        let Ok(scale) = Self::ensure_scale_within_limits(decimals) else {
            return None;
        };

        let multiplier = Decimal::from(10_i64.pow(scale));
        (self.amount * multiplier).to_i128()
    }

    /// Creates a new Money instance from a string amount and currency.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency};
    ///
    /// let money = Money::from_str("123.45", Currency::USD).unwrap();
    /// ```
    /// # Errors
    /// Returns a `rust_decimal::Error` if the input string cannot be parsed
    /// into a valid `Decimal` using `Decimal::from_str_exact`.
    pub fn from_str(amount: &str, currency: Currency) -> Result<Self, rust_decimal::Error> {
        let amount = Decimal::from_str_exact(amount)?;
        Ok(Self::new(amount, currency))
    }

    /// Creates a new Money instance from an integer amount in the currency's minor units.
    ///
    /// This method respects the currency's decimal places:
    /// - USD: 12345 -> $123.45 (2 decimal places)
    /// - JPY: 123 -> ¥123 (0 decimal places)
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency};
    ///
    /// let usd = Money::from_minor_units(12345, Currency::USD).unwrap(); // $123.45
    /// let jpy = Money::from_minor_units(123, Currency::JPY).unwrap();   // ¥123
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `MoneyError::ConversionError` when the currency's configured
    /// precision exceeds the safe scaling limits enforced by `rust_decimal`.
    pub fn from_minor_units(minor_units: i128, currency: Currency) -> Result<Self, MoneyError> {
        let scale = Self::ensure_scale_within_limits(currency.decimal_places())?;
        let amount = Decimal::from_i128_with_scale(minor_units, scale);
        Ok(Self::new(amount, currency))
    }

    /// Returns the amount as a formatted string with currency code.
    ///
    /// This uses `currency.code()` for the currency token, which is the
    /// canonical emission used by both `Display` and serde across enums.
    #[must_use]
    pub fn format(&self) -> String {
        format!("{} {}", self.amount, self.currency.code())
    }

    /// Addition that returns an error for currency mismatch.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency};
    /// use rust_decimal::Decimal;
    ///
    /// let usd = Money::new(Decimal::new(100, 0), Currency::USD);
    /// let eur = Money::new(Decimal::new(100, 0), Currency::EUR);
    ///
    /// // This will return an error
    /// assert!(usd.try_add(&eur).is_err());
    ///
    /// let usd2 = Money::new(Decimal::new(50, 0), Currency::USD);
    /// // This will succeed
    /// assert!(usd.try_add(&usd2).is_ok());
    /// ```
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if `rhs.currency` differs
    /// from `self.currency`.
    /// Note: Using the `+` operator will panic on currency mismatch. Use this
    /// method to handle errors explicitly.
    pub fn try_add(&self, rhs: &Self) -> Result<Self, MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                found: rhs.currency.clone(),
            });
        }
        Ok(Self::new(self.amount + rhs.amount, self.currency.clone()))
    }

    /// Subtraction that returns an error for currency mismatch.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency};
    /// use rust_decimal::Decimal;
    ///
    /// let usd = Money::new(Decimal::new(100, 0), Currency::USD);
    /// let eur = Money::new(Decimal::new(100, 0), Currency::EUR);
    ///
    /// // This will return an error
    /// assert!(usd.try_sub(&eur).is_err());
    ///
    /// let usd2 = Money::new(Decimal::new(50, 0), Currency::USD);
    /// // This will succeed
    /// assert!(usd.try_sub(&usd2).is_ok());
    /// ```
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if `rhs.currency` differs
    /// from `self.currency`.
    /// Note: Using the `-` operator will panic on currency mismatch. Use this
    /// method to handle errors explicitly.
    pub fn try_sub(&self, rhs: &Self) -> Result<Self, MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                found: rhs.currency.clone(),
            });
        }
        Ok(Self::new(self.amount - rhs.amount, self.currency.clone()))
    }

    /// Multiplication that preserves the currency.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency};
    /// use rust_decimal::Decimal;
    ///
    /// let usd = Money::new(Decimal::new(100, 0), Currency::USD);
    /// let result = usd.mul(Decimal::new(2, 0));
    /// assert_eq!(result.amount(), Decimal::new(200, 0));
    /// assert_eq!(result.currency(), &Currency::USD);
    /// ```
    #[must_use]
    pub fn mul(&self, rhs: Decimal) -> Self {
        Self::new(self.amount * rhs, self.currency.clone())
    }

    /// Division that returns an error for division by zero.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency};
    /// use rust_decimal::Decimal;
    ///
    /// let usd = Money::new(Decimal::new(100, 0), Currency::USD);
    ///
    /// // This will return an error
    /// assert!(usd.try_div(Decimal::ZERO).is_err());
    ///
    /// // This will succeed
    /// assert!(usd.try_div(Decimal::new(2, 0)).is_ok());
    /// ```
    /// # Errors
    /// Returns `MoneyError::DivisionByZero` when `rhs` equals zero.
    /// Note: Using the `/` operator will panic on division by zero. Use this
    /// method to handle errors explicitly.
    pub fn try_div(&self, rhs: Decimal) -> Result<Self, MoneyError> {
        if rhs.is_zero() {
            return Err(MoneyError::DivisionByZero);
        }
        Ok(Self::new(self.amount / rhs, self.currency.clone()))
    }

    /// Converts this money to another currency using the provided exchange rate.
    ///
    /// # Arguments
    /// * `rate` - The exchange rate to use for conversion
    ///
    /// # Errors
    /// Returns `MoneyError::IncompatibleExchangeRate` if the money's currency
    /// doesn't match the 'from' currency of the exchange rate.
    ///
    /// # Example
    /// ```
    /// use paft_core::domain::{Money, Currency, ExchangeRate};
    /// use iso_currency::Currency as IsoCurrency;
    /// use rust_decimal::Decimal;
    ///
    /// let usd = Money::new(Decimal::new(100, 0), Currency::Iso(IsoCurrency::USD)); // $100.00
    /// let rate = ExchangeRate::new(Currency::Iso(IsoCurrency::USD), Currency::Iso(IsoCurrency::EUR), Decimal::new(85, 2)).unwrap(); // 0.85 EUR per USD
    ///
    /// let eur = usd.try_convert(&rate).unwrap();
    /// assert_eq!(eur.amount(), Decimal::new(8500, 2)); // €85.00
    /// assert_eq!(eur.currency(), &Currency::Iso(IsoCurrency::EUR));
    /// ```
    /// # Errors
    /// Returns `MoneyError::IncompatibleExchangeRate` if the money's
    /// currency doesn't match the 'from' currency of the exchange rate.
    pub fn try_convert(&self, rate: &ExchangeRate) -> Result<Self, MoneyError> {
        if !rate.is_compatible(self) {
            return Err(MoneyError::IncompatibleExchangeRate {
                from: rate.from.clone(),
                to: rate.to.clone(),
                money_currency: self.currency.clone(),
            });
        }

        let scale = Self::ensure_scale_within_limits(rate.to.decimal_places())?;
        let converted_amount = (self.amount * rate.rate())
            .round_dp_with_strategy(scale, RoundingStrategy::MidpointAwayFromZero);
        Ok(Self::new(converted_amount, rate.to.clone()))
    }

    fn ensure_scale_within_limits(decimals: u8) -> Result<u32, MoneyError> {
        if decimals > MAX_DECIMAL_PRECISION {
            return Err(MoneyError::ConversionError);
        }
        if decimals > MAX_MINOR_UNIT_DECIMALS {
            return Err(MoneyError::ConversionError);
        }
        Ok(u32::from(decimals))
    }
}

// Operator overloading for Money with panicking safety checks for ergonomics
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

        Money::new(self.amount + rhs.amount, self.currency.clone())
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

        Money::new(self.amount - rhs.amount, self.currency.clone())
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Mul<Decimal> for Money {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self::new(self.amount * rhs, self.currency)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Div<Decimal> for Money {
    type Output = Self;

    fn div(self, rhs: Decimal) -> Self::Output {
        assert!(!rhs.is_zero(), "division by zero");
        Self::new(self.amount / rhs, self.currency)
    }
}

#[cfg(feature = "panicking-money-ops")]
impl Div<Decimal> for &Money {
    type Output = Money;

    fn div(self, rhs: Decimal) -> Self::Output {
        assert!(!rhs.is_zero(), "division by zero");
        Money::new(self.amount / rhs, self.currency.clone())
    }
}
