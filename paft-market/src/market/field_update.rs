//! Intent-preserving changes to optional fields in incremental data.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// An incremental change to an optional value.
///
/// On a containing serde field, use
/// `#[serde(default, skip_serializing_if = "FieldUpdate::is_unchanged")]`:
/// omission means [`Unchanged`](Self::Unchanged), a value means [`Set`](Self::Set),
/// and `null` means [`Clear`](Self::Clear). Serializing `Unchanged` on its own
/// fails because a standalone value cannot express omission. `T` must serialize
/// to a non-null value; nullable payloads such as `Option<T>` are unsupported.
///
/// With `dataframe`, price and non-negative decimal updates export `operation`
/// (`UNCHANGED`, `SET`, or `CLEAR`) and nullable decimal `value` columns. Only
/// `SET` has a value; consumers must retain the operation to reconstruct state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FieldUpdate<T> {
    /// Keep the previous value.
    #[default]
    Unchanged,
    /// Replace the previous value with this value.
    Set(T),
    /// Remove the previous value.
    Clear,
}

impl<T> FieldUpdate<T> {
    /// Whether this update leaves the previous value unchanged.
    #[must_use]
    pub const fn is_unchanged(&self) -> bool {
        matches!(self, Self::Unchanged)
    }

    /// Apply this change to a consumer's optional field.
    pub fn apply_to(self, target: &mut Option<T>) {
        match self {
            Self::Unchanged => {}
            Self::Set(value) => *target = Some(value),
            Self::Clear => *target = None,
        }
    }
}

impl<T: Serialize> Serialize for FieldUpdate<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Unchanged => Err(serde::ser::Error::custom(
                "unchanged field updates must be omitted by the containing field",
            )),
            Self::Set(value) => value.serialize(serializer),
            Self::Clear => serializer.serialize_none(),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for FieldUpdate<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(|value| value.map_or(Self::Clear, Self::Set))
    }
}

#[cfg(feature = "dataframe")]
mod dataframe {
    use super::FieldUpdate;
    use paft_decimal::{Decimal, NonNegativeDecimal};
    use paft_money::PriceAmount;
    use paft_utils::dataframe::{Columnar, ToDataFrame};
    use polars::prelude::{DataFrame, DataType, PolarsResult};

    #[derive(df_derive_macros::ToDataFrame)]
    struct DecimalUpdate {
        operation: &'static str,
        #[df_derive(decimal(precision = 38, scale = 10))]
        value: Option<Decimal>,
    }

    macro_rules! impl_decimal_update_dataframe {
        ($ty:ty, $as_decimal:path) => {
            impl From<&FieldUpdate<$ty>> for DecimalUpdate {
                fn from(update: &FieldUpdate<$ty>) -> Self {
                    let (operation, value) = match update {
                        FieldUpdate::Unchanged => ("UNCHANGED", None),
                        FieldUpdate::Set(value) => ("SET", Some(*$as_decimal(value))),
                        FieldUpdate::Clear => ("CLEAR", None),
                    };
                    Self { operation, value }
                }
            }

            impl ToDataFrame for FieldUpdate<$ty> {
                fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    DecimalUpdate::from(self).to_dataframe()
                }

                fn empty_dataframe() -> PolarsResult<DataFrame> {
                    DecimalUpdate::empty_dataframe()
                }

                fn schema() -> PolarsResult<Vec<(String, DataType)>> {
                    DecimalUpdate::schema()
                }
            }

            impl Columnar for FieldUpdate<$ty> {
                fn columnar_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
                    let rows: Vec<_> = items.iter().map(DecimalUpdate::from).collect();
                    DecimalUpdate::columnar_to_dataframe(&rows)
                }

                fn columnar_from_refs(items: &[&Self]) -> PolarsResult<DataFrame> {
                    let rows: Vec<_> = items.iter().copied().map(DecimalUpdate::from).collect();
                    DecimalUpdate::columnar_to_dataframe(&rows)
                }
            }
        };
    }

    impl_decimal_update_dataframe!(PriceAmount, PriceAmount::as_decimal);
    impl_decimal_update_dataframe!(NonNegativeDecimal, NonNegativeDecimal::as_decimal);
}
