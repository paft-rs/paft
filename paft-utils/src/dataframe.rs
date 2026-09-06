//! `DataFrame` conversion traits for paft utilities.
//!
//! This module re-exports the shared `df-derive-core` runtime traits so
//! dataframe impls derived across crates share one trait identity. paft keeps
//! its own `Decimal128Encode` trait to encode PAFT decimal values and constrained
//! newtypes through the shared `paft-decimal` mantissa conversion.

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

/// Internal glue for PAFT rows that must validate before `df-derive` encodes
/// them. A borrowed projection keeps the upstream trait identity and column
/// encoding, including nested provider metadata. Exhaustive destructuring makes
/// adding a model field without updating its projection a compile error.
/// Field types use brackets to pass raw tokens through to the upstream derive;
/// opaque `ty` macro fragments would hide their structure from its type parser.
/// The validation expression returns a fallible result; its error need only
/// implement Display. Conversion happens in the consuming crate, without adding
/// a dependency on that crate's error types here. Batch indices are retained.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_checked_dataframe {
    (
        $name:ident $(<$($generic:ident),+>)? {
            $($(#[$attr:meta])* $field:ident: [$($ty:tt)*]),* $(,)?
        }
        validate |$row:ident| $valid:expr
    ) => {
        const _: () = {
            use $crate::dataframe::{Columnar, ToDataFrame};
            use $crate::dataframe::__private::polars::prelude::{
                DataFrame, DataType, PolarsError, PolarsResult,
            };

            #[derive(df_derive_macros::ToDataFrame)]
            struct Projection<'a $(, $($generic),+)?> {
                $($(#[$attr])* $field: &'a $($ty)*),*
            }

            impl<'a $(, $($generic),+)?> Projection<'a $(, $($generic),+)?> {
                fn checked(index: usize, $row: &'a $name $(<$($generic),+>)?) -> PolarsResult<Self> {
                    let validate = || $valid;
                    validate().map_err(|error| PolarsError::ComputeError(format!(
                        "{}[{index}].{error}", stringify!($name),
                    ).into()))?;
                    let $name { $($field),* } = $row;
                    Ok(Self { $($field),* })
                }
            }

            impl $(<$($generic: ToDataFrame + Columnar),+>)?
                ToDataFrame for $name $(<$($generic),+>)?
            {
                fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    <Self as Columnar>::columnar_from_refs(&[self])
                }

                fn empty_dataframe() -> PolarsResult<DataFrame> {
                    Projection::<$($($generic),+)?>::empty_dataframe()
                }

                fn schema() -> PolarsResult<Vec<(String, DataType)>> {
                    Projection::<$($($generic),+)?>::schema()
                }
            }

            impl $(<$($generic: ToDataFrame + Columnar),+>)?
                Columnar for $name $(<$($generic),+>)?
            {
                fn columnar_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
                    let rows = items.iter().enumerate().map(|(i, row)| Projection::checked(i, row))
                        .collect::<PolarsResult<Vec<_>>>()?;
                    Projection::columnar_to_dataframe(&rows)
                }

                fn columnar_from_refs(items: &[&Self]) -> PolarsResult<DataFrame> {
                    let rows = items.iter().copied().enumerate().map(|(i, row)| Projection::checked(i, row))
                        .collect::<PolarsResult<Vec<_>>>()?;
                    Projection::columnar_to_dataframe(&rows)
                }
            }
        };
    };
}
