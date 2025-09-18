# paft

**P**rovider **A**gnostic **F**inancial **T**ypes for Rust

[![Crates.io](https://img.shields.io/crates/v/paft)](https://crates.io/crates/paft)
[![Docs.rs](https://docs.rs/paft/badge.svg)](https://docs.rs/paft)
[![CI](https://github.com/paft-rs/paft/actions/workflows/ci.yml/badge.svg)](https://github.com/paft-rs/paft/actions/workflows/ci.yml)
[![Downloads](https://img.shields.io/crates/d/paft)](https://crates.io/crates/paft)
[![License](https://img.shields.io/crates/l/paft)](LICENSE)

Standardized Rust types for financial data that work with any provider—Yahoo Finance, Bloomberg, Alpha Vantage, and more.

> 🌟 **Ecosystem Overview**: For the bigger picture, vision, and contributor guidance, see the [workspace README](../README.md).

## Quick Install

```toml
[dependencies]
# Basic installation with all supported data types
# default = ["market", "fundamentals"]
paft = "0.1.1"

# Or, install with all features enabled
paft = { version = "0.1.1", features = ["dataframe"] }

# Or, customize your installation
paft = { version = "0.1.1", default-features = false, features = ["fundamentals", "dataframe"] }
```

## What's Included

### Core Types

- **Instruments**: `Instrument` with hierarchical identifiers, `AssetKind`
- **Market Data**: `Quote`, `Candle`, `HistoryResponse`, `MarketState`  
- **Fundamentals**: Financial statements, earnings, analyst ratings
- **Options**: `OptionContract`, `OptionChain`
- **News & Search**: `NewsArticle`, `SearchResult`
- **ESG & Holders**: ESG scores, institutional holdings

### Key Features

- **Hierarchical Identifiers**: FIGI → ISIN → Symbol@Exchange → Symbol priority
- **Extensible Enums**: Graceful handling of unknown provider values
- **DataFrame Integration**: Optional Polars support with `ToDataFrame` trait  
- **Full Serialization**: serde support for JSON, CSV, and other formats

## Quick Start

### Basic Usage

```rust
// Create instruments with different levels of identification
let apple = Instrument::new(
    "AAPL",
    AssetKind::Equity,
    Some("BBG000B9XRY4".to_string()), // FIGI (best)
    Some("US0378331005".to_string()), // ISIN  
    Some(Exchange::NASDAQ),
);

let bitcoin = Instrument::from_symbol("BTC-USD", AssetKind::Crypto);

// Create market data
let quote = Quote {
    symbol: "AAPL".to_string(),
    shortname: Some("Apple Inc.".to_string()),
    price: Some(Money::new(Decimal::new(19012, 2), Currency::USD)),
    previous_close: Some(Money::new(Decimal::new(18996, 2), Currency::USD)),
    exchange: Some(Exchange::NASDAQ),
    market_state: Some(MarketState::Regular),
    ..Default::default()
};
```

### Hierarchical Identifiers

```rust
// Automatic prioritization: FIGI > ISIN > Symbol@Exchange > Symbol
println!("{}", apple.unique_key());   // "BBG000B9XRY4" (uses FIGI)
println!("{}", bitcoin.unique_key()); // "BTC-USD" (uses symbol)

// Check identification levels
if apple.is_globally_identified() {
    println!("Has FIGI or ISIN - works across all providers");
}

// Access specific identifiers
if let Some(figi) = apple.figi() {
    println!("FIGI: {}", figi);
}
```

### Historical Data

```rust
use paft::{HistoryRequest, Range, Interval};

// Request 6 months of daily data
let request = HistoryRequest::try_from_range(Range::M6, Interval::D1)?;
request.validate()?;
```

### DataFrame Integration

Enable DataFrame support for analysis:

```toml
[dependencies]
paft = { version = "0.1.1", features = ["dataframe"] }
```

```rust
use paft::ToDataFrame;

let quotes = vec![quote1, quote2, quote3];
let df = quotes.to_dataframe()?;
println!("Average price: {:.2}", df.column("price")?.mean()?);
```

## Handling Unknown Values

paft uses extensible enums with `Other(String)` variants to gracefully handle unknown provider values:

```rust
use paft::{Currency, Exchange};

// Handle unknown currencies from providers
match currency {
    Currency::USD => "US Dollar",
    Currency::EUR => "Euro", 
    Currency::Other(code) => {
        // Graceful fallback for unknown currencies
        match code.as_str() {
            "BTC" => "Bitcoin",
            _ => "Unknown currency",
        }
    }
}

// Same pattern for exchanges, asset types, etc.
let exchange = Exchange::Other("BATS".to_string()); // Unknown exchange
```

This pattern ensures your code never breaks when providers return new or unexpected values.

### More Details

- **[Extensible Enums Guide](docs/EXTENSIBLE_ENUMS.md)**: Complete documentation and examples
- **[Best Practices](docs/BEST_PRACTICES.md)**: Guidelines for library authors and consumers  
- **[Working Examples](examples/)**: See extensible enums in action

## License

MIT License. See [crates.io](https://crates.io/crates/paft) for details.
