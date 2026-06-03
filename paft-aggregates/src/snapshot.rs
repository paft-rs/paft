//! Instant-in-time market snapshot model.
//!
//! [`Snapshot`] is a strictly snapshot-shaped view of an instrument: identity,
//! the current session's prices, and the timestamp the snapshot was taken at.
//! Fundamentals, analyst coverage, and ESG fields are intentionally excluded
//! and live in `paft-fundamentals`.

// `Eq` is intentionally NOT derived on the generic payload types: the
// metadata payload `M` is meant to accept user types that don't satisfy
// `Eq` (e.g. HFT timestamps stored as `f64` for hardware-clock latency).
#![allow(clippy::derive_partial_eq_without_eq)]

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_domain::{Instrument, MarketState};
use paft_money::{Currency, PriceAmount};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Strictly instant-in-time snapshot for an instrument.
///
/// All fields except `instrument` are optional to accommodate partially
/// populated data from upstream sources.
///
/// Generic over a provider metadata payload `M`, which is flattened into the
/// serialized representation. Use the [`Snapshot`] alias for the
/// standard shape (no extra metadata).
///
/// **Collision warning:** provider metadata is flattened into the same object
/// as paft fields. Metadata field names must not collide with paft field
/// names; prefer provider-specific prefixes when in doubt.
pub struct GenericSnapshot<M = ()> {
    /// Primary instrument as provided by the data source.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub instrument: Instrument,
    /// Human-friendly instrument name.
    pub name: Option<String>,
    /// Current market session state (for example: Pre, Regular, Post).
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub market_state: Option<MarketState>,
    /// Timestamp (UTC) when this snapshot was taken.
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub as_of: Option<DateTime<Utc>>,
    /// Currency shared by every price amount in this snapshot.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub currency: Currency,
    /// Most recent traded/quoted price.
    pub last: Option<PriceAmount>,
    /// Previous session's official close price.
    pub previous_close: Option<PriceAmount>,
    /// Opening price for the current session.
    pub open: Option<PriceAmount>,
    /// Highest traded price observed during the current session.
    pub day_high: Option<PriceAmount>,
    /// Lowest traded price observed during the current session.
    pub day_low: Option<PriceAmount>,
    /// Today's trading volume.
    pub volume: Option<u64>,
    /// Provider-specific payload, flattened into the serialized form.
    #[serde(flatten, default = "Default::default")]
    pub provider: M,
}

impl<M: Default> GenericSnapshot<M> {
    /// Build a snapshot for the given instrument with all optional fields
    /// unset. `provider` is initialised via `M::default()`.
    #[must_use]
    pub fn new(instrument: Instrument, currency: Currency) -> Self {
        Self {
            instrument,
            name: None,
            market_state: None,
            as_of: None,
            currency,
            last: None,
            previous_close: None,
            open: None,
            day_high: None,
            day_low: None,
            volume: None,
            provider: M::default(),
        }
    }
}

/// Standard `Snapshot` with no extra provider metadata.
pub type Snapshot = GenericSnapshot<()>;
