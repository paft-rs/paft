paft-utils
==========

Shared utilities for the paft workspace: canonical string helpers and optional DataFrame traits.

[![Crates.io](https://img.shields.io/crates/v/paft-utils)](https://crates.io/crates/paft-utils)
[![Docs.rs](https://docs.rs/paft-utils/badge.svg)](https://docs.rs/paft-utils)

- Canonical string utilities: `Canonical`, `canonicalize`, `StringCode`
- Optional Polars helpers: `ToDataFrame`, `ToDataFrameVec`

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.5.1"
```

Advanced (direct dependency):

```toml
[dependencies]
paft-utils = { version = "0.5.1", default-features = false }
```

With DataFrame helpers:

```toml
[dependencies]
paft-utils = { version = "0.5.1", default-features = false, features = ["dataframe"] }
```

Features
--------

- `dataframe`: enable `polars` integration for fast columnar conversions

Quickstart
----------

```rust
use paft_utils::{canonicalize, Canonical};

assert_eq!(canonicalize("Euronext Paris"), "EURONEXT_PARIS");

let c = Canonical::try_new("nasdaq").unwrap();
assert_eq!(c.as_str(), "NASDAQ");
```

Links
-----

- API docs: https://docs.rs/paft-utils
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: https://github.com/paft-rs/paft/blob/main/LICENSE
