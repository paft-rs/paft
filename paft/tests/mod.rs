//! Integration test suite for paft
//!
//! This module contains integration tests for the paft facade crate, covering:
//! - Cross-crate functionality and re-exports
//! - End-to-end workflows using types from multiple crates
//! - Serialization/deserialization across the facade
//! - DataFrame conversion (when feature enabled)
//!
//! Note: Unit tests for individual types are located in their respective crates:
//! - paft-core/tests/ - tests for Currency, Exchange, Money, etc.
//! - paft-market/tests/ - tests for SearchRequest, HistoryRequest, etc.

mod encapsulation;
mod integration;

// Re-export test modules for easy access
