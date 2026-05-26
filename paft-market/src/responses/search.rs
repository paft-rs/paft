//! Search response types.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

use paft_domain::Instrument;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single search result item.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`SearchResult`] alias for the
/// standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericSearchResult<M = ()> {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Display name.
    pub name: Option<String>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard `SearchResult` with no extra provider metadata.
pub type SearchResult = GenericSearchResult<()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// Response containing the merged search results.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation and propagated into each result. Use the
/// [`SearchResponse`] alias for the standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericSearchResponse<M = ()> {
    /// De-duplicated search results.
    pub results: Vec<GenericSearchResult<M>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard `SearchResponse` with no extra provider metadata.
pub type SearchResponse = GenericSearchResponse<()>;
