#![warn(missing_docs)]

//! Shared utility helpers used across the paft workspace.
//!
//! This crate provides:
//! - Canonical string utilities (`Canonical`, `canonicalize`) for enum `Other` variants
//! - Optional dataframe helpers for converting domain structs to `polars` frames
//!
//! # Quickstart
//! ```rust
//! use paft_utils::{canonicalize, Canonical};
//!
//! // Normalize provider strings into canonical tokens
//! assert_eq!(canonicalize("Euronext Paris"), "EURONEXT_PARIS");
//!
//! // Validate non-empty canonical tokens via the `Canonical` wrapper
//! let c = Canonical::try_new("nasdaq").unwrap();
//! assert_eq!(c.as_str(), "NASDAQ");
//! ```
//!
//! # Feature flags
//! - `dataframe`: enable lightweight dataframe traits for `polars`

#[cfg(feature = "dataframe")]
pub mod dataframe;
pub mod string_canonical;

#[cfg(feature = "dataframe")]
pub use dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
pub use string_canonical::{Canonical, CanonicalError, StringCode, canonicalize};
