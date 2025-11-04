paft-domain
===========

Domain modeling primitives for the paft ecosystem: instruments, exchanges, periods, and market state.

[![Crates.io](https://img.shields.io/crates/v/paft-domain)](https://crates.io/crates/paft-domain)
[![Docs.rs](https://docs.rs/paft-domain/badge.svg)](https://docs.rs/paft-domain)

- Strongly-typed identifiers (`Isin`, `Figi`) with enforced validation
- `Instrument` with hierarchical identifiers (FIGI → ISIN → Symbol@Exchange → Symbol)
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

- `bigdecimal`: change money backend from `rust_decimal` to `bigdecimal` via `paft-money`
- `dataframe`: enable DataFrame traits for Polars integration

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
