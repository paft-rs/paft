//! News request parameters and enums.

use std::fmt;
use std::num::NonZeroU32;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
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

impl NewsTab {
    const VARIANTS: &'static [&'static str] = &["NEWS", "ALL", "PRESS_RELEASES"];

    /// Returns the stable wire-format code for this tab.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::News => "NEWS",
            Self::All => "ALL",
            Self::PressReleases => "PRESS_RELEASES",
        }
    }

    fn from_code(value: &str) -> Option<Self> {
        match value {
            "NEWS" => Some(Self::News),
            "ALL" => Some(Self::All),
            "PRESS_RELEASES" => Some(Self::PressReleases),
            _ => None,
        }
    }
}

impl AsRef<str> for NewsTab {
    fn as_ref(&self) -> &str {
        (*self).code()
    }
}

impl fmt::Display for NewsTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).code())
    }
}

impl Serialize for NewsTab {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str((*self).code())
    }
}

impl<'de> Deserialize<'de> for NewsTab {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_code(&value)
            .ok_or_else(|| serde::de::Error::unknown_variant(&value, Self::VARIANTS))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Parameters for fetching instrument news.
pub struct NewsRequest {
    /// Maximum number of articles to fetch.
    pub count: NonZeroU32,
    /// Content tab/category to fetch from the provider.
    pub tab: NewsTab,
}

impl NewsRequest {
    /// Default maximum number of articles to fetch.
    pub const DEFAULT_COUNT: NonZeroU32 = NonZeroU32::new(10).expect("10 is non-zero");
}

impl Default for NewsRequest {
    fn default() -> Self {
        Self {
            count: Self::DEFAULT_COUNT,
            tab: NewsTab::default(),
        }
    }
}
