//! Measurement context for independently reusable financial statement rows.

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};

use crate::FundamentalsError;

/// A balance measured at the close of `date` in the reporting entity's calendar.
///
/// The instant is immediately before the following calendar day begins, not
/// the start of `date`. This is a reporting-calendar boundary, not a UTC event
/// or a publication timestamp. No timezone conversion is implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "dataframe", derive(df_derive_macros::ToDataFrame))]
pub struct StatementInstant {
    /// Inclusive reporting date whose closing balance is measured.
    pub date: NaiveDate,
}

impl StatementInstant {
    /// Creates an instant at the close of a reporting date.
    #[must_use]
    pub const fn new(date: NaiveDate) -> Self {
        Self { date }
    }
}

/// An inclusive interval of reporting-calendar dates, with `start <= end`.
///
/// Measures flows from the start of `start` through the close of `end` in the
/// reporting entity's calendar. A single-day interval is valid. Standalone
/// quarters, cumulative year-to-date periods, and trailing periods are all
/// permitted, but every duration measure in a row must use this same window.
/// The fiscal [`paft_domain::ReportingPeriod`] label never supplies these dates.
/// Adapters with unknown or mixed windows must resolve them before mapping.
///
/// ```
/// use chrono::NaiveDate;
/// use paft_fundamentals::StatementDuration;
/// let quarter = StatementDuration::new(
///     NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
///     NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
/// ).unwrap();
/// assert_eq!(quarter.end().to_string(), "2024-06-30");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "dataframe", derive(df_derive_macros::ToDataFrame))]
pub struct StatementDuration {
    start: NaiveDate,
    end: NaiveDate,
}

impl StatementDuration {
    /// Creates an inclusive reporting interval.
    ///
    /// # Errors
    /// Returns [`FundamentalsError::InvalidStatementDuration`] if `start > end`.
    pub fn new(start: NaiveDate, end: NaiveDate) -> Result<Self, FundamentalsError> {
        if start > end {
            return Err(FundamentalsError::InvalidStatementDuration { start, end });
        }
        Ok(Self { start, end })
    }

    /// Returns the first included reporting date.
    #[must_use]
    pub const fn start(&self) -> NaiveDate {
        self.start
    }

    /// Returns the last included reporting date.
    #[must_use]
    pub const fn end(&self) -> NaiveDate {
        self.end
    }
}

impl<'de> Deserialize<'de> for StatementDuration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Shadow {
            start: NaiveDate,
            end: NaiveDate,
        }
        let shadow = Shadow::deserialize(deserializer)?;
        Self::new(shadow.start, shadow.end).map_err(serde::de::Error::custom)
    }
}
