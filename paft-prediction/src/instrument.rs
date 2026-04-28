//! Logical identity for a prediction-market outcome.

use crate::identifiers::{EventID, OutcomeID};
use paft_domain::DomainError;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Logical identity for a single prediction-market outcome.
///
/// Pairs the event/question identifier with the specific tradeable outcome
/// identifier. Parallels `paft_domain::Instrument` for prediction markets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredictionInstrument {
    /// Identifier of the event/question this outcome belongs to.
    pub event_id: EventID,
    /// Identifier of the specific tradeable outcome.
    pub outcome_id: OutcomeID,
}

impl PredictionInstrument {
    /// Construct a new `PredictionInstrument` from string ids, validating each.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidEventId` or `DomainError::InvalidOutcomeId`
    /// if validation fails.
    pub fn new(event_id: &str, outcome_id: &str) -> Result<Self, DomainError> {
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

    /// Returns the unique key for this prediction outcome (the outcome id).
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.outcome_id.as_ref())
    }
}

impl std::fmt::Display for PredictionInstrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unique_key())
    }
}

#[cfg(feature = "dataframe")]
mod dataframe_impl {
    use super::PredictionInstrument;
    use paft_utils::dataframe::{Columnar, ToDataFrame};
    use polars::datatypes::DataType;
    use polars::prelude::{DataFrame, NamedFrom, PolarsResult, Series};

    impl ToDataFrame for PredictionInstrument {
        fn to_dataframe(&self) -> PolarsResult<DataFrame> {
            <Self as Columnar>::columnar_to_dataframe(std::slice::from_ref(self))
        }

        fn empty_dataframe() -> PolarsResult<DataFrame> {
            DataFrame::new(vec![
                Series::new_empty("event_id".into(), &DataType::String).into(),
                Series::new_empty("outcome_id".into(), &DataType::String).into(),
            ])
        }

        fn schema() -> PolarsResult<Vec<(&'static str, DataType)>> {
            Ok(vec![
                ("event_id", DataType::String),
                ("outcome_id", DataType::String),
            ])
        }
    }

    impl Columnar for PredictionInstrument {
        fn columnar_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
            let event_ids: Vec<String> = items
                .iter()
                .map(|i| i.event_id.as_ref().to_string())
                .collect();
            let outcome_ids: Vec<String> = items
                .iter()
                .map(|i| i.outcome_id.as_ref().to_string())
                .collect();

            let df = DataFrame::new(vec![
                Series::new("event_id".into(), event_ids).into(),
                Series::new("outcome_id".into(), outcome_ids).into(),
            ])?;
            Ok(df)
        }
    }
}
