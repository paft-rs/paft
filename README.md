# paft: Provider Agnostic Financial Types

[![Crates.io](https://img.shields.io/crates/v/paft)](https://crates.io/crates/paft)
[![Docs.rs](https://docs.rs/paft/badge.svg)](https://docs.rs/paft)
[![CI](https://github.com/paft-rs/paft/actions/workflows/ci.yml/badge.svg)](https://github.com/paft-rs/paft/actions/workflows/ci.yml)
[![Downloads](https://img.shields.io/crates/d/paft)](https://crates.io/crates/paft)
[![License](https://img.shields.io/crates/l/paft)](https://crates.io/crates/paft)

**Building the unified ecosystem for financial data in Rust.**

> 🎯 **New to paft?** Start with the [paft crate README](paft/README.md) for practical usage examples and quick setup. This document focuses on the broader vision and ecosystem architecture.

## Vision

The financial data ecosystem is fragmented. Every provider—Yahoo Finance, Alpha Vantage, IEX Cloud, Polygon, etc.—has their own data formats, field names, and API structures. This fragmentation makes it difficult to:

- Switch between providers
- Build applications that work with multiple data sources
- Create reusable financial analysis libraries
- Maintain consistent data handling across projects

**paft** (Provider Agnostic Financial Types) solves this by providing a standardized set of Rust types that represent financial data in a provider-neutral way. The goal is to create an ecosystem where:

1. **Data providers** can build crates that convert their proprietary formats to paft types
2. **Application developers** can write analysis code that works with any provider's paft-compatible output
3. **Library authors** can build on a stable, well-defined foundation of standardized types
4. **The community** benefits from shared tooling and best practices around financial data structures

## The Dream

Imagine a future where financial data providers expose standardized types:

```rust
use paft::{Quote, HistoryRequest, Interval, Range};

// Each provider has their own API, but returns standardized paft types
async fn analyze_with_generic_provider(symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    let provider = GenericProvider::new();
    let quote = provider.get_quote(symbol).await?; // Returns paft::Quote
    let history = provider.get_history(symbol, Range::M6, Interval::D1).await?; // Returns paft::HistoryResponse
    
    analyze_data(quote, history);
    Ok(())
}

async fn analyze_with_alpha_vantage(symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    let av = AlphaVantage::new("your-api-key");
    let quote = av.get_quote(symbol).await?; // Returns paft::Quote
    let history = av.get_daily(symbol, Range::M6).await?; // Returns paft::HistoryResponse
    
    analyze_data(quote, history); // Same analysis function works!
    Ok(())
}

// Your analysis logic works with any provider's paft types
fn analyze_data(quote: paft::Quote, history: paft::HistoryResponse) {
    println!("Current price: ${:.2}", quote.price.as_ref().map(|p| p.amount).unwrap_or_default());
    println!("6-month high: ${:.2}", history.candles.iter().map(|c| c.high).max().unwrap_or_default());
}
```

**Key Point**: paft doesn't create a unified API across providers—each provider keeps their own methods, authentication, rate limits, and data access patterns. What paft provides is **standardized data structures** so your analysis code can work with any provider's output.

## What's Included

### Core Types

- **Instruments**: `Instrument`, `AssetKind` (Equity, Crypto, Fund, Index, etc.)
- **Market Data**: `Quote`, `Candle`, `MarketState`
- **Historical Data**: `HistoryRequest`, `HistoryResponse`, `Interval`, `Range`
- **Money & Currency**: `paft_money::Money`, `paft_money::Currency`
- **Fundamentals**: `CompanyProfile`, `IncomeStatementRow`, `BalanceSheetRow`, `CashflowRow`
- **Options**: `OptionContract`, `OptionChain`
- **News & Search**: `NewsArticle`, `SearchResult`
- **ESG**: `EsgScores`, `EsgInvolvement`
- **Holders**: `InstitutionalHolder`, `InsiderTransaction`

### Advanced Features

- **DataFrame Support**: Optional Polars integration with `ToDataFrame` trait (via `df-derive` proc-macros; enable with the `dataframe` feature)
- **Flexible Enums**: Type-safe enums with fallback variants for unknown values
- **Comprehensive Validation**: Built-in request validation and error handling
- **Serialization**: Full serde support for JSON, CSV, and other formats
- **Unified Error**: Single `paft::Error` enum and `paft::Result<T>` unify errors across crates
- **Feature Flags**:
  - `paft/dataframe`: Enables DataFrame helpers and derives through the facade
  - `paft/isin-validate`: Enables ISIN checksum validation and normalization across domain models (forwards to `paft-domain`).
  - `paft/figi-validate`: Enables FIGI checksum validation in constructors and serde (forwards to `paft-domain`).
  - `paft/ident-validate`: Convenience flag that enables both ISIN and FIGI validation (forwards to `paft-domain`).
  - `paft/panicking-money-ops` (opt-in): Enables ergonomic arithmetic operators on `Money` that panic on currency mismatch or division by zero. By default, operator overloads are disabled and you should use the safe `try_add`, `try_sub`, and `try_div` methods instead.
  - `paft/money-formatting` (opt-in): Locale‑aware money formatting and strict parsing APIs (re‑exports `Locale`/`LocalizedMoney`).
  - `paft/aggregates` (opt-in): Aggregated snapshot and reporting types (`FastInfo`, `Info`, `InfoReport`, `SearchReport`, `DownloadReport`).
  - `paft/bigdecimal` (opt-in): Switches the money backend to `BigDecimal`; `rust_decimal` is the implicit default.
  - `paft/full`: Convenience bundle for `domain`, `market`, `fundamentals`, `aggregates`, and `dataframe`.

  To enable panicking operators via the `paft` facade:

  ```toml
  [dependencies]
  paft = { version = "0.4.0", features = ["panicking-money-ops"] }
  ```

  Note: This feature is opt-in and enables the `+`, `-`, and `/` operators to panic
  on currency mismatch or division by zero. Prefer `try_*` methods in most apps.

  For ergonomics in math-heavy code, enable this only when you control the
  data end to end (e.g., internal pipelines with strict invariants) and are
  absolutely sure all arithmetic uses matching currencies. For external or
  untrusted data, keep this feature disabled and use the `try_*` APIs.

## What's NOT Included (Yet)

**Important**: paft currently focuses on **market data and fundamental analysis**, not trading execution. If you're building backtesting systems, trading bots, or portfolio management tools, you'll need additional types that paft doesn't provide yet:

### Missing Trading Types

- **Orders**: `Order`, `OrderType` (Market, Limit, Stop, etc.), `OrderStatus`
- **Trades**: `Trade`, `TradeExecution`, `Fill`
- **Positions**: `Position`, `Portfolio`, `Holding`
- **Account Data**: `Account`, `Balance`, `Cash`, `Margin`
- **Risk Management**: `RiskMetrics`, `Drawdown`, `SharpeRatio`
- **Strategy Types**: `Strategy`, `Signal`, `BacktestResult`
- **Performance**: `PerformanceMetrics`, `Returns`, `Benchmark`

### Why These Aren't Included

1. **Scope**: paft aims to standardize **market data types**, not trading infrastructure
2. **Complexity**: Trading systems have vastly different requirements (real-time vs backtesting, different brokers, etc.)
3. **Provider Diversity**: Trading APIs vary more than market data APIs
4. **Focus**: Better to do one thing well than many things poorly

### For Trading Applications

If you're building trading systems, consider:

- Using paft for market data ingestion and analysis
- Building your own trading types on top of paft's market data
- Looking into specialized trading crates like `ta` for technical analysis
- Using paft's DataFrame integration for backtesting data preparation

**Collaboration Welcome**: I would warmly welcome any collaboration on adding trading types to paft! If you're interested in contributing order types, portfolio management structures, or backtesting utilities, please reach out via [GitHub Issues](https://github.com/paft-rs/paft/issues) or [Discussions](https://github.com/paft-rs/paft/discussions).

## Quick Start

Ready to use paft in your project? Head to the [paft crate README](paft/README.md) for installation instructions, code examples, and practical usage guidance.

For a deeper dive into specific patterns and concepts, check out our [comprehensive documentation](paft/docs/):

- **[Extensible Enums](paft/docs/EXTENSIBLE_ENUMS.md)**: Understanding paft's graceful enum handling pattern
- **[Best Practices](paft/docs/BEST_PRACTICES.md)**: Guidelines for library authors and consumers
- **[Examples](paft/examples/)**: Working code examples for common patterns

## Ecosystem Architecture

The paft ecosystem is designed around interoperable layers that work together to create a unified financial data experience:

### Core Crates

- **`paft`** - Facade crate re-exporting standardized financial data types, unified error, and forwarded features. This is what most users will depend on directly.
- **`paft-domain`** - Domain models (`Instrument`, `Exchange`, `Period`), typed identifiers (`Isin`, `Figi`), and related errors.
- **`paft-market`** - Market data types, requests, and responses.
- **`paft-fundamentals`** - Fundamentals types (financial statements, ESG, holders, analysis helpers).
- **`paft-aggregates`** - Aggregated snapshot and reporting types for rollups, search, and downloads.
- **`paft-money`** - Currency and money primitives with ISO 4217 integration, safe arithmetic, and pluggable decimal backends.
- **`paft-utils`** - Canonical string utilities and DataFrame traits used across the workspace.
- **`paft-core`** - Infrastructure utilities and serde helpers used internally by the ecosystem.

### Ecosystem Layers

```text
┌─────────────────────────────────────┐
│          Your Application           │
├─────────────────────────────────────┤
│      Analysis & Visualization       │
│      (charts, backtesting, ML)      │
├─────────────────────────────────────┤
│           paft Core Types           │
│   (standardized data structures)    │
├─────────────────────────────────────┤
│         Provider Adapters           │
│    (generic-provider-paft, etc.)    │
├─────────────────────────────────────┤
│        Data Provider APIs           │
│ (Generic, Bloomberg, Alpha Vantage) │
└─────────────────────────────────────┘
```

### Provider Integration Philosophy

paft doesn't create a unified API—each provider maintains their unique methods, authentication, and rate limits. Instead, paft provides **standardized output types** that enable your analysis code to work with any provider's data.

## Building Provider Crates

Data provider crates are the bridge between proprietary APIs and standardized paft types. The recommended architecture balances efficiency with standardization:

### Implementation Strategy

1. **Keep your wire types**: Maintain existing serialization types for API efficiency
2. **Add conversion layer**: Create functions from wire types to paft types  
3. **Expose paft types**: Use paft types as your public API surface
4. **Leverage paft patterns**: Use extensible enums and hierarchical identifiers

### Provider Architecture Example

```rust
// Internal wire types (efficient for serialization)
#[derive(Deserialize)]
struct GenericQuoteWire {
    regularMarketPrice: Option<f64>,
    regularMarketPreviousClose: Option<f64>,
    exchange: Option<String>, // "NASDAQ", "NYSE", etc.
}

// Public API returns paft types
impl GenericProvider {
    pub async fn get_quote(&self, symbol: &str) -> Result<paft::Quote, Error> {
        let wire = self.fetch_quote_wire(symbol).await?;
        Ok(wire.into_paft_quote(symbol))
    }
}

// Conversion handles provider-specific mappings
impl GenericQuoteWire {
    fn into_paft_quote(self, symbol: &str) -> paft::Quote {
        paft::Quote {
            symbol: symbol.to_string(),
            price: self.regularMarketPrice.map(|amount| 
                paft::Money::new(amount.into(), paft::Currency::Iso(paft::IsoCurrency::USD))
            ),
            exchange: self.exchange.as_ref().map(|ex| match ex.as_ref() {
                "NASDAQ" => paft::Exchange::NASDAQ,
                "NYSE" => paft::Exchange::NYSE,
                other => paft::Exchange::Other(other.to_string()), // Graceful handling
            }),
            // ... other mappings
        }
    }
}
```

This approach allows provider crates to focus on their unique value-add (authentication, rate limiting, specialized endpoints) while ensuring output compatibility across the ecosystem.

## Contributing to the Ecosystem

Contributions of all kinds are welcome:

- Core/domain types and provider adapters
- Documentation, examples, and tests
- Performance and ergonomics improvements

Get started:

1. Read the [CONTRIBUTING.md](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md)
2. Open an [issue](https://github.com/paft-rs/paft/issues) or start a [discussion](https://github.com/paft-rs/paft/discussions)
3. Submit a pull request when ready

## Community

- **Discussions**: [GitHub Discussions](https://github.com/paft-rs/paft/discussions)
- **Issues**: [GitHub Issues](https://github.com/paft-rs/paft/issues)

## License

MIT License. See [LICENSE](https://github.com/paft-rs/paft/blob/main/LICENSE) for details.

## Acknowledgments

Inspired by the need for standardized financial data types in the Rust ecosystem. Special thanks to the Polars team for their excellent DataFrame library and the broader Rust community for their support.

---

**Ready to join the ecosystem?**

- **Users**: Start with the [paft crate](paft/README.md) for practical usage
- **Contributors**: Explore [contribution opportunities](#contributing-to-the-ecosystem)
- **Providers**: Build your adapter using our [integration guidelines](#building-provider-crates)

Together, we're building a standardized, interoperable, developer-friendly financial data ecosystem in Rust.

## Projects Using paft

The following open-source projects use paft types in their public APIs:

- [yfinance-rs](https://github.com/gramistella/yfinance-rs) — Ergonomic Yahoo Finance client (also on [crates.io](https://crates.io/crates/yfinance-rs)).

Want to add your project? Open a PR to include it here.
