//! `DataFrame` conversion traits for paft utilities.
//!
//! This module defines the public `ToDataFrame` trait for single items,
//! a public extension trait `ToDataFrameVec` for slices/Vecs,
//! and an internal `Columnar` trait implemented by the derive macro
//! to enable high-performance conversion for `&[T]`.

#[cfg(feature = "dataframe")]
use polars::datatypes::DataType;
#[cfg(feature = "dataframe")]
use polars::frame::DataFrame;
#[cfg(feature = "dataframe")]
use polars::prelude::{AnyValue, PolarsResult};

/// Trait for converting types to Polars `DataFrames`
#[cfg(feature = "dataframe")]
pub trait ToDataFrame {
    /// Convert the type to a Polars `DataFrame`
    ///
    /// # Errors
    /// Implementations return an error if the conversion fails for any reason
    /// such as incompatible schema, null handling, or internal Polars errors.
    fn to_dataframe(&self) -> PolarsResult<DataFrame>;

    /// Create an empty `DataFrame` with the same schema as this type
    ///
    /// # Errors
    /// Implementations return an error if the schema cannot be constructed
    /// without values or if underlying Polars APIs fail.
    fn empty_dataframe() -> PolarsResult<DataFrame>;

    /// Get the schema for this type
    ///
    /// # Errors
    /// Implementations return an error if the schema could not be derived or
    /// requires context that is unavailable.
    fn schema() -> PolarsResult<Vec<(String, DataType)>>;

    /// Returns one `AnyValue` per inner schema column for `self` — the
    /// per-row slice of `to_dataframe()`. The default impl round-trips
    /// through `to_dataframe()`; the derive overrides it to skip the
    /// one-row `DataFrame` allocation.
    ///
    /// # Errors
    /// Returns an error if value extraction fails.
    fn to_inner_values(&self) -> PolarsResult<Vec<AnyValue<'static>>> {
        let df = self.to_dataframe()?;
        let row = df.get(0).unwrap_or_default();
        Ok(row.into_iter().map(AnyValue::into_static).collect())
    }
}

/// Extension trait providing `.to_dataframe()` for slices of `T` (and `Vec<T>` via auto-deref)
#[cfg(feature = "dataframe")]
pub trait ToDataFrameVec {
    /// Convert the slice into a `DataFrame` using a columnar fast path.
    ///
    /// # Errors
    /// Returns an error if the conversion fails for any reason, delegated
    /// from the underlying `Columnar` implementation.
    fn to_dataframe(&self) -> PolarsResult<DataFrame>;
}

/// Internal trait implemented by the derive macro to provide high-performance
/// columnar conversion for `&[T]`. Not intended for user code.
#[doc(hidden)]
#[cfg(feature = "dataframe")]
pub trait Columnar: Sized {
    /// Convert a slice of `Self` to a `DataFrame` via a columnar path.
    ///
    /// Defaults to collecting refs and delegating to `columnar_from_refs`,
    /// which is the canonical entry point for the derive-generated builder.
    ///
    /// # Errors
    /// Returns an error if constructing the `DataFrame` fails.
    fn columnar_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
        let refs: Vec<&Self> = items.iter().collect();
        Self::columnar_from_refs(&refs)
    }

    /// Convert a slice of `&Self` to a `DataFrame` via a columnar path.
    ///
    /// # Errors
    /// Returns an error if constructing the `DataFrame` fails.
    fn columnar_from_refs(items: &[&Self]) -> PolarsResult<DataFrame>;
}

/// Encodes a decimal value into the i128 mantissa expected by polars
/// `DataType::Decimal(_, _)` columns.
///
/// Implementations MUST use round-half-to-even (banker's rounding) on
/// scale-down so the mantissa bytes match what polars's own
/// `str_to_dec128` would produce. Returning `None` indicates the rescaled
/// value does not fit in i128; the caller (the `df-derive` codegen) surfaces
/// this as a `PolarsError::ComputeError`.
#[cfg(feature = "dataframe")]
pub trait Decimal128Encode {
    /// Returns the mantissa as `i128` after rescaling `self` to
    /// `target_scale`, or `None` if the conversion would overflow.
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128>;
}

#[cfg(feature = "dataframe")]
impl Decimal128Encode for rust_decimal::Decimal {
    #[inline]
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        let source_scale = self.scale();
        let mantissa: i128 = self.mantissa();
        if source_scale == target_scale {
            return Some(mantissa);
        }
        if source_scale < target_scale {
            let diff = target_scale - source_scale;
            let pow = 10i128.pow(diff);
            return mantissa.checked_mul(pow);
        }
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
        Some(if neg { -rounded } else { rounded })
    }
}

#[cfg(all(feature = "dataframe", feature = "bigdecimal"))]
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

#[cfg(feature = "dataframe")]
impl<T> ToDataFrameVec for [T]
where
    T: Columnar + ToDataFrame,
{
    fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        if self.is_empty() {
            return <T as ToDataFrame>::empty_dataframe();
        }
        <T as Columnar>::columnar_to_dataframe(self)
    }
}

#[cfg(feature = "dataframe")]
impl ToDataFrame for () {
    fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        use polars::prelude::{NamedFrom, Series};
        let dummy = Series::new("_dummy".into(), &[0i32]);
        let mut df = DataFrame::new_infer_height(vec![dummy.into()])?;
        df.drop_in_place("_dummy")?;
        Ok(df)
    }

    fn empty_dataframe() -> PolarsResult<DataFrame> {
        DataFrame::new_infer_height(vec![])
    }

    fn schema() -> PolarsResult<Vec<(String, DataType)>> {
        Ok(Vec::new())
    }

    fn to_inner_values(&self) -> PolarsResult<Vec<AnyValue<'static>>> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "dataframe")]
impl Columnar for () {
    fn columnar_from_refs(items: &[&Self]) -> PolarsResult<DataFrame> {
        use polars::prelude::Series;
        let n = items.len();
        let dummy = Series::new_empty("_dummy".into(), &DataType::Null)
            .extend_constant(AnyValue::Null, n)?;
        let mut df = DataFrame::new_infer_height(vec![dummy.into()])?;
        df.drop_in_place("_dummy")?;
        Ok(df)
    }
}
