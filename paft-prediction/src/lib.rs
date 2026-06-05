//! Prediction-market data models for the paft ecosystem.
//!
//! `paft-prediction` models provider-neutral prediction concepts:
//! [`PredictionEvent`] groups related markets, [`BinaryMarket`] is one atomic
//! yes/no claim, [`OutcomeInstrument`] is a tradable outcome share/token/contract,
//! [`OutcomePrice`] is a fixed-point fraction of unit payout, and
//! [`BinaryOrderBook`] is normalized to a canonical YES view.
//!
//! Provider-native tickers, slugs, condition ids, CTF token ids, and venue
//! mechanics belong in role-specific opaque identifiers or provider metadata,
//! not in the core domain model.
//!
//! # Feature flags
//! - `dataframe`: derive `ToDataFrame` impls for flat prediction identity types.
//! - `bigdecimal`: switch the shared decimal backend to `bigdecimal` (forwarded
//!   to `paft-decimal`, `paft-money`, and `paft-utils`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod identifiers;
pub mod instrument;
pub mod market;
pub mod market_data;
pub mod price;

pub use error::PredictionError;
pub use identifiers::{
    MAX_PREDICTION_ID_LEN, OtherPredictionVenue, PredictionEventId, PredictionMarketId,
    PredictionOutcomeId, PredictionSeriesId, PredictionVenue,
};
pub use instrument::{BinaryMarketKey, OutcomeInstrument, PredictionEventKey, PredictionMarketKey};
pub use market::{
    BinaryMarket, BinaryResolution, ClaimDescriptor, EventStructure, GenericBinaryMarket,
    GenericMultiOutcomeMarket, GenericPredictionEvent, GenericPredictionMarket,
    GenericScalarMarket, LinkedBinaryRelation, MultiOutcomeMarket, NumericBound, NumericRange,
    OtherBinaryResolution, OtherClaimDescriptor, OtherEventStructure, OtherLinkedBinaryRelation,
    OtherPredictionMarketStatus, OutcomeDescriptor, PredictionEvent, PredictionMarket,
    PredictionMarketStatus, ScalarMarket,
};
pub use market_data::{
    BinaryOrderBook, BinaryOrderDirection, BinaryOutcome, BinaryQuote, BookSide,
    GenericBinaryOrderBook, GenericBinaryQuote, GenericOutcomeOrderBook, GenericPredictionTrade,
    OutcomeOrderBook, PredictionBookLevel, PredictionTrade, TradeAction,
};
pub use price::{ContractQuantity, OutcomePayout, OutcomePrice, PriceBand, PriceGrid, PriceTick};
