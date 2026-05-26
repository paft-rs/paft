//! Historical data request types and helpers.
use std::fmt::Display;

use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::MarketError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Logical lookback range presets.
pub enum Range {
    /// 1 minute
    I1m,
    /// 2 minutes
    I2m,
    /// 5 minutes
    I5m,
    /// 10 minutes
    I10m,
    /// 15 minutes
    I15m,
    /// 30 minutes
    I30m,
    /// 1 hour
    I1h,
    /// 4 hours
    I4h,
    /// 6 hours
    I6h,
    /// 8 hours
    I8h,
    /// 12 hours
    I12h,
    /// 1 day
    #[default]
    D1,
    /// 5 days
    D5,
    /// 1 month
    M1,
    /// 3 months
    M3,
    /// 6 months
    M6,
    /// 1 year
    Y1,
    /// 2 years
    Y2,
    /// 5 years
    Y5,
    /// 10 years
    Y10,
    /// Year to date
    Ytd,
    /// Maximum available
    Max,
}

impl Range {
    /// Returns the stable wire-format code for this range.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::I1m => "1m",
            Self::I2m => "2m",
            Self::I5m => "5m",
            Self::I10m => "10m",
            Self::I15m => "15m",
            Self::I30m => "30m",
            Self::I1h => "1h",
            Self::I4h => "4h",
            Self::I6h => "6h",
            Self::I8h => "8h",
            Self::I12h => "12h",
            Self::D1 => "1d",
            Self::D5 => "5d",
            Self::M1 => "1mo",
            Self::M3 => "3mo",
            Self::M6 => "6mo",
            Self::Y1 => "1y",
            Self::Y2 => "2y",
            Self::Y5 => "5y",
            Self::Y10 => "10y",
            Self::Ytd => "ytd",
            Self::Max => "max",
        }
    }

    fn from_code(value: &str) -> Option<Self> {
        match value {
            "1m" => Some(Self::I1m),
            "2m" => Some(Self::I2m),
            "5m" => Some(Self::I5m),
            "10m" => Some(Self::I10m),
            "15m" => Some(Self::I15m),
            "30m" => Some(Self::I30m),
            "1h" => Some(Self::I1h),
            "4h" => Some(Self::I4h),
            "6h" => Some(Self::I6h),
            "8h" => Some(Self::I8h),
            "12h" => Some(Self::I12h),
            "1d" => Some(Self::D1),
            "5d" => Some(Self::D5),
            "1mo" => Some(Self::M1),
            "3mo" => Some(Self::M3),
            "6mo" => Some(Self::M6),
            "1y" => Some(Self::Y1),
            "2y" => Some(Self::Y2),
            "5y" => Some(Self::Y5),
            "10y" => Some(Self::Y10),
            "ytd" => Some(Self::Ytd),
            "max" => Some(Self::Max),
            _ => None,
        }
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str((*self).code())
    }
}

impl Serialize for Range {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str((*self).code())
    }
}

impl<'de> Deserialize<'de> for Range {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_code(&value).ok_or_else(|| {
            serde::de::Error::unknown_variant(
                &value,
                &[
                    "1m", "2m", "5m", "10m", "15m", "30m", "1h", "4h", "6h", "8h", "12h", "1d",
                    "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max",
                ],
            )
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Supported resolution intervals.
pub enum Interval {
    /// 1 second
    I1s,
    /// 2 seconds
    I2s,
    /// 3 seconds
    I3s,
    /// 5 seconds
    I5s,
    /// 6 seconds
    I6s,
    /// 10 seconds
    I10s,
    /// 15 seconds
    I15s,
    /// 30 seconds
    I30s,
    /// 90 seconds
    I90s,
    /// 1 minute
    #[default]
    I1m,
    /// 2 minutes
    I2m,
    /// 3 minutes
    I3m,
    /// 5 minutes
    I5m,
    /// 6 minutes
    I6m,
    /// 10 minutes
    I10m,
    /// 15 minutes
    I15m,
    /// 30 minutes
    I30m,
    /// 90 minutes
    I90m,
    /// 1 hour
    I1h,
    /// 2 hours
    I2h,
    /// 3 hours
    I3h,
    /// 4 hours
    I4h,
    /// 6 hours
    I6h,
    /// 8 hours
    I8h,
    /// 12 hours
    I12h,
    /// 1 day
    D1,
    /// 5 days (provider-dependent)
    D5,
    /// 1 week
    W1,
    /// 1 month
    M1,
    /// 3 months
    M3,
    /// 6 months
    M6,
    /// 1 year
    Y1,
    /// 2 years
    Y2,
    /// 5 years
    Y5,
    /// 10 years
    Y10,
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str((*self).code())
    }
}

impl Interval {
    /// Returns the stable wire-format code for this interval.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::I1s => "1s",
            Self::I2s => "2s",
            Self::I3s => "3s",
            Self::I5s => "5s",
            Self::I6s => "6s",
            Self::I10s => "10s",
            Self::I15s => "15s",
            Self::I30s => "30s",
            Self::I90s => "90s",
            Self::I1m => "1m",
            Self::I2m => "2m",
            Self::I3m => "3m",
            Self::I5m => "5m",
            Self::I6m => "6m",
            Self::I10m => "10m",
            Self::I15m => "15m",
            Self::I30m => "30m",
            Self::I90m => "90m",
            Self::I1h => "1h",
            Self::I2h => "2h",
            Self::I3h => "3h",
            Self::I4h => "4h",
            Self::I6h => "6h",
            Self::I8h => "8h",
            Self::I12h => "12h",
            Self::D1 => "1d",
            Self::D5 => "5d",
            Self::W1 => "1wk",
            Self::M1 => "1mo",
            Self::M3 => "3mo",
            Self::M6 => "6mo",
            Self::Y1 => "1y",
            Self::Y2 => "2y",
            Self::Y5 => "5y",
            Self::Y10 => "10y",
        }
    }

    fn from_code(value: &str) -> Option<Self> {
        match value {
            "1s" => Some(Self::I1s),
            "2s" => Some(Self::I2s),
            "3s" => Some(Self::I3s),
            "5s" => Some(Self::I5s),
            "6s" => Some(Self::I6s),
            "10s" => Some(Self::I10s),
            "15s" => Some(Self::I15s),
            "30s" => Some(Self::I30s),
            "90s" => Some(Self::I90s),
            "1m" => Some(Self::I1m),
            "2m" => Some(Self::I2m),
            "3m" => Some(Self::I3m),
            "5m" => Some(Self::I5m),
            "6m" => Some(Self::I6m),
            "10m" => Some(Self::I10m),
            "15m" => Some(Self::I15m),
            "30m" => Some(Self::I30m),
            "90m" => Some(Self::I90m),
            "1h" => Some(Self::I1h),
            "2h" => Some(Self::I2h),
            "3h" => Some(Self::I3h),
            "4h" => Some(Self::I4h),
            "6h" => Some(Self::I6h),
            "8h" => Some(Self::I8h),
            "12h" => Some(Self::I12h),
            "1d" => Some(Self::D1),
            "5d" => Some(Self::D5),
            "1wk" => Some(Self::W1),
            "1mo" => Some(Self::M1),
            "3mo" => Some(Self::M3),
            "6mo" => Some(Self::M6),
            "1y" => Some(Self::Y1),
            "2y" => Some(Self::Y2),
            "5y" => Some(Self::Y5),
            "10y" => Some(Self::Y10),
            _ => None,
        }
    }

    /// Is this an intraday interval?
    #[must_use]
    pub const fn is_intraday(self) -> bool {
        matches!(
            self,
            Self::I1s
                | Self::I2s
                | Self::I3s
                | Self::I5s
                | Self::I6s
                | Self::I10s
                | Self::I15s
                | Self::I30s
                | Self::I90s
                | Self::I1m
                | Self::I2m
                | Self::I3m
                | Self::I5m
                | Self::I6m
                | Self::I10m
                | Self::I15m
                | Self::I30m
                | Self::I90m
                | Self::I1h
                | Self::I2h
                | Self::I3h
                | Self::I4h
                | Self::I6h
                | Self::I8h
                | Self::I12h
        )
    }

    /// Returns the interval length in minutes for intraday, otherwise None.
    #[must_use]
    pub const fn minutes(self) -> Option<i64> {
        match self {
            Self::I1m => Some(1),
            Self::I2m => Some(2),
            Self::I3m => Some(3),
            Self::I5m => Some(5),
            Self::I6m => Some(6),
            Self::I10m => Some(10),
            Self::I15m => Some(15),
            Self::I30m => Some(30),
            Self::I90m => Some(90),
            Self::I1h => Some(60),
            Self::I2h => Some(120),
            Self::I3h => Some(180),
            Self::I4h => Some(240),
            Self::I6h => Some(360),
            Self::I8h => Some(480),
            Self::I12h => Some(720),
            _ => None,
        }
    }

    /// Returns the interval length in seconds for intraday, otherwise None.
    #[must_use]
    pub const fn seconds(self) -> Option<i64> {
        match self {
            Self::I1s => Some(1),
            Self::I2s => Some(2),
            Self::I3s => Some(3),
            Self::I5s => Some(5),
            Self::I6s => Some(6),
            Self::I10s => Some(10),
            Self::I15s => Some(15),
            Self::I30s => Some(30),
            Self::I90s => Some(90),
            Self::I1m => Some(60),
            Self::I2m => Some(120),
            Self::I3m => Some(180),
            Self::I5m => Some(300),
            Self::I6m => Some(360),
            Self::I10m => Some(600),
            Self::I15m => Some(900),
            Self::I30m => Some(1_800),
            Self::I90m => Some(5_400),
            Self::I1h => Some(3_600),
            Self::I2h => Some(7_200),
            Self::I3h => Some(10_800),
            Self::I4h => Some(14_400),
            Self::I6h => Some(21_600),
            Self::I8h => Some(28_800),
            Self::I12h => Some(43_200),
            _ => None,
        }
    }
}

impl Serialize for Interval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str((*self).code())
    }
}

impl<'de> Deserialize<'de> for Interval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_code(&value).ok_or_else(|| {
            serde::de::Error::unknown_variant(
                &value,
                &[
                    "1s", "2s", "3s", "5s", "6s", "10s", "15s", "30s", "90s", "1m", "2m", "3m",
                    "5m", "6m", "10m", "15m", "30m", "90m", "1h", "2h", "3h", "4h", "6h", "8h",
                    "12h", "1d", "5d", "1wk", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y",
                ],
            )
        })
    }
}

bitflags! {
    /// Flags to control additional behaviors in history requests.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct HistoryFlags: u8 {
        /// Include pre/post market sessions if supported.
        const INCLUDE_PREPOST = 0b0001;
        /// Include corporate actions (dividends/splits) in the response if supported.
        const INCLUDE_ACTIONS = 0b0010;
        /// Prefer adjusted close for equities when provider supports it.
        const AUTO_ADJUST = 0b0100;
        /// Keep missing candle slots as placeholders depending on consumer.
        const KEEPNA = 0b1000;
    }
}

/// Time specification for historical data requests.
///
/// This enum ensures that only one time specification method is used at a time,
/// making invalid states unrepresentable at compile time.
///
/// Serializes as explicitly tagged JSON:
/// `{ "kind": "range", "range": "6mo" }` or
/// `{ "kind": "period", "start": 1716595200, "end": 1719187200 }`.
/// Period timestamps use Unix seconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeSpec {
    /// Use a logical range preset (e.g., "1 month", "1 year").
    Range(Range),
    /// Use an explicit time period with start and end timestamps.
    Period {
        /// Start timestamp for the period.
        start: DateTime<Utc>,
        /// End timestamp for the period.
        end: DateTime<Utc>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum TimeSpecWire {
    Range {
        range: Range,
    },
    Period {
        #[serde(with = "chrono::serde::ts_seconds")]
        start: DateTime<Utc>,
        #[serde(with = "chrono::serde::ts_seconds")]
        end: DateTime<Utc>,
    },
}

impl Serialize for TimeSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wire = match self {
            Self::Range(range) => TimeSpecWire::Range { range: *range },
            Self::Period { start, end } => TimeSpecWire::Period {
                start: *start,
                end: *end,
            },
        };

        wire.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TimeSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match TimeSpecWire::deserialize(deserializer)? {
            TimeSpecWire::Range { range } => Self::Range(range),
            TimeSpecWire::Period { start, end } => Self::Period { start, end },
        })
    }
}

impl Default for TimeSpec {
    fn default() -> Self {
        Self::Range(Range::M6)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Request parameters for fetching history.
///
/// Use `HistoryRequest::builder()` to create instances.
pub struct HistoryRequest {
    /// Time specification for the request.
    time_spec: TimeSpec,
    /// Desired aggregation interval.
    interval: Interval,
    /// Additional behavior flags.
    flags: HistoryFlags,
}

/// Shadow type used for deserializing [`HistoryRequest`].
///
/// Matches the serialized wire shape, then routes through
/// [`HistoryRequestBuilder::build`] so period validation cannot be skipped.
#[derive(Deserialize)]
struct HistoryRequestShadow {
    time_spec: TimeSpec,
    interval: Interval,
    flags: HistoryFlags,
}

impl<'de> Deserialize<'de> for HistoryRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let shadow = HistoryRequestShadow::deserialize(deserializer)?;

        let mut builder = Self::builder().interval(shadow.interval);
        match shadow.time_spec {
            TimeSpec::Range(range) => builder = builder.range(range),
            TimeSpec::Period { start, end } => builder = builder.period(start, end),
        }
        builder.flags = shadow.flags;

        builder.build().map_err(serde::de::Error::custom)
    }
}

/// Builder for creating validated `HistoryRequest` instances.
#[derive(Debug, Clone)]
pub struct HistoryRequestBuilder {
    time_spec: TimeSpec,
    interval: Interval,
    flags: HistoryFlags,
}

impl HistoryRequestBuilder {
    /// Create a new builder with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            time_spec: TimeSpec::Range(Range::M6),
            interval: Interval::D1,
            flags: HistoryFlags::INCLUDE_ACTIONS | HistoryFlags::AUTO_ADJUST,
        }
    }

    /// Set the logical range.
    #[must_use]
    pub const fn range(mut self, range: Range) -> Self {
        self.time_spec = TimeSpec::Range(range);
        self
    }

    /// Set the explicit period with start and end timestamps.
    #[must_use]
    pub const fn period(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.time_spec = TimeSpec::Period { start, end };
        self
    }

    /// Set the aggregation interval.
    #[must_use]
    pub const fn interval(mut self, interval: Interval) -> Self {
        self.interval = interval;
        self
    }

    /// Set whether to include pre/post market sessions.
    #[must_use]
    pub fn include_prepost(mut self, include: bool) -> Self {
        if include {
            self.flags.insert(HistoryFlags::INCLUDE_PREPOST);
        } else {
            self.flags.remove(HistoryFlags::INCLUDE_PREPOST);
        }
        self
    }

    /// Set whether to include corporate actions.
    #[must_use]
    pub fn include_actions(mut self, include: bool) -> Self {
        if include {
            self.flags.insert(HistoryFlags::INCLUDE_ACTIONS);
        } else {
            self.flags.remove(HistoryFlags::INCLUDE_ACTIONS);
        }
        self
    }

    /// Set whether to prefer adjusted close prices.
    #[must_use]
    pub fn auto_adjust(mut self, adjust: bool) -> Self {
        if adjust {
            self.flags.insert(HistoryFlags::AUTO_ADJUST);
        } else {
            self.flags.remove(HistoryFlags::AUTO_ADJUST);
        }
        self
    }

    /// Set whether to keep missing candle slots.
    #[must_use]
    pub fn keepna(mut self, keep: bool) -> Self {
        if keep {
            self.flags.insert(HistoryFlags::KEEPNA);
        } else {
            self.flags.remove(HistoryFlags::KEEPNA);
        }
        self
    }

    /// Build the `HistoryRequest` with validation.
    ///
    /// Returns an error if:
    /// - `period` start is not before end
    ///
    /// # Errors
    /// Returns `MarketError::InvalidPeriod` when a `Period { start, end }` has `start >= end`.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn build(self) -> Result<HistoryRequest, MarketError> {
        // Validate period constraints
        if let TimeSpec::Period { start, end } = &self.time_spec
            && start >= end
        {
            return Err(MarketError::InvalidPeriod {
                start: start.timestamp(),
                end: end.timestamp(),
            });
        }

        Ok(HistoryRequest {
            time_spec: self.time_spec,
            interval: self.interval,
            flags: self.flags,
        })
    }
}

impl Default for HistoryRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryRequest {
    /// Create a new builder for constructing a `HistoryRequest`.
    #[must_use]
    pub fn builder() -> HistoryRequestBuilder {
        HistoryRequestBuilder::new()
    }

    /// Build a request using a logical `range` and explicit `interval`.
    ///
    /// This is a convenience method that uses the builder pattern internally.
    ///
    /// # Errors
    /// Propagates errors from `HistoryRequestBuilder::build`.
    pub fn try_from_range(range: Range, interval: Interval) -> Result<Self, MarketError> {
        Self::builder().range(range).interval(interval).build()
    }

    /// Build a request using an explicit `[start, end)` epoch-second period and `interval`.
    ///
    /// This is a convenience method that uses the builder pattern internally.
    ///
    /// # Errors
    /// Propagates errors from `HistoryRequestBuilder::build`.
    pub fn try_from_period(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: Interval,
    ) -> Result<Self, MarketError> {
        Self::builder()
            .period(start, end)
            .interval(interval)
            .build()
    }

    /// Get the time specification.
    #[must_use]
    pub const fn time_spec(&self) -> &TimeSpec {
        &self.time_spec
    }

    /// Get the range if using a range specification.
    #[must_use]
    pub const fn range(&self) -> Option<Range> {
        match &self.time_spec {
            TimeSpec::Range(range) => Some(*range),
            TimeSpec::Period { .. } => None,
        }
    }

    /// Get the period if using a period specification.
    #[must_use]
    pub const fn period(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        match &self.time_spec {
            TimeSpec::Range(_) => None,
            TimeSpec::Period { start, end } => Some((*start, *end)),
        }
    }

    /// Get the interval.
    #[must_use]
    pub const fn interval(&self) -> Interval {
        self.interval
    }

    /// Get whether pre/post market sessions are included.
    #[must_use]
    pub const fn include_prepost(&self) -> bool {
        self.flags.contains(HistoryFlags::INCLUDE_PREPOST)
    }

    /// Get whether corporate actions are included.
    #[must_use]
    pub const fn include_actions(&self) -> bool {
        self.flags.contains(HistoryFlags::INCLUDE_ACTIONS)
    }

    /// Get whether auto-adjust is enabled.
    #[must_use]
    pub const fn auto_adjust(&self) -> bool {
        self.flags.contains(HistoryFlags::AUTO_ADJUST)
    }

    /// Get whether missing values are kept.
    #[must_use]
    pub const fn keepna(&self) -> bool {
        self.flags.contains(HistoryFlags::KEEPNA)
    }
}
