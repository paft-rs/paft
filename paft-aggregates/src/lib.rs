//! Aggregated snapshot models for the paft ecosystem.
//!
//! This crate provides instrument snapshot types that integrate with the `paft`
//! ecosystem.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod snapshot;

pub use snapshot::{GenericSnapshot, Snapshot};
