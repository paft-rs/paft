//! Logical identity for prediction events, markets, and outcome instruments.

use crate::error::PredictionError;
use crate::identifiers::{
    PredictionEventId, PredictionMarketId, PredictionOutcomeId, PredictionVenue,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Venue-namespaced key for a prediction event/grouping container.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(df_derive_macros::ToDataFrame))]
pub struct PredictionEventKey {
    /// Prediction venue that issued the event identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub venue: PredictionVenue,
    /// Provider-native event/group identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub event_id: PredictionEventId,
}

impl PredictionEventKey {
    /// Construct a key from already-validated venue and event id values.
    #[must_use]
    pub const fn from_parts(venue: PredictionVenue, event_id: PredictionEventId) -> Self {
        Self { venue, event_id }
    }

    /// Construct a key from string inputs.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] if either input fails validation.
    pub fn new(venue: &str, event_id: &str) -> Result<Self, PredictionError> {
        Ok(Self {
            venue: venue.parse()?,
            event_id: PredictionEventId::new(event_id)?,
        })
    }

    /// Returns a collision-resistant, venue-namespaced identity key.
    #[must_use]
    pub fn unique_key(&self) -> String {
        component_key(self.venue.as_str(), "event", self.event_id.as_str())
    }
}

impl fmt::Display for PredictionEventKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.venue, self.event_id)
    }
}

/// Venue-namespaced key for a prediction market of any shape.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(df_derive_macros::ToDataFrame))]
pub struct PredictionMarketKey {
    /// Prediction venue that issued the market identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub venue: PredictionVenue,
    /// Provider-native market/claim identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub market_id: PredictionMarketId,
}

impl PredictionMarketKey {
    /// Construct a key from already-validated venue and market id values.
    #[must_use]
    pub const fn from_parts(venue: PredictionVenue, market_id: PredictionMarketId) -> Self {
        Self { venue, market_id }
    }

    /// Construct a key from string inputs.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] if either input fails validation.
    pub fn new(venue: &str, market_id: &str) -> Result<Self, PredictionError> {
        Ok(Self {
            venue: venue.parse()?,
            market_id: PredictionMarketId::new(market_id)?,
        })
    }

    /// Returns a collision-resistant, venue-namespaced identity key.
    #[must_use]
    pub fn unique_key(&self) -> String {
        component_key(self.venue.as_str(), "market", self.market_id.as_str())
    }
}

impl fmt::Display for PredictionMarketKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.venue, self.market_id)
    }
}

/// Venue-namespaced key for an atomic binary yes/no market.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(df_derive_macros::ToDataFrame))]
pub struct BinaryMarketKey {
    /// Prediction venue that issued the market identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub venue: PredictionVenue,
    /// Provider-native atomic binary market/claim identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub market_id: PredictionMarketId,
}

impl BinaryMarketKey {
    /// Construct a binary market key from already-validated parts.
    #[must_use]
    pub const fn from_parts(venue: PredictionVenue, market_id: PredictionMarketId) -> Self {
        Self { venue, market_id }
    }

    /// Construct a binary market key from string inputs.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] if either input fails validation.
    pub fn new(venue: &str, market_id: &str) -> Result<Self, PredictionError> {
        Ok(Self {
            venue: venue.parse()?,
            market_id: PredictionMarketId::new(market_id)?,
        })
    }

    /// Returns this binary key as a shape-agnostic market key.
    #[must_use]
    pub fn to_market_key(&self) -> PredictionMarketKey {
        PredictionMarketKey {
            venue: self.venue.clone(),
            market_id: self.market_id.clone(),
        }
    }

    /// Returns the synthetic YES outcome instrument for this binary market.
    ///
    /// This is intended for venues whose provider-native binary instruments are
    /// naturally identified as `YES`/`NO`, such as Kalshi-style adapters. Venues
    /// with provider-issued outcome ids, such as Polymarket CLOB token ids,
    /// should carry those concrete ids in [`BinaryOutcomeInstruments`].
    #[must_use]
    pub fn yes_instrument(&self) -> OutcomeInstrument {
        self.synthetic_outcome_instrument("YES")
    }

    /// Returns the synthetic NO outcome instrument for this binary market.
    ///
    /// This is intended for venues whose provider-native binary instruments are
    /// naturally identified as `YES`/`NO`, such as Kalshi-style adapters. Venues
    /// with provider-issued outcome ids, such as Polymarket CLOB token ids,
    /// should carry those concrete ids in [`BinaryOutcomeInstruments`].
    #[must_use]
    pub fn no_instrument(&self) -> OutcomeInstrument {
        self.synthetic_outcome_instrument("NO")
    }

    fn synthetic_outcome_instrument(&self, outcome_id: &'static str) -> OutcomeInstrument {
        OutcomeInstrument {
            venue: self.venue.clone(),
            market_id: self.market_id.clone(),
            outcome_id: PredictionOutcomeId::from_static_unchecked(outcome_id),
        }
    }

    /// Returns a collision-resistant, venue-namespaced identity key.
    #[must_use]
    pub fn unique_key(&self) -> String {
        component_key(
            self.venue.as_str(),
            "binary_market",
            self.market_id.as_str(),
        )
    }
}

impl fmt::Display for BinaryMarketKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.venue, self.market_id)
    }
}

/// Tradable outcome share/token/contract identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(df_derive_macros::ToDataFrame))]
pub struct OutcomeInstrument {
    /// Prediction venue that issued the outcome instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub venue: PredictionVenue,
    /// Provider-native market/claim identifier this outcome belongs to.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub market_id: PredictionMarketId,
    /// Provider-native outcome instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub outcome_id: PredictionOutcomeId,
}

impl OutcomeInstrument {
    /// Construct an outcome instrument from already-validated parts.
    #[must_use]
    pub const fn from_parts(
        venue: PredictionVenue,
        market_id: PredictionMarketId,
        outcome_id: PredictionOutcomeId,
    ) -> Self {
        Self {
            venue,
            market_id,
            outcome_id,
        }
    }

    /// Construct an outcome instrument from string inputs.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] if any input fails validation.
    pub fn new(venue: &str, market_id: &str, outcome_id: &str) -> Result<Self, PredictionError> {
        Ok(Self {
            venue: venue.parse()?,
            market_id: PredictionMarketId::new(market_id)?,
            outcome_id: PredictionOutcomeId::new(outcome_id)?,
        })
    }

    /// Returns the market key this outcome instrument belongs to.
    #[must_use]
    pub fn market_key(&self) -> PredictionMarketKey {
        PredictionMarketKey {
            venue: self.venue.clone(),
            market_id: self.market_id.clone(),
        }
    }

    /// Returns a collision-resistant, venue-namespaced identity key.
    #[must_use]
    pub fn unique_key(&self) -> String {
        let venue = self.venue.as_str();
        let market = self.market_id.as_str();
        let outcome = self.outcome_id.as_str();
        format!(
            "{}|market:{}:{}|outcome:{}:{}",
            venue,
            market.len(),
            market,
            outcome.len(),
            outcome
        )
    }
}

impl fmt::Display for OutcomeInstrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}/{}", self.venue, self.market_id, self.outcome_id)
    }
}

/// Required YES and NO outcome instruments for an atomic binary market.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BinaryOutcomeInstruments {
    /// Tradable YES claim/share/token/contract.
    pub yes: OutcomeInstrument,
    /// Tradable NO claim/share/token/contract.
    pub no: OutcomeInstrument,
}

impl BinaryOutcomeInstruments {
    /// Construct binary outcome instruments from already-validated values.
    #[must_use]
    pub const fn new(yes: OutcomeInstrument, no: OutcomeInstrument) -> Self {
        Self { yes, no }
    }

    /// Construct synthetic `YES`/`NO` instruments for a binary market key.
    ///
    /// Use this for venues whose provider-native outcome identity is stable as
    /// `YES`/`NO`. Use [`Self::new`] with concrete provider ids when the venue
    /// exposes distinct outcome identifiers.
    #[must_use]
    pub fn synthetic_for_market(key: &BinaryMarketKey) -> Self {
        Self {
            yes: key.yes_instrument(),
            no: key.no_instrument(),
        }
    }
}

fn component_key(venue: &str, role: &str, id: &str) -> String {
    format!("{venue}|{role}:{}:{id}", id.len())
}
