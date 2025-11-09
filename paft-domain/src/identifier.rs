//! Identifier schemes grouping related IDs by asset class.
//!
//! This module defines `IdentifierScheme` and its concrete families like
//! `SecurityId` and `PredictionId`. These types encapsulate the identification
//! semantics for different asset classes and provide a `unique_key()` that
//! selects the most appropriate stable identifier for the scheme.
use super::{EventID, Exchange, Figi, Isin, OutcomeID, Symbol};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

/// Related identifiers for traditional, exchange-traded securities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct SecurityId {
    /// Canonical ticker symbol
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub symbol: Symbol,
    /// Optional trading venue context for disambiguation
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub exchange: Option<Exchange>,
    /// Optional global identifier (preferred)
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub figi: Option<Figi>,
    /// Optional global identifier (fallback)
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub isin: Option<Isin>,
}

impl SecurityId {
    /// Returns a unique key prioritizing global identifiers (FIGI > ISIN > SYMBOL@EXCHANGE > SYMBOL).
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        if let Some(figi) = &self.figi {
            return Cow::Borrowed(figi.as_ref());
        }
        if let Some(isin) = &self.isin {
            return Cow::Borrowed(isin.as_ref());
        }
        if let Some(exchange) = &self.exchange {
            return Cow::Owned(format!("{}@{}", self.symbol, exchange.code()));
        }
        Cow::Borrowed(self.symbol.as_str())
    }
}

/// Related identifiers for decentralized prediction market outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct PredictionId {
    /// Unique ID for the event/question.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub event_id: EventID,
    /// Unique ID for the tradeable outcome/answer.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub outcome_id: OutcomeID,
}

impl PredictionId {
    /// Returns a unique key for the prediction market instrument (`outcome_id`).
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.outcome_id.as_ref())
    }
}

/// The family of identifiers used to locate an asset.
///
/// Each variant groups identifiers that naturally co-exist for that asset class.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub enum IdentifierScheme {
    /// Identifiers for traditional, exchange-traded securities.
    Security(SecurityId),
    /// Identifiers for decentralized prediction market outcomes.
    Prediction(PredictionId),
}
impl IdentifierScheme {
    /// Returns the best unique key for this identifier scheme.
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        match self {
            Self::Security(id) => id.unique_key(),
            Self::Prediction(id) => id.unique_key(),
        }
    }
}
