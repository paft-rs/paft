//! Example demonstrating the hierarchical identifier approach in the Instrument struct.
//!
//! This example shows how the Instrument struct serves as a container for multiple
//! types of identifiers, allowing providers to populate the identifiers they have
//! access to while encouraging the use of better identifiers when available.

#[cfg(feature = "domain")]
use paft::prelude::{AssetKind, Exchange, Instrument};

#[cfg(feature = "domain")]
use std::collections::HashMap;

#[allow(clippy::too_many_lines)]
fn main() {
    #[cfg(feature = "domain")]
    {
        run_example();
    }

    #[cfg(not(feature = "domain"))]
    {
        println!("This example requires the 'domain' feature to be enabled.");
        println!("Run with: cargo run --example hierarchical_identifiers --features domain");
    }
}

#[cfg(feature = "domain")]
fn run_example() {
    println!("=== Hierarchical Identifier Examples ===\n");

    // Example 1: Generic provider (basic symbol + exchange)
    println!("1. Generic Provider (Symbol + Exchange):");
    let generic_instrument =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity);

    println!("   Symbol: {}", generic_instrument.symbol());
    println!(
        "   Exchange: {}",
        generic_instrument.exchange().map_or("-", Exchange::code)
    );
    println!("   Unique Key: {}", generic_instrument.unique_key());
    println!(
        "   Globally Identified: {}",
        generic_instrument.is_globally_identified()
    );
    println!("   Has FIGI: {}", generic_instrument.has_figi());
    println!("   Has ISIN: {}", generic_instrument.has_isin());
    println!();

    // Example 2: Professional data provider (all identifiers)
    println!("2. Professional Data Provider (All Identifiers):");
    let professional_instrument = Instrument::new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4".to_string()), // FIGI
        Some("US0378331005".to_string()), // ISIN
        Some(Exchange::NASDAQ),
    );

    println!("   Symbol: {}", professional_instrument.symbol());
    println!("   FIGI: {:?}", professional_instrument.figi());
    println!("   ISIN: {:?}", professional_instrument.isin());
    println!(
        "   Exchange: {}",
        professional_instrument
            .exchange()
            .map_or("-", Exchange::code)
    );
    println!("   Unique Key: {}", professional_instrument.unique_key());
    println!(
        "   Globally Identified: {}",
        professional_instrument.is_globally_identified()
    );
    println!();

    // Example 3: European provider (ISIN + symbol + exchange)
    println!("3. European Provider (ISIN + Symbol + Exchange):");
    let european_instrument = Instrument::new(
        "ASML",
        AssetKind::Equity,
        None,                             // No FIGI
        Some("NL0010273215".to_string()), // ISIN
        Some(Exchange::Euronext),
    );

    println!("   Symbol: {}", european_instrument.symbol());
    println!("   FIGI: {:?}", european_instrument.figi());
    println!("   ISIN: {:?}", european_instrument.isin());
    println!(
        "   Exchange: {}",
        european_instrument.exchange().map_or("-", Exchange::code)
    );
    println!("   Unique Key: {}", european_instrument.unique_key());
    println!(
        "   Globally Identified: {}",
        european_instrument.is_globally_identified()
    );
    println!();

    // Example 4: Minimal provider (symbol only)
    println!("4. Minimal Provider (Symbol Only):");
    let minimal_instrument = Instrument::from_symbol("BTC-USD", AssetKind::Crypto);

    println!("   Symbol: {}", minimal_instrument.symbol());
    println!("   FIGI: {:?}", minimal_instrument.figi());
    println!("   ISIN: {:?}", minimal_instrument.isin());
    println!(
        "   Exchange: {}",
        minimal_instrument.exchange().map_or("-", Exchange::code)
    );
    println!("   Unique Key: {}", minimal_instrument.unique_key());
    println!(
        "   Globally Identified: {}",
        minimal_instrument.is_globally_identified()
    );
    println!();

    // Example 5: Demonstrating identifier prioritization
    println!("5. Identifier Prioritization:");
    let instruments = vec![
        ("With FIGI", professional_instrument.clone()),
        ("With ISIN only", european_instrument.clone()),
        ("With Symbol+Exchange", generic_instrument.clone()),
        ("Symbol only", minimal_instrument.clone()),
    ];

    for (description, instrument) in instruments {
        println!("   {}: {}", description, instrument.unique_key());
    }
    println!();

    // Example 6: Using instruments in collections
    println!("6. Using Instruments in Collections:");

    let mut instrument_map = HashMap::new();
    instrument_map.insert(professional_instrument.unique_key(), "Apple Inc.");
    instrument_map.insert(european_instrument.unique_key(), "ASML Holding");
    instrument_map.insert(generic_instrument.unique_key(), "Apple (Generic)");
    instrument_map.insert(minimal_instrument.unique_key(), "Bitcoin USD");

    for (key, name) in &instrument_map {
        println!("   {key} -> {name}");
    }
    println!();
}
