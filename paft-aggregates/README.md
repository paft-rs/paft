paft-aggregates
===============

Instant-in-time aggregate snapshot models for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-aggregates)](https://crates.io/crates/paft-aggregates)
[![Docs.rs](https://docs.rs/paft-aggregates/badge.svg)](https://docs.rs/paft-aggregates)
[![Downloads](https://img.shields.io/crates/d/paft-aggregates)](https://crates.io/crates/paft-aggregates)

- `Snapshot`: standard no-metadata alias for `GenericSnapshot<()>`
- `GenericSnapshot<M>`: instrument identity, session prices/ranges, volume,
  market state, UTC `as_of`, and flattened provider metadata
- Fundamentals, analyst coverage, and ESG fields live in `paft-fundamentals`

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = { version = "0.9.0", features = ["aggregates"] }
```

Advanced (direct dependency, minimal features):

```toml
[dependencies]
paft-aggregates = { version = "0.9.0", default-features = false }
```

Alternate decimal backend:

```toml
[dependencies]
paft-aggregates = { version = "0.9.0", default-features = false, features = ["bigdecimal"] }
```

With DataFrame integration:

```toml
[dependencies]
paft-aggregates = { version = "0.9.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

Features
--------

- `bigdecimal`: switch the shared decimal backend from `rust_decimal` to `bigdecimal`
- `panicking-money-ops`: forward to `paft-money` to enable panicking arithmetic operators
- `dataframe`: Polars integration for `Snapshot`; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`

Quickstart
----------

The quickstart below uses direct crate imports. Direct users should also add
the companion crates used by their constructors (`paft-decimal`, `paft-domain`,
and `paft-money`). Facade users can enable `paft/aggregates` and import through
`paft::prelude`.

```rust
use paft_aggregates::Snapshot;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Instrument};
use paft_money::{Currency, PriceAmount, QuantityAmount};

let instrument = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
let mut snapshot = Snapshot::new(instrument, Currency::try_from_str("USD").unwrap());
snapshot.last = Some(PriceAmount::new(Decimal::from(19_012) / Decimal::from(100)));
snapshot.previous_close = Some(PriceAmount::new(Decimal::from(18_996) / Decimal::from(100)));
snapshot.volume = Some(QuantityAmount::from_decimal(Decimal::from(78_900_000)).unwrap());

assert_eq!(snapshot.currency.to_string(), "USD");
assert_eq!(snapshot.last.as_ref().unwrap().to_string(), "190.12");
```

Snapshot notes
--------------

- `instrument` and `currency` are required; all observed market fields are optional.
- Provider metadata is serde-flattened into the snapshot JSON object. Avoid field
  names that collide with paft fields; prefix or nest provider fields when needed.

Links
-----

- API docs: [docs.rs/paft-aggregates](https://docs.rs/paft-aggregates)
- Workspace overview: [GitHub: workspace README](https://github.com/paft-rs/paft/blob/main/README.md)
- License: [LICENSE](../LICENSE)
