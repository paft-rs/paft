//! Feature-dependent decimal abstraction for paft-money.

use std::str::FromStr;

#[cfg(all(feature = "rust-decimal", feature = "bigdecimal"))]
compile_error!(
    "features `rust-decimal` and `bigdecimal` are mutually exclusive; enable only one backend"
);

#[cfg(not(any(feature = "rust-decimal", feature = "bigdecimal")))]
compile_error!(
    "at least one decimal backend feature (`rust-decimal` or `bigdecimal`) must be enabled"
);

/// Rounding strategy supported by money operations.
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

#[cfg(all(feature = "rust-decimal", not(feature = "bigdecimal")))]
mod backend {
    use super::{FromStr, RoundingStrategy};

    pub use rust_decimal::Decimal;
    use rust_decimal::RoundingStrategy as RustRoundingStrategy;
    pub use rust_decimal::prelude::ToPrimitive;

    pub(super) fn parse_decimal(value: &str) -> Option<Decimal> {
        Decimal::from_str(value).ok()
    }

    pub(super) const fn zero() -> Decimal {
        Decimal::ZERO
    }

    pub(super) const fn one() -> Decimal {
        Decimal::ONE
    }

    pub(super) fn from_minor_units(value: i128, scale: u32) -> Decimal {
        Decimal::from_i128_with_scale(value, scale)
    }

    pub(super) fn round_dp_with_strategy(
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

#[cfg(all(feature = "bigdecimal", not(feature = "rust-decimal")))]
mod backend {
    use super::{FromStr, RoundingStrategy};

    pub use bigdecimal::BigDecimal as Decimal;
    use bigdecimal::RoundingMode;
    use num_bigint::BigInt;
    pub use num_traits::ToPrimitive;
    use num_traits::{One, Zero};

    pub(super) fn parse_decimal(value: &str) -> Option<Decimal> {
        Decimal::from_str(value).ok()
    }

    pub(super) fn zero() -> Decimal {
        Decimal::zero()
    }

    pub(super) fn one() -> Decimal {
        Decimal::one()
    }

    pub(super) fn from_minor_units(value: i128, scale: u32) -> Decimal {
        Decimal::new(BigInt::from(value), i64::from(scale))
    }

    pub(super) fn round_dp_with_strategy(
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
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rust-decimal", feature = "bigdecimal")))
)]
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
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rust-decimal", feature = "bigdecimal")))
)]
#[allow(clippy::missing_const_for_fn)]
pub fn zero() -> Decimal {
    backend::zero()
}

/// Returns the one value for the active decimal backend.
#[must_use]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rust-decimal", feature = "bigdecimal")))
)]
#[allow(clippy::missing_const_for_fn)]
pub fn one() -> Decimal {
    backend::one()
}

/// Builds a decimal from an integer count of minor units and the provided scale.
#[must_use]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rust-decimal", feature = "bigdecimal")))
)]
pub fn from_minor_units(value: i128, scale: u32) -> Decimal {
    backend::from_minor_units(value, scale)
}

/// Rounds a decimal to the requested scale using a rounding strategy.
#[must_use]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rust-decimal", feature = "bigdecimal")))
)]
pub fn round_dp_with_strategy(value: &Decimal, scale: u32, strategy: RoundingStrategy) -> Decimal {
    backend::round_dp_with_strategy(value, scale, strategy)
}
