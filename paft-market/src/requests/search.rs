//! Search request and response types.

use serde::{Deserialize, Serialize};

use paft_core::domain::AssetKind;

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
}

/// Builder for creating validated `SearchRequest` instances.
#[derive(Debug, Clone, Default)]
pub struct SearchRequestBuilder {
    query: String,
    kind: Option<AssetKind>,
    limit: Option<usize>,
}

impl SearchRequestBuilder {
    /// Create a new builder with the provided query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            kind: None,
            limit: None,
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

    /// Build the `SearchRequest` with validation.
    ///
    /// Returns an error if:
    /// - Query is empty or only whitespace
    /// - Limit is set to 0
    ///
    /// # Errors
    /// Returns `MarketError::EmptySearchQuery` if the query is empty/whitespace,
    /// or `MarketError::InvalidSearchLimit(0)` if a zero limit is provided.
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
}
