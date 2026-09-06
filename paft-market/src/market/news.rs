//! News article types returned from market data endpoints.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Publication UTC instant, encoded as canonical ISO-8601-style text.
    #[serde(with = "paft_core::serde_helpers::ts_iso8601")]
    pub published_at: DateTime<Utc>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

#[cfg(feature = "dataframe")]
paft_utils::impl_checked_dataframe! {
    GenericNewsArticle<M> {
        uuid: [String],
        title: [String],
        publisher: [Option<String>],
        link: [Option<String>],
        #[df_derive(time_unit = "ns")]
        published_at: [DateTime<Utc>],
        provider: [M],
    }
    validate |row| paft_core::serde_helpers::validate_timestamp_nanos("published_at", &row.published_at)
}

/// Standard `NewsArticle` with no extra provider metadata.
pub type NewsArticle = GenericNewsArticle<()>;
