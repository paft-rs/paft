paft-aggregates
===============

Aggregated snapshot models built on the paft primitives.

[![Crates.io](https://img.shields.io/crates/v/paft-aggregates)](https://crates.io/crates/paft-aggregates)
[![Docs.rs](https://docs.rs/paft-aggregates/badge.svg)](https://docs.rs/paft-aggregates)

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

With DataFrame integration:

```toml
[dependencies]
paft-aggregates = { version = "0.9.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

What’s inside
--------------

- `Snapshot` — strictly instant-in-time snapshot for an instrument: identity, the current session's prices/ranges, and the snapshot timestamp. This is the standard no-metadata alias for `GenericSnapshot<()>`; use `GenericSnapshot<M = ()>` when you need a provider-metadata payload. Fundamentals/analyst/ESG fields belong in `paft-fundamentals`.

Quickstart
----------

```rust
use paft_aggregates::Snapshot;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Instrument};
use paft_money::{Currency, IsoCurrency, PriceAmount, QuantityAmount};

let instrument = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
let mut snapshot = Snapshot::new(instrument, Currency::Iso(IsoCurrency::USD));
snapshot.last = Some(PriceAmount::new(Decimal::from(19012) / Decimal::from(100)));
snapshot.previous_close = Some(PriceAmount::new(Decimal::from(18996) / Decimal::from(100)));
snapshot.volume = Some(QuantityAmount::from_decimal(Decimal::from(78_900_000)).unwrap());

assert_eq!(snapshot.currency.to_string(), "USD");
assert_eq!(snapshot.last.unwrap().to_string(), "190.12");
```

Features
--------

- `bigdecimal`: switch the shared decimal backend used by `paft-money` prices and `paft-utils` dataframe encoding from `rust_decimal` to `bigdecimal`
- `panicking-money-ops`: forwards to `paft-money` to enable panicking arithmetic operators
- `dataframe`: derives Polars dataframe support for `Snapshot`; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`

Links
-----

- API docs: [docs.rs/paft-aggregates](https://docs.rs/paft-aggregates)
- Workspace overview: [GitHub: workspace README](https://github.com/paft-rs/paft/blob/main/README.md)
- License: [LICENSE](../LICENSE)
