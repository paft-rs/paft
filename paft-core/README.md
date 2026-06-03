paft-core
=========

Core infrastructure utilities for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-core)](https://crates.io/crates/paft-core)
[![Docs.rs](https://docs.rs/paft-core/badge.svg)](https://docs.rs/paft-core)

- Workspace-wide error type (`PaftError`)
- Enum macros for canonical string codes (`string_enum_*`, `impl_display_via_code`)
- Reusable serde helpers for timestamp encodings

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.9.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-core = { version = "0.9.0", default-features = false }
```

Serde helpers are included in the minimal crate; there are no optional serde feature flags.

Features
--------

This crate does not currently expose optional features. DataFrame traits now live in `paft-utils` and are forwarded by consumer crates (or the `paft` facade) via their own `dataframe` features.

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
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
