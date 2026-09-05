//! Exactness checks over bounded decimal coefficients, without floating point.

use crate::Decimal;

fn from_coefficient(mut coefficient: i128, mut scale: u32) -> Option<Decimal> {
    if coefficient == 0 {
        return Some(Decimal::ZERO);
    }
    while scale > 0 && coefficient % 10 == 0 {
        coefficient /= 10;
        scale -= 1;
    }
    Decimal::try_from_i128_with_scale(coefficient, scale).ok()
}

fn aligned_coefficients(lhs: &Decimal, rhs: &Decimal) -> Option<(i128, i128, u32)> {
    let lhs = lhs.normalize();
    let rhs = rhs.normalize();
    let scale = lhs.scale().max(rhs.scale());

    // Equal scales need at most 97 bits for the sum. With unequal normalized
    // scales, the finer operand has a nonzero final digit, so the result cannot
    // shed fractional zeros. If alignment exceeds i128, the other 96-bit
    // coefficient cannot cancel enough magnitude to make the result fit.
    let left = lhs
        .mantissa()
        .checked_mul(10_i128.checked_pow(scale - lhs.scale())?)?;
    let right = rhs
        .mantissa()
        .checked_mul(10_i128.checked_pow(scale - rhs.scale())?)?;
    Some((left, right, scale))
}

/// Adds decimals exactly, returning `None` if the exact result cannot fit.
///
/// Never discards significant digits to accommodate a larger magnitude.
#[must_use]
pub fn checked_add_exact(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    let (left, right, scale) = aligned_coefficients(lhs, rhs)?;
    from_coefficient(left.checked_add(right)?, scale)
}

/// Subtracts decimals exactly, returning `None` if the exact result cannot fit.
#[must_use]
pub fn checked_sub_exact(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    let (left, right, scale) = aligned_coefficients(lhs, rhs)?;
    from_coefficient(left.checked_sub(right)?, scale)
}

/// Multiplies decimals exactly, returning `None` if the exact result cannot fit.
///
/// Nonzero underflow is an error, even when upstream multiplication rounds to
/// zero. Insignificant zeros may be removed to fit the coefficient and scale.
#[must_use]
pub fn checked_mul_exact(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    if lhs.is_zero() || rhs.is_zero() {
        return Some(Decimal::ZERO);
    }

    let mut left = lhs.mantissa().unsigned_abs();
    let mut right = rhs.mantissa().unsigned_abs();
    let mut scale = lhs.scale() + rhs.scale();

    // A raw product may require 192 bits even when its numeric value fits.
    // Cancel factors of ten before multiplying, including 2 and 5 split across
    // operands. Once cancellation stops, an overflowing product cannot be
    // rescued by reducing its scale. This loop takes at most 56 iterations.
    while scale > 0 {
        if left.is_multiple_of(10) {
            left /= 10;
        } else if right.is_multiple_of(10) {
            right /= 10;
        } else if left.is_multiple_of(2) && right.is_multiple_of(5) {
            left /= 2;
            right /= 5;
        } else if left.is_multiple_of(5) && right.is_multiple_of(2) {
            left /= 5;
            right /= 2;
        } else {
            break;
        }
        scale -= 1;
    }

    let coefficient = i128::try_from(left.checked_mul(right)?).ok()?;
    let coefficient = if lhs.is_sign_negative() ^ rhs.is_sign_negative() {
        -coefficient
    } else {
        coefficient
    };
    from_coefficient(coefficient, scale)
}

/// Divides decimals exactly, returning `None` for zero divisors or when the
/// exact quotient cannot fit, including nonterminating decimals such as `1/3`.
#[must_use]
pub fn checked_div_exact(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    let quotient = lhs.checked_div(*rhs)?;
    // A rounded reverse multiplication can hide precision loss. Only accept
    // the quotient when exact multiplication reconstructs the dividend.
    (checked_mul_exact(&quotient, rhs)? == *lhs).then_some(quotient)
}
