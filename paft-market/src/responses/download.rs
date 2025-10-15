//! Bulk download response types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::responses::history::HistoryResponse;
use paft_domain::Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Aggregated bulk-download response grouping full history payloads per symbol.
pub struct DownloadResponse {
    /// Map of symbol to full `HistoryResponse` (candles, actions, adjusted, meta).
    pub history: HashMap<Symbol, HistoryResponse>,
}
