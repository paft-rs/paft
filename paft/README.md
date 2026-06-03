# paft

**P**rovider **A**gnostic **F**inancial **T**ypes for Rust

[![Crates.io](https://img.shields.io/crates/v/paft)](https://crates.io/crates/paft)
[![Docs.rs](https://docs.rs/paft/badge.svg)](https://docs.rs/paft)
[![CI](https://github.com/paft-rs/paft/actions/workflows/ci.yml/badge.svg)](https://github.com/paft-rs/paft/actions/workflows/ci.yml)
[![Downloads](https://img.shields.io/crates/d/paft)](https://crates.io/crates/paft)
[![License](https://img.shields.io/crates/l/paft)](../LICENSE)

Standardized Rust types for financial data that work with any provider—Yahoo Finance, Bloomberg, Alpha Vantage, and more.

> 🌟 **Ecosystem Overview**: For the bigger picture, vision, and contributor guidance, see the [workspace README](https://github.com/paft-rs/paft/blob/main/README.md).

## Quick Install

```toml
[dependencies]
# Basic installation with default feature set (domain + market + fundamentals)
paft = "0.9.0"

# Or, add optional analysis helpers (Polars DataFrame support)
paft = { version = "0.9.0", features = ["dataframe"] }

# Or, opt into the aggregates snapshot model as well
paft = { version = "0.9.0", features = ["aggregates"] }

# Or, enable the full bundle of features
paft = { version = "0.9.0", features = ["full"] }

# Or, customize your installation
paft = { version = "0.9.0", default-features = false, features = ["fundamentals", "dataframe"] }

# Switch the money backend to BigDecimal (default is rust_decimal)
paft = { version = "0.9.0", features = ["bigdecimal"] }
```

## Feature Flags

All features are optional—disable the defaults (`default-features = false`) and opt back into what you need.

- `domain` *(default)*: exposes instrument, exchange, period, and other domain models.
- `market` *(default, enables `domain`)*: markets and history types such as `Quote`, `Candle`, and `HistoryRequest`.
- `fundamentals` *(default, enables `domain`)*: fundamentals, ESG, and ownership data structures.
- `aggregates`: exposes the `Snapshot` aggregated instrument-snapshot model.
- `bigdecimal`: swaps the money backend to `BigDecimal` when you require arbitrary precision.
- `dataframe`: forwards DataFrame support from `paft-utils`, providing `ToDataFrame`/`ToDataFrameVec`.
- `prediction`: prediction market data models (`Market`, `Token`).
- `full`: convenience bundle for `domain`, `market`, `fundamentals`, `aggregates`, `prediction`, and `dataframe`.
- `panicking-money-ops`: re-enables `Money` arithmetic operators that panic on mismatched currencies (see below).
- `money-formatting`: forwards to `paft-money/money-formatting` for locale-aware formatting and parsing APIs.
- `tracing`: enables lightweight instrumentation spans in selected constructors and validators for `paft-domain`, `paft-money`, `paft-market`, and `paft-fundamentals`; zero-cost when disabled.

## Migration Notes

- `Instrument` is a flat struct (`symbol`, `exchange`, `figi`, `isin`, `kind`); `IdentifierScheme`, `SecurityId`, and `PredictionID` are gone. Construct with the `from_*` helpers or a struct literal; access identifier fields directly (e.g. `inst.figi.as_ref()`). Prediction-market outcomes now live in `paft-prediction` as `PredictionInstrument`.
- `Instrument::figi` and `Instrument::isin` are typed `Option<Figi>` / `Option<Isin>`. Construct with `Figi::new("...")` and `Isin::new("...")`. When you need `&str`, use helpers like `inst.figi.as_ref().map(AsRef::as_ref)`.
- `CompanyProfile::isin` and `FundProfile::isin` now store `Option<Isin>`; update struct literals to pass `Isin::new(..)?` and adjust deserialization expectations accordingly.
- `Isin::new` and `Figi::new` now always enforce checksum validation. If you previously relied on lenient mode, strip placeholders or keep them in `Symbol` fields instead.
- The new identifier newtypes are `#[serde(transparent)]`, so existing JSON payloads continue to operate with plain strings while now enforcing checksum validation at the boundary.
- `paft-aggregates` no longer ships `FastInfo`/`Info`. Use `Snapshot` for strictly instant-in-time market data — fundamentals/analyst/ESG fields that lived on `Info` belong in the `paft-fundamentals` types.

## What's Included

### Core Types

- **Instruments**: `Instrument` (flat struct: `symbol`, `exchange`, `figi`, `isin`, `kind`), `AssetKind`
- **Market Data**: `Quote`, `Candle`, `HistoryResponse`, `MarketState`
- **Fundamentals**: Financial statements, earnings, analyst ratings, and trend/revision helper rows
- **Options**: `OptionContractKey`, `OptionSide`, `OptionContract`, `OptionGreeks`, `OptionChain`, `OptionUpdate`, `OptionExpirationsResponse`
- **News & Search**: `NewsArticle`, `NewsRequest`, `NewsTab`, `SearchRequest`, `SearchResult`
- **ESG & Holders**: ESG scores, institutional holdings
- **Aggregates** (feature `aggregates`): `Snapshot` instrument snapshots
- **Prediction Markets** (feature `prediction`): `Market`, `Token`

### Key Features

- **Hierarchical Identifiers**: FIGI → ISIN → Symbol@Exchange → Symbol priority
- **Extensible Enums**: Graceful handling of unknown provider values
- **DataFrame Integration**: Optional Polars support with `ToDataFrame` trait  
- **Full Serialization**: serde support for JSON, CSV, and other formats

## Quick Start

### Basic Usage

```rust
use paft::prelude::*;
use paft::money::IsoCurrency;

// Create instruments with different levels of identification
let apple = {
    // Prefer global IDs when available (FIGI > ISIN > Symbol@Exchange > Symbol)
    let symbol = Symbol::new("AAPL").unwrap();
    Instrument::from_figi("BBG000B9XRY4", symbol, AssetKind::Equity).unwrap()
};

let bitcoin = Instrument::from_symbol("BTC-USD", AssetKind::Crypto)
    .expect("valid crypto symbol");

// Create market data. `Quote` carries the full `Instrument`, plus today's
// volume, optional snapshot time, and a provider-metadata escape hatch
// (use `()` for "no metadata").
let usd = Currency::Iso(IsoCurrency::USD);
let quote = Quote {
    instrument: apple.clone(),
    name: Some("Apple Inc.".to_string()),
    currency: usd,
    price: Some(PriceAmount::new(Decimal::from(19012) / Decimal::from(100))),
    bid: None,
    ask: None,
    previous_close: Some(PriceAmount::new(Decimal::from(18996) / Decimal::from(100))),
    day_volume: Some(78_900_000),
    market_state: Some(MarketState::Regular),
    as_of: None,
    provider: (),
};
```

### Hierarchical Identifiers

```rust
// `unique_key()` is kind-aware and namespaced for identity use.
println!("{}", apple.unique_key());   // "EQUITY|FIGI|BBG000B9XRY4"
println!("{}", bitcoin.unique_key()); // "CRYPTO|SYMBOL|7:BTC-USD"

// `display_key()` keeps the compact identifier chain: FIGI > ISIN > Symbol@Exchange > Symbol.
println!("{}", apple.display_key());   // "BBG000B9XRY4"
println!("{}", bitcoin.display_key()); // "BTC-USD"

// Check identification levels — fields are public on the flat struct.
if apple.figi.is_some() || apple.isin.is_some() {
    println!("Has FIGI or ISIN - works across all providers");
}

// Access specific identifiers
if let Some(figi) = apple.figi.as_ref() {
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
paft = { version = "0.9.0", features = ["dataframe"] }
```

```rust
use paft::prelude::*;

let quotes = vec![quote1, quote2, quote3];
let df = quotes.to_dataframe()?;
if let Some(avg) = df.column("price.amount")?.as_materialized_series().mean() {
    println!("Average price: {avg:.2}");
}

// Contextual price fields flatten into amount columns such as `price.amount`;
// the containing record carries denomination in its `currency` column.
```

### Locale-aware money formatting and parsing

Enable the `money-formatting` feature to opt into locale-aware `Display` and strict parsing:

```toml
[dependencies]
paft = { version = "0.9.0", features = ["money-formatting"] }
```

```rust
use paft::money::{Currency, IsoCurrency, Money, Locale};

let m = Money::from_canonical_str("1234.56", Currency::Iso(IsoCurrency::USD))?;
let us = m.format_with_locale(Locale::EnUs)?;
let de = m.format_with_locale(Locale::EnEu)?;
assert_eq!(us, "$1,234.56");
assert_eq!(de, "$1.234,56");

// Strict parsing
let parsed = Money::from_str_locale("$1,234.56", Currency::Iso(IsoCurrency::USD), Locale::EnUs)?;
```

### Money operators and safety

By default, `Money` arithmetic operators (`+`, `-`, `/`, `*`) that would
panic on invalid input are disabled. Use the safe methods instead:

```rust
let sum = a.try_add(&b)?;
let diff = a.try_sub(&b)?;
let half = a.try_div(&Decimal::from(2))?;
```

If you explicitly want the ergonomic panicking operators, enable the
`panicking-money-ops` feature via the `paft` facade (it forwards to `paft-money`):

```toml
[dependencies]
paft = { version = "0.9.0", features = ["panicking-money-ops"] }
```

Note: This feature is opt-in and enables the `+`, `-`, `*`, and `/` operators to
panic on currency mismatch, division by zero, or conversion/metadata failures.
Prefer `try_*` methods in most apps.

For ergonomics in math-heavy code, you may enable this only when you control
the data end to end (e.g., internal pipelines with strict invariants) and are
absolutely sure all arithmetic uses matching currencies. For external or
untrusted data, keep this feature disabled and use the `try_*` APIs.

## Handling Unknown Values

paft uses extensible enums with `Other(Canonical)` variants to gracefully handle unknown provider values:

```rust
use paft::money::IsoCurrency;
use paft::prelude::*;

// Handle unknown currencies from providers
match currency {
    Currency::Iso(IsoCurrency::USD) => "US Dollar",
    Currency::Iso(IsoCurrency::EUR) => "Euro",
    Currency::BTC => "Bitcoin",
    Currency::Other(code) => match code.as_ref() {
        "XBT" => "Bitcoin alias",
        _ => "Unknown currency",
    },
    _ => "Known currency",
}

// Same pattern for exchanges, asset types, etc.
let exchange: Exchange = "DARK_POOL_X".parse().unwrap(); // Unknown exchange handled via Other
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

MIT License. See [LICENSE](../LICENSE) for details.
