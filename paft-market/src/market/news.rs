//! News article types returned from market data endpoints.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A news article associated with an instrument.
pub struct NewsArticle {
    /// A unique identifier for the article.
    pub uuid: String,
    /// The headline of the article.
    pub title: String,
    /// The publisher of the article (e.g., "Reuters", "Associated Press").
    pub publisher: Option<String>,
    /// A direct link to the article.
    pub link: Option<String>,
    /// The Unix timestamp (in seconds) of when the article was published.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub published_at: DateTime<Utc>,
}
