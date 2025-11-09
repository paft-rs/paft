//! Example demonstrating the hierarchical identifier approach in the Instrument struct.
//!
//! This example shows how the Instrument struct serves as a container for multiple
//! types of identifiers, allowing providers to populate the identifiers they have
//! access to while encouraging the use of better identifiers when available.

#[cfg(feature = "domain")]
use paft::prelude::{AssetKind, Exchange, Figi, IdentifierScheme, Instrument, Isin, Symbol};

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
#[allow(clippy::too_many_lines)]
fn run_example() {
    println!("=== Hierarchical Identifier Examples ===\n");

    // Example 1: Generic provider (basic symbol + exchange)
    println!("1. Generic Provider (Symbol + Exchange):");
    let generic_instrument =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity)
            .expect("valid NASDAQ symbol");

    let symbol = match generic_instrument.id() {
        IdentifierScheme::Security(s) => s.symbol.to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   Symbol: {symbol}");
    let exchange = match generic_instrument.id() {
        IdentifierScheme::Security(s) => {
            s.exchange.as_ref().map_or("-", Exchange::code).to_string()
        }
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   Exchange: {exchange}");
    println!("   Unique Key: {}", generic_instrument.unique_key());
    let globally_identified = match generic_instrument.id() {
        IdentifierScheme::Security(s) => s.figi.is_some() || s.isin.is_some(),
        IdentifierScheme::Prediction(_) => false,
    };
    println!("   Globally Identified: {globally_identified}");
    println!();

    // Example 2: Professional data provider (all identifiers)
    println!("2. Professional Data Provider (All Identifiers):");
    let professional_instrument = {
        let symbol = Symbol::new("AAPL").unwrap();
        let figi = Figi::new("BBG000B9XRY4").unwrap();
        let isin = Isin::new("US0378331005").unwrap();
        let id = paft_domain::IdentifierScheme::Security(paft_domain::SecurityId {
            symbol,
            exchange: Some(Exchange::NASDAQ),
            figi: Some(figi),
            isin: Some(isin),
        });
        Instrument::new(id, AssetKind::Equity)
    };

    let symbol = match professional_instrument.id() {
        IdentifierScheme::Security(s) => s.symbol.to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   Symbol: {symbol}");
    let figi = match professional_instrument.id() {
        IdentifierScheme::Security(s) => s
            .figi
            .as_ref()
            .map_or("-", std::convert::AsRef::as_ref)
            .to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   FIGI: {figi}");
    let isin = match professional_instrument.id() {
        IdentifierScheme::Security(s) => s
            .isin
            .as_ref()
            .map_or("-", std::convert::AsRef::as_ref)
            .to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   ISIN: {isin}");
    let exchange = match professional_instrument.id() {
        IdentifierScheme::Security(s) => {
            s.exchange.as_ref().map_or("-", Exchange::code).to_string()
        }
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   Exchange: {exchange}");
    println!("   Unique Key: {}", professional_instrument.unique_key());
    let globally_identified = match professional_instrument.id() {
        IdentifierScheme::Security(s) => s.figi.is_some() || s.isin.is_some(),
        IdentifierScheme::Prediction(_) => false,
    };
    println!("   Globally Identified: {globally_identified}");
    println!();

    // Example 3: European provider (ISIN + symbol + exchange)
    println!("3. European Provider (ISIN + Symbol + Exchange):");
    let european_instrument = {
        let symbol = Symbol::new("ASML").unwrap();
        let isin = Isin::new("NL0010273215").unwrap();
        let id = paft_domain::IdentifierScheme::Security(paft_domain::SecurityId {
            symbol,
            exchange: Some(Exchange::Euronext),
            figi: None,
            isin: Some(isin),
        });
        Instrument::new(id, AssetKind::Equity)
    };

    let symbol = match european_instrument.id() {
        IdentifierScheme::Security(s) => s.symbol.to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   Symbol: {symbol}");
    let figi = match european_instrument.id() {
        IdentifierScheme::Security(s) => s
            .figi
            .as_ref()
            .map_or("-", std::convert::AsRef::as_ref)
            .to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   FIGI: {figi}");
    let isin = match european_instrument.id() {
        IdentifierScheme::Security(s) => s
            .isin
            .as_ref()
            .map_or("-", std::convert::AsRef::as_ref)
            .to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   ISIN: {isin}");
    let exchange = match european_instrument.id() {
        IdentifierScheme::Security(s) => {
            s.exchange.as_ref().map_or("-", Exchange::code).to_string()
        }
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   Exchange: {exchange}");
    println!("   Unique Key: {}", european_instrument.unique_key());
    let globally_identified = match european_instrument.id() {
        IdentifierScheme::Security(s) => s.figi.is_some() || s.isin.is_some(),
        IdentifierScheme::Prediction(_) => false,
    };
    println!("   Globally Identified: {globally_identified}");
    println!();

    // Example 4: Minimal provider (symbol only)
    println!("4. Minimal Provider (Symbol Only):");
    let minimal_instrument =
        Instrument::from_symbol("BTC-USD", AssetKind::Crypto).expect("valid crypto symbol");

    let symbol = match minimal_instrument.id() {
        IdentifierScheme::Security(s) => s.symbol.to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   Symbol: {symbol}");
    let figi = match minimal_instrument.id() {
        IdentifierScheme::Security(s) => s
            .figi
            .as_ref()
            .map_or("-", std::convert::AsRef::as_ref)
            .to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   FIGI: {figi}");
    let isin = match minimal_instrument.id() {
        IdentifierScheme::Security(s) => s
            .isin
            .as_ref()
            .map_or("-", std::convert::AsRef::as_ref)
            .to_string(),
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   ISIN: {isin}");
    let exchange = match minimal_instrument.id() {
        IdentifierScheme::Security(s) => {
            s.exchange.as_ref().map_or("-", Exchange::code).to_string()
        }
        IdentifierScheme::Prediction(_) => "-".to_string(),
    };
    println!("   Exchange: {exchange}");
    println!("   Unique Key: {}", minimal_instrument.unique_key());
    let globally_identified = match minimal_instrument.id() {
        IdentifierScheme::Security(s) => s.figi.is_some() || s.isin.is_some(),
        IdentifierScheme::Prediction(_) => false,
    };
    println!("   Globally Identified: {globally_identified}");
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
