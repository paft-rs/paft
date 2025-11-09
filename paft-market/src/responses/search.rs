//! Search response types.

use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

use paft_domain::{AssetKind, Exchange, Instrument};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single search result item.
pub struct SearchResult {
    /// Instrument identifier.
    pub instrument: Instrument,
    /// Display name.
    pub name: Option<String>,
    /// Exchange identifier.
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
