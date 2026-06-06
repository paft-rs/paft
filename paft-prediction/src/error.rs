//! Error types specific to `paft-prediction`.

use std::fmt;
use thiserror::Error;

/// Details for a binary market whose outcome instruments do not match its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryMarketOutcomeMismatch {
    /// Binary market key venue.
    pub key_venue: String,
    /// Binary market key market id.
    pub key_market_id: String,
    /// YES instrument venue.
    pub yes_venue: String,
    /// YES instrument market id.
    pub yes_market_id: String,
    /// NO instrument venue.
    pub no_venue: String,
    /// NO instrument market id.
    pub no_market_id: String,
}

impl fmt::Display for BinaryMarketOutcomeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mismatched binary market outcomes: market key is {}:{}, YES belongs to {}:{}, NO belongs to {}:{}",
            self.key_venue,
            self.key_market_id,
            self.yes_venue,
            self.yes_market_id,
            self.no_venue,
            self.no_market_id,
        )
    }
}

/// Details for a multi-outcome entry whose instrument does not match its market key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiOutcomeMarketOutcomeMismatch {
    /// Multi-outcome market key venue.
    pub key_venue: String,
    /// Multi-outcome market key market id.
    pub key_market_id: String,
    /// Outcome instrument venue.
    pub outcome_venue: String,
    /// Outcome instrument market id.
    pub outcome_market_id: String,
    /// Outcome instrument outcome id.
    pub outcome_id: String,
}

impl fmt::Display for MultiOutcomeMarketOutcomeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mismatched multi-outcome market outcome: market key is {}:{}, outcome {} belongs to {}:{}",
            self.key_venue,
            self.key_market_id,
            self.outcome_id,
            self.outcome_venue,
            self.outcome_market_id,
        )
    }
}

/// Errors produced by prediction-market constructors and validators.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PredictionError {
    /// Invalid opaque provider identifier.
    #[error(
        "Invalid {kind}: '{value}' - expected a non-empty provider identifier with no whitespace/control characters and at most 256 bytes"
    )]
    InvalidIdentifier {
        /// Human-readable identifier role.
        kind: &'static str,
        /// Original value supplied by the caller.
        value: String,
    },

    /// Invalid prediction venue code.
    #[error(
        "Invalid prediction venue: '{value}' - expected a non-empty venue code with no whitespace/control characters and at most 256 bytes"
    )]
    InvalidVenue {
        /// Original value supplied by the caller.
        value: String,
    },

    /// Metadata code supplied to an `Other*` wrapper is already modeled.
    #[error("Invalid {kind}: '{value}' - value is already modeled")]
    ModeledMetadataCode {
        /// Human-readable metadata-code role.
        kind: &'static str,
        /// Original value supplied by the caller.
        value: String,
    },

    /// Invalid fixed-point outcome price.
    #[error("Invalid outcome price micros: {micros} - expected 0..=1_000_000")]
    InvalidOutcomePrice {
        /// Fixed-point micro value supplied by the caller.
        micros: u32,
    },

    /// Invalid fixed-point price tick.
    #[error("Invalid price tick micros: {micros} - expected 1..=1_000_000")]
    InvalidPriceTick {
        /// Fixed-point micro tick supplied by the caller.
        micros: u32,
    },

    /// Invalid non-zero fixed-point contract/share quantity.
    #[error("Invalid contract quantity microcontracts: {microcontracts} - expected > 0")]
    InvalidContractQuantity {
        /// Fixed-point microcontract count supplied by the caller.
        microcontracts: u64,
    },

    /// Invalid non-zero fixed-point outcome payout.
    #[error("Invalid outcome payout micropayouts: {micropayouts} - expected > 0")]
    InvalidOutcomePayout {
        /// Fixed-point micropayout count supplied by the caller.
        micropayouts: u64,
    },

    /// Invalid decimal input for a fixed-point prediction value.
    #[error("Invalid {kind} decimal '{value}': {reason}")]
    InvalidFixedPointDecimal {
        /// Human-readable fixed-point value role.
        kind: &'static str,
        /// Original decimal text supplied by the caller.
        value: String,
        /// Validation failure reason.
        reason: &'static str,
    },

    /// YES and NO instruments do not belong to the same venue and market.
    #[error(
        "Mismatched binary outcome instruments: YES belongs to {yes_venue}:{yes_market_id}, NO belongs to {no_venue}:{no_market_id}"
    )]
    MismatchedOutcomeInstrumentMarket {
        /// YES instrument venue.
        yes_venue: String,
        /// YES instrument market id.
        yes_market_id: String,
        /// NO instrument venue.
        no_venue: String,
        /// NO instrument market id.
        no_market_id: String,
    },

    /// YES and NO instruments have the same outcome id.
    #[error("Duplicate binary outcome instrument: {venue}:{market_id}/{outcome_id}")]
    DuplicateBinaryOutcomeInstrument {
        /// Outcome instrument venue.
        venue: String,
        /// Outcome instrument market id.
        market_id: String,
        /// Duplicate outcome id.
        outcome_id: String,
    },

    /// Binary outcome instruments do not belong to their binary market key.
    #[error("{0}")]
    MismatchedBinaryMarketOutcomes(
        /// Mismatch details.
        Box<BinaryMarketOutcomeMismatch>,
    ),

    /// Multi-outcome markets must carry at least two outcomes.
    #[error("Invalid multi-outcome market: expected at least 2 outcomes, got {count}")]
    TooFewMultiOutcomeMarketOutcomes {
        /// Number of outcomes supplied.
        count: usize,
    },

    /// A multi-outcome market outcome does not belong to its market key.
    #[error("{0}")]
    MismatchedMultiOutcomeMarketOutcome(
        /// Mismatch details.
        Box<MultiOutcomeMarketOutcomeMismatch>,
    ),

    /// A multi-outcome market repeats an outcome id.
    #[error("Duplicate multi-outcome market outcome: {venue}:{market_id}/{outcome_id}")]
    DuplicateMultiOutcomeMarketOutcome {
        /// Outcome instrument venue.
        venue: String,
        /// Outcome instrument market id.
        market_id: String,
        /// Duplicate outcome id.
        outcome_id: String,
    },

    /// A multi-outcome market resolution does not reference a listed outcome.
    #[error(
        "Invalid multi-outcome market resolution: {venue}:{market_id}/{outcome_id} is not listed"
    )]
    InvalidMultiOutcomeMarketResolution {
        /// Market venue.
        venue: String,
        /// Market id.
        market_id: String,
        /// Unlisted resolution outcome id.
        outcome_id: String,
    },

    /// Event structure metadata is inconsistent with the contained markets.
    #[error("Invalid event structure {structure} with {market_count} markets: {reason}")]
    InvalidEventStructure {
        /// Event-structure code.
        structure: &'static str,
        /// Number of markets contained by the event.
        market_count: usize,
        /// Validation failure reason.
        reason: &'static str,
    },

    /// Invalid price-band structure.
    #[error("Invalid price band: {reason}")]
    InvalidPriceBand {
        /// Validation failure reason.
        reason: &'static str,
    },

    /// Invalid price-grid structure.
    #[error("Invalid price grid: {reason}")]
    InvalidPriceGrid {
        /// Validation failure reason.
        reason: &'static str,
    },

    /// Invalid numeric range descriptor.
    #[error("Invalid numeric range: {reason}")]
    InvalidNumericRange {
        /// Validation failure reason.
        reason: &'static str,
    },

    /// Price does not fall on the applicable grid.
    #[error("Outcome price {micros} micros is not on the price grid")]
    PriceOffGrid {
        /// Off-grid fixed-point price.
        micros: u32,
    },

    /// Order-book levels are not sorted in canonical order.
    #[error("Invalid {book} order book: levels are not sorted in canonical order")]
    UnsortedOrderBook {
        /// Book being validated.
        book: &'static str,
    },

    /// Best bid exceeds best ask in a canonical order book.
    #[error("Invalid {book} order book: best bid {bid_micros} exceeds best ask {ask_micros}")]
    CrossedOrderBook {
        /// Book being validated.
        book: &'static str,
        /// Best bid price in micros.
        bid_micros: u32,
        /// Best ask price in micros.
        ask_micros: u32,
    },
}

impl PredictionError {
    pub(crate) const fn invalid_identifier(kind: &'static str, value: String) -> Self {
        Self::InvalidIdentifier { kind, value }
    }

    pub(crate) const fn modeled_metadata_code(kind: &'static str, value: String) -> Self {
        Self::ModeledMetadataCode { kind, value }
    }

    pub(crate) fn invalid_fixed_point_decimal(
        kind: &'static str,
        value: &str,
        reason: &'static str,
    ) -> Self {
        Self::InvalidFixedPointDecimal {
            kind,
            value: value.to_owned(),
            reason,
        }
    }
}
