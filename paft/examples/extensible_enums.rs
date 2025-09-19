//! Example demonstrating how to properly handle the Other(String) extensible enum pattern
//! in paft. This example shows best practices for consuming paft types that use the
//! extensible enum pattern.

use paft::core::domain::Currency;
#[cfg(feature = "fundamentals")]
use paft::fundamentals::analysis::RecommendationGrade;

fn main() {
    println!("=== paft Extensible Enum Pattern Examples ===\n");

    // Example 1: Handling currencies from different providers
    handle_currencies();

    // Example 2: Working with recommendation grades
    #[cfg(feature = "fundamentals")]
    handle_recommendation_grades();

    // Example 3: Normalizing provider-specific data
    normalize_provider_data();

    // Example 4: Demonstrate FromStr on canonical and alias inputs
    fromstr_demo();
}

/// Example 1: Properly handling currencies with Other variants
fn handle_currencies() {
    println!("1. Handling Currencies:");

    let input_codes = [
        "USD",
        "EUR",
        "BTC",
        "ETH",
        "UNKNOWN_CURRENCY",
        "xbt",
        "bitcoin",
    ];
    let currencies: Vec<Currency> = input_codes
        .iter()
        .map(|c| c.parse::<Currency>().unwrap())
        .collect();

    for currency in currencies {
        match currency {
            // Handle known currencies with full type safety
            Currency::USD => println!("  ✓ US Dollar - major reserve currency"),
            Currency::EUR => println!("  ✓ Euro - major reserve currency"),
            // Handle other known currencies
            other if matches!(other, Currency::BTC | Currency::ETH) => {
                println!("  ✓ {other} - known currency");
            }
            // Handle unknown currencies gracefully
            other => println!(
                "  ⚠ Unknown currency: {} - needs investigation",
                other.code()
            ),
        }
    }
    println!();
}

/// Example 2: Working with recommendation grades
#[cfg(feature = "fundamentals")]
fn handle_recommendation_grades() {
    println!("2. Processing Recommendation Grades:");

    let grades = vec![
        RecommendationGrade::StrongBuy,
        RecommendationGrade::Buy,
        // Common provider synonym now maps to Hold
        "MARKET_PERFORM".parse::<RecommendationGrade>().unwrap(),
        RecommendationGrade::Outperform, // Canonical variant
    ];

    for grade in grades {
        match grade {
            RecommendationGrade::StrongBuy => println!("  ✓ Strong Buy - very bullish"),
            RecommendationGrade::Buy => println!("  ✓ Buy - bullish"),
            RecommendationGrade::Outperform => println!("  ✓ Outperform - bullish"),
            RecommendationGrade::Hold => println!("  ✓ Hold - neutral"),
            other => println!("  ⚠ Unknown grade: {other} - needs interpretation"),
        }
    }
    println!();
}

/// Example 3: Normalizing provider-specific data
fn normalize_provider_data() {
    println!("3. Normalizing Provider-Specific Data:");

    // Simulate data from different providers
    let generic_currencies = vec!["USD", "EUR", "BTC", "DOLLAR"];
    let alpha_vantage_currencies = vec!["US_DOLLAR", "EURO", "BITCOIN", "UNKNOWN", "XBT"];

    println!("  Generic provider currencies:");
    for code in generic_currencies {
        let currency = code.parse::<Currency>().unwrap();
        println!("    {} -> {}", code, format_currency(&currency));
    }

    println!("  Alpha Vantage currencies:");
    for code in alpha_vantage_currencies {
        let currency = code.parse::<Currency>().unwrap();
        println!("    {} -> {}", code, format_currency(&currency));
    }
    println!();
}

/// Example 4: Show `std::str::FromStr` acceptance of canonical and alias inputs
fn fromstr_demo() {
    let canonical = "USD".parse::<Currency>().unwrap();
    let alias = "us dollar".parse::<Currency>().unwrap();
    assert!(canonical.is_canonical());
    assert!(alias.is_canonical());
    println!("FromStr demo: USD == us dollar -> {}", (canonical == alias));
}

/// Normalize generic provider currency codes
/// Format currency for display
fn format_currency(currency: &Currency) -> String {
    match currency {
        Currency::USD => "USD (canonical)".to_string(),
        Currency::EUR => "EUR (canonical)".to_string(),
        _ => format!("{currency} (canonical)"),
    }
}
