//! Historical data request types and helpers.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::error::MarketError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Logical lookback range presets.
pub enum Range {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Supported resolution intervals.
pub enum Interval {
    /// 1 minute
    #[default]
    I1m,
    /// 2 minutes
    I2m,
    /// 5 minutes
    I5m,
    /// 15 minutes
    I15m,
    /// 30 minutes
    I30m,
    /// 90 minutes
    I90m,
    /// 1 hour
    I1h,
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
}

impl Interval {
    /// Is this an intraday interval?
    #[must_use]
    pub const fn is_intraday(self) -> bool {
        use Interval::{I1h, I1m, I2m, I5m, I15m, I30m, I90m};
        matches!(self, I1m | I2m | I5m | I15m | I30m | I90m | I1h)
    }

    /// Returns the interval length in minutes for intraday, otherwise None.
    #[must_use]
    pub const fn minutes(self) -> Option<i64> {
        use Interval::{I1h, I1m, I2m, I5m, I15m, I30m, I90m};
        match self {
            I1m => Some(1),
            I2m => Some(2),
            I5m => Some(5),
            I15m => Some(15),
            I30m => Some(30),
            I1h => Some(60),
            I90m => Some(90),
            _ => None,
        }
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl Default for TimeSpec {
    fn default() -> Self {
        Self::Range(Range::M6)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
