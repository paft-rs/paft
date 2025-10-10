//! `DataFrame` conversion traits for paft utilities.
//!
//! This module defines the public `ToDataFrame` trait for single items,
//! a public extension trait `ToDataFrameVec` for slices/Vecs,
//! and an internal `Columnar` trait implemented by the derive macro
//! to enable high-performance conversion for `&[T]`.

use polars::frame::DataFrame;
use polars::prelude::PolarsResult;
use polars::datatypes::DataType;

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
    fn schema() -> PolarsResult<Vec<(&'static str, DataType)>>;
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
    /// # Errors
    /// Returns an error if constructing the `DataFrame` fails.
    fn columnar_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame>;
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
