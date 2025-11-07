//! Prediction market data models for paft.

use paft_domain::identifiers::{ConditionID, TokenID};
use paft_money::{Money, Currency};
use serde::{Deserialize, Serialize};

/// Represents a single tradeable outcome of a prediction market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub token_id: TokenID,
    pub outcome: String,
}

/// Represents the prediction market itself (the "question").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Market {
    pub condition_id: ConditionID,
    pub tokens: Vec<Token>, // The tradeable outcomes
    pub description: String,
    pub question: String,
    pub market_slug: String,
    pub category: Option<String>,
    pub collateral_currency: Currency,
    pub minimum_order_size: Money,
    pub minimum_tick_size: Money,
    pub is_active: bool,
    pub is_closed: bool,
    // pub end_date: Option<DateTime<Utc>>,
}