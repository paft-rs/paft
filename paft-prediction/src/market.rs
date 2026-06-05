//! Prediction event and market metadata models.

use crate::error::PredictionError;
use crate::identifiers::{
    PredictionEventId, PredictionOutcomeId, PredictionSeriesId, validate_opaque_identifier,
};
use crate::instrument::{
    BinaryMarketKey, BinaryOutcomeInstruments, OutcomeInstrument, PredictionEventKey,
    PredictionMarketKey,
};
use crate::price::{NonZeroContractQuantity, OutcomePayout, PriceGrid};
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
        modeled_by = $modeled_by:path;
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
            /// Returns [`PredictionError::ModeledMetadataCode`] when the code
            /// already names a modeled value for the owning enum.
            pub fn new(input: &str) -> Result<Self, PredictionError> {
                let value = validate_opaque_identifier($kind, input)?;
                if $modeled_by(value.as_str()) {
                    return Err(PredictionError::modeled_metadata_code($kind, input.to_string()));
                }

                Ok(Self(value))
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
    modeled_by = is_modeled_event_structure_code;
);

opaque_metadata_code!(
    /// Linked-binary relation code not modeled by [`LinkedBinaryRelation`].
    pub struct OtherLinkedBinaryRelation;
    kind = "linked binary relation code";
    modeled_by = is_modeled_linked_binary_relation_code;
);

opaque_metadata_code!(
    /// Claim descriptor code not modeled by [`ClaimDescriptor`].
    pub struct OtherClaimDescriptor;
    kind = "claim descriptor code";
    modeled_by = is_modeled_claim_descriptor_code;
);

opaque_metadata_code!(
    /// Market-status code not modeled by [`PredictionMarketStatus`].
    pub struct OtherPredictionMarketStatus;
    kind = "prediction market status code";
    modeled_by = is_modeled_prediction_market_status_code;
);

opaque_metadata_code!(
    /// Binary-resolution code not modeled by [`BinaryResolution`].
    pub struct OtherBinaryResolution;
    kind = "binary resolution code";
    modeled_by = is_modeled_binary_resolution_code;
);

fn is_modeled_code(input: &str, modeled: &[&str]) -> bool {
    modeled.iter().any(|code| input.eq_ignore_ascii_case(code))
}

fn is_modeled_event_structure_code(input: &str) -> bool {
    is_modeled_code(
        input,
        &[
            "single_market",
            "independent_claims",
            "mutually_exclusive",
            "ordered_buckets",
            "linked_binary_claims",
            "composite",
        ],
    )
}

fn is_modeled_linked_binary_relation_code(input: &str) -> bool {
    is_modeled_code(
        input,
        &["sum_to_one", "negative_risk_conversion", "composite_legs"],
    )
}

fn is_modeled_claim_descriptor_code(input: &str) -> bool {
    is_modeled_code(
        input,
        &["text", "categorical", "numeric_range", "composite"],
    )
}

fn is_modeled_prediction_market_status_code(input: &str) -> bool {
    is_modeled_code(
        input,
        &[
            "upcoming",
            "open",
            "paused",
            "closed",
            "resolved",
            "cancelled",
        ],
    )
}

fn is_modeled_binary_resolution_code(input: &str) -> bool {
    is_modeled_code(input, &["yes", "no", "void"])
}

macro_rules! open_string_metadata_enum {
    (
        $name:ident, $other:ident, $kind:literal;
        {
            $($code:literal => $variant:ident),+ $(,)?
        }
    ) => {
        impl $name {
            /// Returns the stable string code for this enum value.
            #[must_use]
            pub fn code(&self) -> &str {
                match self {
                    $(Self::$variant => $code,)+
                    Self::Other(value) => value.as_str(),
                }
            }

            /// Parses a string code, preserving unknown values in `Other`.
            ///
            /// # Errors
            ///
            /// Returns [`PredictionError::InvalidIdentifier`] when the code is
            /// empty, too long, or contains whitespace/control characters.
            pub fn try_from_str(input: &str) -> Result<Self, PredictionError> {
                let value = validate_opaque_identifier($kind, input)?;
                $(if value.as_str().eq_ignore_ascii_case($code) {
                    return Ok(Self::$variant);
                })+

                $other::new(value.as_str()).map(Self::Other)
            }
        }

        impl FromStr for $name {
            type Err = PredictionError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                Self::try_from_str(input)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.code())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.code())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = String::deserialize(deserializer)?;
                Self::try_from_str(&raw).map_err(de::Error::custom)
            }
        }
    };
}

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

open_string_metadata_enum!(
    LinkedBinaryRelation, OtherLinkedBinaryRelation, "linked binary relation code";
    {
        "sum_to_one" => SumToOne,
        "negative_risk_conversion" => NegativeRiskConversion,
        "composite_legs" => CompositeLegs,
    }
);

/// Provider-agnostic status of a prediction market.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

open_string_metadata_enum!(
    PredictionMarketStatus, OtherPredictionMarketStatus, "prediction market status code";
    {
        "upcoming" => Upcoming,
        "open" => Open,
        "paused" => Paused,
        "closed" => Closed,
        "resolved" => Resolved,
        "cancelled" => Cancelled,
    }
);

/// Final binary market resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

open_string_metadata_enum!(
    BinaryResolution, OtherBinaryResolution, "binary resolution code";
    {
        "yes" => Yes,
        "no" => No,
        "void" => Void,
    }
);

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct NumericRange {
    /// Lower interval bound.
    pub lower: NumericBound,
    /// Upper interval bound.
    pub upper: NumericBound,
    /// Optional unit for the numeric value, such as `USD`.
    pub unit: Option<String>,
}

impl NumericRange {
    /// Construct a numeric range and validate its interval.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidNumericRange`] when finite bounds are
    /// descending or when equal finite endpoints do not include the single
    /// endpoint value.
    pub fn new(
        lower: NumericBound,
        upper: NumericBound,
        unit: Option<String>,
    ) -> Result<Self, PredictionError> {
        let range = Self { lower, upper, unit };
        range.validate()?;
        Ok(range)
    }

    /// Validate the range interval.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidNumericRange`] when finite bounds are
    /// descending or when equal finite endpoints do not include the single
    /// endpoint value.
    pub fn validate(&self) -> Result<(), PredictionError> {
        let Some((lower, lower_included)) = self.lower.finite_value() else {
            return Ok(());
        };
        let Some((upper, upper_included)) = self.upper.finite_value() else {
            return Ok(());
        };

        if lower > upper {
            return Err(PredictionError::InvalidNumericRange {
                reason: "lower bound must be less than or equal to upper bound",
            });
        }

        if lower == upper && !(lower_included && upper_included) {
            return Err(PredictionError::InvalidNumericRange {
                reason: "zero-width finite ranges must include both endpoints",
            });
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for NumericRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct NumericRangeShadow {
            lower: NumericBound,
            upper: NumericBound,
            unit: Option<String>,
        }

        let shadow = NumericRangeShadow::deserialize(deserializer)?;
        Self::new(shadow.lower, shadow.upper, shadow.unit).map_err(de::Error::custom)
    }
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

impl NumericBound {
    const fn finite_value(&self) -> Option<(&Decimal, bool)> {
        match self {
            Self::Included(value) => Some((value, true)),
            Self::Excluded(value) => Some((value, false)),
            Self::Unbounded => None,
        }
    }
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
// Keep market variants inline so construction and pattern matching stay direct;
// this is metadata, not a high-cardinality in-memory book representation.
#[allow(clippy::large_enum_variant)]
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
    /// Required tradable YES and NO outcome instruments for this market.
    pub outcomes: BinaryOutcomeInstruments,
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
    /// Non-zero minimum accepted order quantity, if known.
    pub min_order_quantity: Option<NonZeroContractQuantity>,
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
        outcomes: BinaryOutcomeInstruments,
        title: String,
        claim: ClaimDescriptor,
        status: PredictionMarketStatus,
        collateral_currency: Currency,
        unit_payout: OutcomePayout,
    ) -> Self {
        Self {
            key,
            outcomes,
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
    /// Non-zero minimum accepted order quantity, if known.
    pub min_order_quantity: Option<NonZeroContractQuantity>,
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
