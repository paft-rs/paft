//! Full-precision quantity type for market sizes and volumes.

use crate::decimal::{self, Decimal};
use paft_decimal::{DecimalConstraintError, NonNegativeDecimal};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

/// Full-precision non-negative quantity whose unit is supplied by surrounding context.
///
/// Use `QuantityAmount` for provider-agnostic sizes and volumes where the
/// quantity unit may be shares, contracts, base units, quote units, lots, or a
/// fractional venue-specific unit. Count-only fields with inherently integral
/// semantics can keep using integer types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
#[cfg_attr(not(feature = "bigdecimal"), derive(Copy))]
pub struct QuantityAmount {
    #[cfg_attr(feature = "dataframe", df_derive(decimal(precision = 38, scale = 10)))]
    amount: NonNegativeDecimal,
}

impl QuantityAmount {
    /// Creates a contextual quantity amount from a non-negative decimal.
    #[must_use]
    pub const fn new(amount: NonNegativeDecimal) -> Self {
        Self { amount }
    }

    /// Creates a contextual quantity amount from a decimal.
    ///
    /// # Errors
    ///
    /// Returns [`DecimalConstraintError`] when `amount < 0`.
    pub fn from_decimal(amount: Decimal) -> Result<Self, DecimalConstraintError> {
        Ok(Self::new(NonNegativeDecimal::new(amount)?))
    }

    /// Returns the wrapped non-negative decimal by reference.
    #[must_use]
    pub const fn as_non_negative_decimal(&self) -> &NonNegativeDecimal {
        &self.amount
    }

    /// Returns the wrapped decimal by reference.
    #[must_use]
    pub const fn as_decimal(&self) -> &Decimal {
        self.amount.as_decimal()
    }

    /// Returns the wrapped non-negative decimal.
    #[must_use]
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn into_inner(self) -> NonNegativeDecimal {
        self.amount
    }

    /// Returns the wrapped non-negative decimal.
    #[must_use]
    #[cfg(feature = "bigdecimal")]
    pub fn into_inner(self) -> NonNegativeDecimal {
        self.amount
    }
}

impl AsRef<NonNegativeDecimal> for QuantityAmount {
    fn as_ref(&self) -> &NonNegativeDecimal {
        self.as_non_negative_decimal()
    }
}

impl TryFrom<Decimal> for QuantityAmount {
    type Error = DecimalConstraintError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Self::from_decimal(value)
    }
}

impl From<NonNegativeDecimal> for QuantityAmount {
    fn from(value: NonNegativeDecimal) -> Self {
        Self::new(value)
    }
}

impl From<QuantityAmount> for NonNegativeDecimal {
    fn from(value: QuantityAmount) -> Self {
        value.into_inner()
    }
}

impl Hash for QuantityAmount {
    fn hash<H: Hasher>(&self, state: &mut H) {
        decimal::to_canonical_string(self.as_decimal()).hash(state);
    }
}

impl fmt::Display for QuantityAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&decimal::to_canonical_string(self.as_decimal()))
    }
}
