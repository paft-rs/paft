//! Financial period primitives.
//!
//! Provides a structured `Period` type with parsing/formatting helpers and an
//! extensible fallback variant.

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};
use std::borrow::Cow;
use std::fmt;

use super::string_canonical::Canonical;
use crate::error::PaftError;
use chrono::NaiveDate;

// Compile-time compiled regex patterns for Period parsing
static QUARTERLY_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?i)^(\d{4})[-\s]?Q(\d+)$").expect("Invalid quarterly regex pattern")
});

static YEAR_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?i)^(?:FY|FISCAL\s+)?(\d{4})$").expect("Invalid year regex pattern")
});

static DATE_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^(\d{4})[-/](\d{1,2})[-/](\d{1,2})$").expect("Invalid date regex pattern")
});

static US_DATE_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{4})$").expect("Invalid US date regex pattern")
});

static DAY_FIRST_DATE_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^(\d{1,2})-(\d{1,2})-(\d{4})$").expect("Invalid day-first date regex pattern")
});

/// Financial period enumeration with structured variants and extensible fallback.
///
/// This enum provides type-safe handling of financial periods while gracefully
/// handling unknown or provider-specific period formats through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII where applicable)
/// - Parser accepts a superset of tokens (aliases, case-insensitive where appropriate)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix) and must be non-empty
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Period {
    /// Quarterly period with year and quarter number
    Quarter {
        /// The year of the quarter
        year: i32,
        /// The quarter number (1-4)
        quarter: u8,
    },
    /// Annual period with year
    Year {
        /// The year of the annual period
        year: i32,
    },
    /// Specific date
    Date(
        /// Calendar date (UTC newline-free)
        NaiveDate,
    ),
    /// Unknown or provider-specific period format
    Other(Canonical),
}

impl Period {
    /// Returns the canonical display/serde code for this period.
    #[must_use]
    pub fn code(&self) -> Cow<'_, str> {
        match self {
            Self::Quarter { year, quarter } => Cow::Owned(format!("{year}Q{quarter}")),
            Self::Year { year } => Cow::Owned(year.to_string()),
            Self::Date(date) => Cow::Owned(date.format("%Y-%m-%d").to_string()),
            Self::Other(s) => Cow::Borrowed(s.as_ref()),
        }
    }
    /// Returns the year for this period, if applicable
    #[must_use]
    pub const fn year(&self) -> Option<i32> {
        match self {
            Self::Quarter { year, .. } | Self::Year { year } => Some(*year),
            _ => None,
        }
    }

    /// Returns the quarter number for quarterly periods
    #[must_use]
    pub const fn quarter(&self) -> Option<u8> {
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

impl Period {
    /// Parse quarterly period format: "2023Q4", "2023-Q4", "2023 Q4"
    fn parse_quarterly(s: &str) -> Option<Self> {
        let captures = QUARTERLY_REGEX.captures(s)?;
        let year_str = &captures[1];
        let quarter_str = &captures[2];

        let year = year_str.parse::<i32>().ok()?;
        let quarter = quarter_str.parse::<u8>().ok()?;

        // Validate quarter is between 1-4
        if (1..=4).contains(&quarter) {
            Some(Self::Quarter { year, quarter })
        } else {
            None
        }
    }

    /// Parse year period format: "2023", "FY2023", "Fiscal 2023"
    fn parse_year(s: &str) -> Option<Self> {
        let captures = YEAR_REGEX.captures(s)?;
        let year_str = &captures[1];

        let year = year_str.parse::<i32>().ok()?;
        Some(Self::Year { year })
    }

    /// Parse date period format: "2023-12-31", "12/31/2023", "31-12-2023"
    fn parse_date(s: &str) -> Option<Self> {
        // Try ISO date format first: "2023-12-31"
        if let Some(captures) = DATE_REGEX.captures(s) {
            let year_str = &captures[1];
            let month_str = &captures[2];
            let day_str = &captures[3];

            let year = year_str.parse::<i32>().ok()?;
            let month = month_str.parse::<u32>().ok()?;
            let day = day_str.parse::<u32>().ok()?;

            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Some(Self::Date(date));
            }
        }

        // Try US date format: "12/31/2023"
        if let Some(captures) = US_DATE_REGEX.captures(s) {
            let month_str = &captures[1];
            let day_str = &captures[2];
            let year_str = &captures[3];

            let month = month_str.parse::<u32>().ok()?;
            let day = day_str.parse::<u32>().ok()?;
            let year = year_str.parse::<i32>().ok()?;

            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Some(Self::Date(date));
            }
        }

        // Try day-first format: "31-12-2023"
        if let Some(captures) = DAY_FIRST_DATE_REGEX.captures(s) {
            let day_str = &captures[1];
            let month_str = &captures[2];
            let year_str = &captures[3];

            let day = day_str.parse::<u32>().ok()?;
            let month = month_str.parse::<u32>().ok()?;
            let year = year_str.parse::<i32>().ok()?;

            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Some(Self::Date(date));
            }
        }

        None
    }
}

impl From<Period> for String {
    fn from(val: Period) -> Self {
        val.code().into_owned()
    }
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.code())
    }
}

impl Serialize for Period {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.code())
    }
}

impl<'de> Deserialize<'de> for Period {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse::<Self>().map_err(DeError::custom)
    }
}

impl std::str::FromStr for Period {
    type Err = PaftError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(PaftError::InvalidPeriodFormat {
                format: s.to_string(),
            });
        }

        if let Some(period) = Self::parse_quarterly(trimmed) {
            return Ok(period);
        }

        if let Some(period) = Self::parse_year(trimmed) {
            return Ok(period);
        }

        if let Some(period) = Self::parse_date(trimmed) {
            return Ok(period);
        }

        if QUARTERLY_REGEX.is_match(trimmed)
            || YEAR_REGEX.is_match(trimmed)
            || DATE_REGEX.is_match(trimmed)
            || US_DATE_REGEX.is_match(trimmed)
            || DAY_FIRST_DATE_REGEX.is_match(trimmed)
        {
            return Err(PaftError::InvalidPeriodFormat {
                format: s.to_string(),
            });
        }

        Ok(Self::Other(Canonical::try_new(trimmed)?))
    }
}

impl TryFrom<String> for Period {
    type Error = PaftError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.as_str().parse()
    }
}
