//! Corporate action types under the `paft_market::market::action` namespace.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_money::Money;
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Corporate action attached to a history series.
pub enum Action {
    /// Cash dividend.
    Dividend {
        /// Timestamp.
        #[serde(with = "chrono::serde::ts_seconds")]
        ts: DateTime<Utc>,
        /// Amount paid per share.
        amount: Money,
    },
    /// Stock split.
    Split {
        /// Timestamp.
        #[serde(with = "chrono::serde::ts_seconds")]
        ts: DateTime<Utc>,
        /// Split numerator.
        numerator: u32,
        /// Split denominator.
        denominator: u32,
    },
    /// Capital gain distribution.
    CapitalGain {
        /// Timestamp.
        #[serde(with = "chrono::serde::ts_seconds")]
        ts: DateTime<Utc>,
        /// Distribution amount.
        gain: Money,
    },
}

#[cfg(feature = "dataframe")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
struct ActionRow {
    pub action_type: String,
    pub ts: DateTime<Utc>,
    pub amount: Option<Money>,
    pub numerator: Option<u32>,
    pub denominator: Option<u32>,
    pub gain: Option<Money>,
}

#[cfg(feature = "dataframe")]
impl From<&Action> for ActionRow {
    fn from(action: &Action) -> Self {
        match action {
            Action::Dividend { ts, amount } => Self {
                action_type: "Dividend".to_string(),
                ts: *ts,
                amount: Some(amount.clone()),
                numerator: None,
                denominator: None,
                gain: None,
            },
            Action::Split {
                ts,
                numerator,
                denominator,
            } => Self {
                action_type: "Split".to_string(),
                ts: *ts,
                amount: None,
                numerator: Some(*numerator),
                denominator: Some(*denominator),
                gain: None,
            },
            Action::CapitalGain { ts, gain } => Self {
                action_type: "CapitalGain".to_string(),
                ts: *ts,
                amount: None,
                numerator: None,
                denominator: None,
                gain: Some(gain.clone()),
            },
        }
    }
}

#[cfg(feature = "dataframe")]
impl ToDataFrame for Action {
    fn to_dataframe(&self) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        ActionRow::from(self).to_dataframe()
    }

    fn empty_dataframe() -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        ActionRow::empty_dataframe()
    }

    fn schema() -> polars::prelude::PolarsResult<Vec<(&'static str, polars::datatypes::DataType)>> {
        ActionRow::schema()
    }
}

#[cfg(feature = "dataframe")]
impl Columnar for Action {
    fn columnar_to_dataframe(
        items: &[Self],
    ) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        let rows: Vec<ActionRow> = items.iter().map(ActionRow::from).collect();
        rows.as_slice().to_dataframe()
    }
}
