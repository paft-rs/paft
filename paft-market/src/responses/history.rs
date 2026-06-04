//! History response types.

use std::num::NonZeroU16;

use paft_money::{Currency, PriceAmount, QuantityAmount};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::requests::history::Interval;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_domain::Instrument;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Open, high, low, and close price amounts for one denominated bar.
pub struct Ohlc {
    /// Open price.
    pub open: PriceAmount,
    /// High price.
    pub high: PriceAmount,
    /// Low price.
    pub low: PriceAmount,
    /// Close price in the history response's primary OHLC price basis.
    pub close: PriceAmount,
}

impl Ohlc {
    /// Build an OHLC price vector.
    #[must_use]
    pub const fn new(
        open: PriceAmount,
        high: PriceAmount,
        low: PriceAmount,
        close: PriceAmount,
    ) -> Self {
        Self {
            open,
            high,
            low,
            close,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// A single OHLCV bar at timestamp `ts` (Unix milliseconds).
///
/// Volume may be `None` when unavailable.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`Candle`] alias for the standard
/// shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericCandle<M = ()> {
    /// Timestamp for the bar as Unix milliseconds.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub ts: DateTime<Utc>,
    /// Currency shared by every price amount in this candle.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub currency: Currency,
    /// Primary OHLC price amounts.
    #[serde(flatten)]
    #[cfg_attr(feature = "dataframe", df_derive(flatten))]
    pub ohlc: Ohlc,
    /// Raw provider close price, if available.
    ///
    /// This field is separate from the primary OHLC fields and may therefore
    /// have a different basis from [`GenericHistoryResponse::price_basis`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_unadj: Option<PriceAmount>,
    /// Volume if available, in the provider's stated quantity unit.
    pub volume: Option<QuantityAmount>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericCandle<M> {
    /// Build a candle from the OHLC quadruple at the given timestamp.
    /// `close_unadj` and `volume` default to `None`; `provider` is initialised
    /// via `M::default()`.
    #[must_use]
    pub fn new(ts: DateTime<Utc>, currency: Currency, ohlc: Ohlc) -> Self {
        Self {
            ts,
            currency,
            ohlc,
            close_unadj: None,
            volume: None,
            provider: M::default(),
        }
    }
}

/// Standard `Candle` with no extra provider metadata.
pub type Candle = GenericCandle<()>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Streaming candle update event.
///
/// Generic over an update-level provider metadata payload `U`, which is
/// flattened into the serialized representation, and a candle-level metadata
/// payload `C`. Use the [`CandleUpdate`] alias for the standard shape
/// (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericCandleUpdate<U = (), C = ()> {
    /// Instrument identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Interval represented by the candle.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub interval: Interval,
    /// The candle payload for the interval.
    pub candle: GenericCandle<C>,
    /// true when the bar is closed/final as per the upstream provider WebSocket
    pub is_final: bool,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: U,
}

impl<U: Default, C> GenericCandleUpdate<U, C> {
    /// Build a candle-update event from its required parts.
    /// `provider` is initialised via `U::default()`.
    #[must_use]
    pub fn new(
        instrument: Instrument,
        interval: Interval,
        candle: GenericCandle<C>,
        is_final: bool,
    ) -> Self {
        Self {
            instrument,
            interval,
            candle,
            is_final,
            provider: U::default(),
        }
    }
}

/// Standard `CandleUpdate` with no extra provider metadata.
pub type CandleUpdate = GenericCandleUpdate<(), ()>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
/// Price basis of the primary OHLC fields in a history response.
pub enum OhlcPriceBasis {
    /// The same basis applies to open, high, low, and close.
    Uniform {
        /// Uniform OHLC price basis.
        basis: PriceBasis,
    },
    /// Different bases apply to individual OHLC fields.
    PerField {
        /// Open price basis.
        open: PriceBasis,
        /// High price basis.
        high: PriceBasis,
        /// Low price basis.
        low: PriceBasis,
        /// Close price basis.
        close: PriceBasis,
    },
}

impl OhlcPriceBasis {
    /// Build a uniform raw OHLC basis.
    #[must_use]
    pub const fn raw() -> Self {
        Self::Uniform {
            basis: PriceBasis::Raw,
        }
    }

    /// Build a uniform OHLC basis.
    #[must_use]
    pub const fn uniform(price_basis: PriceBasis) -> Self {
        Self::Uniform { basis: price_basis }
    }

    /// Build a per-field OHLC basis.
    #[must_use]
    pub const fn per_field(
        open: PriceBasis,
        high: PriceBasis,
        low: PriceBasis,
        close: PriceBasis,
    ) -> Self {
        Self::PerField {
            open,
            high,
            low,
            close,
        }
    }

    /// Return the open, high, low, and close bases in order.
    #[must_use]
    pub const fn fields(&self) -> (&PriceBasis, &PriceBasis, &PriceBasis, &PriceBasis) {
        match self {
            Self::Uniform { basis } => (basis, basis, basis, basis),
            Self::PerField {
                open,
                high,
                low,
                close,
            } => (open, high, low, close),
        }
    }
}

impl Default for OhlcPriceBasis {
    fn default() -> Self {
        Self::raw()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
/// Basis of a returned price value.
pub enum PriceBasis {
    /// Raw provider value.
    #[default]
    Raw,
    /// Price adjusted for corporate actions.
    ///
    /// This explicitly records the non-empty set of corporate-action classes
    /// included in the returned price. Use [`PriceBasis::ProviderAdjusted`]
    /// when paft cannot truthfully identify the included causes.
    CorporateActionAdjusted {
        /// Adjustment anchor used for the returned values.
        anchor: AdjustmentAnchor,
        /// Corporate-action classes included in the adjustment.
        causes: CorporateActionAdjustmentCauses,
    },
    /// Price adjusted across contract rolls.
    ContractRollAdjusted {
        /// Adjustment anchor used for the returned values.
        anchor: AdjustmentAnchor,
        /// Adjustment method used across rolls.
        method: AdjustmentMethod,
    },
    /// Opaque provider-adjusted value using provider-supplied adjustment factors.
    ///
    /// Use this when paft cannot truthfully state which adjustment causes are
    /// included.
    ProviderAdjusted {
        /// Adjustment anchor used for the returned values.
        anchor: AdjustmentAnchor,
    },
}

impl PriceBasis {
    /// Build a raw price basis.
    #[must_use]
    pub const fn raw() -> Self {
        Self::Raw
    }

    /// Build a split-adjusted basis with an explicit anchor.
    #[must_use]
    pub const fn split_adjusted(anchor: AdjustmentAnchor) -> Self {
        Self::CorporateActionAdjusted {
            anchor,
            causes: CorporateActionAdjustmentCauses::splits(),
        }
    }

    /// Build a split-adjusted basis anchored to the provider's latest basis.
    #[must_use]
    pub const fn split_adjusted_latest() -> Self {
        Self::split_adjusted(AdjustmentAnchor::ProviderLatestBasis)
    }

    /// Build a corporate-action-adjusted basis.
    #[must_use]
    pub const fn corporate_action_adjusted(
        anchor: AdjustmentAnchor,
        causes: CorporateActionAdjustmentCauses,
    ) -> Self {
        Self::CorporateActionAdjusted { anchor, causes }
    }

    /// Build an opaque provider-adjusted basis with an explicit anchor.
    #[must_use]
    pub const fn provider_adjusted(anchor: AdjustmentAnchor) -> Self {
        Self::ProviderAdjusted { anchor }
    }

    /// Build an opaque provider-adjusted basis anchored to the provider's latest basis.
    #[must_use]
    pub const fn provider_latest_adjusted() -> Self {
        Self::ProviderAdjusted {
            anchor: AdjustmentAnchor::ProviderLatestBasis,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "date",
    rename_all = "snake_case",
    deny_unknown_fields
)]
#[non_exhaustive]
/// Anchor basis used by adjusted prices.
pub enum AdjustmentAnchor {
    /// Adjusted to the provider's latest/raw-price basis.
    ///
    /// The exact anchor observation may be provider-defined.
    ProviderLatestBasis,
    /// Adjusted to the first observation in the returned series.
    FirstReturnedObservation,
    /// Adjusted to the last observation in the returned series.
    LastReturnedObservation,
    /// Adjusted to a specific calendar date.
    Date(NaiveDate),
    /// Provider-defined anchor when the exact basis is opaque.
    ProviderDefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
/// Corporate-action class included in an adjusted price.
pub enum CorporateActionAdjustmentCause {
    /// Stock split effects.
    Split,
    /// Cash dividend effects.
    Dividend,
    /// Capital-gain distribution effects.
    CapitalGain,
}

impl CorporateActionAdjustmentCause {
    const ALL: [Self; 3] = [Self::Split, Self::Dividend, Self::CapitalGain];

    /// Return the stable wire-format code for this cause.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Split => "split",
            Self::Dividend => "dividend",
            Self::CapitalGain => "capital_gain",
        }
    }

    const fn bit(self) -> u16 {
        match self {
            Self::Split => CorporateActionAdjustmentCauses::SPLIT_BIT,
            Self::Dividend => CorporateActionAdjustmentCauses::DIVIDEND_BIT,
            Self::CapitalGain => CorporateActionAdjustmentCauses::CAPITAL_GAIN_BIT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Non-empty set of corporate-action classes included in an adjusted price.
pub struct CorporateActionAdjustmentCauses(NonZeroU16);

impl CorporateActionAdjustmentCauses {
    const SPLIT_BIT: u16 = 0b0001;
    const DIVIDEND_BIT: u16 = 0b0010;
    const CAPITAL_GAIN_BIT: u16 = 0b0100;
    const KNOWN_BITS: u16 = Self::SPLIT_BIT | Self::DIVIDEND_BIT | Self::CAPITAL_GAIN_BIT;

    /// Split-only corporate-action adjustment.
    pub const SPLITS: Self = Self::from_nonzero_bits(Self::SPLIT_BIT);
    /// Dividend-only corporate-action adjustment.
    pub const DIVIDENDS: Self = Self::from_nonzero_bits(Self::DIVIDEND_BIT);
    /// Capital-gain-only corporate-action adjustment.
    pub const CAPITAL_GAINS: Self = Self::from_nonzero_bits(Self::CAPITAL_GAIN_BIT);
    /// Adjustment including every modeled corporate-action class.
    pub const ALL: Self = Self::from_nonzero_bits(Self::KNOWN_BITS);

    const fn from_nonzero_bits(bits: u16) -> Self {
        match NonZeroU16::new(bits) {
            Some(bits) => Self(bits),
            None => panic!("corporate-action adjustment causes must be non-empty"),
        }
    }

    fn from_bits(bits: u16) -> Option<Self> {
        if bits == 0 || bits & !Self::KNOWN_BITS != 0 {
            return None;
        }

        NonZeroU16::new(bits).map(Self)
    }

    /// Build a split-only adjustment cause set.
    #[must_use]
    pub const fn splits() -> Self {
        Self::SPLITS
    }

    /// Build a dividend-only adjustment cause set.
    #[must_use]
    pub const fn dividends() -> Self {
        Self::DIVIDENDS
    }

    /// Build a capital-gain-only adjustment cause set.
    #[must_use]
    pub const fn capital_gains() -> Self {
        Self::CAPITAL_GAINS
    }

    /// Build an adjustment cause set including all modeled corporate actions.
    #[must_use]
    pub const fn all() -> Self {
        Self::ALL
    }

    /// Build an adjustment cause set from an iterator.
    ///
    /// Returns `None` when the iterator is empty.
    #[must_use]
    pub fn from_causes(
        causes: impl IntoIterator<Item = CorporateActionAdjustmentCause>,
    ) -> Option<Self> {
        let mut bits = 0;
        for cause in causes {
            bits |= cause.bit();
        }

        Self::from_bits(bits)
    }

    /// Return whether this set contains `cause`.
    #[must_use]
    pub const fn contains(self, cause: CorporateActionAdjustmentCause) -> bool {
        self.0.get() & cause.bit() != 0
    }

    /// Return the union of two non-empty cause sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self::from_nonzero_bits(self.0.get() | other.0.get())
    }

    /// Iterate through included causes in stable wire-format order.
    pub fn iter(self) -> impl Iterator<Item = CorporateActionAdjustmentCause> {
        CorporateActionAdjustmentCause::ALL
            .into_iter()
            .filter(move |cause| self.contains(*cause))
    }

    /// Return the number of included causes.
    #[must_use]
    pub const fn len(self) -> usize {
        self.0.get().count_ones() as usize
    }

    /// Return whether this set is empty.
    ///
    /// Always `false`; the type invariant forbids empty sets.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        false
    }
}

impl Serialize for CorporateActionAdjustmentCauses {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for cause in self.iter() {
            seq.serialize_element(&cause)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for CorporateActionAdjustmentCauses {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let causes = Vec::<CorporateActionAdjustmentCause>::deserialize(deserializer)?;
        let mut bits = 0;

        for cause in causes {
            let bit = cause.bit();
            if bits & bit != 0 {
                return Err(serde::de::Error::custom(format!(
                    "duplicate corporate-action adjustment cause: {}",
                    cause.code()
                )));
            }
            bits |= bit;
        }

        Self::from_bits(bits).ok_or_else(|| {
            serde::de::Error::custom("corporate-action adjustment causes must be non-empty")
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
/// Method used to adjust prices across discontinuities.
pub enum AdjustmentMethod {
    /// Multiplicative ratio adjustment.
    Multiplicative,
    /// Additive difference adjustment.
    Additive,
    /// Provider-defined adjustment method.
    ProviderDefined,
}

impl AdjustmentMethod {
    /// Return the stable wire-format code for this method.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Multiplicative => "multiplicative",
            Self::Additive => "additive",
            Self::ProviderDefined => "provider_defined",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Optional metadata describing the history series.
pub struct HistoryMeta {
    /// IANA timezone identifier.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub timezone: Option<Tz>,
    /// UTC offset in seconds.
    pub utc_offset_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// A complete history response including candles, actions, and metadata.
///
/// Generic over a response-level provider metadata payload `R`, which is
/// flattened into the serialized representation, and a candle-level metadata
/// payload `C`. Use the [`HistoryResponse`] alias for the standard shape
/// (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericHistoryResponse<R = (), C = ()> {
    /// Candles as supplied by a provider.
    ///
    /// Providers are expected to order these by non-decreasing timestamp, but
    /// direct struct construction and deserialization do not enforce that. Use
    /// [`Self::is_chronologically_ordered`] when consumers need to validate
    /// ordering.
    pub candles: Vec<GenericCandle<C>>,
    /// Corporate actions aligned to candles.
    pub actions: Vec<crate::market::action::Action>,
    /// Price basis of the primary open, high, low, and close fields.
    pub price_basis: OhlcPriceBasis,
    /// Optional metadata including timezone.
    pub meta: Option<HistoryMeta>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: R,
}

impl<R, C> GenericHistoryResponse<R, C> {
    /// Return `true` when candle timestamps are non-decreasing.
    ///
    /// Duplicate timestamps are considered ordered so callers can preserve the
    /// provider's tie ordering.
    #[must_use]
    pub fn is_chronologically_ordered(&self) -> bool {
        self.candles.windows(2).all(|pair| pair[0].ts <= pair[1].ts)
    }
}

/// Standard `HistoryResponse` with no extra provider metadata.
pub type HistoryResponse = GenericHistoryResponse<(), ()>;
