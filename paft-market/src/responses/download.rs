//! Bulk download response types.

use serde::{Deserialize, Serialize};

use crate::market::action::Action;
use crate::responses::history::{Candle, HistoryMeta};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Aggregated bulk-download response grouping series, metadata, and actions per symbol.
pub struct DownloadResponse {
    /// Map of symbol to merged candle series.
    pub series: std::collections::HashMap<String, Vec<Candle>>,
    /// Map of symbol to optional series metadata.
    pub meta: std::collections::HashMap<String, Option<HistoryMeta>>,
    /// Map of symbol to corporate actions.
    pub actions: std::collections::HashMap<String, Vec<Action>>,
    /// Whether returned prices are adjusted when applicable.
    pub adjusted: bool,
}
