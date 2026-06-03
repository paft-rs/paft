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
