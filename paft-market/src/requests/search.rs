//! Search request and response types.

use serde::{Deserialize, Serialize};

use paft_domain::AssetKind;

use crate::error::MarketError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Request to search for instruments by free-text query.
///
/// Use `SearchRequest::builder()` to create instances.
pub struct SearchRequest {
    /// Free-text user query (symbol, name, etc.). Must be non-empty.
    query: String,
    /// Optional asset-kind filter. If set, routers/connectors may restrict results.
    kind: Option<AssetKind>,
    /// Optional maximum number of results to return after routing/merge.
    limit: Option<usize>,
    /// Optional ISO language code (e.g., "en", "fr").
    lang: Option<String>,
    /// Optional region code to scope results (e.g., "US", "EU").
    region: Option<String>,
}

/// Builder for creating validated `SearchRequest` instances.
#[derive(Debug, Clone, Default)]
pub struct SearchRequestBuilder {
    query: String,
    kind: Option<AssetKind>,
    limit: Option<usize>,
    lang: Option<String>,
    region: Option<String>,
}

impl SearchRequestBuilder {
    /// Create a new builder with the provided query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            kind: None,
            limit: None,
            lang: None,
            region: None,
        }
    }

    /// Set the asset kind filter.
    #[must_use]
    pub const fn kind(mut self, kind: AssetKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Set the result limit.
    #[must_use]
    pub const fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the preferred language for search results.
    #[must_use]
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = Some(lang.into());
        self
    }

    /// Set the region to scope search results.
    #[must_use]
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Build the `SearchRequest` with validation.
    ///
    /// Returns an error if:
    /// - Query is empty or only whitespace
    /// - Limit is set to 0
    ///
    /// # Errors
    /// Returns `MarketError::EmptySearchQuery` if the query is empty/whitespace,
    /// or `MarketError::InvalidSearchLimit(0)` if a zero limit is provided.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn build(self) -> Result<SearchRequest, MarketError> {
        // Validate request invariants
        if self.query.trim().is_empty() {
            return Err(MarketError::EmptySearchQuery);
        }
        if let Some(lim) = self.limit
            && lim == 0
        {
            return Err(MarketError::InvalidSearchLimit(lim));
        }

        Ok(SearchRequest {
            query: self.query,
            kind: self.kind,
            limit: self.limit,
            lang: self.lang,
            region: self.region,
        })
    }
}

impl SearchRequest {
    /// Create a new builder for constructing a `SearchRequest`.
    pub fn builder(query: impl Into<String>) -> SearchRequestBuilder {
        SearchRequestBuilder::new(query)
    }

    /// Construct a new request with the provided query.
    ///
    /// This is a convenience method that uses the builder pattern internally.
    ///
    /// # Errors
    /// Propagates errors from `SearchRequestBuilder::build`.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(query), err)
    )]
    pub fn new(query: impl Into<String>) -> Result<Self, MarketError> {
        Self::builder(query).build()
    }

    /// Get the search query.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get the asset kind filter if set.
    #[must_use]
    pub const fn kind(&self) -> Option<AssetKind> {
        self.kind
    }

    /// Get the result limit if set.
    #[must_use]
    pub const fn limit(&self) -> Option<usize> {
        self.limit
    }

    /// Get the language if set.
    #[must_use]
    pub fn lang(&self) -> Option<&str> {
        self.lang.as_deref()
    }

    /// Get the region if set.
    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
}
