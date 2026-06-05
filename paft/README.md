paft
====

Facade crate for Provider Agnostic Financial Types.

[![Crates.io](https://img.shields.io/crates/v/paft)](https://crates.io/crates/paft)
[![Docs.rs](https://docs.rs/paft/badge.svg)](https://docs.rs/paft)
[![Downloads](https://img.shields.io/crates/d/paft)](https://crates.io/crates/paft)

Use this crate when you want one dependency that re-exports the paft workspace:
money and decimal primitives, domain identifiers, market data, fundamentals,
optional aggregate snapshots, optional prediction-market types, and a unified
`paft::Error` / `paft::Result`.

- `paft::prelude` imports the common public types enabled by your features
- `paft::money` exposes currency, money, price, and quantity primitives
- `paft::domain`, `paft::market`, and `paft::fundamentals` are enabled by default
- `paft::aggregates`, `paft::prediction`, and `paft::dataframe` are feature-gated
- `paft::Decimal` follows the active decimal backend

Install
-------

Most applications should start with the default feature set:

```toml
[dependencies]
paft = "0.9.0"
```

The default features are `domain`, `market`, and `fundamentals`. For a smaller
dependency, disable defaults and opt into only what you use:

```toml
[dependencies]
paft = { version = "0.9.0", default-features = false, features = ["domain"] }
```

Add one or more optional features to the default set:

```toml
[dependencies]
paft = { version = "0.9.0", features = ["aggregates", "prediction", "dataframe"] }
```

Use `full` when you want every domain crate plus DataFrame support:

```toml
[dependencies]
paft = { version = "0.9.0", features = ["full"] }
```

Use `bigdecimal` when you need an arbitrary-precision decimal backend:

```toml
[dependencies]
paft = { version = "0.9.0", features = ["bigdecimal"] }
```

Features
--------

- `domain` (default): instruments, exchanges, identifiers, periods, horizons, and market state
- `market` (default, enables `domain`): quotes, history, options, order books, news, search, downloads, and request builders
- `fundamentals` (default, enables `domain`): profiles, statements, analysis rows, holders, ESG, and key statistics
- `aggregates` (enables `domain`): `Snapshot` instrument snapshots
- `prediction`: prediction-market `Market`, `Token`, and `PredictionInstrument` types
- `dataframe`: Polars DataFrame traits and implementations for enabled paft types
- `bigdecimal`: switch the shared decimal backend from `rust_decimal` to `bigdecimal`
- `money-formatting`: locale-aware `Money` formatting and strict parsing APIs
- `panicking-money-ops`: opt in to `Money` arithmetic operators that panic on invalid operations
- `tracing`: lightweight instrumentation in selected constructors and validators
- `full`: convenience bundle for `domain`, `market`, `fundamentals`, `aggregates`, `prediction`, and `dataframe`

Quickstart
----------

```rust
use paft::money::IsoCurrency;
use paft::prelude::*;

fn run() -> Result<()> {
    let instrument =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity)?;
    let currency = Currency::Iso(IsoCurrency::USD);

    let mut quote = Quote::new(instrument.clone(), currency.clone());
    quote.name = Some("Apple Inc.".to_string());
    quote.price = Some(PriceAmount::new(Decimal::from(19012) / Decimal::from(100)));
    quote.day_volume = Some(QuantityAmount::from_decimal(Decimal::from(78_900_000))?);
    quote.market_state = Some(MarketState::Regular);

    assert_eq!(quote.instrument.display_key(), "AAPL@NASDAQ");

    let request = HistoryRequest::builder()
        .range(Range::M6)
        .interval(Interval::D1)
        .prefer_adjusted_prices(true)
        .build()?;

    assert_eq!(request.interval(), Interval::D1);
    assert!(request.prefer_adjusted_prices());

    Ok(())
}

run().unwrap();
```

Global security identifiers are strongly typed. Use the flat `Instrument`
fields when FIGI or ISIN data is available:

```rust
use paft::prelude::*;

let instrument = Instrument {
    symbol: Symbol::new("AAPL").unwrap(),
    exchange: Some(Exchange::NASDAQ),
    figi: Some(Figi::new("BBG000B9XRY4").unwrap()),
    isin: Some(Isin::new("US0378331005").unwrap()),
    kind: AssetKind::Equity,
};

assert_eq!(instrument.unique_key(), "EQUITY|FIGI|BBG000B9XRY4");
assert_eq!(instrument.display_key(), "BBG000B9XRY4");
```

DataFrame support
-----------------

Enable the `dataframe` feature and import the traits from the facade prelude:

```toml
[dependencies]
paft = { version = "0.9.0", features = ["dataframe"] }
```

```rust
use paft::prelude::*;

let quotes = vec![quote];
let df = quotes.to_dataframe()?;
```

Contextual price fields flatten into amount columns such as `price.amount`;
the containing record carries denomination in its `currency` column. Provider
metadata columns are namespaced under `provider.*`.

Money formatting
----------------

Enable `money-formatting` when you need localized presentation or strict
locale-aware parsing. Canonical `Display` remains stable as `"<amount> <CODE>"`.

```toml
[dependencies]
paft = { version = "0.9.0", features = ["money-formatting"] }
```

```rust
use paft::money::{Currency, IsoCurrency, Locale, Money};

let usd = Currency::Iso(IsoCurrency::USD);
let money = Money::from_canonical_str("1234.56", usd.clone())?;

assert_eq!(money.format(), "1234.56 USD");
assert_eq!(money.format_with_locale(Locale::EnUs)?, "$1,234.56");

let parsed = Money::from_str_locale("$1,234.56", usd, Locale::EnUs)?;
assert_eq!(parsed, money);
```

Examples
--------

- [`examples/v09_ergonomics.rs`](examples/v09_ergonomics.rs): default facade usage without provider metadata
- [`examples/provider_metadata.rs`](examples/provider_metadata.rs): generic provider metadata payloads
- [`examples/metadata_dataframe.rs`](examples/metadata_dataframe.rs): DataFrame export with provider metadata
- [`examples/nested_metadata_propagation.rs`](examples/nested_metadata_propagation.rs): independent provider metadata at each nested layer
- [`examples/extensible_enums.rs`](examples/extensible_enums.rs): unknown provider tokens and typed `Other` wrappers

Links
-----

- API docs: https://docs.rs/paft
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
