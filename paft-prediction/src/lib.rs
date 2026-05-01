//! Prediction market data models for paft.

pub mod identifiers;
pub mod instrument;

pub use identifiers::{EventID, OutcomeID};
pub use instrument::PredictionInstrument;

use paft_money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;

/// Represents a single tradeable outcome of a prediction market.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Token {
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub outcome_id: OutcomeID,
    pub outcome: String,
}

/// Represents the prediction market itself (the "question").
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Market {
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub event_id: EventID,
    pub tokens: Vec<Token>, // The tradeable outcomes
    pub description: String,
    pub question: String,
    pub market_slug: String,
    pub category: Option<String>,
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub collateral_currency: Currency,
    pub minimum_order_size: Money,
    pub minimum_tick_size: Money,
    pub is_active: bool,
    pub is_closed: bool,
    // pub end_date: Option<DateTime<Utc>>,
}
