//! Prediction-market data models for the paft ecosystem.
//!
//! Provides validated identifier newtypes ([`EventID`], [`OutcomeID`]), a
//! [`PredictionInstrument`] that pairs them, and the higher-level [`Market`] and
//! [`Token`] aggregates used to describe a tradeable outcome of a prediction
//! market. Money values reuse [`paft_money`] so amounts compose cleanly with
//! the rest of the workspace.
//!
//! # Feature flags
//! - `dataframe`: derive `ToDataFrame` impls so prediction types can be
//!   exported via `polars`.
//! - `bigdecimal`: switch the underlying money backend to `bigdecimal` (forwarded
//!   to `paft-money` and `paft-utils`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod identifiers;
pub mod instrument;

pub use error::PredictionError;
pub use identifiers::{EventID, OutcomeID};
pub use instrument::PredictionInstrument;

use paft_money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

/// A single tradeable outcome of a [`Market`].
///
/// `Token` is the unit traders actually buy and sell — for a binary market,
/// "Yes" and "No" are each modelled as a `Token` underneath the same
/// [`Market`].
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Token {
    /// Provider-issued identifier for the tradeable outcome.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub outcome_id: OutcomeID,
    /// Human-readable label for the outcome (e.g. `"Yes"`, `"No"`,
    /// `"Candidate A"`).
    pub outcome: String,
}

/// A prediction market — the question being predicted and the [`Token`]s used
/// to bet on its outcomes.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct Market {
    /// On-chain or provider-issued identifier of the underlying event.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub event_id: EventID,
    /// Tradeable outcomes belonging to this market.
    pub tokens: Vec<Token>,
    /// Long-form description of the market and its resolution criteria.
    pub description: String,
    /// Short-form question phrasing shown to traders.
    pub question: String,
    /// Provider-specific URL slug used to address this market.
    pub market_slug: String,
    /// Optional category or topic the market belongs to.
    pub category: Option<String>,
    /// Currency in which positions are collateralised and `PnL` is denominated.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub collateral_currency: Currency,
    /// Smallest order size accepted by the venue for this market.
    pub minimum_order_size: Money,
    /// Smallest price increment accepted by the venue for this market.
    pub minimum_tick_size: Money,
    /// Whether the market is currently accepting orders.
    pub is_active: bool,
    /// Whether the market has been closed (resolution may or may not have
    /// occurred).
    pub is_closed: bool,
    // pub end_date: Option<DateTime<Utc>>,
}
