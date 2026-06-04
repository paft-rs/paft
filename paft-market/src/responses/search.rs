//! Search response types.

use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

use paft_domain::Instrument;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Response containing the merged search results.
///
/// Generic over a response-level provider metadata payload `R`, which is
/// flattened into the serialized representation, and a result-level metadata
/// payload `S`. Use the [`SearchResponse`] alias for the standard shape
/// (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericSearchResponse<R = (), S = ()> {
    /// De-duplicated search results.
    pub results: Vec<GenericSearchResult<S>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: R,
}

/// Standard `SearchResponse` with no extra provider metadata.
pub type SearchResponse = GenericSearchResponse<(), ()>;
