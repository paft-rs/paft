paft-money
==========

Currency and money primitives for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-money)](https://crates.io/crates/paft-money)
[![Docs.rs](https://docs.rs/paft-money/badge.svg)](https://docs.rs/paft-money)

- `Currency` with ISO 4217 integration and extensible fallback
- `Money` with safe arithmetic and explicit conversions via `ExchangeRate`
- Backend-agnostic decimals (`rust-decimal` by default or `bigdecimal`)
- Runtime currency metadata overlays for non-ISO minor units (e.g., `XAU`, `XDR`)

Install
-------

```toml
[dependencies]
paft-money = "0.3.0"
```

Features
--------

- `rust-decimal` (default): fast, fixed-size decimals (up to 28 fractional digits)
- `bigdecimal`: arbitrary precision decimals
- `dataframe`: Polars integration (`ToDataFrame`/`ToDataFrameVec`)
- `panicking-money-ops`: opt-in operator overloading that panics on invalid operations

Quickstart
----------

```rust
use iso_currency::Currency as IsoCurrency;
use paft_money::{Currency, Money};

let price = Money::from_str("12.34", Currency::Iso(IsoCurrency::USD))?;
let tax   = Money::from_str("1.23",  Currency::Iso(IsoCurrency::USD))?;
let total = price.try_add(&tax)?;
assert_eq!(total.format(), "13.57 USD");
# Ok::<(), paft_money::MoneyError>(())
```

Links
-----

- API docs: https://docs.rs/paft-money
- Workspace overview: ../README.md
