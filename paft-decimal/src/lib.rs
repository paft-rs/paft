//! Fixed-width decimal helpers shared across the `paft` workspace.
//!
//! [`Decimal`] is always [`rust_decimal::Decimal`]: a 96-bit coefficient with
//! scale 0 through 28. Magnitude and fractional precision share that coefficient
//! budget. PAFT parsing and canonical serde preserve the numeric value or fail;
//! native `Decimal` parsing, serde, and arithmetic retain upstream semantics.
//! Constructors receiving an existing decimal cannot validate its history.
//!
//! Exact arithmetic helpers reject unrepresentable results. Ordinary checked
//! helpers may round; settlement and `DataFrame` rounding are separate boundaries.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::borrow::Cow;

mod constrained;
mod exact;

pub use exact::{checked_add_exact, checked_div_exact, checked_mul_exact, checked_sub_exact};

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

impl From<RoundingStrategy> for rust_decimal::RoundingStrategy {
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

pub use rust_decimal::{Decimal, prelude::ToPrimitive};

const DECIMAL128_PRECISION: u32 = 38;
const MAX_I128_MANTISSA: i128 = 10_i128.pow(DECIMAL128_PRECISION);

fn rust_decimal_to_scaled_units(value: &rust_decimal::Decimal, target_scale: u32) -> Option<i128> {
    let source_scale = value.scale();
    let mantissa = value.mantissa();
    match source_scale.cmp(&target_scale) {
        std::cmp::Ordering::Equal => Some(mantissa),
        std::cmp::Ordering::Less => {
            let diff = target_scale - source_scale;
            let pow = 10_i128.checked_pow(diff)?;
            mantissa.checked_mul(pow)
        }
        std::cmp::Ordering::Greater => {
            let diff = source_scale - target_scale;
            let pow = 10_i128.checked_pow(diff)?;
            if mantissa % pow != 0 {
                return None;
            }
            Some(mantissa / pow)
        }
    }
}

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

/// Maximum decimal scale; magnitude and fractional precision share 96 coefficient bits.
pub const MAX_DECIMAL_PRECISION: u8 = 28;

/// Returns the maximum decimal scale (28), subject to the 96-bit coefficient limit.
#[must_use]
pub const fn max_decimal_precision() -> u8 {
    MAX_DECIMAL_PRECISION
}

/// Copies a decimal value.
#[must_use]
pub const fn clone_decimal(value: &Decimal) -> Decimal {
    *value
}

/// Returns the number of fractional digits in the decimal's representation.
#[must_use]
pub fn fractional_digit_count(value: &Decimal) -> i64 {
    i64::from(value.scale())
}

/// Adds two decimals using upstream checked arithmetic.
///
/// May round to fit decimal precision; `None` indicates overflow. Use
/// [`checked_add_exact`] when rounding is not permitted.
#[must_use]
pub fn checked_add(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    lhs.checked_add(*rhs)
}

/// Subtracts two decimals using upstream checked arithmetic.
///
/// May round to fit decimal precision; `None` indicates overflow. Use
/// [`checked_sub_exact`] when rounding is not permitted.
#[must_use]
pub fn checked_sub(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    lhs.checked_sub(*rhs)
}

/// Multiplies two decimals using upstream checked arithmetic.
///
/// May round, including underflow to zero; `None` indicates overflow. Use
/// [`checked_mul_exact`] when rounding is not permitted.
#[must_use]
pub fn checked_mul(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    lhs.checked_mul(*rhs)
}

/// Divides two decimals using upstream checked arithmetic.
///
/// May round, including underflow to zero; `None` indicates overflow or division
/// by zero. Use [`checked_div_exact`] when rounding is not permitted.
#[must_use]
pub fn checked_div(lhs: &Decimal, rhs: &Decimal) -> Option<Decimal> {
    lhs.checked_div(*rhs)
}

/// Encodes decimal-like values into Polars-compatible decimal128 mantissas.
pub trait Decimal128Mantissa {
    /// Returns the mantissa after rescaling to `target_scale`, or `None` when
    /// the result exceeds decimal128 precision. Scale-down uses half-even
    /// rounding; this is not an exact conversion. Use [`try_to_scaled_units`]
    /// when rounding is not permitted.
    fn try_to_i128_mantissa(&self, target_scale: u32) -> Option<i128>;
}

impl Decimal128Mantissa for Decimal {
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

/// Failure to ingest a plain decimal string into PAFT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecimalParseError {
    /// The input does not follow PAFT's plain decimal grammar.
    InvalidSyntax,
    /// The numeric value cannot fit PAFT's fixed decimal representation exactly.
    NotRepresentable,
}

impl std::fmt::Display for DecimalParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidSyntax => "invalid plain decimal syntax",
            Self::NotRepresentable => "numeric value is not exactly representable by PAFT",
        })
    }
}

impl std::error::Error for DecimalParseError {}

/// Parses a plain decimal string without changing its numeric value.
///
/// Preserves representable input scale. Insignificant fractional trailing zeros
/// may be removed to fit; nonzero digits are never discarded. Surrounding
/// whitespace and an optional leading sign are accepted. Scientific notation,
/// digit separators, and internal whitespace are rejected.
///
/// Unlike native `Decimal` parsing or serde, this is a PAFT exact-ingestion
/// boundary. Exactness concerns the numeric value, not the original spelling.
///
/// # Errors
/// Returns [`DecimalParseError::InvalidSyntax`] for invalid grammar and
/// [`DecimalParseError::NotRepresentable`] when the value cannot fit exactly.
pub fn parse_decimal(value: &str) -> Result<Decimal, DecimalParseError> {
    let normalized = normalize_decimal_literal(value).ok_or(DecimalParseError::InvalidSyntax)?;
    Decimal::from_str_exact(&normalized)
        .or_else(|_| {
            // Only fractional trailing zeros may be removed without changing value.
            let reduced = if normalized.contains('.') {
                normalized.trim_end_matches('0').trim_end_matches('.')
            } else {
                &normalized
            };
            Decimal::from_str_exact(reduced)
        })
        .map_err(|_| DecimalParseError::NotRepresentable)
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

/// Returns the decimal zero value.
#[must_use]
pub const fn zero() -> Decimal {
    Decimal::ZERO
}

/// Returns the decimal one value.
#[must_use]
pub const fn one() -> Decimal {
    Decimal::ONE
}

/// Builds a decimal from an integer count of minor units and the provided scale.
///
/// # Panics
/// Panics when the scale exceeds 28 or the coefficient exceeds 96 bits. Use
/// [`try_from_scaled_units`] when the input is not already known to fit.
#[must_use]
pub fn from_minor_units(value: i128, scale: u32) -> Decimal {
    Decimal::from_i128_with_scale(value, scale)
}

/// Builds a decimal from an integer coefficient and scale without rounding.
///
/// Returns `None` when the supplied scale exceeds 28 or the coefficient exceeds
/// 96 bits, even if reducing trailing zeros could represent the numeric value.
#[must_use]
pub fn try_from_scaled_units(value: i128, scale: u32) -> Option<Decimal> {
    Decimal::try_from_i128_with_scale(value, scale).ok()
}

/// Converts a decimal into exact base-10 scaled integer units.
///
/// Returns `None` when converting to `target_scale` would require rounding or
/// when the exact scaled unit count cannot be stored in `i128`.
#[must_use]
pub fn try_to_scaled_units(value: &Decimal, target_scale: u32) -> Option<i128> {
    rust_decimal_to_scaled_units(value, target_scale)
}

/// Rounds a decimal to the requested scale using a rounding strategy.
#[must_use]
pub fn round_dp_with_strategy(value: &Decimal, scale: u32, strategy: RoundingStrategy) -> Decimal {
    value.round_dp_with_strategy(scale, strategy.into())
}

/// Serde helpers for exact ingestion and canonical decimal strings.
///
/// These modules serialize decimals as canonical strings rendered by
/// [`to_canonical_string`] and deserialize with [`parse_decimal`]. Unlike
/// native decimal serde, these adapters reject values PAFT cannot represent
/// exactly, including nonzero digits beyond its precision limit.
pub mod serde {
    use super::{Cow, Decimal, DecimalParseError};
    use serde::{Deserialize, Serializer, de};

    fn invalid_decimal<E>(value: &str, error: DecimalParseError) -> E
    where
        E: de::Error,
    {
        E::custom(format_args!("{error}: `{value}`"))
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
            parse_decimal(&value).map_err(|error| invalid_decimal(&value, error))
        }
    }

    /// Serde adapter for an optional canonical decimal string.
    pub mod option_canonical_str {
        use super::{Cow, Decimal, Deserialize, Serializer, invalid_decimal};
        use crate::{parse_decimal, to_canonical_string};
        use serde::Serialize;

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
                .map(|value| parse_decimal(&value).map_err(|error| invalid_decimal(&value, error)))
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

    let mut repr = value.to_string();
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
        to_canonical_string, try_from_scaled_units, try_to_scaled_units,
    };

    #[test]
    fn parse_rejects_scientific_notation() {
        assert!(parse_decimal("1e3").is_err());
        assert!(parse_decimal("2E-3").is_err());
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
    fn parse_uses_plain_decimal_grammar() {
        for (literal, canonical) in [
            (".5", "0.5"),
            ("1.", "1"),
            ("+1", "1"),
            ("-0.00", "0"),
            ("001.2300", "1.23"),
            (" \t\n+001.2300\r", "1.23"),
        ] {
            let parsed = parse_decimal(literal)
                .unwrap_or_else(|error| panic!("{literal} should parse: {error}"));
            assert_eq!(to_canonical_string(&parsed), canonical);
        }
    }

    #[test]
    fn parse_rejects_non_plain_decimal_grammar() {
        for literal in [
            "", " ", "+", "-", ".", "+.", "-.", "+-1", "++1", "--1", "1_000", "1e3", "2E-3", "1 2",
            "1.2.3",
        ] {
            assert!(parse_decimal(literal).is_err(), "{literal} should fail");
        }
    }

    #[test]
    fn parse_rejects_duplicate_explicit_signs() {
        assert!(parse_decimal("+-1").is_err());
        assert!(parse_decimal("++1").is_err());
        assert!(parse_decimal("+").is_err());
        assert!(parse_decimal("+1").is_ok());
        assert!(parse_decimal("-1").is_ok());
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

    #[test]
    fn try_to_scaled_units_accepts_exact_values() {
        let value = parse_decimal("123.4560").unwrap();
        assert_eq!(try_to_scaled_units(&value, 6), Some(123_456_000));
        assert_eq!(try_to_scaled_units(&value, 3), Some(123_456));

        let negative = parse_decimal("-1.25").unwrap();
        assert_eq!(try_to_scaled_units(&negative, 2), Some(-125));
    }

    #[test]
    fn try_to_scaled_units_rejects_inexact_values_instead_of_rounding() {
        let above_half = parse_decimal("1.250001").unwrap();
        assert_eq!(try_to_scaled_units(&above_half, 1), None);

        let tie = parse_decimal("1.25").unwrap();
        assert_eq!(try_to_scaled_units(&tie, 1), None);
    }

    #[test]
    fn try_from_scaled_units_rejects_rust_decimal_limits() {
        assert!(try_from_scaled_units(i128::MAX, 0).is_none());
        assert!(try_from_scaled_units(1, 29).is_none());
    }
}
