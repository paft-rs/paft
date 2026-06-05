paft-utils
==========

Shared utilities used by paft crates: canonical string tokens for open enums and optional Polars DataFrame traits.

[![Crates.io](https://img.shields.io/crates/v/paft-utils)](https://crates.io/crates/paft-utils)
[![Docs.rs](https://docs.rs/paft-utils/badge.svg)](https://docs.rs/paft-utils)
[![Downloads](https://img.shields.io/crates/d/paft-utils)](https://crates.io/crates/paft-utils)

- Canonical string utilities: `Canonical`, `CanonicalError`, `MAX_CANONICAL_TOKEN_LEN`, `StringCode`, `canonicalize`, `has_canonical_token_boundaries`
- DataFrame traits behind `dataframe`: `Columnar`, `Decimal128Encode`, `ToDataFrame`, `ToDataFrameVec`

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.9.0"
```

Direct dependency for enum/token helpers:

```toml
[dependencies]
paft-utils = { version = "0.9.0", default-features = false }
```

With DataFrame helpers:

```toml
[dependencies]
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] }
```

With DataFrame helpers and the `bigdecimal` decimal backend:

```toml
[dependencies]
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe", "bigdecimal"] }
```

Features
--------

Default features are empty.

- `dataframe`: enable the shared `df-derive-core`/Polars trait runtime and `Decimal128Encode` for the active `paft-decimal` backend
- `bigdecimal`: when combined with `dataframe`, switch decimal128 encoding from `rust_decimal::Decimal` to `bigdecimal::BigDecimal`

Quickstart
----------

```rust
use paft_utils::{canonicalize, Canonical};

assert_eq!(canonicalize("Euronext Paris"), "EURONEXT_PARIS");

let c = Canonical::try_new("nasdaq").unwrap();
assert_eq!(c.as_str(), "NASDAQ");
```

Canonical codes
---------------

`canonicalize` normalizes provider strings into `[A-Z0-9]+(?:_[A-Z0-9]+)*`
tokens. It can return an empty string when the input has no ASCII
alphanumerics; use `Canonical::try_new` for values that will be stored or
serialized as an open-enum `Other` code.

```rust
use paft_utils::{Canonical, canonicalize};

assert_eq!(canonicalize("S&P 500").as_ref(), "S_P_500");
assert_eq!(canonicalize("!!!").as_ref(), "");

let token = Canonical::try_new(" dark-pool x ").unwrap();
assert_eq!(token.as_str(), "DARK_POOL_X");
assert!(Canonical::try_new("!!!").is_err());
```

Enum parsers use `has_canonical_token_boundaries` before resolving aliases so
inputs such as `"$USD"` or `"CLOSED!"` are not silently mapped to modeled enum
variants.

DataFrame traits
----------------

Enable `dataframe` when a downstream crate or derive macro needs the shared
trait identity. The traits are available from `paft_utils::dataframe` and are
also re-exported at the crate root when the feature is enabled.

`Decimal128Encode` produces the i128 mantissa expected by Polars decimal
columns, using half-even rounding on scale-down and returning `None` when the
value cannot fit Polars decimal precision.

Links
-----

- API docs: https://docs.rs/paft-utils
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
