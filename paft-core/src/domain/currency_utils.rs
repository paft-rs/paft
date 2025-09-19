//! Utilities and helpers for working with `Currency` values.

use std::collections::HashMap;
use std::fmt;
use std::sync::{LazyLock, RwLock};

use super::Currency;
use super::string_canonical::canonicalize;

/// Maximum precision supported by `rust_decimal` for safe scaling operations.
pub(crate) const MAX_DECIMAL_PRECISION: u8 = 28;
/// Maximum precision that can be converted into an `i64` minor-unit scale (10^18).
pub(crate) const MAX_MINOR_UNIT_DECIMALS: u8 = 18;

/// Errors that can occur when configuring minor-unit overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinorUnitError {
    /// The requested precision exceeds what `rust_decimal` supports (28 fractional digits).
    ExceedsDecimalPrecision {
        /// Requested fractional digits.
        decimals: u8,
    },
    /// The requested precision would overflow `10_i64.pow(decimals)` used for minor units.
    ExceedsMinorUnitScale {
        /// Requested fractional digits.
        decimals: u8,
    },
}

impl fmt::Display for MinorUnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExceedsDecimalPrecision { decimals } => write!(
                f,
                "decimal precision {decimals} exceeds rust_decimal maximum of {MAX_DECIMAL_PRECISION}"
            ),
            Self::ExceedsMinorUnitScale { decimals } => write!(
                f,
                "decimal precision {decimals} exceeds minor-unit scaling limit of {MAX_MINOR_UNIT_DECIMALS}"
            ),
        }
    }
}

impl std::error::Error for MinorUnitError {}

/// Built-in precision overrides for commonly used non-ISO currency codes.
///
/// These values cover high-volume crypto assets and stablecoins that do not
/// have dedicated `Currency` enum variants but require non-standard
/// decimal precision.
const BUILTIN_MINOR_UNIT_OVERRIDES: &[(&str, u8)] = &[
    // Stablecoins
    ("USDC", 6),
    ("USDT", 6),
    // Major crypto assets
    ("BNB", 8),
    ("ADA", 6),
    ("SOL", 9),
    ("XRP", 6),
    ("DOT", 10),
    ("DOGE", 8),
    ("AVAX", 8),
    ("LINK", 8),
    ("LTC", 8),
    ("MATIC", 8),
    ("UNI", 8),
];

static MINOR_UNIT_OVERRIDES: LazyLock<RwLock<HashMap<String, u8>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (code, decimals) in BUILTIN_MINOR_UNIT_OVERRIDES {
        map.insert((*code).to_string(), *decimals);
    }
    RwLock::new(map)
});

/// Attempts to normalize a currency code to a canonical variant or common `Other` value.
///
/// # Errors
///
/// Returns an error on empty input to preserve `Currency` invariants.
pub fn try_normalize_currency_code(code: &str) -> Result<Currency, crate::error::PaftError> {
    Currency::try_from_str(code)
}

/// Returns the configured minor-unit precision for the provided currency code, if any.
#[must_use]
pub fn currency_minor_units(code: &str) -> Option<u8> {
    let canonical = canonicalize(code);
    MINOR_UNIT_OVERRIDES
        .read()
        .ok()
        .and_then(|map| map.get(&canonical).copied())
}

/// Registers or updates the minor-unit precision for a currency code.
///
/// Returns the previously configured precision, if one existed.
///
/// # Errors
///
/// Returns [`MinorUnitError`] when the requested precision would exceed either the
/// `rust_decimal` precision limit (28 fractional digits) or the safe minor-unit
/// scaling limit (18 fractional digits, used in `10_i64.pow` conversions).
pub fn set_currency_minor_units(code: &str, decimals: u8) -> Result<Option<u8>, MinorUnitError> {
    if decimals > MAX_DECIMAL_PRECISION {
        return Err(MinorUnitError::ExceedsDecimalPrecision { decimals });
    }
    if decimals > MAX_MINOR_UNIT_DECIMALS {
        return Err(MinorUnitError::ExceedsMinorUnitScale { decimals });
    }

    let canonical = canonicalize(code);
    Ok(MINOR_UNIT_OVERRIDES
        .write()
        .map_or(None, |mut map| map.insert(canonical, decimals)))
}

/// Removes any configured precision override for a currency code.
pub fn clear_currency_minor_units(code: &str) -> Option<u8> {
    let canonical = canonicalize(code);
    MINOR_UNIT_OVERRIDES
        .write()
        .map_or(None, |mut map| map.remove(&canonical))
}
