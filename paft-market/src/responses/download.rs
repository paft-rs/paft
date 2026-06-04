//! Bulk download response types.

use serde::{Deserialize, Serialize};

use crate::responses::history::GenericHistoryResponse;
use paft_domain::{Instrument, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A single instrument-scoped history entry within a bulk download.
///
/// Generic over an entry-level provider metadata payload `E`, which is
/// flattened into the serialized representation, plus history-response and
/// candle metadata payloads. Use the [`DownloadEntry`] alias for the standard
/// shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericDownloadEntry<E = (), H = (), C = ()> {
    /// Full instrument identity (symbol, kind, optional identifiers/venue).
    pub instrument: Instrument,
    /// Full `HistoryResponse` (candles, actions, price basis, meta).
    pub history: GenericHistoryResponse<H, C>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: E,
}

/// Standard `DownloadEntry` with no extra provider metadata.
pub type DownloadEntry = GenericDownloadEntry<(), (), ()>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Aggregated bulk-download response as an array of instrument-keyed entries.
///
/// Generic over a response-level provider metadata payload `R`, which is
/// flattened into the serialized representation, plus entry, history-response,
/// and candle metadata payloads. Use the [`DownloadResponse`] alias for the
/// standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericDownloadResponse<R = (), E = (), H = (), C = ()> {
    /// Entries keyed by full `Instrument` identity.
    pub entries: Vec<GenericDownloadEntry<E, H, C>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: R,
}

/// Standard `DownloadResponse` with no extra provider metadata.
pub type DownloadResponse = GenericDownloadResponse<(), (), (), ()>;

impl<R, E, H, C> GenericDownloadResponse<R, E, H, C> {
    /// Zero-copy iterator over entries as (&Instrument, &`GenericHistoryResponse<H, C>`).
    pub fn iter(&self) -> impl Iterator<Item = (&Instrument, &GenericHistoryResponse<H, C>)> + '_ {
        self.entries
            .iter()
            .map(|entry| (&entry.instrument, &entry.history))
    }

    /// Zero-copy iterator exposing a symbol-centric view (duplicates may exist).
    pub fn iter_by_symbol(
        &self,
    ) -> impl Iterator<Item = (&Symbol, &GenericHistoryResponse<H, C>)> + '_ {
        self.entries
            .iter()
            .map(|entry| (&entry.instrument.symbol, &entry.history))
    }
}
