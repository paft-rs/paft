//! `DataFrame` conversion traits for paft utilities.
//!
//! This module re-exports the shared `df-derive-core` runtime traits so
//! dataframe impls derived across crates share one trait identity. paft keeps
//! its own `Decimal128Encode` trait so it can provide decimal backend impls
//! for the active `paft-decimal` backend without making downstream crates
//! branch on the concrete decimal type.

pub use df_derive_core::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};

/// Hidden dependency re-exports used by `df-derive` generated code when the
/// paft runtime is the selected default dataframe facade.
#[doc(hidden)]
pub mod __private {
    pub use df_derive_core::dataframe::__private::{polars, polars_arrow};
}

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
    /// `target_scale`, or `None` if the scale exceeds polars decimal
    /// precision or the conversion would overflow.
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128>;
}

impl<T> Decimal128Encode for T
where
    T: paft_decimal::Decimal128Mantissa,
{
    #[inline]
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        paft_decimal::Decimal128Mantissa::try_to_i128_mantissa(self, target_scale)
    }
}
