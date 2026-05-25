use crate::currency::Currency;
#[cfg(not(feature = "bigdecimal"))]
use crate::currency_utils::MAX_DECIMAL_PRECISION;
use crate::decimal::{self, Decimal, RoundingStrategy};
use crate::error::MoneyError;
use crate::money::Money;

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

#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
pub fn checked_add_decimal(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
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
pub fn checked_sub_decimal(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
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
pub fn checked_mul_decimal(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
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
pub fn checked_div_decimal(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    #[cfg(not(feature = "bigdecimal"))]
    {
        lhs.checked_div(*rhs)
    }
    #[cfg(feature = "bigdecimal")]
    {
        Some(lhs / rhs)
    }
}

#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
pub fn decimal_from_scaled_units(units: i128, scale: u32) -> Result<Decimal, MoneyError> {
    #[cfg(not(feature = "bigdecimal"))]
    if scale > u32::from(MAX_DECIMAL_PRECISION) {
        return Err(MoneyError::ConversionError);
    }

    Ok(decimal::from_minor_units(units, scale))
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

pub fn canonical_format(amount: &Decimal, currency: &Currency) -> String {
    format!(
        "{} {}",
        decimal::to_canonical_string(amount),
        currency.code()
    )
}
