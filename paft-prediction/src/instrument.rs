//! Logical identity for a prediction-market outcome.

use crate::error::PredictionError;
use crate::identifiers::{EventID, OutcomeID};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Logical identity for a single prediction-market outcome.
///
/// Pairs the event/question identifier with the specific tradeable outcome
/// identifier. Parallels `paft_domain::Instrument` for prediction markets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(df_derive_macros::ToDataFrame))]
pub struct PredictionInstrument {
    /// Identifier of the event/question this outcome belongs to.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub event_id: EventID,
    /// Identifier of the specific tradeable outcome.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub outcome_id: OutcomeID,
}

impl PredictionInstrument {
    /// Construct a new `PredictionInstrument` from string ids, validating each.
    ///
    /// # Errors
    /// Returns [`PredictionError::InvalidEventId`] or
    /// [`PredictionError::InvalidOutcomeId`] if validation fails.
    pub fn new(event_id: &str, outcome_id: &str) -> Result<Self, PredictionError> {
        Ok(Self {
            event_id: EventID::new(event_id)?,
            outcome_id: OutcomeID::new(outcome_id)?,
        })
    }

    /// Construct a `PredictionInstrument` from already-validated ids.
    #[must_use]
    pub const fn from_ids(event_id: EventID, outcome_id: OutcomeID) -> Self {
        Self {
            event_id,
            outcome_id,
        }
    }

    /// Returns the unique key for this prediction outcome (`event_id/outcome_id`).
    ///
    /// The key includes the event id because outcome ids are only assumed to be
    /// unique within an event.
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        let event_id = self.event_id.as_ref();
        let outcome_id = self.outcome_id.as_ref();
        let mut key = String::with_capacity(event_id.len() + 1 + outcome_id.len());
        key.push_str(event_id);
        key.push('/');
        key.push_str(outcome_id);

        Cow::Owned(key)
    }
}

impl std::fmt::Display for PredictionInstrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.event_id.as_ref())?;
        f.write_str("/")?;
        f.write_str(self.outcome_id.as_ref())
    }
}
