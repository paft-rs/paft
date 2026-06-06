//! Prediction event and market metadata models.

use crate::error::{
    BinaryMarketOutcomeMismatch, MultiOutcomeMarketOutcomeMismatch, PredictionError,
};
use crate::identifiers::{PredictionOutcomeId, validate_opaque_identifier};
use crate::instrument::{
    BinaryMarketKey, BinaryOutcomeInstruments, OutcomeInstrument, PredictionEventKey,
    PredictionMarketKey, PredictionSeriesKey,
};
use crate::price::{NonZeroContractQuantity, NonZeroOutcomePayout, OutcomePayout, PriceGrid};
use chrono::{DateTime, Utc};
use paft_decimal::Decimal;
use paft_money::Currency;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;
use std::{collections::HashSet, fmt, str::FromStr};

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
    /// Binary-settlement code not modeled by [`BinarySettlement`].
    pub struct OtherBinaryResolution;
    kind = "binary settlement code";
    modeled_by = is_modeled_binary_settlement_code;
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

fn is_modeled_binary_settlement_code(input: &str) -> bool {
    is_modeled_code(input, &["yes", "no", "payout_vector", "void"])
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
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
pub enum EventStructure {
    /// Event is intended to group exactly one market.
    SingleMarket,
    /// Markets are related only by provider grouping/context.
    IndependentClaims,
    /// Markets are intended to represent mutually exclusive outcomes.
    MutuallyExclusive {
        /// Whether the outcome set is exhaustive.
        exhaustive: bool,
    },
    /// Binary markets are intended to represent ordered numeric buckets.
    OrderedBuckets {
        /// Whether the bucket set is exhaustive.
        exhaustive: bool,
    },
    /// Markets are intended to be linked by a named binary-claim relation.
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

/// Explicit settlement payouts for a binary market.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPayoutVector {
    /// Resolved payout for a YES outcome instrument.
    pub yes: OutcomePayout,
    /// Resolved payout for a NO outcome instrument.
    pub no: OutcomePayout,
}

impl BinaryPayoutVector {
    /// Build an explicit binary settlement payout vector.
    #[must_use]
    pub const fn new(yes: OutcomePayout, no: OutcomePayout) -> Self {
        Self { yes, no }
    }
}

/// Final binary market settlement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
#[non_exhaustive]
pub enum BinarySettlement {
    /// YES pays the market's full winning payout; NO pays zero.
    Yes,
    /// NO pays the market's full winning payout; YES pays zero.
    No,
    /// Explicit payout vector, including partial, cancelled, or scalar-like settlement.
    PayoutVector(BinaryPayoutVector),
    /// Market voided/cancelled without a simple payout vector.
    Void,
    /// Provider-specific binary settlement result.
    Other(OtherBinaryResolution),
}

impl BinarySettlement {
    /// Derive YES/NO payouts for settlements that define a payout vector.
    #[must_use]
    pub const fn resolved_payouts(
        &self,
        winning_payout: NonZeroOutcomePayout,
    ) -> Option<BinaryPayoutVector> {
        match self {
            Self::Yes => Some(BinaryPayoutVector {
                yes: winning_payout.to_payout(),
                no: OutcomePayout::ZERO,
            }),
            Self::No => Some(BinaryPayoutVector {
                yes: OutcomePayout::ZERO,
                no: winning_payout.to_payout(),
            }),
            Self::PayoutVector(vector) => Some(*vector),
            Self::Void | Self::Other(_) => None,
        }
    }
}

/// Description of the claim represented by a prediction market.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
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
    lower: NumericBound,
    /// Upper interval bound.
    upper: NumericBound,
    /// Optional unit for the numeric value, such as `USD`.
    unit: Option<String>,
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

    /// Return the lower interval bound.
    #[must_use]
    pub const fn lower(&self) -> &NumericBound {
        &self.lower
    }

    /// Return the upper interval bound.
    #[must_use]
    pub const fn upper(&self) -> &NumericBound {
        &self.upper
    }

    /// Return the optional unit for the numeric value, such as `USD`.
    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    /// Consume this range and return its validated parts.
    #[must_use]
    pub fn into_parts(self) -> (NumericBound, NumericBound, Option<String>) {
        (self.lower, self.upper, self.unit)
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
        #[serde(deny_unknown_fields)]
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
#[serde(bound(deserialize = "E: Default + Deserialize<'de>, M: Default + Deserialize<'de>"))]
pub struct GenericPredictionEvent<E = (), M = ()> {
    /// Venue-namespaced event key.
    pub key: PredictionEventKey,
    /// Main event title.
    pub title: String,
    /// Optional event subtitle/secondary text.
    pub subtitle: Option<String>,
    /// Optional provider/category topic.
    pub category: Option<String>,
    /// Optional venue-namespaced recurring-series/group key.
    pub series_key: Option<PredictionSeriesKey>,
    /// Relationship metadata for the event's markets.
    ///
    /// This is not enforced by construction because events are often built
    /// before their market list is populated. Call [`Self::validate_structure`]
    /// after filling `markets` to check currently modeled cardinality rules.
    pub structure: EventStructure,
    /// Markets grouped under this event.
    pub markets: Vec<GenericPredictionMarket<M>>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: E,
}

impl<E: Default, M> GenericPredictionEvent<E, M> {
    /// Build an empty prediction event with the given key, title, and structure.
    ///
    /// The event starts with no markets even when `structure` implies a market
    /// cardinality. Populate `markets`, then call [`Self::validate_structure`]
    /// when the event is expected to be semantically complete.
    #[must_use]
    pub fn new(key: PredictionEventKey, title: String, structure: EventStructure) -> Self {
        Self {
            key,
            title,
            subtitle: None,
            category: None,
            series_key: None,
            structure,
            markets: Vec::new(),
            provider: E::default(),
        }
    }
}

impl<E, M> GenericPredictionEvent<E, M> {
    /// Validate that the event's market count matches modeled structure rules.
    ///
    /// This intentionally validates only structure-level cardinality. It does
    /// not try to prove provider-specific semantics, such as whether an
    /// ordered-bucket event's binary claims all carry numeric range descriptors.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidEventStructure`] when the event's
    /// market count is inconsistent with its [`EventStructure`].
    pub fn validate_structure(&self) -> Result<(), PredictionError> {
        validate_event_structure(&self.structure, self.markets.len())
    }
}

fn validate_event_structure(
    structure: &EventStructure,
    market_count: usize,
) -> Result<(), PredictionError> {
    let invalid = |structure, reason| PredictionError::InvalidEventStructure {
        structure,
        market_count,
        reason,
    };

    match structure {
        EventStructure::SingleMarket if market_count != 1 => Err(invalid(
            "single_market",
            "expected exactly one contained market",
        )),
        EventStructure::MutuallyExclusive { .. } if market_count < 2 => Err(invalid(
            "mutually_exclusive",
            "expected at least two contained markets",
        )),
        EventStructure::OrderedBuckets { .. } if market_count < 2 => Err(invalid(
            "ordered_buckets",
            "expected at least two contained markets",
        )),
        EventStructure::LinkedBinaryClaims { .. } if market_count < 2 => Err(invalid(
            "linked_binary_claims",
            "expected at least two contained markets",
        )),
        EventStructure::SingleMarket
        | EventStructure::IndependentClaims
        | EventStructure::MutuallyExclusive { .. }
        | EventStructure::OrderedBuckets { .. }
        | EventStructure::LinkedBinaryClaims { .. }
        | EventStructure::Composite
        | EventStructure::Other { .. } => Ok(()),
    }
}

/// Standard prediction event with no provider metadata.
pub type PredictionEvent = GenericPredictionEvent<(), ()>;

/// Prediction market shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "market", rename_all = "snake_case")]
#[serde(bound(deserialize = "M: Default + Deserialize<'de>"))]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GenericBinaryMarket<M = ()> {
    /// Venue-namespaced binary market key.
    key: BinaryMarketKey,
    /// Required tradable YES and NO outcome instruments for this market.
    outcomes: BinaryOutcomeInstruments,
    /// Venue-namespaced event/group key when this market belongs to an event.
    pub event_key: Option<PredictionEventKey>,
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
    /// Non-zero payout per winning contract/share, denominated by `collateral_currency`.
    pub winning_payout: NonZeroOutcomePayout,
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
    /// Binary settlement result, if known.
    pub settlement: Option<BinarySettlement>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericBinaryMarket<M> {
    /// Build a binary market with the minimum required metadata.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::MismatchedBinaryMarketOutcomes`] when the
    /// outcome instruments do not belong to `key`.
    pub fn new(
        key: BinaryMarketKey,
        outcomes: BinaryOutcomeInstruments,
        title: String,
        claim: ClaimDescriptor,
        status: PredictionMarketStatus,
        collateral_currency: Currency,
        winning_payout: NonZeroOutcomePayout,
    ) -> Result<Self, PredictionError> {
        validate_binary_market_outcomes(&key, &outcomes)?;

        Ok(Self {
            key,
            outcomes,
            event_key: None,
            title,
            yes_label: None,
            no_label: None,
            claim,
            status,
            collateral_currency,
            winning_payout,
            price_grid: None,
            min_order_quantity: None,
            open_time: None,
            close_time: None,
            settlement_time: None,
            settlement: None,
            provider: M::default(),
        })
    }
}

impl<'de, M> Deserialize<'de> for GenericBinaryMarket<M>
where
    M: Default + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct GenericBinaryMarketShadow<M> {
            key: BinaryMarketKey,
            outcomes: BinaryOutcomeInstruments,
            event_key: Option<PredictionEventKey>,
            title: String,
            yes_label: Option<String>,
            no_label: Option<String>,
            claim: ClaimDescriptor,
            status: PredictionMarketStatus,
            collateral_currency: Currency,
            winning_payout: NonZeroOutcomePayout,
            price_grid: Option<PriceGrid>,
            min_order_quantity: Option<NonZeroContractQuantity>,
            #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
            open_time: Option<DateTime<Utc>>,
            #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
            close_time: Option<DateTime<Utc>>,
            #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
            settlement_time: Option<DateTime<Utc>>,
            settlement: Option<BinarySettlement>,
            #[serde(flatten, default = "Default::default")]
            provider: M,
        }

        let shadow = GenericBinaryMarketShadow::deserialize(deserializer)?;
        validate_binary_market_outcomes(&shadow.key, &shadow.outcomes)
            .map_err(de::Error::custom)?;

        Ok(Self {
            key: shadow.key,
            outcomes: shadow.outcomes,
            event_key: shadow.event_key,
            title: shadow.title,
            yes_label: shadow.yes_label,
            no_label: shadow.no_label,
            claim: shadow.claim,
            status: shadow.status,
            collateral_currency: shadow.collateral_currency,
            winning_payout: shadow.winning_payout,
            price_grid: shadow.price_grid,
            min_order_quantity: shadow.min_order_quantity,
            open_time: shadow.open_time,
            close_time: shadow.close_time,
            settlement_time: shadow.settlement_time,
            settlement: shadow.settlement,
            provider: shadow.provider,
        })
    }
}

impl<M> GenericBinaryMarket<M> {
    /// Return the venue-namespaced binary market key.
    #[must_use]
    pub const fn key(&self) -> &BinaryMarketKey {
        &self.key
    }

    /// Return the required tradable YES and NO outcome instruments.
    #[must_use]
    pub const fn outcomes(&self) -> &BinaryOutcomeInstruments {
        &self.outcomes
    }

    /// Derive resolved YES/NO payouts from this market's settlement, when possible.
    #[must_use]
    pub fn resolved_payouts(&self) -> Option<BinaryPayoutVector> {
        self.settlement
            .as_ref()?
            .resolved_payouts(self.winning_payout)
    }
}

fn validate_binary_market_outcomes(
    key: &BinaryMarketKey,
    outcomes: &BinaryOutcomeInstruments,
) -> Result<(), PredictionError> {
    let yes = outcomes.yes();
    let no = outcomes.no();
    if yes.venue == key.venue
        && no.venue == key.venue
        && yes.market_id == key.market_id
        && no.market_id == key.market_id
    {
        return Ok(());
    }

    Err(PredictionError::MismatchedBinaryMarketOutcomes(Box::new(
        BinaryMarketOutcomeMismatch {
            key_venue: key.venue.to_string(),
            key_market_id: key.market_id.to_string(),
            yes_venue: yes.venue.to_string(),
            yes_market_id: yes.market_id.to_string(),
            no_venue: no.venue.to_string(),
            no_market_id: no.market_id.to_string(),
        },
    )))
}

/// Standard binary market with no provider metadata.
pub type BinaryMarket = GenericBinaryMarket<()>;

/// Native multi-answer market metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GenericMultiOutcomeMarket<M = ()> {
    /// Venue-namespaced market key.
    key: PredictionMarketKey,
    /// Venue-namespaced event/group key when this market belongs to an event.
    pub event_key: Option<PredictionEventKey>,
    /// Market title.
    pub title: String,
    /// Outcomes available in the market.
    outcomes: Vec<OutcomeDescriptor>,
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
    resolution: Option<PredictionOutcomeId>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericMultiOutcomeMarket<M> {
    /// Build a native multi-outcome market with the minimum required metadata.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] when fewer than two outcomes are supplied,
    /// an outcome instrument does not belong to `key`, or outcome ids repeat.
    pub fn new(
        key: PredictionMarketKey,
        title: String,
        outcomes: Vec<OutcomeDescriptor>,
        status: PredictionMarketStatus,
        collateral_currency: Currency,
        unit_payout: OutcomePayout,
    ) -> Result<Self, PredictionError> {
        validate_multi_outcome_market(&key, &outcomes, None)?;

        Ok(Self {
            key,
            event_key: None,
            title,
            outcomes,
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
        })
    }
}

impl<M> GenericMultiOutcomeMarket<M> {
    /// Return the venue-namespaced market key.
    #[must_use]
    pub const fn key(&self) -> &PredictionMarketKey {
        &self.key
    }

    /// Return the outcome descriptors in provider order.
    #[must_use]
    pub fn outcomes(&self) -> &[OutcomeDescriptor] {
        &self.outcomes
    }

    /// Return the winning outcome id, if resolved.
    #[must_use]
    pub const fn resolution(&self) -> Option<&PredictionOutcomeId> {
        self.resolution.as_ref()
    }

    /// Set or clear the winning outcome id.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidMultiOutcomeMarketResolution`] when
    /// `resolution` is not one of the listed outcome ids.
    pub fn set_resolution(
        &mut self,
        resolution: Option<PredictionOutcomeId>,
    ) -> Result<(), PredictionError> {
        validate_multi_outcome_market_resolution(&self.key, &self.outcomes, resolution.as_ref())?;
        self.resolution = resolution;
        Ok(())
    }
}

impl<'de, M> Deserialize<'de> for GenericMultiOutcomeMarket<M>
where
    M: Default + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct GenericMultiOutcomeMarketShadow<M> {
            key: PredictionMarketKey,
            event_key: Option<PredictionEventKey>,
            title: String,
            outcomes: Vec<OutcomeDescriptor>,
            status: PredictionMarketStatus,
            collateral_currency: Currency,
            unit_payout: OutcomePayout,
            price_grid: Option<PriceGrid>,
            min_order_quantity: Option<NonZeroContractQuantity>,
            #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
            open_time: Option<DateTime<Utc>>,
            #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
            close_time: Option<DateTime<Utc>>,
            #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
            settlement_time: Option<DateTime<Utc>>,
            resolution: Option<PredictionOutcomeId>,
            #[serde(flatten, default = "Default::default")]
            provider: M,
        }

        let shadow = GenericMultiOutcomeMarketShadow::deserialize(deserializer)?;
        validate_multi_outcome_market(&shadow.key, &shadow.outcomes, shadow.resolution.as_ref())
            .map_err(de::Error::custom)?;

        Ok(Self {
            key: shadow.key,
            event_key: shadow.event_key,
            title: shadow.title,
            outcomes: shadow.outcomes,
            status: shadow.status,
            collateral_currency: shadow.collateral_currency,
            unit_payout: shadow.unit_payout,
            price_grid: shadow.price_grid,
            min_order_quantity: shadow.min_order_quantity,
            open_time: shadow.open_time,
            close_time: shadow.close_time,
            settlement_time: shadow.settlement_time,
            resolution: shadow.resolution,
            provider: shadow.provider,
        })
    }
}

fn validate_multi_outcome_market(
    key: &PredictionMarketKey,
    outcomes: &[OutcomeDescriptor],
    resolution: Option<&PredictionOutcomeId>,
) -> Result<(), PredictionError> {
    if outcomes.len() < 2 {
        return Err(PredictionError::TooFewMultiOutcomeMarketOutcomes {
            count: outcomes.len(),
        });
    }

    let mut outcome_ids = HashSet::with_capacity(outcomes.len());
    for outcome in outcomes {
        let instrument = &outcome.instrument;
        if instrument.venue != key.venue || instrument.market_id != key.market_id {
            return Err(PredictionError::MismatchedMultiOutcomeMarketOutcome(
                Box::new(MultiOutcomeMarketOutcomeMismatch {
                    key_venue: key.venue.to_string(),
                    key_market_id: key.market_id.to_string(),
                    outcome_venue: instrument.venue.to_string(),
                    outcome_market_id: instrument.market_id.to_string(),
                    outcome_id: instrument.outcome_id.to_string(),
                }),
            ));
        }

        if !outcome_ids.insert(&instrument.outcome_id) {
            return Err(PredictionError::DuplicateMultiOutcomeMarketOutcome {
                venue: instrument.venue.to_string(),
                market_id: instrument.market_id.to_string(),
                outcome_id: instrument.outcome_id.to_string(),
            });
        }
    }

    validate_multi_outcome_market_resolution(key, outcomes, resolution)
}

fn validate_multi_outcome_market_resolution(
    key: &PredictionMarketKey,
    outcomes: &[OutcomeDescriptor],
    resolution: Option<&PredictionOutcomeId>,
) -> Result<(), PredictionError> {
    let Some(resolution) = resolution else {
        return Ok(());
    };

    if outcomes
        .iter()
        .any(|outcome| outcome.instrument.outcome_id == *resolution)
    {
        return Ok(());
    }

    Err(PredictionError::InvalidMultiOutcomeMarketResolution {
        venue: key.venue.to_string(),
        market_id: key.market_id.to_string(),
        outcome_id: resolution.to_string(),
    })
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
    /// Venue-namespaced event/group key when this market belongs to an event.
    pub event_key: Option<PredictionEventKey>,
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
