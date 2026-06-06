paft-core
=========

Shared error, enum, and serde building blocks for paft crates.

[![Crates.io](https://img.shields.io/crates/v/paft-core)](https://crates.io/crates/paft-core)
[![Docs.rs](https://docs.rs/paft-core/badge.svg)](https://docs.rs/paft-core)
[![Downloads](https://img.shields.io/crates/d/paft-core)](https://crates.io/crates/paft-core)

- Shared enum parsing error (`PaftError`)
- Canonical string enum macros (`string_enum_*`, `impl_display_via_code`)
- `other_string_code_type` for typed open-enum fallback codes
- Serde helpers for timestamp encodings

Install
-------

Most applications should depend on the facade crate:

```toml
[dependencies]
paft = "0.9.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-core = { version = "0.9.0", default-features = false }
```

Use `paft-core` directly when defining paft-compatible crates or local enum
models that need the macro toolkit.

Features
--------

`paft-core` has no optional features. Serde support and timestamp helpers are
always available; DataFrame traits live in `paft-utils`.

Quickstart
----------

```rust
use paft_core::PaftError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Side {
    Buy,
    Sell,
}

paft_core::string_enum_closed_with_code!(
    Side,
    "Side",
    { "BUY" => Side::Buy, "SELL" => Side::Sell }
);
paft_core::impl_display_via_code!(Side);

assert_eq!(Side::Buy.code(), "BUY");
assert_eq!("sell".parse::<Side>().unwrap(), Side::Sell);
assert!(matches!("".parse::<Side>(), Err(PaftError::InvalidEnumValue { .. })));
```

Open enums with typed `Other`
-----------------------------

Use open enum macros for provider-facing concepts where upstreams can add new
tokens. The typed `OtherX` wrapper preserves unknown values while rejecting
tokens the enum already models. Unknown tokens are normalized into bounded
canonical strings.

```rust
use paft_core::PaftError;

paft_core::other_string_code_type!(
    /// Provider-specific venue not modeled by `Venue`.
    pub struct OtherVenue for Venue;
    type Error = PaftError;
    parse(input) => input.parse::<Venue>();
    invalid(input) => PaftError::InvalidEnumValue {
        enum_name: "Venue",
        value: input.to_string(),
    };
);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Venue {
    Nasdaq,
    Nyse,
    Other(OtherVenue),
}

paft_core::string_enum_with_code!(
    Venue, Other(OtherVenue), "Venue",
    {
        "NASDAQ" => Venue::Nasdaq,
        "NYSE" => Venue::Nyse
    },
    {
        "NASDAQ_GS" => Venue::Nasdaq,
        "NEW_YORK_STOCK_EXCHANGE" => Venue::Nyse
    }
);
paft_core::impl_display_via_code!(Venue);

assert_eq!("nasdaq-gs".parse::<Venue>().unwrap(), Venue::Nasdaq);
assert_eq!("dark pool".parse::<Venue>().unwrap().to_string(), "DARK_POOL");
assert!(OtherVenue::new("NASDAQ").is_err());
```

Links
-----

- API docs: https://docs.rs/paft-core
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
