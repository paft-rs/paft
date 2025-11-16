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

use std::str::FromStr;

/// Rounding strategy supported by decimal operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn round_dp_with_strategy(
        value: &Decimal,
        scale: u32,
        strategy: RoundingStrategy,
    ) -> Decimal {
        let strategy: RustRoundingStrategy = strategy.into();
        value.round_dp_with_strategy(scale, strategy)
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
    let normalized = trimmed.strip_prefix('+').unwrap_or(trimmed);
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
#[must_use]
pub fn from_minor_units(value: i128, scale: u32) -> Decimal {
    backend::from_minor_units(value, scale)
}

/// Rounds a decimal to the requested scale using a rounding strategy.
#[must_use]
pub fn round_dp_with_strategy(value: &Decimal, scale: u32, strategy: RoundingStrategy) -> Decimal {
    backend::round_dp_with_strategy(value, scale, strategy)
}

/// Converts a decimal into a canonical string without scientific notation and
/// without gratuitous trailing zeros.
#[must_use]
pub fn to_canonical_string(value: &Decimal) -> String {
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
    use super::*;
    use std::str::FromStr;

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
    fn canonical_string_trims_trailing_zeros() {
        let value = parse_decimal("123.4500").unwrap();
        assert_eq!(to_canonical_string(&value), "123.45");
        let integer = parse_decimal("1000").unwrap();
        assert_eq!(to_canonical_string(&integer), "1000");
    }
}
