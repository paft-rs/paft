use crate::currency::Currency;
use crate::decimal::{self, Decimal, RoundingStrategy};
use crate::error::MoneyError;
use crate::money::Money;

pub trait CurrencyAmount {
    fn raw_amount(&self) -> &Decimal;

    fn raw_currency(&self) -> &Currency;
}

#[inline]
pub fn copy_decimal(value: &Decimal) -> Decimal {
    decimal::clone_decimal(value)
}

/// Number of fractional digits represented by the active decimal backend.
pub fn decimal_scale(value: &Decimal) -> i64 {
    decimal::fractional_digit_count(value)
}

fn checked_add_decimal_inner(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    decimal::checked_add(lhs, rhs)
}

fn checked_sub_decimal_inner(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    decimal::checked_sub(lhs, rhs)
}

fn checked_mul_decimal_inner(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    decimal::checked_mul(lhs, rhs)
}

fn checked_div_decimal_inner(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    decimal::checked_div(lhs, rhs)
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
