//! Aggregated snapshot and report models for the paft ecosystem.
//!
//! This crate provides summary and reporting types that integrate with the `paft` ecosystem.
//!
//! # Quickstart
//! ```rust
//! use paft_aggregates::{DownloadReport, InfoReport, SearchReport};
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod info;
pub mod reports;

pub use info::{FastInfo, Info};
pub use reports::{DownloadReport, InfoReport, SearchReport};
