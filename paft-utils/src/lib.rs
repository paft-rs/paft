#![warn(missing_docs)]

//! Shared utility helpers used across the paft workspace.

#[cfg(feature = "dataframe")]
pub mod dataframe;
pub mod string_canonical;

#[cfg(feature = "dataframe")]
pub use dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
pub use string_canonical::{Canonical, CanonicalError, StringCode, canonicalize};
