use crate::currency::Currency;
use crate::decimal::{self, Decimal, RoundingStrategy};
use crate::error::MoneyError;
use crate::money::Money;

pub trait CurrencyAmount {
    fn raw_amount(&self) -> &Decimal;

    fn raw_currency(&self) -> &Currency;
}

// `Decimal` is `Copy` under `rust_decimal` but not under `bigdecimal`. The two
// definitions below let call sites say `copy_decimal(&value)` without
// sprinkling backend cfgs through the money types.
#[cfg(not(feature = "bigdecimal"))]
#[inline]
pub const fn copy_decimal(value: &Decimal) -> Decimal {
    *value
}

#[cfg(feature = "bigdecimal")]
#[inline]
pub fn copy_decimal(value: &Decimal) -> Decimal {
    value.clone()
}

/// Number of fractional digits represented by the active decimal backend.
pub fn decimal_scale(value: &Decimal) -> i64 {
    #[cfg(not(feature = "bigdecimal"))]
    {
        i64::from(value.scale())
    }
    #[cfg(feature = "bigdecimal")]
    {
        value.fractional_digit_count()
    }
}

#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
fn checked_add_decimal_inner(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_add(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs + rhs)
    }
}

#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
fn checked_sub_decimal_inner(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_sub(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs - rhs)
    }
}

#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
fn checked_mul_decimal_inner(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_mul(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs * rhs)
    }
}

#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
fn checked_div_decimal_inner(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_div(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs / rhs)
    }
}

pub fn checked_add_decimal(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    checked_add_decimal_inner(lhs, rhs).ok_or(MoneyError::ConversionError)
}

pub fn checked_sub_decimal(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    checked_sub_decimal_inner(lhs, rhs).ok_or(MoneyError::ConversionError)
}

pub fn checked_mul_decimal(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    checked_mul_decimal_inner(lhs, rhs).ok_or(MoneyError::ConversionError)
}

pub fn checked_div_decimal(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    if rhs == &decimal::zero() {
        return Err(MoneyError::DivisionByZero);
    }
    checked_div_decimal_inner(lhs, rhs).ok_or(MoneyError::ConversionError)
}

pub fn checked_add_amounts<T: CurrencyAmount>(lhs: &T, rhs: &T) -> Result<Decimal, MoneyError> {
    ensure_same_currency(lhs.raw_currency(), rhs.raw_currency())?;
    checked_add_decimal(lhs.raw_amount(), rhs.raw_amount())
}

pub fn checked_sub_amounts<T: CurrencyAmount>(lhs: &T, rhs: &T) -> Result<Decimal, MoneyError> {
    ensure_same_currency(lhs.raw_currency(), rhs.raw_currency())?;
    checked_sub_decimal(lhs.raw_amount(), rhs.raw_amount())
}

pub fn parse_canonical_decimal(amount: &str) -> Result<Decimal, MoneyError> {
    decimal::parse_decimal(amount).ok_or(MoneyError::InvalidDecimal)
}

pub fn decimal_from_scaled_units(units: i128, scale: u32) -> Result<Decimal, MoneyError> {
    decimal::try_from_scaled_units(units, scale).ok_or(MoneyError::ConversionError)
}

pub fn round_to_money(
    amount: &Decimal,
    currency: Currency,
    rounding: RoundingStrategy,
    target_fraction_digits: Option<u32>,
) -> Result<Money, MoneyError> {
    let currency_scale = u32::from(currency.decimal_places()?);
    let effective_scale = match target_fraction_digits {
        Some(digits) if digits > currency_scale => return Err(MoneyError::ConversionError),
        Some(digits) => digits,
        None => currency_scale,
    };

    let rounded = decimal::round_dp_with_strategy(amount, effective_scale, rounding);
    Money::new(rounded, currency)
}

fn canonical_format(amount: &Decimal, currency: &Currency) -> String {
    format!(
        "{} {}",
        decimal::to_canonical_string(amount),
        currency.code()
    )
}

pub fn canonical_amount_format(amount: &impl CurrencyAmount) -> String {
    canonical_format(amount.raw_amount(), amount.raw_currency())
}

fn ensure_same_currency(expected: &Currency, found: &Currency) -> Result<(), MoneyError> {
    if expected != found {
        return Err(MoneyError::CurrencyMismatch {
            expected: expected.clone(),
            found: found.clone(),
        });
    }
    Ok(())
}
