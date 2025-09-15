//! News request parameters and enums.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Tabs/categories for news requests.
pub enum NewsTab {
    /// The latest news articles. (Default)
    #[default]
    News,
    /// All news-related content.
    All,
    /// Official press releases.
    PressReleases,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Parameters for fetching instrument news.
pub struct NewsRequest {
    /// Maximum number of articles to fetch.
    pub count: u32,
    /// Content tab/category to fetch from the provider.
    pub tab: NewsTab,
}

impl Default for NewsRequest {
    fn default() -> Self {
        Self {
            count: 10,
            tab: NewsTab::default(),
        }
    }
}
