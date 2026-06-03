// Pedagogical example — shape and clarity are intentional.
#![allow(
    clippy::unnecessary_wraps,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::missing_panics_doc
)]

//! Extensible enum pattern in paft.
//!
//! Many provider-facing paft enums (`Currency`, `Exchange`, `AssetKind`,
//! `ReportingPeriod`, `RecommendationGrade`, ...) are open: they have a fixed set of
//! canonical variants plus a single typed `Other` fallback for tokens the
//! library does not model directly. Parsing never fails on unknown tokens —
//! they round-trip losslessly through the enum's typed unknown-code wrapper.
//!
//! Run with:
//!     cargo run -p paft --example extensible_enums --features full
//!
//! What this example demonstrates:
//! 1. Parsing well-known codes (`USD`, `EUR`) into ISO canonical variants.
//! 2. Parsing crypto aliases (`BTC`, `ETH`, plus the `XBT`/`BITCOIN` synonyms)
//!    into the canonical non-ISO variants — or into `Other` when no canonical
//!    equivalent exists.
//! 3. Working with `RecommendationGrade`, including a provider synonym
//!    (`MARKET_PERFORM`) that maps onto a canonical variant.
//! 4. The honest output for unknown provider codes: `Other(...)`.
//!    The provider sent the token, paft normalized it, and we preserve it
//!    even though it is not in our known set.

#[cfg(feature = "fundamentals")]
use paft::fundamentals::analysis::RecommendationGrade;
use paft::money::IsoCurrency;
use paft::prelude::Currency;

fn main() {
    println!("=== paft Extensible Enum Pattern Examples ===\n");

    // Example 1: Handling currencies from different providers
    handle_currencies();

    // Example 2: Working with recommendation grades
    #[cfg(feature = "fundamentals")]
    handle_recommendation_grades();

    // Example 3: Normalizing provider-specific data
    normalize_provider_data();
}

/// Example 1: Handling currencies. Some inputs (`USD`, `EUR`, `BTC`, `ETH`)
/// parse directly into canonical variants. Others (`xbt`, `bitcoin`,
/// `UNKNOWN_CURRENCY`) are preserved verbatim as typed `Other` values —
/// the parser uppercases and normalizes them but does not promote them onto
/// `Currency::BTC`. Mapping aliases onto canonical variants is the job of
/// your provider adapter (see also `handle_currencies` and the workspace
/// README provider mapping rules).
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
        if currency == Currency::Iso(IsoCurrency::USD) {
            println!("  US Dollar - major reserve currency");
        } else if currency == Currency::Iso(IsoCurrency::EUR) {
            println!("  Euro - major reserve currency");
        } else if currency == Currency::BTC || currency == Currency::ETH {
            println!("  {currency} - canonical non-ISO variant");
        } else {
            // Anything that didn't match above lands in a typed Other wrapper.
            println!(
                "  Unknown currency: {} - kept as Other, needs investigation",
                currency.code()
            );
        }
    }
    println!();
}

/// Example 2: Working with recommendation grades.
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
            RecommendationGrade::StrongBuy => println!("  Strong Buy - very bullish"),
            RecommendationGrade::Buy => println!("  Buy - bullish"),
            RecommendationGrade::Outperform => println!("  Outperform - bullish"),
            RecommendationGrade::Hold => println!("  Hold - neutral"),
            other => println!("  Unknown grade: {other} - needs interpretation"),
        }
    }
    println!();
}

/// Example 3: Normalizing provider-specific data. The output here intentionally
/// distinguishes between true canonical variants and typed `Other` fallbacks.
/// Provider tokens like `BITCOIN`, `DOLLAR`, `EURO`, `XBT` are preserved
/// verbatim by the parser but are NOT canonical — your provider adapter is the
/// right place to map them onto canonical variants.
fn normalize_provider_data() {
    println!("3. Normalizing Provider-Specific Data:");

    // Simulate data from different providers
    let generic_currencies = vec!["USD", "EUR", "BTC", "DOLLAR"];
    let alpha_vantage_currencies = vec!["US_DOLLAR", "EURO", "BITCOIN", "UNKNOWN", "XBT"];

    println!("  Generic provider currencies:");
    for code in generic_currencies {
        let currency = code.parse::<Currency>().unwrap();
        println!("    {} -> {}", code, classify(&currency));
    }

    println!("  Alpha Vantage currencies:");
    for code in alpha_vantage_currencies {
        let currency = code.parse::<Currency>().unwrap();
        println!("    {} -> {}", code, classify(&currency));
    }
    println!();
}

/// Classify a parsed currency: was it a canonical variant, or did it fall
/// through to `Other`? This is the honest version of what
/// most consumer code wants to know.
fn classify(currency: &Currency) -> String {
    match currency {
        // ISO 4217 codes — canonical, well-defined.
        Currency::Iso(iso) => format!("{} (canonical, ISO 4217)", iso.code()),
        // Non-ISO canonical variants paft models directly.
        Currency::BTC | Currency::ETH | Currency::XMR | Currency::USDC | Currency::USDT => {
            format!("{} (canonical, non-ISO)", currency.code())
        }
        // Everything else: provider-supplied token paft preserved verbatim.
        Currency::Other(canonical) => format!("Other({canonical}) - not in known set"),
        _ => format!("{} (canonical, non-ISO)", currency.code()),
    }
}
