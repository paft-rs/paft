//! Financial period primitives.
//!
//! Provides a structured `Period` type with parsing/formatting helpers and an
//! extensible fallback variant.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;

use crate::error::DomainError;
use chrono::{Datelike, NaiveDate};
use paft_utils::Canonical;

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
    /// Internal: establish a stable ordering precedence across variants.
    /// Date < Quarter < Year < Other
    const fn type_rank(&self) -> u8 {
        match self {
            Self::Date(_) => 0,
            Self::Quarter { .. } => 1,
            Self::Year { .. } => 2,
            Self::Other(_) => 3,
        }
    }

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

    /// Returns the next chronological quarter bucket after this period, if applicable.
    ///
    /// - For `Date`, computes the quarter containing the date, then returns the next quarter.
    /// - For `Quarter`, returns the next quarter (wrapping to Q1 of the next year).
    /// - For `Year`, returns `Q1` of the next year.
    /// - For `Other`, returns `None`.
    #[must_use]
    pub fn next_quarter(&self) -> Option<Self> {
        match self {
            Self::Date(d) => {
                let (y, q) = Self::quarter_for_date(*d);
                let (ny, nq) = Self::increment_quarter(y, q);
                Some(Self::Quarter {
                    year: ny,
                    quarter: nq,
                })
            }
            Self::Quarter { year, quarter } => {
                let (ny, nq) = Self::increment_quarter(*year, *quarter);
                Some(Self::Quarter {
                    year: ny,
                    quarter: nq,
                })
            }
            Self::Year { year } => Some(Self::Quarter {
                year: *year + 1,
                quarter: 1,
            }),
            Self::Other(_) => None,
        }
    }

    /// Returns the last calendar date of the year this period belongs to.
    ///
    /// - For `Date`, uses the date's year
    /// - For `Quarter`, uses the quarter's year
    /// - For `Year`, uses that year
    /// - For `Other`, returns `None`
    #[must_use]
    pub fn year_end(&self) -> Option<NaiveDate> {
        let y = match self {
            Self::Date(d) => d.year(),
            Self::Quarter { year, .. } | Self::Year { year } => *year,
            Self::Other(_) => return None,
        };
        NaiveDate::from_ymd_opt(y, 12, 31)
    }

    /// Returns true if both values describe the same time bucket.
    ///
    /// Cross-variant rules:
    /// - Year vs Date: true if date.year == year
    /// - Year vs Quarter: true if quarter.year == year
    /// - Quarter vs Date: true if date falls within that quarter of that year
    /// - Other vs Other: true if canonical strings match
    /// - Otherwise, same-variant exact equality
    #[must_use]
    pub fn is_same_bucket_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Year { year: ay }, Self::Year { year: by }) => ay == by,
            (Self::Year { year }, Self::Date(d)) | (Self::Date(d), Self::Year { year }) => {
                d.year() == *year
            }
            (Self::Year { year }, Self::Quarter { year: qy, .. })
            | (Self::Quarter { year: qy, .. }, Self::Year { year }) => qy == year,

            (
                Self::Quarter {
                    year: ay,
                    quarter: aq,
                },
                Self::Quarter {
                    year: by,
                    quarter: bq,
                },
            ) => ay == by && aq == bq,
            (Self::Quarter { year, quarter }, Self::Date(d))
            | (Self::Date(d), Self::Quarter { year, quarter }) => {
                let (dy, dq) = Self::quarter_for_date(*d);
                dy == *year && dq == *quarter
            }

            (Self::Date(a), Self::Date(b)) => a == b,
            (Self::Other(a), Self::Other(b)) => a.as_ref() == b.as_ref(),
            _ => false,
        }
    }

    fn quarter_for_date(d: NaiveDate) -> (i32, u8) {
        let y = d.year();
        let m = d.month();
        let q = match m {
            1..=3 => 1,
            4..=6 => 2,
            7..=9 => 3,
            _ => 4,
        };
        (y, q)
    }

    const fn increment_quarter(year: i32, quarter: u8) -> (i32, u8) {
        if quarter < 4 {
            (year, quarter + 1)
        } else {
            (year + 1, 1)
        }
    }
}

// Per-format parser results.
//
// `Some(Ok(p))` means the input fully matched the format and produced a valid
// `Period`. `Some(Err(()))` means the input matched the format structurally
// (i.e., the original regex would have matched) but the captured values were
// invalid (e.g., `2023Q5`, `2023-13-01`); the caller treats this as
// `InvalidPeriodFormat`. `None` means the input does not match this format
// and the caller should try the next one.
type PeriodAttempt = Option<Result<Period, ()>>;

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
fn date_or_err(year: i32, month: u32, day: u32) -> Result<Period, ()> {
    NaiveDate::from_ymd_opt(year, month, day)
        .map(Period::Date)
        .ok_or(())
}

impl Period {
    /// Parse quarterly period format: "2023Q4", "2023-Q4", "2023 Q4".
    fn parse_quarterly(s: &str) -> PeriodAttempt {
        let b = s.as_bytes();
        // Minimum form is `YYYYQ#` (6 bytes).
        if b.len() < 6 {
            return None;
        }

        let year = read_4_digits(b, 0)?;
        let mut idx = 4;

        // Optional single ASCII separator: '-' or whitespace.
        if b[idx] == b'-' || b[idx].is_ascii_whitespace() {
            idx += 1;
            if idx >= b.len() {
                return None;
            }
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
        match quarter {
            1..=4 => Some(Ok(Self::Quarter { year, quarter })),
            _ => Some(Err(())),
        }
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
        let year = read_4_digits(b, digits_start)?;
        Some(Self::Year { year })
    }

    /// Parse date period: ISO `YYYY[-/]M[M][-/]D[D]`, US `M[M]/D[D]/YYYY`,
    /// or day-first `D[D]-M[M]-YYYY`.
    fn parse_date(s: &str) -> PeriodAttempt {
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

impl From<Period> for String {
    fn from(val: Period) -> Self {
        val.code().into_owned()
    }
}

impl Ord for Period {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.type_rank().cmp(&other.type_rank()) {
            Ordering::Equal => match (self, other) {
                (Self::Date(a), Self::Date(b)) => a.cmp(b),
                (
                    Self::Quarter {
                        year: ay,
                        quarter: aq,
                    },
                    Self::Quarter {
                        year: by,
                        quarter: bq,
                    },
                ) => (ay, aq).cmp(&(by, bq)),
                (Self::Year { year: ay }, Self::Year { year: by }) => ay.cmp(by),
                (Self::Other(a), Self::Other(b)) => a.as_ref().cmp(b.as_ref()),
                _ => Ordering::Equal,
            },
            ord => ord,
        }
    }
}

impl PartialOrd for Period {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    type Err = DomainError;

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
        Ok(Self::Other(canonical))
    }
}

impl TryFrom<String> for Period {
    type Error = DomainError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.as_str().parse()
    }
}
