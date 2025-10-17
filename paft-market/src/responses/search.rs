//! Search response types.

use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

use paft_domain::{AssetKind, Exchange, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single search result item.
pub struct SearchResult {
    /// Symbol identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub symbol: Symbol,
    /// Display name.
    pub name: Option<String>,
    /// Exchange identifier with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub exchange: Option<Exchange>,
    /// Classified asset kind.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub kind: AssetKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Response containing the merged search results.
pub struct SearchResponse {
    /// De-duplicated search results.
    pub results: Vec<SearchResult>,
}
