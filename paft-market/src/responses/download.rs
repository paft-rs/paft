//! Bulk download response types.

use serde::{Deserialize, Serialize};

use crate::responses::history::HistoryResponse;
use paft_domain::{Instrument, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A single instrument-scoped history entry within a bulk download.
pub struct DownloadEntry {
    /// Full instrument identity (symbol, kind, optional identifiers/venue).
    pub instrument: Instrument,
    /// Full `HistoryResponse` (candles, actions, adjusted, meta).
    pub history: HistoryResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Aggregated bulk-download response as an array of instrument-keyed entries.
pub struct DownloadResponse {
    /// Entries keyed by full `Instrument` identity.
    pub entries: Vec<DownloadEntry>,
}

impl DownloadResponse {
    /// Zero-copy iterator over entries as (&Instrument, &`HistoryResponse`).
    pub fn iter(&self) -> impl Iterator<Item = (&Instrument, &HistoryResponse)> + '_ {
        self.entries
            .iter()
            .map(|entry| (&entry.instrument, &entry.history))
    }

    /// Zero-copy iterator exposing a symbol-centric view (duplicates may exist).
    pub fn iter_by_symbol(&self) -> impl Iterator<Item = (&Symbol, &HistoryResponse)> + '_ {
        self.entries
            .iter()
            .map(|entry| (entry.instrument.symbol(), &entry.history))
    }
}
