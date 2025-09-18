//! Example demonstrating how to properly handle the Other(String) extensible enum pattern
//! in paft. This example shows best practices for consuming paft types that use the
//! extensible enum pattern.

use paft::core::domain::{AssetKind, Currency, normalize_currency_code};
#[cfg(feature = "fundamentals")]
use paft::fundamentals::analysis::RecommendationGrade;

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
                _ => println!("  ⚠ Unknown currency: {code} - needs investigation"),
            },

            // Handle other known currencies
            _ => println!("  ✓ {currency} - known currency"),
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
            match asset {
                AssetKind::Equity => println!("  ✓ Equity - standard stock analysis"),
                AssetKind::Crypto => println!("  ✓ Crypto - blockchain analysis"),
                AssetKind::Fund => println!("  ✓ Fund - portfolio analysis"),
                AssetKind::Index => println!("  ✓ Index - benchmark analysis"),
                AssetKind::Forex => println!("  ✓ Forex - currency pairs"),
                AssetKind::Bond => println!("  ✓ Bond - fixed income analysis"),
                AssetKind::Commodity => println!("  ✓ Commodity - raw materials market"),
                AssetKind::Option => println!("  ✓ Option - derivatives analysis"),
                AssetKind::Other(_) => unreachable!(),
            }
        } else {
            // Handle unknown asset types
            if let AssetKind::Other(unknown_type) = asset {
                match unknown_type.as_str() {
                    "NFT" => println!("  ✓ NFT - non-fungible token analysis"),
                    "COMMODITY_FUTURE" => println!("  ✓ Commodity Future - derivative analysis"),
                    _ => println!("  ⚠ Unknown asset type: {unknown_type} - needs classification",),
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
                _ => println!("  ⚠ Unknown grade: {grade_str} - needs interpretation"),
            },
            _ => println!("  ✓ Other known grade: {grade}"),
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
        let currency = normalize_currency_code(code);
        println!("    {} -> {}", code, format_currency(&currency));
    }

    println!("  Alpha Vantage currencies:");
    for code in alpha_vantage_currencies {
        let currency = normalize_currency_code(code);
        println!("    {} -> {}", code, format_currency(&currency));
    }
    println!();
}

/// Normalize generic provider currency codes
/// Format currency for display
fn format_currency(currency: &Currency) -> String {
    match currency {
        Currency::USD => "USD (canonical)".to_string(),
        Currency::EUR => "EUR (canonical)".to_string(),
        Currency::Other(code) => format!("Other({code})"),
        _ => format!("{currency} (canonical)"),
    }
}
