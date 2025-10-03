# paft

**P**rovider **A**gnostic **F**inancial **T**ypes for Rust

[![Crates.io](https://img.shields.io/crates/v/paft)](https://crates.io/crates/paft)
[![Docs.rs](https://docs.rs/paft/badge.svg)](https://docs.rs/paft)
[![CI](https://github.com/paft-rs/paft/actions/workflows/ci.yml/badge.svg)](https://github.com/paft-rs/paft/actions/workflows/ci.yml)
[![Downloads](https://img.shields.io/crates/d/paft)](https://crates.io/crates/paft)
[![License](https://img.shields.io/crates/l/paft)](LICENSE)

Standardized Rust types for financial data that work with any provider—Yahoo Finance, Bloomberg, Alpha Vantage, and more.

> 🌟 **Ecosystem Overview**: For the bigger picture, vision, and contributor guidance, see the [workspace README](https://github.com/paft-rs/paft/blob/main/README.md).

## Quick Install

```toml
[dependencies]
# Basic installation with all supported data types
# default = ["domain", "market", "fundamentals", "rust-decimal"]
paft = "0.3.1"

# Or, install with all features enabled
paft = { version = "0.3.1", features = ["dataframe"] }

# Or, customize your installation
paft = { version = "0.3.1", default-features = false, features = ["fundamentals", "dataframe"] }
```

## Feature Flags

All features are optional—disable the defaults (`default-features = false`) and opt back into what you need.

- `domain` *(default)*: exposes instrument, exchange, period, and other domain models.
- `market` *(default, enables `domain`)*: markets and history types such as `Quote`, `Candle`, and `HistoryRequest`.
- `fundamentals` *(default, enables `domain`)*: fundamentals, ESG, and ownership data structures.
- `rust-decimal` *(default)*: uses `rust_decimal` as the money backend; mutually exclusive with `bigdecimal`.
- `bigdecimal`: swaps the money backend to `BigDecimal` when you require arbitrary precision.
- `dataframe`: forwards DataFrame support from `paft-utils`, providing `ToDataFrame`/`ToDataFrameVec`.
- `full`: convenience bundle for `domain`, `market`, `fundamentals`, and `dataframe`.
- `panicking-money-ops`: re-enables `Money` arithmetic operators that panic on mismatched currencies (see below).
- `isin-validate`: forwards to `paft-domain/isin-validate`, enabling ISIN checksum validation and normalization everywhere (including deserialization).
- `figi-validate`: forwards to `paft-domain/figi-validate`, enabling FIGI checksum validation in constructors and serde.
- `ident-validate`: convenience flag that enables both ISIN and FIGI validation (forwards to `paft-domain`).

## Migration Notes

- `Instrument::figi` and `Instrument::isin` are now typed as `Option<Figi>` / `Option<Isin>`. Use `Figi::new("...")` and `Isin::new("...")` to construct validated identifiers, and call `figi_str()` / `isin_str()` when you need a borrowed `&str`.
- `CompanyProfile::isin` and `FundProfile::isin` now store `Option<Isin>`; update struct literals to pass `Isin::new(..)?` and adjust deserialization expectations accordingly.
- The new identifier newtypes are `#[serde(transparent)]`, so existing JSON payloads continue to operate with plain strings while gaining validation where enabled.

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
use paft::prelude::*;
use rust_decimal::Decimal;

// Create instruments with different levels of identification
let apple = Instrument::try_new(
    "AAPL",
    AssetKind::Equity,
    Some("BBG000B9XRY4"), // FIGI (best)
    Some("US0378331005"),            // ISIN
    Some(Exchange::NASDAQ),
)
.expect("valid instrument");

let bitcoin = Instrument::from_symbol("BTC-USD", AssetKind::Crypto);

// Create market data
let quote = Quote {
    symbol: "AAPL".to_string(),
    shortname: Some("Apple Inc.".to_string()),
    price: Some(Money::from_str("190.12", Currency::Iso(IsoCurrency::USD)).unwrap()),
    previous_close: Some(Money::from_str("189.96", Currency::Iso(IsoCurrency::USD)).unwrap()),
    exchange: Some(Exchange::NASDAQ),
    market_state: Some(MarketState::Regular),
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
use paft::prelude::*;

// Request 6 months of daily data (validated in constructor)
let request = HistoryRequest::try_from_range(Range::M6, Interval::D1).unwrap();
```

### DataFrame Integration

Enable DataFrame support for analysis:

```toml
[dependencies]
paft = { version = "0.3.1", features = ["dataframe"] }
```

```rust
use paft::prelude::*;

let quotes = vec![quote1, quote2, quote3];
let df = quotes.to_dataframe()?;
println!("Average price: {:.2}", df.column("price")?.mean()?);
```

### Money operators and safety

By default, `Money` arithmetic operators (`+`, `-`, `/`, `*`) that would
panic on invalid input are disabled. Use the safe methods instead:

```rust
let sum = a.try_add(&b)?;
let diff = a.try_sub(&b)?;
let half = a.try_div(Decimal::from(2))?;
```

If you explicitly want the ergonomic panicking operators, enable the
`panicking-money-ops` feature via the `paft` facade (it forwards to `paft-money`):

```toml
[dependencies]
paft = { version = "0.3.1", features = ["panicking-money-ops"] }
```

Note: This feature is opt-in and enables the `+`, `-`, and `/` operators to panic
on currency mismatch or division by zero. Prefer `try_*` methods in most apps.

For ergonomics in math-heavy code, you may enable this only when you control
the data end to end (e.g., internal pipelines with strict invariants) and are
absolutely sure all arithmetic uses matching currencies. For external or
untrusted data, keep this feature disabled and use the `try_*` APIs.

## Handling Unknown Values

paft uses extensible enums with `Other(Canonical)` variants to gracefully handle unknown provider values:

```rust
use paft::prelude::*;

// Handle unknown currencies from providers
match currency {
    Currency::Iso(IsoCurrency::USD) => "US Dollar",
    Currency::Iso(IsoCurrency::EUR) => "Euro",
    Currency::Other(code) => match code.as_ref() {
        "BTC" => "Bitcoin",
        _ => "Unknown currency",
    },
}

// Same pattern for exchanges, asset types, etc.
let exchange: Exchange = "BATS".parse().unwrap(); // Unknown exchange handled via Other
```

This pattern ensures your code never breaks when providers return new or unexpected values.

## Canonical Codes vs Human Labels

Enums ship with three complementary string representations:

- **Wire**: `code()` returns the canonical token used in APIs and serialization.
- **Display**: `to_string()` mirrors `code()` so logging and dataframes stay consistent.
- **Human**: Opt-in helpers such as `Currency::full_name()`, `AssetKind::full_name()`, and `MarketState::full_name()` provide sentence-case labels for UI surfaces.

Keep the rule of thumb: *wire = code = Display; human prose = explicit helper*.

### More Details

- **[Extensible Enums Guide](docs/EXTENSIBLE_ENUMS.md)**: Complete documentation and examples
- **[Best Practices](docs/BEST_PRACTICES.md)**: Guidelines for library authors and consumers  
- **[Working Examples](examples/)**: See extensible enums in action

## License

MIT License. See [LICENSE](https://github.com/paft-rs/paft/blob/main/LICENSE) for details.
