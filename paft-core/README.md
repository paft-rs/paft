paft-core
=========

Core infrastructure utilities for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-core)](https://crates.io/crates/paft-core)
[![Docs.rs](https://docs.rs/paft-core/badge.svg)](https://docs.rs/paft-core)

- Workspace-wide error type (`PaftError`)
- Enum macros for canonical string codes (`string_enum_*`, `impl_display_via_code`)
- Reusable serde helpers for timestamp encodings
- Optional re-exports for lightweight DataFrame traits

Install
-------

```toml
[dependencies]
paft-core = "0.3.0"
```

Features
--------

- `dataframe`: Re-exports `ToDataFrame`/`ToDataFrameVec` traits from `paft-utils`.

Quickstart
----------

```rust
use paft_core::{PaftError, string_enum_closed_with_code, impl_display_via_code};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Side { Buy, Sell }

paft_core::string_enum_closed_with_code!(
    Side, "Side",
    { "BUY" => Side::Buy, "SELL" => Side::Sell }
);
paft_core::impl_display_via_code!(Side);

assert_eq!(Side::Buy.code(), "BUY");
assert_eq!("sell".parse::<Side>().unwrap(), Side::Sell);
assert!(matches!("".parse::<Side>(), Err(PaftError::InvalidEnumValue { .. })));
```

Links
-----

- API docs: https://docs.rs/paft-core
- Workspace overview: ../README.md
