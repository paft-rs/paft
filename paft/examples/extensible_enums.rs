//! Example demonstrating how to properly handle the Other(String) extensible enum pattern
//! in paft. This example shows best practices for consuming paft types that use the
//! extensible enum pattern.

#[cfg(feature = "fundamentals")]
use paft::fundamentals::RecommendationGrade;
use paft::{AssetKind, Currency};

fn main() {
    println!("=== paft Extensible Enum Pattern Examples ===\n");

    // Example 1: Handling currencies from different providers
    handle_currencies();

    // Example 2: Processing asset kinds with unknown types
    handle_asset_kinds();

    // Example 3: Working with recommendation grades
    #[cfg(feature = "fundamentals")]
    handle_recommendation_grades();

    // Example 4: Normalizing provider-specific data
    normalize_provider_data();
}

/// Example 1: Properly handling currencies with Other variants
fn handle_currencies() {
    println!("1. Handling Currencies:");

    let currencies = vec![
        Currency::USD,                                   // Canonical variant
        Currency::EUR,                                   // Canonical variant
        Currency::Other("BTC".to_string()),              // Crypto currency
        Currency::Other("ETH".to_string()),              // Crypto currency
        Currency::Other("UNKNOWN_CURRENCY".to_string()), // Truly unknown
    ];

    for currency in currencies {
        match currency {
            // Handle known currencies with full type safety
            Currency::USD => println!("  ✓ US Dollar - major reserve currency"),
            Currency::EUR => println!("  ✓ Euro - major reserve currency"),

            // Handle unknown currencies gracefully
            Currency::Other(code) => match code.as_str() {
                "BTC" => println!("  ✓ Bitcoin - crypto currency"),
                "ETH" => println!("  ✓ Ethereum - crypto currency"),
                _ => println!("  ⚠ Unknown currency: {} - needs investigation", code),
            },

            // Handle other known currencies
            _ => println!("  ✓ {} - known currency", currency),
        }
    }
    println!();
}

/// Example 2: Processing asset kinds with helper methods
fn handle_asset_kinds() {
    println!("2. Processing Asset Kinds:");

    let assets = vec![
        AssetKind::Equity,                                // Canonical
        AssetKind::Crypto,                                // Canonical
        AssetKind::Other("NFT".to_string()),              // New asset type
        AssetKind::Other("COMMODITY_FUTURE".to_string()), // Provider-specific
    ];

    for asset in assets {
        if asset.is_canonical() {
            // Safe to use exhaustive matching since we know it's canonical
            match asset {
                AssetKind::Equity => println!("  ✓ Equity - standard stock analysis"),
                AssetKind::Crypto => println!("  ✓ Crypto - blockchain analysis"),
                _ => unreachable!(), // Safe because is_canonical() returned true
            }
        } else {
            // Handle unknown asset types
            if let AssetKind::Other(unknown_type) = asset {
                match unknown_type.as_str() {
                    "NFT" => println!("  ✓ NFT - non-fungible token analysis"),
                    "COMMODITY_FUTURE" => println!("  ✓ Commodity Future - derivative analysis"),
                    _ => println!(
                        "  ⚠ Unknown asset type: {} - needs classification",
                        unknown_type
                    ),
                }
            }
        }
    }
    println!();
}

/// Example 3: Working with recommendation grades
#[cfg(feature = "fundamentals")]
fn handle_recommendation_grades() {
    println!("3. Processing Recommendation Grades:");

    let grades = vec![
        RecommendationGrade::StrongBuy,
        RecommendationGrade::Buy,
        RecommendationGrade::Other("MARKET_PERFORM".to_string()), // Provider-specific
        RecommendationGrade::Other("OUTPERFORM".to_string()),     // Alternative naming
    ];

    for grade in grades {
        match grade {
            RecommendationGrade::StrongBuy => println!("  ✓ Strong Buy - very bullish"),
            RecommendationGrade::Buy => println!("  ✓ Buy - bullish"),
            RecommendationGrade::Other(grade_str) => match grade_str.as_str() {
                "MARKET_PERFORM" => println!("  ✓ Market Perform - neutral (mapped from Other)"),
                "OUTPERFORM" => println!("  ✓ Outperform - bullish (mapped from Other)"),
                _ => println!("  ⚠ Unknown grade: {} - needs interpretation", grade_str),
            },
            _ => println!("  ✓ Other known grade: {}", grade),
        }
    }
    println!();
}

/// Example 4: Normalizing provider-specific data
fn normalize_provider_data() {
    println!("4. Normalizing Provider-Specific Data:");

    // Simulate data from different providers
    let generic_currencies = vec!["USD", "EUR", "BTC", "DOLLAR"];
    let alpha_vantage_currencies = vec!["US_DOLLAR", "EURO", "BITCOIN", "UNKNOWN"];

    println!("  Generic provider currencies:");
    for code in generic_currencies {
        let currency = normalize_generic_provider_currency(code);
        println!("    {} -> {}", code, format_currency(&currency));
    }

    println!("  Alpha Vantage currencies:");
    for code in alpha_vantage_currencies {
        let currency = normalize_alpha_vantage_currency(code);
        println!("    {} -> {}", code, format_currency(&currency));
    }
    println!();
}

/// Normalize generic provider currency codes
fn normalize_generic_provider_currency(code: &str) -> Currency {
    match code.to_uppercase().as_str() {
        "DOLLAR" => Currency::USD, // Map to canonical variant
        _ => Currency::from(code.to_string()),
    }
}

/// Normalize Alpha Vantage currency codes
fn normalize_alpha_vantage_currency(code: &str) -> Currency {
    match code.to_uppercase().as_str() {
        "US_DOLLAR" => Currency::USD, // Map to canonical variant
        "EURO" => Currency::EUR,      // Map to canonical variant
        "BITCOIN" => Currency::Other("BTC".to_string()), // Normalize crypto naming
        _ => Currency::from(code.to_string()),
    }
}

/// Format currency for display
fn format_currency(currency: &Currency) -> String {
    match currency {
        Currency::USD => "USD (canonical)".to_string(),
        Currency::EUR => "EUR (canonical)".to_string(),
        Currency::Other(code) => format!("Other({})", code),
        _ => format!("{} (canonical)", currency),
    }
}

/// Example of a utility function that encourages ecosystem convergence
pub mod currency_utils {
    use super::Currency;

    /// Attempts to normalize a currency code to a canonical variant or common Other value
    pub fn normalize_currency_code(code: &str) -> Currency {
        match code.to_uppercase().as_str() {
            // Map to canonical variants when possible
            "DOLLAR" | "US_DOLLAR" | "USD" => Currency::USD,
            "EURO" | "EUR" => Currency::EUR,
            "POUND" | "GBP" => Currency::GBP,

            // Normalize crypto currencies to common Other values
            "BITCOIN" | "XBT" => Currency::Other("BTC".to_string()),
            "ETHEREUM" => Currency::Other("ETH".to_string()),

            // Preserve other values as-is
            _ => Currency::from(code.to_string()),
        }
    }

    /// Returns true if the currency is commonly used in financial applications
    pub fn is_common_currency(currency: &Currency) -> bool {
        match currency {
            // Major fiat currencies
            Currency::USD | Currency::EUR | Currency::GBP | Currency::JPY => true,

            // Common crypto currencies
            Currency::Other(code) => matches!(code.as_str(), "BTC" | "ETH"),

            _ => false,
        }
    }

    /// Returns a human-readable description of the currency
    pub fn describe_currency(currency: &Currency) -> String {
        match currency {
            Currency::USD => "US Dollar".to_string(),
            Currency::EUR => "Euro".to_string(),
            Currency::GBP => "British Pound".to_string(),
            Currency::JPY => "Japanese Yen".to_string(),
            Currency::Other(code) => match code.as_str() {
                "BTC" => "Bitcoin".to_string(),
                "ETH" => "Ethereum".to_string(),
                _ => format!("Unknown currency ({})", code),
            },
            _ => format!("{}", currency),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "fundamentals")]
    #[cfg(feature = "fundamentals")]
    #[test]
    fn test_currency_normalization() {
        assert_eq!(
            currency_utils::normalize_currency_code("DOLLAR"),
            Currency::USD
        );
        assert_eq!(
            currency_utils::normalize_currency_code("BITCOIN"),
            Currency::Other("BTC".to_string())
        );
        assert_eq!(
            currency_utils::normalize_currency_code("UNKNOWN"),
            Currency::Other("UNKNOWN".to_string())
        );
    }

    #[cfg(feature = "fundamentals")]
    #[test]
    fn test_common_currency_detection() {
        assert!(currency_utils::is_common_currency(&Currency::USD));
        assert!(currency_utils::is_common_currency(&Currency::Other(
            "BTC".to_string()
        )));
        assert!(!currency_utils::is_common_currency(&Currency::Other(
            "UNKNOWN".to_string()
        )));
    }

    #[cfg(feature = "fundamentals")]
    #[test]
    fn test_currency_description() {
        assert_eq!(
            currency_utils::describe_currency(&Currency::USD),
            "US Dollar"
        );
        assert_eq!(
            currency_utils::describe_currency(&Currency::Other("BTC".to_string())),
            "Bitcoin"
        );
        assert_eq!(
            currency_utils::describe_currency(&Currency::Other("UNKNOWN".to_string())),
            "Unknown currency (UNKNOWN)"
        );
    }
}
