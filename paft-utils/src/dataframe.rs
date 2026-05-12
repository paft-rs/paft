//! `DataFrame` conversion traits for paft utilities.
//!
//! This module re-exports the shared `df-derive-core` runtime traits so
//! dataframe impls derived across crates share one trait identity. paft keeps
//! its own `Decimal128Encode` trait so it can provide decimal backend impls
//! for foreign types such as `rust_decimal::Decimal` and
//! `bigdecimal::BigDecimal`.

pub use df_derive_core::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};

/// Encodes a decimal value into the i128 mantissa expected by polars
/// `DataType::Decimal(_, _)` columns.
///
/// Implementations MUST use round-half-to-even (banker's rounding) on
/// scale-down so the mantissa bytes match what polars's own
/// `str_to_dec128` would produce. Returning `None` indicates the rescaled
/// value does not fit in i128; the caller (the `df-derive` codegen) surfaces
/// this as a `PolarsError::ComputeError`.
pub trait Decimal128Encode {
    /// Returns the mantissa as `i128` after rescaling `self` to
    /// `target_scale`, or `None` if the conversion would overflow.
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128>;
}

/// Polars `Decimal(38, _)` columns require `|mantissa| < 10^38` (matches
/// polars's own `dec128_fits` strict-less-than precision check). `i128::MAX`
/// is `~1.7 × 10^38`, so the band `10^38 ≤ |m| ≤ i128::MAX` would otherwise
/// slip through `i128` arithmetic and violate the column's declared
/// precision. Both backend implementations reject any rescaled mantissa
/// whose absolute value reaches this constant.
const MAX_I128_MANTISSA: i128 = 10_i128.pow(38);

impl Decimal128Encode for rust_decimal::Decimal {
    #[inline]
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        let source_scale = self.scale();
        let mantissa: i128 = self.mantissa();
        let rescaled = match source_scale.cmp(&target_scale) {
            std::cmp::Ordering::Equal => mantissa,
            std::cmp::Ordering::Less => {
                let diff = target_scale - source_scale;
                let pow = 10i128.pow(diff);
                mantissa.checked_mul(pow)?
            }
            std::cmp::Ordering::Greater => {
                let diff = source_scale - target_scale;
                let pow = 10i128.pow(diff).cast_unsigned();
                let neg = mantissa < 0;
                let abs = mantissa.unsigned_abs();
                let q = (abs / pow).cast_signed();
                let r = abs % pow;
                let half = pow / 2;
                let rounded = match r.cmp(&half) {
                    std::cmp::Ordering::Greater => q + 1,
                    std::cmp::Ordering::Less => q,
                    std::cmp::Ordering::Equal => q + (q & 1),
                };
                if neg { -rounded } else { rounded }
            }
        };
        // Match the `bigdecimal` arm and polars's own `dec128_fits`: reject
        // any mantissa in the band `[10^38, i128::MAX]` so we never emit a
        // value that violates the declared `Decimal(38, _)` precision.
        if rescaled.unsigned_abs() >= MAX_I128_MANTISSA.cast_unsigned() {
            return None;
        }
        Some(rescaled)
    }
}

#[cfg(feature = "bigdecimal")]
impl Decimal128Encode for bigdecimal::BigDecimal {
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        let target = i64::from(target_scale);
        let rescaled = self.with_scale_round(target, bigdecimal::RoundingMode::HalfEven);
        // Polars `Decimal(38, _)` requires |mantissa| < 10^38. `i128::try_from`
        // allows up to `i128::MAX ≈ 1.7 × 10^38`, so the boundary band
        // 10^38 ≤ |m| ≤ i128::MAX would otherwise slip through and violate
        // the column's declared precision. Match polars's `dec128_fits` (strict
        // less-than) by rejecting any rescaled value with more than 38 digits.
        if rescaled.digits() > 38 {
            return None;
        }
        let (bigint, _) = rescaled.into_bigint_and_exponent();
        i128::try_from(bigint).ok()
    }
}
