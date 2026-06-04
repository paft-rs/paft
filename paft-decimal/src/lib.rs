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
    use super::{FromStr, RoundingStrategy, rust_decimal_to_i128_mantissa};

    pub use rust_decimal::Decimal;
    use rust_decimal::RoundingStrategy as RustRoundingStrategy;
    pub use rust_decimal::prelude::ToPrimitive;

    pub fn parse_decimal(value: &str) -> Option<Decimal> {
        Decimal::from_str(value).ok()
    }

    pub const MAX_DECIMAL_PRECISION: u8 = 28;

    pub const fn clone_decimal(value: &Decimal) -> Decimal {
        *value
    }

    pub fn fractional_digit_count(value: &Decimal) -> i64 {
        i64::from(value.scale())
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

    pub fn checked_add(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
        lhs.checked_add(*rhs)
    }

    pub fn checked_sub(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
        lhs.checked_sub(*rhs)
    }

    pub fn checked_mul(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
        lhs.checked_mul(*rhs)
    }

    pub fn checked_div(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
        lhs.checked_div(*rhs)
    }

    pub fn try_to_i128_mantissa(value: &Decimal, target_scale: u32) -> Option<i128> {
        rust_decimal_to_i128_mantissa(value, target_scale)
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
    use super::{DECIMAL128_PRECISION, FromStr, RoundingStrategy};

    pub use bigdecimal::BigDecimal as Decimal;
    use bigdecimal::RoundingMode;
    use num_bigint::BigInt;
    pub use num_traits::ToPrimitive;
    use num_traits::{One, Zero};

    pub fn parse_decimal(value: &str) -> Option<Decimal> {
        Decimal::from_str(value).ok()
    }

    pub const MAX_DECIMAL_PRECISION: u8 = u8::MAX;

    pub fn clone_decimal(value: &Decimal) -> Decimal {
        value.clone()
    }

    pub fn fractional_digit_count(value: &Decimal) -> i64 {
        value.fractional_digit_count()
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

    #[allow(clippy::unnecessary_wraps)]
    pub fn checked_add(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
        Some(lhs + rhs)
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn checked_sub(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
        Some(lhs - rhs)
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn checked_mul(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
        Some(lhs * rhs)
    }

    pub fn checked_div(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
        if rhs.is_zero() {
            return None;
        }

        Some(lhs / rhs)
    }

    pub fn try_to_i128_mantissa(value: &Decimal, target_scale: u32) -> Option<i128> {
        if target_scale > DECIMAL128_PRECISION {
            return None;
        }

        let target = i64::from(target_scale);
        let rescaled = value.with_scale_round(target, bigdecimal::RoundingMode::HalfEven);
        if rescaled.digits() > 38 {
            return None;
        }
        let (bigint, _) = rescaled.into_bigint_and_exponent();
        i128::try_from(bigint).ok()
    }
}

pub use backend::{Decimal, ToPrimitive};

const DECIMAL128_PRECISION: u32 = 38;
const MAX_I128_MANTISSA: i128 = 10_i128.pow(DECIMAL128_PRECISION);

fn rust_decimal_to_i128_mantissa(value: &rust_decimal::Decimal, target_scale: u32) -> Option<i128> {
    if target_scale > DECIMAL128_PRECISION {
        return None;
    }

    let source_scale = value.scale();
    let mantissa: i128 = value.mantissa();
    let rescaled = match source_scale.cmp(&target_scale) {
        std::cmp::Ordering::Equal => mantissa,
        std::cmp::Ordering::Less => {
            let diff = target_scale - source_scale;
            let pow = 10_i128.checked_pow(diff)?;
            mantissa.checked_mul(pow)?
        }
        std::cmp::Ordering::Greater => {
            let diff = source_scale - target_scale;
            let pow = 10_i128.checked_pow(diff)?.cast_unsigned();
            let neg = mantissa < 0;
            let abs = mantissa.unsigned_abs();
            let q = (abs / pow).cast_signed();
            let r = abs % pow;
            let half = pow / 2;
            let rounded = match r.cmp(&half) {
                std::cmp::Ordering::Greater => q + 1,
                std::cmp::Ordering::Less => q,
                std::cmp::Ordering::Equal => q + (q & 1),
            };
            if neg { -rounded } else { rounded }
        }
    };
    if rescaled.unsigned_abs() >= MAX_I128_MANTISSA.cast_unsigned() {
        return None;
    }
    Some(rescaled)
}

/// Maximum fractional precision supported by the active decimal backend.
pub const MAX_DECIMAL_PRECISION: u8 = backend::MAX_DECIMAL_PRECISION;

/// Returns the maximum fractional precision supported by the active decimal backend.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn max_decimal_precision() -> u8 {
    backend::MAX_DECIMAL_PRECISION
}

/// Clones a decimal value using the active backend's cheapest owned-value path.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn clone_decimal(value: &Decimal) -> Decimal {
    backend::clone_decimal(value)
}

/// Returns the number of fractional digits represented by the active backend.
#[must_use]
pub fn fractional_digit_count(value: &Decimal) -> i64 {
    backend::fractional_digit_count(value)
}

/// Adds two decimals if the active backend can represent the result.
#[must_use]
pub fn checked_add(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    backend::checked_add(lhs, rhs)
}

/// Subtracts two decimals if the active backend can represent the result.
#[must_use]
pub fn checked_sub(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    backend::checked_sub(lhs, rhs)
}

/// Multiplies two decimals if the active backend can represent the result.
#[must_use]
pub fn checked_mul(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    backend::checked_mul(lhs, rhs)
}

/// Divides two decimals if the active backend can represent the result.
#[must_use]
pub fn checked_div(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    backend::checked_div(lhs, rhs)
}

/// Encodes decimal-like values into Polars-compatible decimal128 mantissas.
pub trait Decimal128Mantissa {
    /// Returns the mantissa after rescaling to `target_scale`, or `None` when
    /// the result exceeds decimal128 precision or the active backend cannot
    /// represent the conversion.
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128>;
}

impl Decimal128Mantissa for Decimal {
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        backend::try_to_i128_mantissa(self, target_scale)
    }
}

#[cfg(feature = "bigdecimal")]
impl Decimal128Mantissa for rust_decimal::Decimal {
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        rust_decimal_to_i128_mantissa(self, target_scale)
    }
}

impl Decimal128Mantissa for NonNegativeDecimal {
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        self.as_decimal().try_to_i128_mantissa(target_scale)
    }
}

impl Decimal128Mantissa for PositiveDecimal {
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        self.as_decimal().try_to_i128_mantissa(target_scale)
    }
}

impl Decimal128Mantissa for Ratio {
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128> {
        self.as_decimal().try_to_i128_mantissa(target_scale)
    }
}

/// Parses a plain decimal string using the active backend.
///
/// Surrounding whitespace is ignored, an optional leading sign is accepted, and
/// scientific notation, digit separators, and internal whitespace are rejected
/// so both decimal backends share identical parsing semantics.
#[must_use]
pub fn parse_decimal(value: &str) -> Option<Decimal> {
    let normalized = normalize_decimal_literal(value)?;
    backend::parse_decimal(&normalized)
}

fn normalize_decimal_literal(value: &str) -> Option<Cow<'_, str>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (sign, unsigned) = match trimmed.as_bytes().first() {
        Some(b'+') => ("", &trimmed[1..]),
        Some(b'-') => ("-", &trimmed[1..]),
        Some(_) => ("", trimmed),
        None => return None,
    };

    if unsigned.is_empty() {
        return None;
    }

    let mut seen_dot = false;
    let mut seen_digit = false;
    for byte in unsigned.bytes() {
        match byte {
            b'0'..=b'9' => seen_digit = true,
            b'.' if !seen_dot => seen_dot = true,
            _ => return None,
        }
    }

    if !seen_digit {
        return None;
    }

    let needs_leading_zero = unsigned.starts_with('.');
    let needs_trailing_zero = unsigned.ends_with('.');
    if needs_leading_zero || needs_trailing_zero {
        let mut normalized = String::with_capacity(trimmed.len() + 2);
        normalized.push_str(sign);
        if needs_leading_zero {
            normalized.push('0');
        }
        normalized.push_str(unsigned);
        if needs_trailing_zero {
            normalized.push('0');
        }
        Some(Cow::Owned(normalized))
    } else if sign == "-" {
        Some(Cow::Borrowed(trimmed))
    } else {
        Some(Cow::Borrowed(unsigned))
    }
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
    let zero = zero();
    if value == &zero {
        return "0".to_owned();
    }

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

    use super::{
        Decimal, RoundingStrategy, checked_div, parse_decimal, round_dp_with_strategy,
        to_canonical_string, try_from_scaled_units,
    };

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
    fn parse_uses_backend_stable_plain_decimal_grammar() {
        for (literal, canonical) in [
            (".5", "0.5"),
            ("1.", "1"),
            ("+1", "1"),
            ("-0.00", "0"),
            ("001.2300", "1.23"),
            (" \t\n+001.2300\r", "1.23"),
        ] {
            let parsed = parse_decimal(literal).unwrap_or_else(|| panic!("{literal} should parse"));
            assert_eq!(to_canonical_string(&parsed), canonical);
        }
    }

    #[test]
    fn parse_rejects_non_plain_decimal_grammar() {
        for literal in [
            "", " ", "+", "-", ".", "+.", "-.", "+-1", "++1", "--1", "1_000", "1e3", "2E-3", "1 2",
            "1.2.3",
        ] {
            assert!(parse_decimal(literal).is_none(), "{literal} should fail");
        }
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
    fn canonical_string_normalizes_zero_sign() {
        let negative_zero = parse_decimal("-0.00").unwrap();
        assert_eq!(to_canonical_string(&negative_zero), "0");

        let rounded_negative_zero = round_dp_with_strategy(
            &parse_decimal("-0.0049").unwrap(),
            2,
            RoundingStrategy::ToZero,
        );
        assert_eq!(to_canonical_string(&rounded_negative_zero), "0");
    }

    #[test]
    fn checked_div_returns_none_for_zero_divisor() {
        let lhs = parse_decimal("10").unwrap();
        let zero = parse_decimal("0.00").unwrap();
        assert!(checked_div(&lhs, &zero).is_none());

        let two = parse_decimal("2").unwrap();
        let quotient = checked_div(&lhs, &two).unwrap();
        assert_eq!(to_canonical_string(&quotient), "5");
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
