//! Historical data request types and helpers.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
use paft_core::error::PaftError;

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

/// Two-variant switch used instead of multiple independent bools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Switch {
    /// Feature disabled
    #[default]
    Off,
    /// Feature enabled
    On,
}

impl From<bool> for Switch {
    fn from(b: bool) -> Self {
        if b { Self::On } else { Self::Off }
    }
}

impl Switch {
    /// Returns true if the switch is On.
    #[must_use]
    pub const fn is_on(self) -> bool {
        matches!(self, Self::On)
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
    /// Include pre/post market sessions if supported.
    include_prepost: Switch,
    /// Include corporate actions (dividends/splits) in the response if supported.
    include_actions: Switch,
    /// Prefer adjusted close for equities when provider supports it.
    auto_adjust: Switch,
    /// Keep missing candle slots as `NaN`/placeholder depending on consumer.
    keepna: Switch,
}

/// Builder for creating validated `HistoryRequest` instances.
#[derive(Debug, Clone)]
pub struct HistoryRequestBuilder {
    time_spec: TimeSpec,
    interval: Interval,
    include_prepost: Switch,
    include_actions: Switch,
    auto_adjust: Switch,
    keepna: Switch,
}

impl HistoryRequestBuilder {
    /// Create a new builder with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            time_spec: TimeSpec::Range(Range::M6),
            interval: Interval::D1,
            include_prepost: Switch::Off,
            include_actions: Switch::On,
            auto_adjust: Switch::On,
            keepna: Switch::Off,
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
    pub const fn include_prepost(mut self, include: bool) -> Self {
        self.include_prepost = if include { Switch::On } else { Switch::Off };
        self
    }

    /// Set whether to include corporate actions.
    #[must_use]
    pub const fn include_actions(mut self, include: bool) -> Self {
        self.include_actions = if include { Switch::On } else { Switch::Off };
        self
    }

    /// Set whether to prefer adjusted close prices.
    #[must_use]
    pub const fn auto_adjust(mut self, adjust: bool) -> Self {
        self.auto_adjust = if adjust { Switch::On } else { Switch::Off };
        self
    }

    /// Set whether to keep missing candle slots.
    #[must_use]
    pub const fn keepna(mut self, keep: bool) -> Self {
        self.keepna = if keep { Switch::On } else { Switch::Off };
        self
    }

    /// Build the `HistoryRequest` with validation.
    ///
    /// Returns an error if:
    /// - `period` start is not before end
    ///
    /// # Errors
    /// Returns `PaftError::InvalidPeriod` when a `Period { start, end }` has `start >= end`.
    pub fn build(self) -> Result<HistoryRequest, PaftError> {
        // Validate period constraints
        if let TimeSpec::Period { start, end } = &self.time_spec
            && start >= end
        {
            return Err(PaftError::InvalidPeriod {
                start: start.timestamp(),
                end: end.timestamp(),
            });
        }

        Ok(HistoryRequest {
            time_spec: self.time_spec,
            interval: self.interval,
            include_prepost: self.include_prepost,
            include_actions: self.include_actions,
            auto_adjust: self.auto_adjust,
            keepna: self.keepna,
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
    pub const fn builder() -> HistoryRequestBuilder {
        HistoryRequestBuilder::new()
    }

    /// Build a request using a logical `range` and explicit `interval`.
    ///
    /// This is a convenience method that uses the builder pattern internally.
    ///
    /// # Errors
    /// Propagates errors from `HistoryRequestBuilder::build`.
    pub fn try_from_range(range: Range, interval: Interval) -> Result<Self, PaftError> {
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
    ) -> Result<Self, PaftError> {
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
        self.include_prepost.is_on()
    }

    /// Get whether corporate actions are included.
    #[must_use]
    pub const fn include_actions(&self) -> bool {
        self.include_actions.is_on()
    }

    /// Get whether auto-adjust is enabled.
    #[must_use]
    pub const fn auto_adjust(&self) -> bool {
        self.auto_adjust.is_on()
    }

    /// Get whether missing values are kept.
    #[must_use]
    pub const fn keepna(&self) -> bool {
        self.keepna.is_on()
    }
}
