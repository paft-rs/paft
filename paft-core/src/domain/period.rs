//! Financial period primitives.
//!
//! Provides a structured `Period` type with parsing/formatting helpers and an
//! extensible fallback variant.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::PaftError;
use chrono::{DateTime, Utc};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Specific date timestamp
    Date(
        /// Date and time in UTC
        #[serde(with = "chrono::serde::ts_seconds")]
        DateTime<Utc>,
    ),
    /// Unknown or provider-specific period format
    Other(String),
}

impl Period {
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

            if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day)
                && let Some(datetime) = date.and_hms_opt(0, 0, 0)
            {
                return Some(Self::Date(datetime.and_utc()));
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

            if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day)
                && let Some(datetime) = date.and_hms_opt(0, 0, 0)
            {
                return Some(Self::Date(datetime.and_utc()));
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

            if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day)
                && let Some(datetime) = date.and_hms_opt(0, 0, 0)
            {
                return Some(Self::Date(datetime.and_utc()));
            }
        }

        None
    }
}

impl TryFrom<String> for Period {
    type Error = PaftError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim();

        // Try to parse quarterly format
        if let Some(period) = Self::parse_quarterly(trimmed) {
            return Ok(period);
        }

        // Try to parse year format
        if let Some(period) = Self::parse_year(trimmed) {
            return Ok(period);
        }

        // Try to parse date format
        if let Some(period) = Self::parse_date(trimmed) {
            return Ok(period);
        }

        // If no patterns match, check if any pattern matched but had invalid values
        // This helps distinguish between "unknown format" and "invalid values"
        if QUARTERLY_REGEX.is_match(trimmed)
            || YEAR_REGEX.is_match(trimmed)
            || DATE_REGEX.is_match(trimmed)
            || US_DATE_REGEX.is_match(trimmed)
            || DAY_FIRST_DATE_REGEX.is_match(trimmed)
        {
            return Err(PaftError::InvalidPeriodFormat { format: s });
        }

        // If no patterns match, store as Other (unknown format)
        Ok(Self::Other(s.to_uppercase()))
    }
}

impl From<Period> for String {
    fn from(val: Period) -> Self {
        match val {
            Period::Quarter { year, quarter } => format!("{year}Q{quarter}"),
            Period::Year { year } => year.to_string(),
            Period::Date(datetime) => datetime.date_naive().format("%Y-%m-%d").to_string(),
            Period::Other(s) => s,
        }
    }
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.clone().into();
        write!(f, "{s}")
    }
}
