paft-utils
==========

Shared utilities for the paft workspace: canonical string helpers and optional DataFrame traits.

[![Crates.io](https://img.shields.io/crates/v/paft-utils)](https://crates.io/crates/paft-utils)
[![Docs.rs](https://docs.rs/paft-utils/badge.svg)](https://docs.rs/paft-utils)

- Canonical string utilities: `Canonical`, `CanonicalError`, `MAX_CANONICAL_TOKEN_LEN`, `canonicalize`, `StringCode`
- Optional Polars helpers: `Columnar`, `Decimal128Encode`, `ToDataFrame`, `ToDataFrameVec`

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.9.0"
```

Advanced (direct dependency):

```toml
[dependencies]
paft-utils = { version = "0.9.0", default-features = false }
```

With DataFrame helpers:

```toml
[dependencies]
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] }
```

Features
--------

- `dataframe`: enable `polars` integration for fast columnar conversions
- `bigdecimal`: enable `Decimal128Encode` for `bigdecimal::BigDecimal` when combined with `dataframe`

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

`Canonical` is the storage invariant behind typed unknown enum wrappers such as
`OtherCurrency` and `OtherExchange`: values are non-empty, trimmed, uppercase
ASCII, no longer than `MAX_CANONICAL_TOKEN_LEN`, and use single underscores
between words.

```rust
use paft_utils::{Canonical, canonicalize};

assert_eq!(canonicalize(" dark-pool x "), "DARK_POOL_X");
assert!(Canonical::try_new("   ").is_err());
```

Links
-----

- API docs: https://docs.rs/paft-utils
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
