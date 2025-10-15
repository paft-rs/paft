//! Search response types.

use serde::{Deserialize, Serialize};

use paft_domain::{AssetKind, Exchange, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// A single search result item.
pub struct SearchResult {
    /// Symbol identifier.
    pub symbol: Symbol,
    /// Display name.
    pub name: Option<String>,
    /// Exchange identifier with canonical variants and extensible fallback.
    pub exchange: Option<Exchange>,
    /// Classified asset kind.
    pub kind: AssetKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Response containing the merged search results.
pub struct SearchResponse {
    /// De-duplicated search results.
    pub results: Vec<SearchResult>,
}
