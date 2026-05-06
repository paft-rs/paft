use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::currency::Currency;
use crate::decimal::{self, Decimal, RoundingStrategy};
use crate::error::MoneyError;
use crate::money::Money;

/// High-precision decimal amount with an optional [`Currency`] hint.
///
/// `PartialEq`, `Eq`, and `Hash` consider both the amount and the
/// currency hint as a tuple. Two `MoneyAmount` values with equal numeric
/// magnitude but different hints (e.g. `100 USD` vs `100 EUR`) are
/// **not** equal, because they are not interchangeable in the contexts
/// where the hint matters (currency-aware sums, dataframe keys, dedup).
/// Hashing matches `Eq`, so storing them in `HashMap`/`HashSet` does
/// the right thing.
#[derive(Debug, Clone)]
pub struct MoneyAmount {
    amount: Decimal,
    currency_hint: Option<Currency>,
}

impl MoneyAmount {
    /// Creates a `MoneyAmount` without a currency hint.
    #[must_use]
    pub const fn new(amount: Decimal) -> Self {
        Self {
            amount,
            currency_hint: None,
        }
    }

    /// Parses a decimal string using [`decimal::parse_decimal`].
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::InvalidDecimal`] when the string cannot be parsed losslessly.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(amount: &str) -> Result<Self, MoneyError> {
        let decimal = decimal::parse_decimal(amount).ok_or(MoneyError::InvalidDecimal)?;
        Ok(Self::new(decimal))
    }

    /// Creates a `MoneyAmount` from integer minor units and a scale.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::ConversionError`] when the scale exceeds the active backend precision.
    pub fn from_minor_units(units: i128, scale: u32) -> Result<Self, MoneyError> {
        #[cfg(not(feature = "bigdecimal"))]
        {
            use crate::currency_utils::MAX_DECIMAL_PRECISION;

            if scale > u32::from(MAX_DECIMAL_PRECISION) {
                return Err(MoneyError::ConversionError);
            }
        }
        let decimal = decimal::from_minor_units(units, scale);
        Ok(Self::new(decimal))
    }

    /// Returns the zero amount with no currency hint.
    #[must_use]
    pub fn zero() -> Self {
        Self::new(decimal::zero())
    }

    /// Returns the underlying [`Decimal`], cloning when required by the backend.
    ///
    /// `const`-qualified under the default `rust_decimal` backend, which is a
    /// pure copy. Under `bigdecimal`, the body invokes `Clone::clone` and
    /// must run at runtime.
    #[must_use]
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn amount(&self) -> Decimal {
        copy_decimal(&self.amount)
    }

    /// Returns the underlying [`Decimal`], cloning when required by the backend.
    #[must_use]
    #[cfg(feature = "bigdecimal")]
    pub fn amount(&self) -> Decimal {
        copy_decimal(&self.amount)
    }

    /// Returns the optional currency hint attached to this amount.
    #[must_use]
    pub const fn currency_hint(&self) -> Option<&Currency> {
        self.currency_hint.as_ref()
    }

    /// Produces a new `MoneyAmount` with the same numeric value and a provided hint.
    ///
    /// See [`MoneyAmount::amount`] for the `const`-fn split rationale.
    #[must_use]
    #[cfg(not(feature = "bigdecimal"))]
    pub const fn with_currency_hint(&self, currency: Currency) -> Self {
        Self {
            amount: copy_decimal(&self.amount),
            currency_hint: Some(currency),
        }
    }

    /// Produces a new `MoneyAmount` with the same numeric value and a provided hint.
    #[must_use]
    #[cfg(feature = "bigdecimal")]
    pub fn with_currency_hint(&self, currency: Currency) -> Self {
        Self {
            amount: copy_decimal(&self.amount),
            currency_hint: Some(currency),
        }
    }

    /// Adds another amount, combining currency hints when possible.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        let amount = copy_decimal(&self.amount) + copy_decimal(&other.amount);
        let currency_hint =
            merge_currency_hints(self.currency_hint.as_ref(), other.currency_hint.as_ref());
        Self {
            amount,
            currency_hint,
        }
    }

    /// Subtracts another amount, combining currency hints when possible.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        let amount = copy_decimal(&self.amount) - copy_decimal(&other.amount);
        let currency_hint =
            merge_currency_hints(self.currency_hint.as_ref(), other.currency_hint.as_ref());
        Self {
            amount,
            currency_hint,
        }
    }

    /// Multiplies the amount by a decimal factor, preserving the hint.
    #[must_use]
    pub fn mul(&self, factor: Decimal) -> Self {
        let amount = copy_decimal(&self.amount) * factor;
        Self {
            amount,
            currency_hint: self.currency_hint.clone(),
        }
    }

    /// Divides the amount by a decimal divisor, preserving the hint.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::DivisionByZero`] when `divisor` is zero.
    pub fn div(&self, divisor: Decimal) -> Result<Self, MoneyError> {
        if divisor == decimal::zero() {
            return Err(MoneyError::DivisionByZero);
        }
        let amount = copy_decimal(&self.amount) / divisor;
        Ok(Self {
            amount,
            currency_hint: self.currency_hint.clone(),
        })
    }

    /// Converts the amount into [`Money`], rounding to the currency exponent with
    /// [`RoundingStrategy::MidpointAwayFromZero`].
    ///
    /// # Errors
    ///
    /// Propagates the errors returned by [`Money::new`].
    pub fn to_money(&self, currency: Currency) -> Result<Money, MoneyError> {
        self.to_money_with(currency, RoundingStrategy::MidpointAwayFromZero, None)
    }

    /// Converts the amount into [`Money`] using an explicit rounding strategy and precision.
    ///
    /// # Errors
    ///
    /// - Returns [`MoneyError::MetadataNotFound`] when the currency is missing metadata.
    /// - Returns [`MoneyError::ConversionError`] when `target_fraction_digits` exceeds the
    ///   currency exponent.
    pub fn to_money_with(
        &self,
        currency: Currency,
        rounding: RoundingStrategy,
        target_fraction_digits: Option<u32>,
    ) -> Result<Money, MoneyError> {
        let exponent = currency.decimal_places()?;
        let currency_scale = u32::from(exponent);

        let effective_scale = match target_fraction_digits {
            Some(digits) => {
                if digits > currency_scale {
                    return Err(MoneyError::ConversionError);
                }
                digits
            }
            None => currency_scale,
        };

        let rounded = decimal::round_dp_with_strategy(&self.amount, effective_scale, rounding);
        Money::new(rounded, currency)
    }
}

impl From<Decimal> for MoneyAmount {
    fn from(value: Decimal) -> Self {
        Self::new(value)
    }
}

impl From<Money> for MoneyAmount {
    fn from(money: Money) -> Self {
        Self {
            amount: money.amount(),
            currency_hint: Some(money.currency().clone()),
        }
    }
}

impl PartialEq for MoneyAmount {
    fn eq(&self, other: &Self) -> bool {
        self.amount == other.amount && self.currency_hint == other.currency_hint
    }
}

impl Eq for MoneyAmount {}

impl Hash for MoneyAmount {
    fn hash<H: Hasher>(&self, state: &mut H) {
        decimal::to_canonical_string(&self.amount).hash(state);
        self.currency_hint.hash(state);
    }
}

impl fmt::Display for MoneyAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&decimal::to_canonical_string(&self.amount))
    }
}

impl Serialize for MoneyAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(&self.amount, serializer)
    }
}

impl<'de> Deserialize<'de> for MoneyAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let amount = <Decimal as serde::Deserialize>::deserialize(deserializer)?;
        Ok(Self::new(amount))
    }
}

impl FromStr for MoneyAmount {
    type Err = MoneyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

fn merge_currency_hints(lhs: Option<&Currency>, rhs: Option<&Currency>) -> Option<Currency> {
    match (lhs, rhs) {
        (Some(a), Some(b)) if a == b => Some(a.clone()),
        (Some(a), None) | (None, Some(a)) => Some(a.clone()),
        _ => None,
    }
}

// Two cfg-gated definitions instead of one body with `cfg!` arms — the
// `rust_decimal` path is a pure copy and is `const`-eligible, while the
// `bigdecimal` path performs a heap allocation through `Clone`.
#[cfg(not(feature = "bigdecimal"))]
#[inline]
const fn copy_decimal(value: &Decimal) -> Decimal {
    *value
}

#[cfg(feature = "bigdecimal")]
#[inline]
fn copy_decimal(value: &Decimal) -> Decimal {
    value.clone()
}
