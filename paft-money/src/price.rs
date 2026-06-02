//! Full-precision price type for quoted per-unit values.

use crate::amount::MonetaryAmount;
use crate::currency::Currency;
use crate::decimal::{self, Decimal, RoundingStrategy};
use crate::error::MoneyError;
use crate::exact::{
    CurrencyAmount, canonical_amount_format, checked_add_amounts, checked_div_decimal,
    checked_mul_decimal, checked_sub_amounts, copy_decimal, decimal_from_scaled_units,
    parse_canonical_decimal, round_to_money,
};
use crate::money::Money;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

/// Full-precision currency price for per-unit quoted values.
///
/// `Price` always carries a [`Currency`] but does not enforce the currency's
/// settlement exponent. Use it for quotes, OHLC values, option strikes, EPS,
/// dividend-per-share values, and other provider-quoted per-unit data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Price {
    amount: Decimal,
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    currency: Currency,
}

impl Price {
    /// Creates a full-precision price.
    #[must_use]
    pub const fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Parses a canonical decimal string and attaches the given currency.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::InvalidDecimal`] when the string cannot be parsed losslessly.
    pub fn from_canonical_str(amount: &str, currency: Currency) -> Result<Self, MoneyError> {
        let decimal = parse_canonical_decimal(amount)?;
        Ok(Self::new(decimal, currency))
    }

    /// Creates a price from integer units and an explicit scale.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::ConversionError`] when the scale exceeds the active backend precision.
    pub fn from_scaled_units(
        units: i128,
        scale: u32,
        currency: Currency,
    ) -> Result<Self, MoneyError> {
        Ok(Self::new(
            decimal_from_scaled_units(units, scale)?,
            currency,
        ))
    }

    /// Returns the zero price for the given currency.
    #[must_use]
    pub fn zero(currency: Currency) -> Self {
        Self::new(decimal::zero(), currency)
    }

    /// Returns the underlying decimal amount.
    #[must_use]
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn amount(&self) -> Decimal {
        copy_decimal(&self.amount)
    }

    /// Returns the underlying decimal amount.
    #[must_use]
    #[cfg(feature = "bigdecimal")]
    pub fn amount(&self) -> Decimal {
        copy_decimal(&self.amount)
    }

    /// Returns the price currency.
    #[must_use]
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Returns a canonical string with currency code (`"<amount> <CODE>"`).
    #[must_use]
    pub fn format(&self) -> String {
        canonical_amount_format(self)
    }

    /// Adds another price with the same currency.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::CurrencyMismatch`] when currencies differ and
    /// [`MoneyError::ConversionError`] when the active decimal backend overflows.
    pub fn try_add(&self, rhs: &Self) -> Result<Self, MoneyError> {
        let amount = checked_add_amounts(self, rhs)?;
        Ok(Self::new(amount, self.currency.clone()))
    }

    /// Subtracts another price with the same currency.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::CurrencyMismatch`] when currencies differ and
    /// [`MoneyError::ConversionError`] when the active decimal backend overflows.
    pub fn try_sub(&self, rhs: &Self) -> Result<Self, MoneyError> {
        let amount = checked_sub_amounts(self, rhs)?;
        Ok(Self::new(amount, self.currency.clone()))
    }

    /// Multiplies the price by a scalar and returns another price.
    ///
    /// For price-times-quantity totals, use [`Price::try_total`] instead.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::ConversionError`] when the active decimal backend overflows.
    pub fn try_mul(&self, factor: &Decimal) -> Result<Self, MoneyError> {
        let amount = checked_mul_decimal(&self.amount, factor)?;
        Ok(Self::new(amount, self.currency.clone()))
    }

    /// Divides the price by a scalar.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::DivisionByZero`] when `divisor` is zero and
    /// [`MoneyError::ConversionError`] when the active decimal backend overflows.
    pub fn try_div(&self, divisor: &Decimal) -> Result<Self, MoneyError> {
        let amount = checked_div_decimal(&self.amount, divisor)?;
        Ok(Self::new(amount, self.currency.clone()))
    }

    /// Multiplies this price by a quantity and returns an exact monetary total.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::ConversionError`] when the active decimal backend overflows.
    pub fn try_total(&self, quantity: &Decimal) -> Result<MonetaryAmount, MoneyError> {
        let amount = checked_mul_decimal(&self.amount, quantity)?;
        Ok(MonetaryAmount::new(amount, self.currency.clone()))
    }

    /// Converts the price into [`Money`], rounding to the currency exponent with
    /// [`RoundingStrategy::MidpointAwayFromZero`].
    ///
    /// # Errors
    ///
    /// Propagates the errors returned by [`Money::new`].
    pub fn to_money(&self) -> Result<Money, MoneyError> {
        self.to_money_with(RoundingStrategy::MidpointAwayFromZero, None)
    }

    /// Converts the price into [`Money`] using an explicit rounding strategy and precision.
    ///
    /// # Errors
    ///
    /// - Returns [`MoneyError::MetadataNotFound`] when the currency is missing metadata.
    /// - Returns [`MoneyError::ConversionError`] when `target_fraction_digits` exceeds the
    ///   currency exponent.
    pub fn to_money_with(
        &self,
        rounding: RoundingStrategy,
        target_fraction_digits: Option<u32>,
    ) -> Result<Money, MoneyError> {
        round_to_money(
            &self.amount,
            self.currency.clone(),
            rounding,
            target_fraction_digits,
        )
    }
}

impl CurrencyAmount for Price {
    fn raw_amount(&self) -> &Decimal {
        &self.amount
    }

    fn raw_currency(&self) -> &Currency {
        &self.currency
    }
}

impl Hash for Price {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.currency.hash(state);
        decimal::to_canonical_string(&self.amount).hash(state);
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.format())
    }
}

impl From<Money> for Price {
    fn from(money: Money) -> Self {
        Self::new(money.amount(), money.currency().clone())
    }
}
