//! Utilities and helpers for working with `Currency` values.
//!
//! This module also provides the metadata overlay registry used when ISO 4217 is
//! silent about a currency's minor-unit exponent (e.g., metals like `XAU`, funds
//! like `XDR`). Use [`set_currency_metadata`] to register a name and decimal
//! places for such currencies so that [`Currency::decimal_places`](crate::currency::Currency::decimal_places)
//! can resolve a scale. If no overlay exists, money operations that require a
//! scale will return `MoneyError::MetadataNotFound`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::{LazyLock, RwLock};

use paft_utils::canonicalize;

use crate::currency::Currency;
use crate::error::MoneyParseError;

/// Maximum precision supported by the active decimal backend for safe scaling operations.
///
/// * With the default `rust-decimal` backend this reflects the 28 fractional digits that
///   `rust_decimal::Decimal` can represent safely.
#[cfg(not(feature = "bigdecimal"))]
pub const MAX_DECIMAL_PRECISION: u8 = 28;
/// Maximum precision supported by the active decimal backend for safe scaling operations.
#[cfg(feature = "bigdecimal")]
pub const MAX_DECIMAL_PRECISION: u8 = u8::MAX;
/// Maximum precision that can be converted into an `i64` minor-unit scale (10^18).
///
/// This is bounded by `10_i128.pow(scale)` fitting into an `i128`, ensuring minor-unit
/// conversions remain safe regardless of backend precision.
pub const MAX_MINOR_UNIT_DECIMALS: u8 = 18;

/// Metadata describing additional information for custom currencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyMetadata {
    /// Human-readable name for the currency.
    pub full_name: Cow<'static, str>,
    /// Number of decimal places (minor units) for the currency.
    pub minor_units: u8,
}

/// Errors that can occur when configuring minor-unit overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinorUnitError {
    /// The requested precision exceeds the decimal backend's supported limit.
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
                "decimal precision {decimals} exceeds maximum of {MAX_DECIMAL_PRECISION}"
            ),
            Self::ExceedsMinorUnitScale { decimals } => write!(
                f,
                "decimal precision {decimals} exceeds minor-unit scaling limit of {MAX_MINOR_UNIT_DECIMALS}"
            ),
        }
    }
}

impl std::error::Error for MinorUnitError {}

/// Built-in metadata for commonly used non-ISO currency codes.
const BUILTIN_CURRENCY_METADATA: &[(&str, &str, u8)] = &[
    ("USDC", "USD Coin", 6),
    ("USDT", "Tether", 6),
    ("BNB", "BNB", 8),
    ("ADA", "Cardano", 6),
    ("SOL", "Solana", 9),
    ("XRP", "XRP", 6),
    ("DOT", "Polkadot", 10),
    ("DOGE", "Dogecoin", 8),
    ("AVAX", "Avalanche", 8),
    ("LINK", "Chainlink", 8),
    ("LTC", "Litecoin", 8),
    ("MATIC", "Polygon", 8),
    ("UNI", "Uniswap", 8),
];

static BUILTIN_METADATA: LazyLock<HashMap<String, CurrencyMetadata>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (code, full_name, decimals) in BUILTIN_CURRENCY_METADATA {
        let canonical = canonicalize(code);
        map.insert(
            canonical,
            CurrencyMetadata {
                full_name: Cow::Borrowed(*full_name),
                minor_units: *decimals,
            },
        );
    }
    map
});

static CUSTOM_METADATA: LazyLock<RwLock<HashMap<String, CurrencyMetadata>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Attempts to normalize a currency code to a canonical variant or common `Other` value.
///
/// # Errors
/// Returns `MoneyParseError::InvalidEnumValue` when the code is empty or cannot be canonicalized.
pub fn try_normalize_currency_code(code: &str) -> Result<Currency, MoneyParseError> {
    Currency::try_from_str(code)
}

/// Registers metadata for a custom currency.
///
/// # Errors
/// Returns a `MinorUnitError` when the requested precision exceeds supported limits.
pub fn set_currency_metadata(
    code: &str,
    full_name: impl Into<String>,
    minor_units: u8,
) -> Result<Option<CurrencyMetadata>, MinorUnitError> {
    #[cfg(not(feature = "bigdecimal"))]
    if minor_units > MAX_DECIMAL_PRECISION {
        return Err(MinorUnitError::ExceedsDecimalPrecision {
            decimals: minor_units,
        });
    }
    if minor_units > MAX_MINOR_UNIT_DECIMALS {
        return Err(MinorUnitError::ExceedsMinorUnitScale {
            decimals: minor_units,
        });
    }

    let canonical = canonicalize(code);
    let full_name: String = full_name.into();
    let metadata = CurrencyMetadata {
        full_name: Cow::Owned(full_name),
        minor_units,
    };

    Ok(CUSTOM_METADATA
        .write()
        .map_or(None, |mut map| map.insert(canonical, metadata)))
}

/// Retrieves metadata for a custom currency, if registered.
pub fn currency_metadata(code: &str) -> Option<CurrencyMetadata> {
    let canonical = canonicalize(code);
    if let Some(custom) = CUSTOM_METADATA
        .read()
        .ok()
        .and_then(|map| map.get(&canonical).cloned())
    {
        return Some(custom);
    }

    BUILTIN_METADATA.get(&canonical).cloned()
}

/// Removes metadata for a custom currency and any associated minor-unit overrides.
pub fn clear_currency_metadata(code: &str) -> Option<CurrencyMetadata> {
    let canonical = canonicalize(code);
    CUSTOM_METADATA
        .write()
        .map_or(None, |mut map| map.remove(&canonical))
}
