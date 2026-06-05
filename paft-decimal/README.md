paft-decimal
============

Backend-agnostic decimal helpers for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-decimal)](https://crates.io/crates/paft-decimal)
[![Docs.rs](https://docs.rs/paft-decimal/badge.svg)](https://docs.rs/paft-decimal)
[![Downloads](https://img.shields.io/crates/d/paft-decimal)](https://crates.io/crates/paft-decimal)

- `Decimal` aliases the active backend: `rust_decimal::Decimal` by default, or
  `bigdecimal::BigDecimal` with the `bigdecimal` feature
- Backend-stable helpers for plain decimal parsing, canonical rendering,
  rounding, checked arithmetic, and scaled-unit construction
- Constrained decimal newtypes: `NonNegativeDecimal`, `PositiveDecimal`, and
  `Ratio`
- Serde adapters for canonical decimal strings
- `Decimal128Mantissa` for decimal128 mantissa encoding used by DataFrame
  integrations

Install
-------

Use the facade crate when you only need the decimal types it re-exports:

```toml
[dependencies]
paft = "0.9.0"
```

Depend directly when you need helpers such as `parse_decimal`,
`from_minor_units`, or the serde adapters:

```toml
[dependencies]
paft-decimal = "0.9.0"
```

Alternate decimal backend:

```toml
[dependencies]
paft-decimal = { version = "0.9.0", features = ["bigdecimal"] }
```

Features
--------

- `bigdecimal`: switch the active `Decimal` type from `rust_decimal::Decimal`
  to `bigdecimal::BigDecimal` for arbitrary precision decimals

Quickstart
----------

```rust
use paft_decimal::{self as decimal, NonNegativeDecimal, Ratio, RoundingStrategy};

let value = decimal::parse_decimal("00123.4500").unwrap();
assert_eq!(decimal::to_canonical_string(&value), "123.45");

let rounded =
    decimal::round_dp_with_strategy(&value, 1, RoundingStrategy::MidpointAwayFromZero);
assert_eq!(decimal::to_canonical_string(&rounded), "123.5");

let size = NonNegativeDecimal::new(decimal::from_minor_units(10, 0)).unwrap();
assert_eq!(size.to_string(), "10");

let pct = Ratio::new(decimal::parse_decimal("0.135").unwrap()).unwrap();
assert_eq!(pct.to_string(), "0.135");
assert!(Ratio::new(decimal::parse_decimal("1.2").unwrap()).is_err());
```

Serde Adapters
--------------

Use the serde helpers when a decimal-backed field must keep the same JSON wire
format under both decimal backends:

```rust
use paft_decimal::Decimal;

#[derive(serde::Serialize, serde::Deserialize)]
struct Payload {
    #[serde(with = "paft_decimal::serde::canonical_str")]
    amount: Decimal,
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    ratio: Option<Decimal>,
}
```

Links
-----

- API docs: https://docs.rs/paft-decimal
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
