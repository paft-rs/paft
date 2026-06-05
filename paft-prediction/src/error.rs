//! Error types specific to `paft-prediction`.

use thiserror::Error;

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
