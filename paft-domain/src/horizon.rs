//! Lookback horizon primitives.
//!
//! [`Horizon`] models relative lookback windows such as `7d`, `1mo`, or `1y`.
//! It is intentionally separate from [`crate::ReportingPeriod`], which models
//! fiscal or reporting labels such as `2023Q4`, `2023`, or `2023-12-31`.

use std::borrow::Cow;
use std::fmt;
use std::num::NonZeroU32;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};

use crate::error::DomainError;
use paft_utils::Canonical;

paft_core::other_string_code_type!(
    /// Provider-specific horizon token that is not modeled by [`Horizon`].
    pub struct OtherHorizon for Horizon;
    type Error = DomainError;
    parse(input) => input.parse::<Horizon>();
    invalid(input) => DomainError::InvalidHorizonFormat {
        format: input.to_string(),
    };
);

/// Relative lookback horizon with compact, stable wire codes.
///
/// Modeled horizons serialize as lower-case compact strings:
/// - Days: `"<n>d"`, e.g. `"7d"`
/// - Months: `"<n>mo"`, e.g. `"3mo"`
/// - Years: `"<n>y"`, e.g. `"1y"`
///
/// Unknown provider-specific horizons round-trip through [`Horizon::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Horizon {
    /// A non-zero number of days.
    Days(NonZeroU32),
    /// A non-zero number of months.
    Months(NonZeroU32),
    /// A non-zero number of years.
    Years(NonZeroU32),
    /// Unknown or provider-specific horizon format.
    Other(OtherHorizon),
}

impl Horizon {
    /// Builds a day horizon.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidHorizonCount`] when `days` is zero.
    pub fn days(days: u32) -> Result<Self, DomainError> {
        Ok(Self::Days(nonzero_count(days)?))
    }

    /// Builds a month horizon.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidHorizonCount`] when `months` is zero.
    pub fn months(months: u32) -> Result<Self, DomainError> {
        Ok(Self::Months(nonzero_count(months)?))
    }

    /// Builds a year horizon.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidHorizonCount`] when `years` is zero.
    pub fn years(years: u32) -> Result<Self, DomainError> {
        Ok(Self::Years(nonzero_count(years)?))
    }

    /// Builds an unknown horizon token, rejecting tokens modeled by [`Horizon`].
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`Horizon`] variant.
    ///
    /// Partial modeled-looking provider labels that do not match a supported
    /// horizon parser, such as `7 d`, may still be accepted as
    /// [`Horizon::Other`].
    pub fn other(input: &str) -> Result<Self, DomainError> {
        OtherHorizon::new(input).map(Self::Other)
    }

    /// Returns the canonical display/serde code for this horizon.
    #[must_use]
    pub fn code(&self) -> Cow<'_, str> {
        match self {
            Self::Days(days) => Cow::Owned(format!("{days}d")),
            Self::Months(months) => Cow::Owned(format!("{months}mo")),
            Self::Years(years) => Cow::Owned(format!("{years}y")),
            Self::Other(other) => Cow::Borrowed(other.as_ref()),
        }
    }

    /// Returns true when this value represents a modeled horizon.
    #[must_use]
    pub const fn is_canonical(&self) -> bool {
        !matches!(self, Self::Other(_))
    }

    /// Returns the non-zero horizon count for modeled horizons.
    #[must_use]
    pub const fn count(&self) -> Option<NonZeroU32> {
        match self {
            Self::Days(count) | Self::Months(count) | Self::Years(count) => Some(*count),
            Self::Other(_) => None,
        }
    }

    /// Returns the day count for day horizons.
    #[must_use]
    pub const fn days_count(&self) -> Option<NonZeroU32> {
        match self {
            Self::Days(days) => Some(*days),
            Self::Months(_) | Self::Years(_) | Self::Other(_) => None,
        }
    }

    /// Returns the month count for month horizons.
    #[must_use]
    pub const fn months_count(&self) -> Option<NonZeroU32> {
        match self {
            Self::Months(months) => Some(*months),
            Self::Days(_) | Self::Years(_) | Self::Other(_) => None,
        }
    }

    /// Returns the year count for year horizons.
    #[must_use]
    pub const fn years_count(&self) -> Option<NonZeroU32> {
        match self {
            Self::Years(years) => Some(*years),
            Self::Days(_) | Self::Months(_) | Self::Other(_) => None,
        }
    }
}

fn nonzero_count(count: u32) -> Result<NonZeroU32, DomainError> {
    NonZeroU32::new(count).ok_or(DomainError::InvalidHorizonCount { count })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HorizonUnit {
    Days,
    Months,
    Years,
}

fn parse_modeled(input: &str) -> Option<Result<Horizon, DomainError>> {
    let split_at = input
        .as_bytes()
        .iter()
        .position(|byte| !byte.is_ascii_digit())?;
    if split_at == 0 || split_at == input.len() {
        return None;
    }

    let suffix = &input[split_at..];
    let unit = if suffix.eq_ignore_ascii_case("d")
        || suffix.eq_ignore_ascii_case("day")
        || suffix.eq_ignore_ascii_case("days")
    {
        HorizonUnit::Days
    } else if suffix.eq_ignore_ascii_case("mo")
        || suffix.eq_ignore_ascii_case("mon")
        || suffix.eq_ignore_ascii_case("month")
        || suffix.eq_ignore_ascii_case("months")
    {
        HorizonUnit::Months
    } else if suffix.eq_ignore_ascii_case("y")
        || suffix.eq_ignore_ascii_case("yr")
        || suffix.eq_ignore_ascii_case("yrs")
        || suffix.eq_ignore_ascii_case("year")
        || suffix.eq_ignore_ascii_case("years")
    {
        HorizonUnit::Years
    } else {
        return None;
    };

    let Ok(count) = input[..split_at].parse::<u32>() else {
        return Some(Err(DomainError::InvalidHorizonFormat {
            format: input.to_string(),
        }));
    };

    let count = match nonzero_count(count) {
        Ok(count) => count,
        Err(error) => return Some(Err(error)),
    };

    let horizon = match unit {
        HorizonUnit::Days => Horizon::Days(count),
        HorizonUnit::Months => Horizon::Months(count),
        HorizonUnit::Years => Horizon::Years(count),
    };
    Some(Ok(horizon))
}

impl From<Horizon> for String {
    fn from(value: Horizon) -> Self {
        value.code().into_owned()
    }
}

impl fmt::Display for Horizon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.code())
    }
}

impl Serialize for Horizon {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.code())
    }
}

impl<'de> Deserialize<'de> for Horizon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse::<Self>().map_err(DeError::custom)
    }
}

impl std::str::FromStr for Horizon {
    type Err = DomainError;

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidHorizonFormat {
                format: input.to_string(),
            });
        }

        if let Some(parsed) = parse_modeled(trimmed) {
            return parsed;
        }

        let invalid = || DomainError::InvalidHorizonFormat {
            format: input.to_string(),
        };
        let canonical = Canonical::try_new(trimmed).map_err(|_| invalid())?;

        if parse_modeled(canonical.as_ref()).is_some() {
            return Err(invalid());
        }

        Ok(Self::Other(OtherHorizon::from_canonical_unchecked(
            canonical,
        )))
    }
}

impl TryFrom<String> for Horizon {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}
