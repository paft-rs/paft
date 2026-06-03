//! Decimal newtypes for invariants that hold across providers.

use std::fmt;

use ::serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{Decimal, one, serde::canonical_str, zero};

/// Error returned when a decimal does not satisfy a constrained decimal type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecimalConstraintError {
    type_name: &'static str,
    expected: &'static str,
    value: Decimal,
}

impl DecimalConstraintError {
    #[cfg(not(feature = "bigdecimal"))]
    const fn new(type_name: &'static str, expected: &'static str, value: &Decimal) -> Self {
        Self {
            type_name,
            expected,
            value: *value,
        }
    }

    #[cfg(feature = "bigdecimal")]
    fn new(type_name: &'static str, expected: &'static str, value: &Decimal) -> Self {
        Self {
            type_name,
            expected,
            value: value.clone(),
        }
    }

    /// Name of the constrained decimal type that rejected the value.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Human-readable description of the accepted range.
    #[must_use]
    pub const fn expected(&self) -> &'static str {
        self.expected
    }

    /// Rejected decimal value.
    #[must_use]
    pub const fn value(&self) -> &Decimal {
        &self.value
    }
}

impl fmt::Display for DecimalConstraintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} expected {}, got {}",
            self.type_name, self.expected, self.value
        )
    }
}

impl std::error::Error for DecimalConstraintError {}

fn is_non_negative(value: &Decimal) -> bool {
    let zero = zero();
    value >= &zero
}

fn is_positive(value: &Decimal) -> bool {
    let zero = zero();
    value > &zero
}

fn is_ratio(value: &Decimal) -> bool {
    let zero = zero();
    let one = one();
    value >= &zero && value <= &one
}

/// Decimal constrained to `x >= 0`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
#[cfg_attr(not(feature = "bigdecimal"), derive(Copy))]
pub struct NonNegativeDecimal(Decimal);

impl NonNegativeDecimal {
    const EXPECTED: &'static str = "a decimal greater than or equal to 0";

    /// Builds a non-negative decimal.
    ///
    /// # Errors
    /// Returns [`DecimalConstraintError`] when `value < 0`.
    pub fn new(value: Decimal) -> Result<Self, DecimalConstraintError> {
        if is_non_negative(&value) {
            Ok(Self(value))
        } else {
            Err(DecimalConstraintError::new(
                "NonNegativeDecimal",
                Self::EXPECTED,
                &value,
            ))
        }
    }

    /// Returns the wrapped decimal by reference.
    #[must_use]
    pub const fn as_decimal(&self) -> &Decimal {
        &self.0
    }

    /// Returns the wrapped decimal.
    #[must_use]
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn into_inner(self) -> Decimal {
        self.0
    }

    /// Returns the wrapped decimal.
    #[must_use]
    #[cfg(feature = "bigdecimal")]
    pub fn into_inner(self) -> Decimal {
        self.0
    }
}

impl AsRef<Decimal> for NonNegativeDecimal {
    fn as_ref(&self) -> &Decimal {
        self.as_decimal()
    }
}

impl TryFrom<Decimal> for NonNegativeDecimal {
    type Error = DecimalConstraintError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NonNegativeDecimal> for Decimal {
    fn from(value: NonNegativeDecimal) -> Self {
        value.into_inner()
    }
}

impl fmt::Display for NonNegativeDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for NonNegativeDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        canonical_str::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for NonNegativeDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = canonical_str::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// Decimal constrained to `x > 0`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
#[cfg_attr(not(feature = "bigdecimal"), derive(Copy))]
pub struct PositiveDecimal(Decimal);

impl PositiveDecimal {
    const EXPECTED: &'static str = "a decimal greater than 0";

    /// Builds a positive decimal.
    ///
    /// # Errors
    /// Returns [`DecimalConstraintError`] when `value <= 0`.
    pub fn new(value: Decimal) -> Result<Self, DecimalConstraintError> {
        if is_positive(&value) {
            Ok(Self(value))
        } else {
            Err(DecimalConstraintError::new(
                "PositiveDecimal",
                Self::EXPECTED,
                &value,
            ))
        }
    }

    /// Returns the wrapped decimal by reference.
    #[must_use]
    pub const fn as_decimal(&self) -> &Decimal {
        &self.0
    }

    /// Returns the wrapped decimal.
    #[must_use]
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn into_inner(self) -> Decimal {
        self.0
    }

    /// Returns the wrapped decimal.
    #[must_use]
    #[cfg(feature = "bigdecimal")]
    pub fn into_inner(self) -> Decimal {
        self.0
    }
}

impl AsRef<Decimal> for PositiveDecimal {
    fn as_ref(&self) -> &Decimal {
        self.as_decimal()
    }
}

impl TryFrom<Decimal> for PositiveDecimal {
    type Error = DecimalConstraintError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PositiveDecimal> for Decimal {
    fn from(value: PositiveDecimal) -> Self {
        value.into_inner()
    }
}

impl From<PositiveDecimal> for NonNegativeDecimal {
    fn from(value: PositiveDecimal) -> Self {
        Self(value.into_inner())
    }
}

impl fmt::Display for PositiveDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for PositiveDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        canonical_str::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for PositiveDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = canonical_str::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// Decimal constrained to `0 <= x <= 1`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
#[cfg_attr(not(feature = "bigdecimal"), derive(Copy))]
pub struct Ratio(Decimal);

impl Ratio {
    const EXPECTED: &'static str = "a decimal between 0 and 1 inclusive";

    /// Builds a ratio.
    ///
    /// # Errors
    /// Returns [`DecimalConstraintError`] when `value < 0` or `value > 1`.
    pub fn new(value: Decimal) -> Result<Self, DecimalConstraintError> {
        if is_ratio(&value) {
            Ok(Self(value))
        } else {
            Err(DecimalConstraintError::new("Ratio", Self::EXPECTED, &value))
        }
    }

    /// Returns the wrapped decimal by reference.
    #[must_use]
    pub const fn as_decimal(&self) -> &Decimal {
        &self.0
    }

    /// Returns the wrapped decimal.
    #[must_use]
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn into_inner(self) -> Decimal {
        self.0
    }

    /// Returns the wrapped decimal.
    #[must_use]
    #[cfg(feature = "bigdecimal")]
    pub fn into_inner(self) -> Decimal {
        self.0
    }
}

impl AsRef<Decimal> for Ratio {
    fn as_ref(&self) -> &Decimal {
        self.as_decimal()
    }
}

impl TryFrom<Decimal> for Ratio {
    type Error = DecimalConstraintError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Ratio> for Decimal {
    fn from(value: Ratio) -> Self {
        value.into_inner()
    }
}

impl From<Ratio> for NonNegativeDecimal {
    fn from(value: Ratio) -> Self {
        Self(value.into_inner())
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Ratio {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        canonical_str::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Ratio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = canonical_str::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}
