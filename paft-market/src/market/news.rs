//! News article types returned from market data endpoints.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A news article associated with an instrument.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`NewsArticle`] alias for the
/// standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericNewsArticle<M = ()> {
    /// A unique identifier for the article.
    pub uuid: String,
    /// The headline of the article.
    pub title: String,
    /// The publisher of the article (e.g., "Reuters", "Associated Press").
    pub publisher: Option<String>,
    /// A direct link to the article.
    pub link: Option<String>,
    /// The Unix timestamp in milliseconds of when the article was published.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub published_at: DateTime<Utc>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard `NewsArticle` with no extra provider metadata.
pub type NewsArticle = GenericNewsArticle<()>;
