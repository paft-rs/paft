//! Prediction event and market metadata models.

use crate::error::PredictionError;
use crate::identifiers::{
    PredictionEventId, PredictionOutcomeId, PredictionSeriesId, validate_opaque_identifier,
};
use crate::instrument::{
    BinaryMarketKey, OutcomeInstrument, PredictionEventKey, PredictionMarketKey,
};
use crate::price::{ContractQuantity, OutcomePayout, PriceGrid};
use chrono::{DateTime, Utc};
use paft_decimal::Decimal;
use paft_money::Currency;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;
use std::{fmt, str::FromStr};

macro_rules! opaque_metadata_code {
    (
        $(#[$meta:meta])*
        pub struct $name:ident;
        kind = $kind:literal;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(SmolStr);

        impl $name {
            /// Construct a new provider-specific metadata code.
            ///
            /// # Errors
            ///
            /// Returns [`PredictionError::InvalidIdentifier`] when the trimmed
            /// code is empty, too long, or contains whitespace/control characters.
            pub fn new(input: &str) -> Result<Self, PredictionError> {
                validate_opaque_identifier($kind, input).map(Self)
            }

            /// Returns the preserved provider metadata code.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl FromStr for $name {
            type Err = PredictionError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                Self::new(input)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = String::deserialize(deserializer)?;
                Self::new(&raw).map_err(de::Error::custom)
            }
        }
    };
}

opaque_metadata_code!(
    /// Event-structure code not modeled by [`EventStructure`].
    pub struct OtherEventStructure;
    kind = "event structure code";
);

opaque_metadata_code!(
    /// Linked-binary relation code not modeled by [`LinkedBinaryRelation`].
    pub struct OtherLinkedBinaryRelation;
    kind = "linked binary relation code";
);

opaque_metadata_code!(
    /// Claim descriptor code not modeled by [`ClaimDescriptor`].
    pub struct OtherClaimDescriptor;
    kind = "claim descriptor code";
);

opaque_metadata_code!(
    /// Market-status code not modeled by [`PredictionMarketStatus`].
    pub struct OtherPredictionMarketStatus;
    kind = "prediction market status code";
);

opaque_metadata_code!(
    /// Binary-resolution code not modeled by [`BinaryResolution`].
    pub struct OtherBinaryResolution;
    kind = "binary resolution code";
);

/// High-level relationship among markets inside an event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum EventStructure {
    /// Event contains exactly one market.
    SingleMarket,
    /// Markets are related only by provider grouping/context.
    IndependentClaims,
    /// Markets represent mutually exclusive outcomes.
    MutuallyExclusive {
        /// Whether the outcome set is exhaustive.
        exhaustive: bool,
    },
    /// Binary markets represent ordered numeric buckets.
    OrderedBuckets {
        /// Whether the bucket set is exhaustive.
        exhaustive: bool,
    },
    /// Markets are linked by a named binary-claim relation.
    LinkedBinaryClaims {
        /// The relation that links the binary claims.
        relation: LinkedBinaryRelation,
    },
    /// Event has composite/conditional structure not captured by simpler variants.
    Composite,
    /// Provider-specific event relation.
    Other {
        /// Preserved provider event-structure code.
        value: OtherEventStructure,
    },
}

/// Relationship among linked binary claims.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LinkedBinaryRelation {
    /// Linked outcomes sum to one unit of payout/probability.
    SumToOne,
    /// Provider supports negative-risk conversion among binary claims.
    NegativeRiskConversion,
    /// Linked markets represent legs of a composite claim.
    CompositeLegs,
    /// Provider-specific linked-claim relation.
    Other(OtherLinkedBinaryRelation),
}

/// Provider-agnostic status of a prediction market.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PredictionMarketStatus {
    /// Market is listed but not yet open.
    Upcoming,
    /// Market is accepting orders.
    Open,
    /// Market is temporarily paused.
    Paused,
    /// Market is closed to trading.
    Closed,
    /// Market has resolved.
    Resolved,
    /// Market was cancelled/voided by the venue.
    Cancelled,
    /// Provider-specific status.
    Other(OtherPredictionMarketStatus),
}

/// Final binary market resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BinaryResolution {
    /// YES won.
    Yes,
    /// NO won.
    No,
    /// Market was voided/cancelled and did not resolve yes/no.
    Void,
    /// Provider-specific binary resolution.
    Other(OtherBinaryResolution),
}

/// Description of the claim represented by a prediction market.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ClaimDescriptor {
    /// Free-form textual claim.
    Text {
        /// Claim description or resolution condition.
        description: String,
    },
    /// Categorical label supplied by the adapter/provider.
    Categorical {
        /// Category or answer label.
        label: String,
    },
    /// Numeric interval claim.
    NumericRange {
        /// Parsed numeric interval.
        range: NumericRange,
        /// Optional display label for the interval.
        label: Option<String>,
    },
    /// Composite claim that should be interpreted with provider metadata.
    Composite,
    /// Provider-specific claim descriptor.
    Other {
        /// Preserved provider claim-descriptor code.
        value: OtherClaimDescriptor,
    },
}

/// Parsed numeric interval for a range/bucket market.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NumericRange {
    /// Lower interval bound.
    pub lower: NumericBound,
    /// Upper interval bound.
    pub upper: NumericBound,
    /// Optional unit for the numeric value, such as `USD`.
    pub unit: Option<String>,
}

/// Inclusive, exclusive, or unbounded numeric range endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NumericBound {
    /// Inclusive endpoint.
    Included(#[serde(with = "paft_decimal::serde::canonical_str")] Decimal),
    /// Exclusive endpoint.
    Excluded(#[serde(with = "paft_decimal::serde::canonical_str")] Decimal),
    /// Unbounded endpoint.
    Unbounded,
}

/// Grouping/container for related prediction markets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericPredictionEvent<E = (), M = ()> {
    /// Venue-namespaced event key.
    pub key: PredictionEventKey,
    /// Main event title.
    pub title: String,
    /// Optional event subtitle/secondary text.
    pub subtitle: Option<String>,
    /// Optional provider/category topic.
    pub category: Option<String>,
    /// Optional recurring-series/group identifier.
    pub series_id: Option<PredictionSeriesId>,
    /// Relationship among the event's markets.
    pub structure: EventStructure,
    /// Markets grouped under this event.
    pub markets: Vec<GenericPredictionMarket<M>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: E,
}

impl<E: Default, M> GenericPredictionEvent<E, M> {
    /// Build an empty prediction event with the given key, title, and structure.
    #[must_use]
    pub fn new(key: PredictionEventKey, title: String, structure: EventStructure) -> Self {
        Self {
            key,
            title,
            subtitle: None,
            category: None,
            series_id: None,
            structure,
            markets: Vec::new(),
            provider: E::default(),
        }
    }
}

/// Standard prediction event with no provider metadata.
pub type PredictionEvent = GenericPredictionEvent<(), ()>;

/// Prediction market shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "market", rename_all = "snake_case")]
#[non_exhaustive]
pub enum GenericPredictionMarket<M = ()> {
    /// Atomic yes/no claim.
    Binary(GenericBinaryMarket<M>),
    /// Native multi-answer market.
    MultiOutcome(GenericMultiOutcomeMarket<M>),
    /// Scalar/numeric market.
    Scalar(GenericScalarMarket<M>),
}

/// Standard prediction market enum with no provider metadata.
pub type PredictionMarket = GenericPredictionMarket<()>;

/// Atomic yes/no prediction market metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericBinaryMarket<M = ()> {
    /// Venue-namespaced binary market key.
    pub key: BinaryMarketKey,
    /// Provider-native event/group id when this market belongs to an event.
    pub event_id: Option<PredictionEventId>,
    /// Market title.
    pub title: String,
    /// Optional label for the YES outcome.
    pub yes_label: Option<String>,
    /// Optional label for the NO outcome.
    pub no_label: Option<String>,
    /// Claim descriptor, optionally structured by an adapter.
    pub claim: ClaimDescriptor,
    /// Market lifecycle status.
    pub status: PredictionMarketStatus,
    /// Currency used for collateral and settlement.
    pub collateral_currency: Currency,
    /// Payout per winning contract/share, denominated by `collateral_currency`.
    pub unit_payout: OutcomePayout,
    /// Market-specific price grid, if known.
    pub price_grid: Option<PriceGrid>,
    /// Minimum accepted order quantity, if known.
    pub min_order_quantity: Option<ContractQuantity>,
    /// Time when the market opens to trading.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub open_time: Option<DateTime<Utc>>,
    /// Time when the market closes to trading.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub close_time: Option<DateTime<Utc>>,
    /// Time when the market settled/resolved.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub settlement_time: Option<DateTime<Utc>>,
    /// Binary resolution, if known.
    pub resolution: Option<BinaryResolution>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericBinaryMarket<M> {
    /// Build a binary market with the minimum required metadata.
    #[must_use]
    pub fn new(
        key: BinaryMarketKey,
        title: String,
        claim: ClaimDescriptor,
        status: PredictionMarketStatus,
        collateral_currency: Currency,
        unit_payout: OutcomePayout,
    ) -> Self {
        Self {
            key,
            event_id: None,
            title,
            yes_label: None,
            no_label: None,
            claim,
            status,
            collateral_currency,
            unit_payout,
            price_grid: None,
            min_order_quantity: None,
            open_time: None,
            close_time: None,
            settlement_time: None,
            resolution: None,
            provider: M::default(),
        }
    }
}

/// Standard binary market with no provider metadata.
pub type BinaryMarket = GenericBinaryMarket<()>;

/// Native multi-answer market metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericMultiOutcomeMarket<M = ()> {
    /// Venue-namespaced market key.
    pub key: PredictionMarketKey,
    /// Provider-native event/group id when this market belongs to an event.
    pub event_id: Option<PredictionEventId>,
    /// Market title.
    pub title: String,
    /// Outcomes available in the market.
    pub outcomes: Vec<OutcomeDescriptor>,
    /// Market lifecycle status.
    pub status: PredictionMarketStatus,
    /// Currency used for collateral and settlement.
    pub collateral_currency: Currency,
    /// Payout per winning contract/share, denominated by `collateral_currency`.
    pub unit_payout: OutcomePayout,
    /// Market-specific price grid, if known.
    pub price_grid: Option<PriceGrid>,
    /// Minimum accepted order quantity, if known.
    pub min_order_quantity: Option<ContractQuantity>,
    /// Time when the market opens to trading.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub open_time: Option<DateTime<Utc>>,
    /// Time when the market closes to trading.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub close_time: Option<DateTime<Utc>>,
    /// Time when the market settled/resolved.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub settlement_time: Option<DateTime<Utc>>,
    /// Winning outcome id, if resolved to one outcome.
    pub resolution: Option<PredictionOutcomeId>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard multi-outcome market with no provider metadata.
pub type MultiOutcomeMarket = GenericMultiOutcomeMarket<()>;

/// Outcome metadata inside a native multi-answer market.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutcomeDescriptor {
    /// Tradable outcome instrument.
    pub instrument: OutcomeInstrument,
    /// Human-readable outcome label.
    pub label: String,
}

/// Scalar/numeric prediction market metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericScalarMarket<M = ()> {
    /// Venue-namespaced market key.
    pub key: PredictionMarketKey,
    /// Provider-native event/group id when this market belongs to an event.
    pub event_id: Option<PredictionEventId>,
    /// Market title.
    pub title: String,
    /// Optional unit for the resolved scalar value.
    pub unit: Option<String>,
    /// Market lifecycle status.
    pub status: PredictionMarketStatus,
    /// Time when the market opens to trading.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub open_time: Option<DateTime<Utc>>,
    /// Time when the market closes to trading.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub close_time: Option<DateTime<Utc>>,
    /// Time when the market settled/resolved.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub settlement_time: Option<DateTime<Utc>>,
    /// Resolved scalar value, if known.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub resolution_value: Option<Decimal>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

/// Standard scalar market with no provider metadata.
pub type ScalarMarket = GenericScalarMarket<()>;
