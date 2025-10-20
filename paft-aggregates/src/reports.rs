//! Report envelopes for market operations.
//!
//! These types wrap upstream responses with additional context such as
//! validation or processing warnings, making them suitable for logging,
//! storage, or user-facing reporting.

use super::Info;
use paft_domain::Symbol;
use paft_market::responses::download::DownloadResponse;
use paft_market::responses::search::SearchResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// Summary of instrument information retrieval.
///
/// Carries the requested `symbol`, the resolved [`Info`] snapshot if
/// available, and any non-fatal warnings encountered during processing.
pub struct InfoReport {
    /// Requested symbol.
    pub symbol: Symbol,
    /// Snapshot payload, if successfully resolved.
    pub info: Option<Info>,
    /// Non-fatal issues encountered while building the report.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Summary of a symbol search operation.
///
/// Contains the upstream search `response` when present and any associated
/// `warnings`.
pub struct SearchReport {
    /// Upstream search response payload.
    pub response: Option<SearchResponse>,
    /// Non-fatal issues encountered while building the report.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Summary of historical data download.
///
/// Wraps a [`DownloadResponse`] payload when present and any `warnings`
/// captured during retrieval or normalization.
pub struct DownloadReport {
    /// Aggregated download payload.
    pub response: Option<DownloadResponse>,
    /// Non-fatal issues encountered while building the report.
    pub warnings: Vec<String>,
}
