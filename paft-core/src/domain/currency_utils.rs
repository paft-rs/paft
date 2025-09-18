//! Utilities and helpers for working with `Currency` values.

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use super::Currency;

/// Built-in precision overrides for commonly used non-ISO currency codes.
///
/// These values cover high-volume crypto assets and stablecoins that do not
/// have dedicated `Currency` enum variants but require non-standard
/// decimal precision.
const BUILTIN_MINOR_UNIT_OVERRIDES: &[(&str, u32)] = &[
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

static MINOR_UNIT_OVERRIDES: LazyLock<RwLock<HashMap<String, u32>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (code, decimals) in BUILTIN_MINOR_UNIT_OVERRIDES {
        map.insert((*code).to_string(), *decimals);
    }
    RwLock::new(map)
});

fn canonicalize(code: &str) -> String {
    code.trim().to_uppercase()
}

/// Attempts to normalize a currency code to a canonical variant or common `Other` value.
#[must_use]
pub fn normalize_currency_code(code: &str) -> Currency {
    let normalized = canonicalize(code);

    match normalized.as_str() {
        // Map to canonical variants when possible
        "DOLLAR" | "US_DOLLAR" | "USD" => Currency::USD,
        "EURO" | "EUR" => Currency::EUR,
        "POUND" | "GBP" => Currency::GBP,

        // Normalize crypto currencies to common Other values
        "BITCOIN" | "XBT" => Currency::BTC,
        "ETHEREUM" => Currency::ETH,

        // Preserve other values as-is
        _ => Currency::from(normalized),
    }
}

/// Returns `true` if the currency is commonly used in financial applications.
#[must_use]
pub fn is_common_currency(currency: &Currency) -> bool {
    match currency {
        // Major fiat currencies and commonly used cryptos
        Currency::USD
        | Currency::EUR
        | Currency::GBP
        | Currency::JPY
        | Currency::BTC
        | Currency::ETH => true,
        Currency::Other(code) => matches!(code.as_str(), "BTC" | "ETH"),

        _ => false,
    }
}

/// Returns a human-readable description of the currency.
#[must_use]
pub fn describe_currency(currency: &Currency) -> String {
    match currency {
        Currency::USD => "US Dollar".to_string(),
        Currency::EUR => "Euro".to_string(),
        Currency::GBP => "British Pound".to_string(),
        Currency::JPY => "Japanese Yen".to_string(),
        Currency::Other(code) => format!("Unknown currency ({code})"),
        _ => format!("{currency}"),
    }
}

/// Returns the configured minor-unit precision for the provided currency code, if any.
#[must_use]
pub fn currency_minor_units(code: &str) -> Option<u32> {
    let canonical = canonicalize(code);
    MINOR_UNIT_OVERRIDES
        .read()
        .ok()
        .and_then(|map| map.get(&canonical).copied())
}

/// Registers or updates the minor-unit precision for a currency code.
///
/// Returns the previously configured precision, if one existed.
pub fn set_currency_minor_units(code: &str, decimals: u32) -> Option<u32> {
    let canonical = canonicalize(code);
    MINOR_UNIT_OVERRIDES
        .write()
        .map_or(None, |mut map| map.insert(canonical, decimals))
}

/// Removes any configured precision override for a currency code.
pub fn clear_currency_minor_units(code: &str) -> Option<u32> {
    let canonical = canonicalize(code);
    MINOR_UNIT_OVERRIDES
        .write()
        .map_or(None, |mut map| map.remove(&canonical))
}
