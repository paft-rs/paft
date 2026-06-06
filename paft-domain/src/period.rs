//! Financial period primitives.
//!
//! Provides separate reporting/fiscal period labels and calendar period buckets.

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, Visitor},
};
use std::borrow::Cow;
use std::fmt;

use crate::error::DomainError;
use chrono::{Datelike, NaiveDate};
use paft_utils::Canonical;

/// Valid year component for structured financial periods.
///
/// `ReportingPeriod` accepts calendar-style four-digit years in `0..=9999`. The lower
/// bound preserves the crate's existing parser behavior for tokens like
/// `0000`; the upper bound keeps structured period display/serde canonical as
/// exactly four year digits.
///
/// Standalone serde emits the same four-digit canonical string as
/// [`std::fmt::Display`].
/// Deserialization also accepts integer years for compatibility and normalizes
/// them on the next serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PeriodYear(u16);

impl PeriodYear {
    /// Smallest valid structured period year.
    pub const MIN: u16 = 0;

    /// Largest valid structured period year.
    pub const MAX: u16 = 9999;

    /// Builds a validated period year.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodYear`] when `year` is outside
    /// `0..=9999`.
    pub fn new(year: i32) -> Result<Self, DomainError> {
        let Ok(year_u16) = u16::try_from(year) else {
            return Err(DomainError::InvalidPeriodYear { year });
        };

        if year_u16 <= Self::MAX {
            Ok(Self(year_u16))
        } else {
            Err(DomainError::InvalidPeriodYear { year })
        }
    }

    /// Returns the year as an `i32`, matching [`chrono::Datelike::year`].
    #[must_use]
    #[allow(clippy::cast_lossless)]
    pub const fn get(self) -> i32 {
        self.0 as i32
    }

    /// Returns the year as the compact unsigned storage type.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

impl TryFrom<i32> for PeriodYear {
    type Error = DomainError;

    fn try_from(year: i32) -> Result<Self, Self::Error> {
        Self::new(year)
    }
}

impl TryFrom<u16> for PeriodYear {
    type Error = DomainError;

    fn try_from(year: u16) -> Result<Self, Self::Error> {
        Self::new(i32::from(year))
    }
}

impl From<PeriodYear> for i32 {
    fn from(year: PeriodYear) -> Self {
        year.get()
    }
}

impl From<PeriodYear> for u16 {
    fn from(year: PeriodYear) -> Self {
        year.as_u16()
    }
}

impl fmt::Display for PeriodYear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}", self.0)
    }
}

impl Serialize for PeriodYear {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PeriodYear {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PeriodYearVisitor)
    }
}

struct PeriodYearVisitor;

impl Visitor<'_> for PeriodYearVisitor {
    type Value = PeriodYear;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a canonical four-digit period year string or integer in 0..=9999")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        parse_period_year_code(value).map_err(DeError::custom)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        period_year_from_i64(value).map_err(DeError::custom)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        period_year_from_u64(value).map_err(DeError::custom)
    }
}

fn parse_period_year_code(value: &str) -> Result<PeriodYear, DomainError> {
    let bytes = value.as_bytes();
    if bytes.len() != 4 || !bytes.iter().all(u8::is_ascii_digit) {
        return Err(DomainError::InvalidPeriodFormat {
            format: value.to_string(),
        });
    }

    let year = i32::from(bytes[0] - b'0') * 1_000
        + i32::from(bytes[1] - b'0') * 100
        + i32::from(bytes[2] - b'0') * 10
        + i32::from(bytes[3] - b'0');

    PeriodYear::new(year)
}

fn period_year_from_i64(value: i64) -> Result<PeriodYear, DomainError> {
    let Ok(year) = i32::try_from(value) else {
        return Err(DomainError::InvalidPeriodFormat {
            format: value.to_string(),
        });
    };

    PeriodYear::new(year)
}

fn period_year_from_u64(value: u64) -> Result<PeriodYear, DomainError> {
    let Ok(year) = i32::try_from(value) else {
        return Err(DomainError::InvalidPeriodFormat {
            format: value.to_string(),
        });
    };

    PeriodYear::new(year)
}

fn parse_period_date_code(value: &str) -> Result<PeriodDate, DomainError> {
    let invalid = || DomainError::InvalidPeriodFormat {
        format: value.to_string(),
    };

    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(invalid());
    }

    let Some(year) = read_4_digits(bytes, 0) else {
        return Err(invalid());
    };

    if !bytes[5..7].iter().all(u8::is_ascii_digit) || !bytes[8..10].iter().all(u8::is_ascii_digit) {
        return Err(invalid());
    }

    let month = u32::from(bytes[5] - b'0') * 10 + u32::from(bytes[6] - b'0');
    let day = u32::from(bytes[8] - b'0') * 10 + u32::from(bytes[9] - b'0');
    let date = NaiveDate::from_ymd_opt(year, month, day).ok_or_else(invalid)?;
    PeriodDate::new(date)
}

fn parse_quarter_of_year_code(value: &str) -> Result<QuarterOfYear, DomainError> {
    let bytes = value.as_bytes();
    if bytes.len() != 1 || !bytes[0].is_ascii_digit() {
        return Err(DomainError::InvalidPeriodFormat {
            format: value.to_string(),
        });
    }

    QuarterOfYear::new(bytes[0] - b'0')
}

fn quarter_of_year_from_i64(value: i64) -> Result<QuarterOfYear, DomainError> {
    let Ok(quarter) = u8::try_from(value) else {
        return Err(DomainError::InvalidPeriodFormat {
            format: value.to_string(),
        });
    };

    QuarterOfYear::new(quarter)
}

fn quarter_of_year_from_u64(value: u64) -> Result<QuarterOfYear, DomainError> {
    let Ok(quarter) = u8::try_from(value) else {
        return Err(DomainError::InvalidPeriodFormat {
            format: value.to_string(),
        });
    };

    QuarterOfYear::new(quarter)
}

/// Valid date component for structured financial periods.
///
/// The wrapped [`NaiveDate`] always has a year in `0..=9999`, matching
/// [`PeriodYear`] and the four-digit canonical `YYYY-MM-DD` period format.
///
/// Standalone serde emits the same canonical `YYYY-MM-DD` string as
/// [`std::fmt::Display`] and deserializes that canonical form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PeriodDate(NaiveDate);

impl PeriodDate {
    /// Builds a validated period date.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodYear`] when the date's year is
    /// outside `0..=9999`.
    pub fn new(date: NaiveDate) -> Result<Self, DomainError> {
        PeriodYear::new(date.year())?;
        Ok(Self(date))
    }

    /// Returns the wrapped date.
    #[must_use]
    pub const fn get(self) -> NaiveDate {
        self.0
    }
}

impl TryFrom<NaiveDate> for PeriodDate {
    type Error = DomainError;

    fn try_from(date: NaiveDate) -> Result<Self, Self::Error> {
        Self::new(date)
    }
}

impl From<PeriodDate> for NaiveDate {
    fn from(date: PeriodDate) -> Self {
        date.get()
    }
}

impl fmt::Display for PeriodDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl Serialize for PeriodDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PeriodDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        parse_period_date_code(&raw).map_err(DeError::custom)
    }
}

/// Valid quarter-of-year component for structured financial periods.
///
/// Standalone serde emits the same canonical string as [`std::fmt::Display`].
/// Deserialization also accepts integer quarters for compatibility and
/// normalizes them on the next serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QuarterOfYear(u8);

impl QuarterOfYear {
    /// First quarter.
    pub const Q1: Self = Self(1);

    /// Second quarter.
    pub const Q2: Self = Self(2);

    /// Third quarter.
    pub const Q3: Self = Self(3);

    /// Fourth quarter.
    pub const Q4: Self = Self(4);

    /// Smallest valid quarter number.
    pub const MIN: u8 = 1;

    /// Largest valid quarter number.
    pub const MAX: u8 = 4;

    /// Builds a validated quarter-of-year.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodQuarter`] when `quarter` is outside
    /// `1..=4`.
    pub const fn new(quarter: u8) -> Result<Self, DomainError> {
        if quarter >= Self::MIN && quarter <= Self::MAX {
            Ok(Self(quarter))
        } else {
            Err(DomainError::InvalidPeriodQuarter { quarter })
        }
    }

    /// Returns the quarter number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for QuarterOfYear {
    type Error = DomainError;

    fn try_from(quarter: u8) -> Result<Self, Self::Error> {
        Self::new(quarter)
    }
}

impl From<QuarterOfYear> for u8 {
    fn from(quarter: QuarterOfYear) -> Self {
        quarter.get()
    }
}

impl fmt::Display for QuarterOfYear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for QuarterOfYear {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for QuarterOfYear {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(QuarterOfYearVisitor)
    }
}

struct QuarterOfYearVisitor;

impl Visitor<'_> for QuarterOfYearVisitor {
    type Value = QuarterOfYear;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a canonical quarter string or integer in 1..=4")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        parse_quarter_of_year_code(value).map_err(DeError::custom)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        quarter_of_year_from_i64(value).map_err(DeError::custom)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        quarter_of_year_from_u64(value).map_err(DeError::custom)
    }
}

paft_core::other_string_code_type!(
    /// Provider-specific period token that is not modeled by [`ReportingPeriod`].
    pub struct OtherPeriod for ReportingPeriod;
    type Error = DomainError;
    parse(input) => input.parse::<ReportingPeriod>();
    invalid(input) => DomainError::InvalidPeriodFormat {
        format: input.to_string(),
    };
);

/// Reporting or fiscal period label with structured variants and extensible fallback.
///
/// `ReportingPeriod` models labels reported by issuers, analysts, or providers:
/// `2023Q4`, `FY2023`, `2023-12-31`, and provider-specific ranges are labels,
/// not calendar boundary claims. A fiscal `2023Q4` may not overlap calendar Q4.
/// Use [`CalendarPeriod`] when you need date boundary helpers.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII where applicable)
/// - Parser accepts a superset of tokens (aliases, case-insensitive where appropriate)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix)
/// - `Display` output matches the canonical form for structured variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
///
/// Canonical outputs:
/// - Quarters: `YYYYQ#` (e.g., `2023Q4`)
/// - Years: `YYYY` (e.g., `2023`)
/// - Dates: `YYYY-MM-DD` (ISO 8601)
/// - `Other` stores and emits `canonicalize`-style tokens
///
/// `Display` and serde always emit the canonical forms listed above. The parser
/// accepts common provider variants (e.g., `FY2023`, `2023-Q4`, `12/31/2023`) and
/// normalizes to the single canonical emission for round-trip stability.
///
/// `ReportingPeriod` intentionally does not implement `Ord` or date-boundary
/// helpers: cross-granularity ordering needs caller-chosen semantics (fiscal
/// calendar, exact date, provider-specific `Other`, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReportingPeriod {
    /// Quarterly period with year and quarter number
    Quarter {
        /// The year of the quarter
        year: PeriodYear,
        /// The quarter number (1-4)
        quarter: QuarterOfYear,
    },
    /// Annual period with year
    Year {
        /// The year of the annual period
        year: PeriodYear,
    },
    /// Specific date
    Date(
        /// Validated calendar date
        PeriodDate,
    ),
    /// Unknown or provider-specific period format
    Other(OtherPeriod),
}

impl ReportingPeriod {
    /// Builds a validated quarterly period.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodYear`] or
    /// [`DomainError::InvalidPeriodQuarter`] when either component is outside
    /// its accepted range.
    pub fn quarterly(year: i32, quarter: u8) -> Result<Self, DomainError> {
        Ok(Self::Quarter {
            year: PeriodYear::new(year)?,
            quarter: QuarterOfYear::new(quarter)?,
        })
    }

    /// Builds a validated annual period.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodYear`] when `year` is outside
    /// `0..=9999`.
    pub fn annual(year: i32) -> Result<Self, DomainError> {
        Ok(Self::Year {
            year: PeriodYear::new(year)?,
        })
    }

    /// Builds a validated date period.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodYear`] when `date.year()` is
    /// outside `0..=9999`.
    pub fn date(date: NaiveDate) -> Result<Self, DomainError> {
        Ok(Self::Date(PeriodDate::new(date)?))
    }

    /// Builds an unknown period token, rejecting tokens modeled by [`ReportingPeriod`].
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is empty, cannot be canonicalized, parses to
    /// a modeled [`ReportingPeriod`] variant, or matches a supported structured
    /// period shape with invalid components.
    ///
    /// Partial modeled-looking provider labels that do not match a supported
    /// structured parser, such as `FY`, may still be accepted as
    /// [`ReportingPeriod::Other`].
    pub fn other(input: &str) -> Result<Self, DomainError> {
        OtherPeriod::new(input).map(Self::Other)
    }

    /// Returns the canonical display/serde code for this period.
    #[must_use]
    pub fn code(&self) -> Cow<'_, str> {
        match self {
            Self::Quarter { year, quarter } => Cow::Owned(format!("{year}Q{quarter}")),
            Self::Year { year } => Cow::Owned(year.to_string()),
            Self::Date(date) => Cow::Owned(date.to_string()),
            Self::Other(s) => Cow::Borrowed(s.as_ref()),
        }
    }
    /// Returns the year for this period, if applicable
    #[must_use]
    pub const fn year(&self) -> Option<i32> {
        match self {
            Self::Quarter { year, .. } | Self::Year { year } => Some(year.get()),
            _ => None,
        }
    }

    /// Returns the validated year component for this period, if applicable.
    #[must_use]
    pub const fn period_year(&self) -> Option<PeriodYear> {
        match self {
            Self::Quarter { year, .. } | Self::Year { year } => Some(*year),
            _ => None,
        }
    }

    /// Returns the quarter number for quarterly periods
    #[must_use]
    pub const fn quarter(&self) -> Option<u8> {
        match self {
            Self::Quarter { quarter, .. } => Some(quarter.get()),
            _ => None,
        }
    }

    /// Returns the validated quarter component for quarterly periods.
    #[must_use]
    pub const fn quarter_of_year(&self) -> Option<QuarterOfYear> {
        match self {
            Self::Quarter { quarter, .. } => Some(*quarter),
            _ => None,
        }
    }

    /// Returns true if this is a quarterly period
    #[must_use]
    pub const fn is_quarterly(&self) -> bool {
        matches!(self, Self::Quarter { .. })
    }

    /// Returns true if this is an annual period
    #[must_use]
    pub const fn is_annual(&self) -> bool {
        matches!(self, Self::Year { .. })
    }

    /// Returns true if this is a specific date period
    #[must_use]
    pub const fn is_date(&self) -> bool {
        matches!(self, Self::Date(_))
    }
}

/// Calendar period bucket with date-boundary helpers.
///
/// `CalendarPeriod` is closed over actual calendar years, quarters, and dates.
/// It intentionally has no provider-specific `Other` variant and rejects fiscal
/// aliases such as `FY2023`; use [`ReportingPeriod`] for fiscal/provider labels.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CalendarPeriod {
    /// Calendar quarter with year and quarter number.
    Quarter {
        /// The calendar year of the quarter.
        year: PeriodYear,
        /// The quarter number (1-4).
        quarter: QuarterOfYear,
    },
    /// Calendar year.
    Year {
        /// The calendar year.
        year: PeriodYear,
    },
    /// Specific calendar date.
    Date(
        /// Validated calendar date.
        PeriodDate,
    ),
}

impl CalendarPeriod {
    /// Builds a validated calendar quarter.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodYear`] or
    /// [`DomainError::InvalidPeriodQuarter`] when either component is outside
    /// its accepted range.
    pub fn quarterly(year: i32, quarter: u8) -> Result<Self, DomainError> {
        Ok(Self::Quarter {
            year: PeriodYear::new(year)?,
            quarter: QuarterOfYear::new(quarter)?,
        })
    }

    /// Builds a validated calendar year.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodYear`] when `year` is outside
    /// `0..=9999`.
    pub fn annual(year: i32) -> Result<Self, DomainError> {
        Ok(Self::Year {
            year: PeriodYear::new(year)?,
        })
    }

    /// Builds a validated calendar date.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPeriodYear`] when `date.year()` is
    /// outside `0..=9999`.
    pub fn date(date: NaiveDate) -> Result<Self, DomainError> {
        Ok(Self::Date(PeriodDate::new(date)?))
    }

    /// Returns the canonical display/serde code for this calendar period.
    #[must_use]
    pub fn code(&self) -> Cow<'_, str> {
        match self {
            Self::Quarter { year, quarter } => Cow::Owned(format!("{year}Q{quarter}")),
            Self::Year { year } => Cow::Owned(year.to_string()),
            Self::Date(date) => Cow::Owned(date.to_string()),
        }
    }

    /// Returns the year for this calendar period, if applicable.
    #[must_use]
    pub const fn year(&self) -> Option<i32> {
        match self {
            Self::Quarter { year, .. } | Self::Year { year } => Some(year.get()),
            Self::Date(_) => None,
        }
    }

    /// Returns the validated year component for this calendar period, if applicable.
    #[must_use]
    pub const fn period_year(&self) -> Option<PeriodYear> {
        match self {
            Self::Quarter { year, .. } | Self::Year { year } => Some(*year),
            Self::Date(_) => None,
        }
    }

    /// Returns the quarter number for calendar quarters.
    #[must_use]
    pub const fn quarter(&self) -> Option<u8> {
        match self {
            Self::Quarter { quarter, .. } => Some(quarter.get()),
            Self::Year { .. } | Self::Date(_) => None,
        }
    }

    /// Returns the validated quarter component for calendar quarters.
    #[must_use]
    pub const fn quarter_of_year(&self) -> Option<QuarterOfYear> {
        match self {
            Self::Quarter { quarter, .. } => Some(*quarter),
            Self::Year { .. } | Self::Date(_) => None,
        }
    }

    /// Returns true if this is a calendar quarter.
    #[must_use]
    pub const fn is_quarterly(&self) -> bool {
        matches!(self, Self::Quarter { .. })
    }

    /// Returns true if this is a calendar year.
    #[must_use]
    pub const fn is_annual(&self) -> bool {
        matches!(self, Self::Year { .. })
    }

    /// Returns true if this is a specific calendar date.
    #[must_use]
    pub const fn is_date(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    /// Returns the next chronological quarter bucket after this calendar period.
    ///
    /// - For `Date`, computes the quarter containing the date, then returns the next quarter.
    /// - For `Quarter`, returns the next quarter (wrapping to Q1 of the next year).
    /// - For `Year`, returns `Q1` of the next year.
    #[must_use]
    pub fn next_quarter(&self) -> Option<Self> {
        match self {
            Self::Date(d) => {
                let (year, quarter) = quarter_for_date(d.get())?;
                let (year, quarter) = increment_quarter(year, quarter)?;
                Some(Self::Quarter { year, quarter })
            }
            Self::Quarter { year, quarter } => {
                let (year, quarter) = increment_quarter(*year, *quarter)?;
                Some(Self::Quarter { year, quarter })
            }
            Self::Year { year } => {
                let next_year = PeriodYear::new(year.get() + 1).ok()?;
                Some(Self::Quarter {
                    year: next_year,
                    quarter: QuarterOfYear::Q1,
                })
            }
        }
    }

    /// Returns the last calendar date of the year this period belongs to.
    ///
    /// - For `Date`, uses the date's year.
    /// - For `Quarter`, uses the quarter's calendar year.
    /// - For `Year`, uses that year.
    #[must_use]
    pub fn year_end(&self) -> NaiveDate {
        let y = match self {
            Self::Date(d) => d.get().year(),
            Self::Quarter { year, .. } | Self::Year { year } => year.get(),
        };
        expect_valid_date(y, 12, 31)
    }

    /// Returns the first calendar date covered by this period.
    ///
    /// - For `Date`, returns the date itself.
    /// - For `Quarter`, returns the first day of that calendar quarter.
    /// - For `Year`, returns January 1 of that year.
    #[must_use]
    pub const fn start_date(&self) -> NaiveDate {
        match self {
            Self::Date(d) => d.get(),
            Self::Quarter { year, quarter } => {
                let month = match quarter.get() {
                    1 => 1,
                    2 => 4,
                    3 => 7,
                    4 => 10,
                    _ => unreachable!(),
                };
                expect_valid_date(year.get(), month, 1)
            }
            Self::Year { year } => expect_valid_date(year.get(), 1, 1),
        }
    }

    /// Returns the last calendar date covered by this period.
    ///
    /// - For `Date`, returns the date itself.
    /// - For `Quarter`, returns the last day of that calendar quarter.
    /// - For `Year`, returns December 31 of that year.
    #[must_use]
    pub const fn end_date(&self) -> NaiveDate {
        match self {
            Self::Date(d) => d.get(),
            Self::Quarter { year, quarter } => {
                let (month, day) = match quarter.get() {
                    1 => (3, 31),
                    2 => (6, 30),
                    3 => (9, 30),
                    4 => (12, 31),
                    _ => unreachable!(),
                };
                expect_valid_date(year.get(), month, day)
            }
            Self::Year { year } => expect_valid_date(year.get(), 12, 31),
        }
    }

    /// Returns true if this calendar period overlaps `other`.
    ///
    /// Calendar periods are closed ranges over dates, so adjacent quarters do
    /// not overlap, while a year overlaps every quarter and date inside that
    /// calendar year.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start_date() <= other.end_date() && other.start_date() <= self.end_date()
    }

    /// Returns true if this calendar period fully contains `other`.
    ///
    /// Containment is directional: a year contains its quarters and dates, but
    /// a quarter or date does not contain the year.
    #[must_use]
    pub fn contains(&self, other: &Self) -> bool {
        self.start_date() <= other.start_date() && self.end_date() >= other.end_date()
    }

    /// Returns true if both values are the same exact calendar bucket.
    ///
    /// Cross-granularity containment is not an exact match: a calendar year
    /// and one of its quarters overlap, but they are not the same bucket.
    #[must_use]
    pub fn is_same_exact_bucket_as(&self, other: &Self) -> bool {
        self == other
    }
}

// Per-format parser results.
//
// `Some(Ok(p))` means the input fully matched the format and produced a valid
// `ReportingPeriod`. `Some(Err(()))` means the input matched the format structurally
// (i.e., the original regex would have matched) but the captured values were
// invalid (e.g., `2023Q5`, `2023-13-01`); the caller treats this as
// `InvalidPeriodFormat`. `None` means the input does not match this format
// and the caller should try the next one.
type ReportingPeriodAttempt = Option<Result<ReportingPeriod, ()>>;

#[inline]
const fn expect_valid_date(year: i32, month: u32, day: u32) -> NaiveDate {
    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
        unreachable!();
    };
    date
}

#[inline]
fn read_4_digits(b: &[u8], start: usize) -> Option<i32> {
    if start + 4 > b.len() {
        return None;
    }
    let mut v: i32 = 0;
    for &c in &b[start..start + 4] {
        if !c.is_ascii_digit() {
            return None;
        }
        v = v * 10 + i32::from(c - b'0');
    }
    Some(v)
}

#[inline]
fn read_1_or_2_digits(b: &[u8], start: usize) -> Option<(u32, usize)> {
    let &first = b.get(start)?;
    if !first.is_ascii_digit() {
        return None;
    }
    let d1 = u32::from(first - b'0');
    if let Some(&second) = b.get(start + 1)
        && second.is_ascii_digit()
    {
        Some((d1 * 10 + u32::from(second - b'0'), 2))
    } else {
        Some((d1, 1))
    }
}

#[inline]
fn date_or_err(year: i32, month: u32, day: u32) -> Result<ReportingPeriod, ()> {
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or(())
        .and_then(|date| ReportingPeriod::date(date).map_err(|_| ()))
}

fn calendar_year(s: &str) -> Option<PeriodYear> {
    let b = s.as_bytes();
    if b.len() != 4 {
        return None;
    }
    PeriodYear::new(read_4_digits(b, 0)?).ok()
}

fn quarter_for_date(d: NaiveDate) -> Option<(PeriodYear, QuarterOfYear)> {
    let year = PeriodYear::new(d.year()).ok()?;
    let m = d.month();
    let quarter = match m {
        1..=3 => QuarterOfYear::Q1,
        4..=6 => QuarterOfYear::Q2,
        7..=9 => QuarterOfYear::Q3,
        _ => QuarterOfYear::Q4,
    };
    Some((year, quarter))
}

fn increment_quarter(
    year: PeriodYear,
    quarter: QuarterOfYear,
) -> Option<(PeriodYear, QuarterOfYear)> {
    if quarter.get() < QuarterOfYear::MAX {
        let next_quarter = QuarterOfYear::new(quarter.get() + 1).ok()?;
        Some((year, next_quarter))
    } else {
        let next_year = PeriodYear::new(year.get() + 1).ok()?;
        Some((next_year, QuarterOfYear::Q1))
    }
}

impl ReportingPeriod {
    /// Parse quarterly period format: "2023Q4", "2023-Q4", "2023 Q4",
    /// "2023  Q4", "2023\tQ4", "2023 \t Q4".
    fn parse_quarterly(s: &str) -> ReportingPeriodAttempt {
        let b = s.as_bytes();
        // Minimum form is `YYYYQ#` (6 bytes).
        if b.len() < 6 {
            return None;
        }

        let year = PeriodYear::new(read_4_digits(b, 0)?).ok()?;
        let mut idx = 4;

        // Optional separator between the year and the `Q`:
        //   - a single `-`, or
        //   - a run of ASCII whitespace (matches `parse_year`'s "Fiscal "
        //     handling — `is_ascii_whitespace` covers space, tab, CR, LF and
        //     form feed but, importantly, no Unicode whitespace).
        // The two forms are mutually exclusive: we don't mix `-` with spaces.
        if b[idx] == b'-' {
            idx += 1;
        } else {
            while idx < b.len() && b[idx].is_ascii_whitespace() {
                idx += 1;
            }
        }
        if idx >= b.len() {
            return None;
        }

        // Case-insensitive 'Q'.
        if b[idx] != b'Q' && b[idx] != b'q' {
            return None;
        }
        idx += 1;

        let q_bytes = b.get(idx..)?;
        if q_bytes.is_empty() {
            return None;
        }

        // Valid quarters are always exactly one digit. Multi-digit runs of
        // digits structurally match the original `Q\d+` regex but are
        // out-of-range, so they're a structural-only match (caller turns into
        // `InvalidPeriodFormat`). A multi-byte tail with any non-digit is
        // simply not a quarterly token at all.
        if q_bytes.len() > 1 {
            return q_bytes.iter().all(u8::is_ascii_digit).then_some(Err(()));
        }

        let c = q_bytes[0];
        if !c.is_ascii_digit() {
            return None;
        }
        let quarter = c - b'0';
        let Ok(quarter) = QuarterOfYear::new(quarter) else {
            return Some(Err(()));
        };

        Some(Ok(Self::Quarter { year, quarter }))
    }

    /// Parse year period format: "2023", "FY2023", "Fiscal 2023".
    fn parse_year(s: &str) -> Option<Self> {
        let b = s.as_bytes();
        let digits_start = match b.len() {
            4 => 0,
            6 if b[..2].eq_ignore_ascii_case(b"FY") => 2,
            n if n >= 11 && b[..6].eq_ignore_ascii_case(b"FISCAL") => {
                let mut i = 6;
                while i < n && b[i].is_ascii_whitespace() {
                    i += 1;
                }
                if i == 6 {
                    return None;
                }
                i
            }
            _ => return None,
        };

        if b.len() - digits_start != 4 {
            return None;
        }
        let year = PeriodYear::new(read_4_digits(b, digits_start)?).ok()?;
        Some(Self::Year { year })
    }

    /// Parse date period: ISO `YYYY[-/]M[M][-/]D[D]`, US `M[M]/D[D]/YYYY`,
    /// or day-first `D[D]-M[M]-YYYY`.
    fn parse_date(s: &str) -> ReportingPeriodAttempt {
        let b = s.as_bytes();
        if !(8..=10).contains(&b.len()) {
            return None;
        }

        // ISO: `YYYY[-/]M[M][-/]D[D]`.
        if let Some(year) = read_4_digits(b, 0)
            && (b[4] == b'-' || b[4] == b'/')
        {
            let sep = b[4];
            let (month, m_len) = read_1_or_2_digits(b, 5)?;
            let after_m = 5 + m_len;
            if b.get(after_m).copied() == Some(sep) {
                let (day, d_len) = read_1_or_2_digits(b, after_m + 1)?;
                if after_m + 1 + d_len == b.len() {
                    return Some(date_or_err(year, month, day));
                }
            }
            // Leading `YYYY[-/]` cannot match the US or day-first shapes
            // (those need 1-2 digits before the first separator), so a
            // partial ISO match means no date format matches.
            return None;
        }

        // US (`/`-separated, year last) and day-first (`-`-separated, year
        // last) share a common prefix of 1-2 digits + separator + 1-2 digits
        // + same separator + 4-digit year.
        let (first, first_len) = read_1_or_2_digits(b, 0)?;
        let sep = *b.get(first_len)?;
        if sep != b'/' && sep != b'-' {
            return None;
        }

        let (second, second_len) = read_1_or_2_digits(b, first_len + 1)?;
        let after_second = first_len + 1 + second_len;
        if b.get(after_second).copied() != Some(sep) {
            return None;
        }

        let year_start = after_second + 1;
        if b.len() - year_start != 4 {
            return None;
        }
        let year = read_4_digits(b, year_start)?;

        let (month, day) = if sep == b'/' {
            (first, second)
        } else {
            (second, first)
        };

        Some(date_or_err(year, month, day))
    }
}

impl From<ReportingPeriod> for String {
    fn from(val: ReportingPeriod) -> Self {
        val.code().into_owned()
    }
}

impl fmt::Display for ReportingPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.code())
    }
}

impl Serialize for ReportingPeriod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.code())
    }
}

impl<'de> Deserialize<'de> for ReportingPeriod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse::<Self>().map_err(DeError::custom)
    }
}

impl std::str::FromStr for ReportingPeriod {
    type Err = DomainError;

    /// Invariant: the canonical form of a `ReportingPeriod::Other` produced here is
    /// guaranteed not to parse as any structured variant on a subsequent
    /// deserialize. Without this, inputs like `"-2023Q4"` (rejected by the
    /// structured parsers because of the leading `-`) would canonicalize to
    /// `"2023Q4"` and serialize back to a string that re-parses as
    /// `ReportingPeriod::Quarter`, breaking round-trip identity.
    ///
    /// To maintain the invariant without accepting malformed aliases, we
    /// re-run the structured parsers on the canonicalized form before
    /// returning `Other`. If any parser recognizes the canonical form, we
    /// return `InvalidPeriodFormat`; otherwise a malformed input such as
    /// `"-2023Q4"` would silently become `ReportingPeriod::Quarter`.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(DomainError::InvalidPeriodFormat {
                format: s.to_string(),
            });
        }

        let invalid = || DomainError::InvalidPeriodFormat {
            format: s.to_string(),
        };

        match Self::parse_quarterly(trimmed) {
            Some(Ok(period)) => return Ok(period),
            Some(Err(())) => return Err(invalid()),
            None => {}
        }

        if let Some(period) = Self::parse_year(trimmed) {
            return Ok(period);
        }

        match Self::parse_date(trimmed) {
            Some(Ok(period)) => return Ok(period),
            Some(Err(())) => return Err(invalid()),
            None => {}
        }

        let canonical = Canonical::try_new(trimmed).map_err(|_| invalid())?;

        // Re-run the structured parsers against the canonical token. Any
        // structured match is rejected: supported aliases have already matched
        // above, so reaching this point means canonicalization would otherwise
        // convert a malformed spelling into a modeled value.
        let canonical_str = canonical.as_ref();
        if Self::parse_quarterly(canonical_str).is_some() {
            return Err(invalid());
        }
        if Self::parse_year(canonical_str).is_some() {
            return Err(invalid());
        }
        if Self::parse_date(canonical_str).is_some() {
            return Err(invalid());
        }

        Ok(Self::Other(OtherPeriod::from_canonical_unchecked(
            canonical,
        )))
    }
}

impl TryFrom<String> for ReportingPeriod {
    type Error = DomainError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.as_str().parse()
    }
}

impl TryFrom<ReportingPeriod> for CalendarPeriod {
    type Error = DomainError;

    fn try_from(period: ReportingPeriod) -> Result<Self, Self::Error> {
        match period {
            ReportingPeriod::Quarter { year, quarter } => Ok(Self::Quarter { year, quarter }),
            ReportingPeriod::Year { year } => Ok(Self::Year { year }),
            ReportingPeriod::Date(date) => Ok(Self::Date(date)),
            ReportingPeriod::Other(other) => Err(DomainError::InvalidPeriodFormat {
                format: other.to_string(),
            }),
        }
    }
}

impl From<CalendarPeriod> for ReportingPeriod {
    fn from(period: CalendarPeriod) -> Self {
        match period {
            CalendarPeriod::Quarter { year, quarter } => Self::Quarter { year, quarter },
            CalendarPeriod::Year { year } => Self::Year { year },
            CalendarPeriod::Date(date) => Self::Date(date),
        }
    }
}

impl From<CalendarPeriod> for String {
    fn from(val: CalendarPeriod) -> Self {
        val.code().into_owned()
    }
}

impl fmt::Display for CalendarPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.code())
    }
}

impl Serialize for CalendarPeriod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.code())
    }
}

impl<'de> Deserialize<'de> for CalendarPeriod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse::<Self>().map_err(DeError::custom)
    }
}

impl std::str::FromStr for CalendarPeriod {
    type Err = DomainError;

    /// Parses calendar-only period tokens.
    ///
    /// Unlike [`ReportingPeriod`], this parser rejects fiscal aliases such as
    /// `FY2023` and unknown provider-specific labels.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(DomainError::InvalidPeriodFormat {
                format: s.to_string(),
            });
        }

        let invalid = || DomainError::InvalidPeriodFormat {
            format: s.to_string(),
        };

        match ReportingPeriod::parse_quarterly(trimmed) {
            Some(Ok(ReportingPeriod::Quarter { year, quarter })) => {
                return Ok(Self::Quarter { year, quarter });
            }
            Some(Ok(_)) => unreachable!("quarter parser only emits quarter periods"),
            Some(Err(())) => return Err(invalid()),
            None => {}
        }

        if let Some(year) = calendar_year(trimmed) {
            return Ok(Self::Year { year });
        }

        match ReportingPeriod::parse_date(trimmed) {
            Some(Ok(ReportingPeriod::Date(date))) => return Ok(Self::Date(date)),
            Some(Ok(_)) => unreachable!("date parser only emits date periods"),
            Some(Err(())) => return Err(invalid()),
            None => {}
        }

        Err(invalid())
    }
}

impl TryFrom<String> for CalendarPeriod {
    type Error = DomainError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.as_str().parse()
    }
}
