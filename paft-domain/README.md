paft-domain
===========

Domain modeling primitives for the paft ecosystem: instruments, exchanges, periods, and market state.

[![Crates.io](https://img.shields.io/crates/v/paft-domain)](https://crates.io/crates/paft-domain)
[![Docs.rs](https://docs.rs/paft-domain/badge.svg)](https://docs.rs/paft-domain)

- Strongly-typed identifiers for securities and prediction markets (`Symbol`, `Figi`, `Isin`, `EventID`, `OutcomeID`) with enforced validation
- `Instrument` with hierarchical identifiers for securities (FIGI → ISIN → Symbol@Exchange → Symbol)
- Canonical, serde-stable enums (`Exchange`, `AssetKind`, `MarketState`)
- `Period` parsing for quarters, years, and dates with a canonical wire format

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.8.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-domain = { version = "0.8.0", default-features = false }
```

Alternate decimal backend: enable on dependent crates (e.g., via the facade):

```toml
[dependencies]
paft = { version = "0.8.0", features = ["bigdecimal"] }
```

Enable DataFrame helpers as needed:

```toml
[dependencies]
paft-domain = { version = "0.8.0", default-features = false, features = ["dataframe"] }
```

Features
--------

- `tracing`: enable lightweight instrumentation on constructors and validators
- `dataframe`: enable DataFrame traits for Polars integration

Quickstart
----------

```rust
use paft_domain::{
    AssetKind, Exchange, Figi, IdentifierScheme, Instrument, Isin, Period, SecurityId, Symbol,
};

// Minimal: instrument from symbol + exchange
let aapl = Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity)
    .unwrap();

// Globally-identified: provide FIGI/ISIN (preferred over symbol)
let id = SecurityId {
    symbol: Symbol::new("AAPL").unwrap(),
    exchange: Some(Exchange::NASDAQ),
    figi: Some(Figi::new("BBG000B9XRY4").unwrap()),
    isin: Some(Isin::new("US0378331005").unwrap()),
};
let aapl_pro = Instrument::new(IdentifierScheme::Security(id), AssetKind::Equity);
assert_eq!(aapl_pro.unique_key(), "BBG000B9XRY4");

// Period parsing with canonical output (wire = Display)
let q4 = "2023-Q4".parse::<Period>().unwrap();
assert_eq!(q4.to_string(), "2023Q4");
```

Prediction markets
------------------

```rust
use paft_domain::Instrument;

// Create an instrument for a prediction market outcome
let pm = Instrument::from_prediction_market(
    "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de",
    "73470541315377973562501025254719659796416871135081220986683321361000395461644",
).unwrap();

// Unique key for prediction markets is the outcome_id
assert_eq!(
    pm.unique_key(),
    "73470541315377973562501025254719659796416871135081220986683321361000395461644"
);
```

Links
-----

- API docs: https://docs.rs/paft-domain
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: https://github.com/paft-rs/paft/blob/main/LICENSE
