//! Bulk download response types.

use serde::{Deserialize, Serialize};

use crate::responses::history::GenericHistoryResponse;
use paft_domain::{Instrument, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A single instrument-scoped history entry within a bulk download.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into the inner history
/// response. Use the [`DownloadEntry`] alias for the standard shape
/// (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericDownloadEntry<M = ()> {
    /// Full instrument identity (symbol, kind, optional identifiers/venue).
    pub instrument: Instrument,
    /// Full `HistoryResponse` (candles, actions, price basis, meta).
    pub history: GenericHistoryResponse<M>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard `DownloadEntry` with no extra provider metadata.
pub type DownloadEntry = GenericDownloadEntry<()>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Aggregated bulk-download response as an array of instrument-keyed entries.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into each entry. Use the
/// [`DownloadResponse`] alias for the standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericDownloadResponse<M = ()> {
    /// Entries keyed by full `Instrument` identity.
    pub entries: Vec<GenericDownloadEntry<M>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard `DownloadResponse` with no extra provider metadata.
pub type DownloadResponse = GenericDownloadResponse<()>;

impl<M> GenericDownloadResponse<M> {
    /// Zero-copy iterator over entries as (&Instrument, &`GenericHistoryResponse<M>`).
    pub fn iter(&self) -> impl Iterator<Item = (&Instrument, &GenericHistoryResponse<M>)> + '_ {
        self.entries
            .iter()
            .map(|entry| (&entry.instrument, &entry.history))
    }

    /// Zero-copy iterator exposing a symbol-centric view (duplicates may exist).
    pub fn iter_by_symbol(
        &self,
    ) -> impl Iterator<Item = (&Symbol, &GenericHistoryResponse<M>)> + '_ {
        self.entries
            .iter()
            .map(|entry| (&entry.instrument.symbol, &entry.history))
    }
}
