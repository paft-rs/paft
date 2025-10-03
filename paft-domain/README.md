paft-domain
===========

Domain modeling primitives for the paft ecosystem: instruments, exchanges, periods, and market state.

[![Crates.io](https://img.shields.io/crates/v/paft-domain)](https://crates.io/crates/paft-domain)
[![Docs.rs](https://docs.rs/paft-domain/badge.svg)](https://docs.rs/paft-domain)

- Strongly-typed identifiers (`Isin`, `Figi`) with optional validation
- `Instrument` with hierarchical identifiers (FIGI → ISIN → Symbol@Exchange → Symbol)
- Canonical, serde-stable enums (`Exchange`, `AssetKind`, `MarketState`)
- `Period` parsing for quarters, years, and dates with a canonical wire format

Install
-------

```toml
[dependencies]
paft-domain = "0.3.0"
```

Features
--------

- `rust-decimal` (default) | `bigdecimal`: choose the money backend via `paft-money`
- `dataframe`: enable DataFrame traits for Polars integration
- `isin-validate`: strict ISIN normalization/validation
- `figi-validate`: strict FIGI checksum validation
- `ident-validate`: convenience feature enabling both validations

Quickstart
----------

```rust
use paft_domain::{Instrument, AssetKind, Exchange, Period};

// Instrument with optional global identifiers
let aapl = Instrument::try_new(
    "AAPL",
    AssetKind::Equity,
    Some("BBG000B9XRY4"), // FIGI
    Some("US0378331005"), // ISIN
    Some(Exchange::NASDAQ),
).unwrap();

assert!(aapl.is_globally_identified());
assert_eq!(aapl.unique_key(), "BBG000B9XRY4");

// Period parsing with canonical output
let q4 = "2023-Q4".parse::<Period>().unwrap();
assert_eq!(q4.to_string(), "2023Q4");
```

Links
-----

- API docs: https://docs.rs/paft-domain
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: https://github.com/paft-rs/paft/blob/main/LICENSE
