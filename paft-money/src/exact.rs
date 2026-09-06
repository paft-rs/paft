use crate::currency::Currency;
use crate::decimal::{self, Decimal, RoundingStrategy};
use crate::error::MoneyError;
use crate::money::Money;

pub trait CurrencyAmount {
    fn raw_amount(&self) -> &Decimal;

    fn raw_currency(&self) -> &Currency;
}

#[inline]
pub const fn copy_decimal(value: &Decimal) -> Decimal {
    decimal::clone_decimal(value)
}

/// Number of fractional digits in the decimal's representation.
pub fn decimal_scale(value: &Decimal) -> i64 {
    decimal::fractional_digit_count(value)
}

pub fn checked_add_decimal(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    decimal::checked_add(lhs, rhs).ok_or(MoneyError::ConversionError)
}

pub fn checked_sub_decimal(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    decimal::checked_sub(lhs, rhs).ok_or(MoneyError::ConversionError)
}

pub fn checked_mul_decimal(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    decimal::checked_mul(lhs, rhs).ok_or(MoneyError::ConversionError)
}

pub fn checked_div_decimal(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    if rhs == &decimal::zero() {
        return Err(MoneyError::DivisionByZero);
    }
    decimal::checked_div(lhs, rhs).ok_or(MoneyError::ConversionError)
}

pub fn checked_mul_decimal_exact(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    decimal::checked_mul_exact(lhs, rhs).ok_or(MoneyError::NotRepresentable)
}

pub fn checked_div_decimal_exact(lhs: &Decimal, rhs: &Decimal) -> Result<Decimal, MoneyError> {
    if rhs.is_zero() {
        return Err(MoneyError::DivisionByZero);
    }
    decimal::checked_div_exact(lhs, rhs).ok_or(MoneyError::NotRepresentable)
}

pub fn checked_add_amounts<T: CurrencyAmount>(lhs: &T, rhs: &T) -> Result<Decimal, MoneyError> {
    ensure_same_currency(lhs.raw_currency(), rhs.raw_currency())?;
    decimal::checked_add_exact(lhs.raw_amount(), rhs.raw_amount())
        .ok_or(MoneyError::NotRepresentable)
}

pub fn checked_sub_amounts<T: CurrencyAmount>(lhs: &T, rhs: &T) -> Result<Decimal, MoneyError> {
    ensure_same_currency(lhs.raw_currency(), rhs.raw_currency())?;
    decimal::checked_sub_exact(lhs.raw_amount(), rhs.raw_amount())
        .ok_or(MoneyError::NotRepresentable)
}

pub fn parse_canonical_decimal(amount: &str) -> Result<Decimal, MoneyError> {
    decimal::parse_decimal(amount).map_err(|error| match error {
        decimal::DecimalParseError::NotRepresentable => MoneyError::NotRepresentable,
        _ => MoneyError::InvalidDecimal,
    })
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
    round_to_money_after_resolving(amount, currency, rounding, target_fraction_digits, || {})
}

// The callback provides a deterministic synchronization point for the regression
// test. The production call supplies a no-op, with no registry lock held here.
fn round_to_money_after_resolving(
    amount: &Decimal,
    currency: Currency,
    rounding: RoundingStrategy,
    target_fraction_digits: Option<u32>,
    after_resolving: impl FnOnce(),
) -> Result<Money, MoneyError> {
    let (minor_units, currency_scale) = Money::scale_for_currency(&currency)?;
    after_resolving();
    let effective_scale = match target_fraction_digits {
        Some(digits) if digits > currency_scale => return Err(MoneyError::ConversionError),
        Some(digits) => digits,
        None => currency_scale,
    };

    let rounded = decimal::round_dp_with_strategy(amount, effective_scale, rounding);
    Ok(Money::from_quantized_parts(rounded, currency, minor_units))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, clear_currency_metadata, override_currency_metadata};
    use std::sync::mpsc;

    #[test]
    fn settlement_conversion_uses_one_metadata_snapshot() {
        let _guard = crate::currency_utils::tests::SERIALIZE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let code = "SETTLEMENT_SNAPSHOT_TEST";
        let currency = Currency::other(code).unwrap();
        let amount = parse_canonical_decimal("1.99").unwrap();
        // Cover default precision and an explicit coarser target. The captured
        // scale must remain 2 even when the amount is rounded to fewer places.
        for (target, expected) in [(None, "1.99"), (Some(1), "1.9")] {
            override_currency_metadata(code, code, 2, code, true, Locale::EnUs).unwrap();
            let (resolved_tx, resolved_rx) = mpsc::channel();
            let (changed_tx, changed_rx) = mpsc::channel();
            std::thread::scope(|scope| {
                scope.spawn(move || {
                    resolved_rx.recv().unwrap();
                    override_currency_metadata(code, code, 0, code, true, Locale::EnUs).unwrap();
                    changed_tx.send(()).unwrap();
                });
                let money = round_to_money_after_resolving(
                    &amount,
                    currency.clone(),
                    RoundingStrategy::ToZero,
                    target,
                    || {
                        resolved_tx.send(()).unwrap();
                        changed_rx.recv().unwrap();
                    },
                )
                .unwrap();
                assert_eq!(money.amount(), parse_canonical_decimal(expected).unwrap());
                assert_eq!(money.minor_units(), 2);
                assert_eq!(money.currency(), &currency);
            });
            let replacement =
                round_to_money(&amount, currency.clone(), RoundingStrategy::ToZero, None).unwrap();
            assert_eq!(replacement.amount(), decimal::one());
            assert_eq!(replacement.minor_units(), 0);
        }
        clear_currency_metadata(code);
    }
}
