//! Backend-agnostic decimal helpers shared across the `paft` workspace.
//!
//! The crate wraps [`rust_decimal`](https://docs.rs/rust_decimal) by default and can
//! switch to [`bigdecimal`](https://docs.rs/bigdecimal) via the optional
//! `bigdecimal` feature. It exposes a consistent [`Decimal`] type alongside rounding
//! strategies and utility helpers for parsing, scaling, and canonical rendering
//! without pulling in higher-level money abstractions.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::cargo_common_metadata)]

use std::{borrow::Cow, str::FromStr};

mod constrained;

pub use constrained::{DecimalConstraintError, NonNegativeDecimal, PositiveDecimal, Ratio};

/// Rounding strategy supported by decimal operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RoundingStrategy {
    /// Round halves toward the nearest even digit.
    MidpointNearestEven,
    /// Round halves away from zero.
    MidpointAwayFromZero,
    /// Round halves toward zero.
    MidpointTowardZero,
    /// Always round toward zero.
    ToZero,
    /// Always round away from zero.
    AwayFromZero,
    /// Always round toward negative infinity.
    ToNegativeInfinity,
    /// Always round toward positive infinity.
    ToPositiveInfinity,
}

#[cfg(not(feature = "bigdecimal"))]
mod backend {
    use super::{FromStr, RoundingStrategy};

    pub use rust_decimal::Decimal;
    use rust_decimal::RoundingStrategy as RustRoundingStrategy;
    pub use rust_decimal::prelude::ToPrimitive;

    pub fn parse_decimal(value: &str) -> Option<Decimal> {
        Decimal::from_str(value).ok()
    }

    pub const fn zero() -> Decimal {
        Decimal::ZERO
    }

    pub const fn one() -> Decimal {
        Decimal::ONE
    }

    pub fn from_minor_units(value: i128, scale: u32) -> Decimal {
        Decimal::from_i128_with_scale(value, scale)
    }

    pub fn try_from_scaled_units(value: i128, scale: u32) -> Option<Decimal> {
        Decimal::try_from_i128_with_scale(value, scale).ok()
    }

    pub fn round_dp_with_strategy(
        value: &Decimal,
        scale: u32,
        strategy: RoundingStrategy,
    ) -> Decimal {
        let strategy: RustRoundingStrategy = strategy.into();
        value.round_dp_with_strategy(scale, strategy)
    }

    pub fn to_plain_string(value: &Decimal) -> String {
        value.to_string()
    }

    impl From<RoundingStrategy> for RustRoundingStrategy {
        fn from(value: RoundingStrategy) -> Self {
            match value {
                RoundingStrategy::MidpointNearestEven => Self::MidpointNearestEven,
                RoundingStrategy::MidpointAwayFromZero => Self::MidpointAwayFromZero,
                RoundingStrategy::MidpointTowardZero => Self::MidpointTowardZero,
                RoundingStrategy::ToZero => Self::ToZero,
                RoundingStrategy::AwayFromZero => Self::AwayFromZero,
                RoundingStrategy::ToNegativeInfinity => Self::ToNegativeInfinity,
                RoundingStrategy::ToPositiveInfinity => Self::ToPositiveInfinity,
            }
        }
    }
}

#[cfg(feature = "bigdecimal")]
mod backend {
    use super::{FromStr, RoundingStrategy};

    pub use bigdecimal::BigDecimal as Decimal;
    use bigdecimal::RoundingMode;
    use num_bigint::BigInt;
    pub use num_traits::ToPrimitive;
    use num_traits::{One, Zero};

    pub fn parse_decimal(value: &str) -> Option<Decimal> {
        Decimal::from_str(value).ok()
    }

    pub fn zero() -> Decimal {
        Decimal::zero()
    }

    pub fn one() -> Decimal {
        Decimal::one()
    }

    pub fn from_minor_units(value: i128, scale: u32) -> Decimal {
        Decimal::new(BigInt::from(value), i64::from(scale))
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn try_from_scaled_units(value: i128, scale: u32) -> Option<Decimal> {
        Some(Decimal::new(BigInt::from(value), i64::from(scale)))
    }

    pub fn round_dp_with_strategy(
        value: &Decimal,
        scale: u32,
        strategy: RoundingStrategy,
    ) -> Decimal {
        let mode = match strategy {
            RoundingStrategy::MidpointNearestEven => RoundingMode::HalfEven,
            RoundingStrategy::MidpointAwayFromZero => RoundingMode::HalfUp,
            RoundingStrategy::MidpointTowardZero => RoundingMode::HalfDown,
            RoundingStrategy::ToZero => RoundingMode::Down,
            RoundingStrategy::AwayFromZero => RoundingMode::Up,
            RoundingStrategy::ToNegativeInfinity => RoundingMode::Floor,
            RoundingStrategy::ToPositiveInfinity => RoundingMode::Ceiling,
        };

        value.with_scale_round(i64::from(scale), mode)
    }

    pub fn to_plain_string(value: &Decimal) -> String {
        value.to_plain_string()
    }
}

pub use backend::{Decimal, ToPrimitive};

/// Parses a decimal string using the active backend.
///
/// Whitespace is ignored, an optional leading `+` is accepted, and scientific
/// notation is rejected so both decimal backends share identical parsing
/// semantics.
#[must_use]
pub fn parse_decimal(value: &str) -> Option<Decimal> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.contains(['e', 'E']) {
        return None;
    }
    let normalized = if let Some(rest) = trimmed.strip_prefix('+') {
        if rest.is_empty() || rest.starts_with(['+', '-']) {
            return None;
        }
        rest
    } else {
        trimmed
    };
    backend::parse_decimal(normalized)
}

/// Returns the zero value for the active decimal backend.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn zero() -> Decimal {
    backend::zero()
}

/// Returns the one value for the active decimal backend.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn one() -> Decimal {
    backend::one()
}

/// Builds a decimal from an integer count of minor units and the provided scale.
///
/// # Panics
///
/// With the default backend, panics when the scale or integer coefficient cannot
/// be represented by `rust_decimal`. Use [`try_from_scaled_units`] when the input
/// is not already known to fit the active backend.
#[must_use]
pub fn from_minor_units(value: i128, scale: u32) -> Decimal {
    backend::from_minor_units(value, scale)
}

/// Builds a decimal from an integer coefficient and scale if the active backend can represent it.
///
/// Returns `None` when the default backend rejects either the scale or the
/// 96-bit mantissa. The `bigdecimal` backend accepts every `i128` coefficient
/// and `u32` scale.
#[must_use]
#[cfg_attr(feature = "bigdecimal", allow(clippy::unnecessary_wraps))]
pub fn try_from_scaled_units(value: i128, scale: u32) -> Option<Decimal> {
    backend::try_from_scaled_units(value, scale)
}

/// Rounds a decimal to the requested scale using a rounding strategy.
#[must_use]
pub fn round_dp_with_strategy(value: &Decimal, scale: u32, strategy: RoundingStrategy) -> Decimal {
    backend::round_dp_with_strategy(value, scale, strategy)
}

/// Serde helpers for backend-stable decimal wire formats.
///
/// These modules serialize decimals as canonical strings rendered by
/// [`to_canonical_string`] and deserialize with [`parse_decimal`], avoiding
/// backend-native differences such as scale preservation or exponent output.
pub mod serde {
    use super::{Cow, Decimal};
    use ::serde::{Deserialize, Serializer, de};

    fn invalid_decimal<E>(value: &str) -> E
    where
        E: de::Error,
    {
        E::custom(format_args!("invalid decimal string `{value}`"))
    }

    /// Serde adapter for a required canonical decimal string.
    pub mod canonical_str {
        use super::{Cow, Decimal, Deserialize, Serializer, invalid_decimal};
        use crate::{parse_decimal, to_canonical_string};

        /// Serializes a decimal as a canonical string.
        ///
        /// # Errors
        /// Returns the serializer error when writing the string fails.
        pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&to_canonical_string(value))
        }

        /// Deserializes a decimal from a string accepted by [`crate::parse_decimal`].
        ///
        /// # Errors
        /// Returns the deserializer error when the input is not a string or
        /// when [`crate::parse_decimal`] rejects the string.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            let value = Cow::<str>::deserialize(deserializer)?;
            parse_decimal(&value).ok_or_else(|| invalid_decimal(&value))
        }
    }

    /// Serde adapter for an optional canonical decimal string.
    pub mod option_canonical_str {
        use super::{Cow, Decimal, Deserialize, Serializer, invalid_decimal};
        use crate::{parse_decimal, to_canonical_string};
        use ::serde::Serialize;

        /// Serializes an optional decimal as a canonical string or `null`.
        ///
        /// # Errors
        /// Returns the serializer error when writing the option fails.
        pub fn serialize<S>(value: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let canonical = value.as_ref().map(to_canonical_string);
            canonical.serialize(serializer)
        }

        /// Deserializes an optional decimal from strings accepted by [`crate::parse_decimal`].
        ///
        /// # Errors
        /// Returns the deserializer error when the input is not `null` or a
        /// string, or when [`crate::parse_decimal`] rejects the string.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            Option::<Cow<'de, str>>::deserialize(deserializer)?
                .map(|value| parse_decimal(&value).ok_or_else(|| invalid_decimal(&value)))
                .transpose()
        }
    }
}

/// Converts a decimal into a canonical string without scientific notation and
/// without gratuitous trailing zeros.
#[must_use]
pub fn to_canonical_string(value: &Decimal) -> String {
    let mut repr = backend::to_plain_string(value);
    if let Some(dot) = repr.find('.') {
        let mut end = repr.len();
        while end > dot + 1 && repr.as_bytes()[end - 1] == b'0' {
            end -= 1;
        }
        if end == dot + 1 {
            end -= 1;
        }
        repr.truncate(end);
    }
    repr
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{Decimal, parse_decimal, to_canonical_string, try_from_scaled_units};

    #[test]
    fn parse_rejects_scientific_notation() {
        assert!(parse_decimal("1e3").is_none());
        assert!(parse_decimal("2E-3").is_none());
    }

    #[test]
    fn parse_accepts_standard_forms() {
        assert_eq!(
            parse_decimal("  +123.4500 ").unwrap(),
            parse_decimal("123.45").unwrap()
        );
        assert_eq!(
            parse_decimal("-42.1").unwrap(),
            Decimal::from_str("-42.1").unwrap()
        );
    }

    #[test]
    fn parse_rejects_duplicate_explicit_signs() {
        assert!(parse_decimal("+-1").is_none());
        assert!(parse_decimal("++1").is_none());
        assert!(parse_decimal("+").is_none());
        assert!(parse_decimal("+1").is_some());
        assert!(parse_decimal("-1").is_some());
    }

    #[test]
    fn canonical_string_trims_trailing_zeros() {
        let value = parse_decimal("123.4500").unwrap();
        assert_eq!(to_canonical_string(&value), "123.45");
        let integer = parse_decimal("1000").unwrap();
        assert_eq!(to_canonical_string(&integer), "1000");
    }

    #[test]
    fn canonical_decimal_serde_uses_strings() {
        #[derive(::serde::Serialize, ::serde::Deserialize, PartialEq, Debug)]
        struct Payload {
            #[serde(with = "crate::serde::canonical_str")]
            value: Decimal,
            #[serde(default, with = "crate::serde::option_canonical_str")]
            optional: Option<Decimal>,
        }

        let payload = Payload {
            value: parse_decimal("123.4500").unwrap(),
            optional: Some(parse_decimal("0.5000").unwrap()),
        };

        let value = serde_json::to_value(&payload).unwrap();
        assert_eq!(value["value"], serde_json::json!("123.45"));
        assert_eq!(value["optional"], serde_json::json!("0.5"));
        assert_eq!(serde_json::from_value::<Payload>(value).unwrap(), payload);

        let missing_optional = serde_json::json!({ "value": "+1.2300" });
        let parsed = serde_json::from_value::<Payload>(missing_optional).unwrap();
        assert_eq!(to_canonical_string(&parsed.value), "1.23");
        assert_eq!(parsed.optional, None);
    }

    #[test]
    fn try_from_scaled_units_accepts_representable_values() {
        let value = try_from_scaled_units(123_456, 3).unwrap();
        assert_eq!(to_canonical_string(&value), "123.456");
    }

    #[cfg(not(feature = "bigdecimal"))]
    #[test]
    fn try_from_scaled_units_rejects_rust_decimal_limits() {
        assert!(try_from_scaled_units(i128::MAX, 0).is_none());
        assert!(try_from_scaled_units(1, 29).is_none());
    }
}
