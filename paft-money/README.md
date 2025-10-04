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

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.4.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-money = { version = "0.4.0", default-features = false, features = ["rust-decimal"] }
```

Alternate decimal backend:

```toml
[dependencies]
paft-money = { version = "0.4.0", default-features = false, features = ["bigdecimal"] }
```

With DataFrame integration or panicking ops:

```toml
[dependencies]
paft-money = { version = "0.4.0", default-features = false, features = ["rust-decimal", "dataframe", "panicking-money-ops"] }
```

Features
--------

- `rust-decimal` (default): fast, fixed-size decimals (up to 28 fractional digits)
- `bigdecimal`: arbitrary precision decimals
- `dataframe`: Polars integration (`ToDataFrame`/`ToDataFrameVec`)
- `panicking-money-ops`: opt-in operator overloading that panics on invalid operations
- `money-formatting`: locale-aware formatting and strict parsing for `Money`

Quickstart
----------

```rust
use iso_currency::Currency as IsoCurrency;
use paft_money::{Currency, Money};

let price = Money::from_canonical_str("12.34", Currency::Iso(IsoCurrency::USD))?;
let tax   = Money::from_canonical_str("1.23",  Currency::Iso(IsoCurrency::USD))?;
let total = price.try_add(&tax)?;
assert_eq!(total.format(), "13.57 USD");
# Ok::<(), paft_money::MoneyError>(())
```

Locale-aware formatting
-----------------------

When you enable the optional `money-formatting` feature, localized output lives behind explicit APIs so the canonical `Display` remains stable (`"<amount> <CODE>"`).

```rust
# #[cfg(feature = "money-formatting")]
# {
use iso_currency::Currency as IsoCurrency;
use paft_money::{Currency, Locale, Money};

let eur = Money::from_canonical_str("1234.56", Currency::Iso(IsoCurrency::EUR)).unwrap();
assert_eq!(format!("{eur}"), "1234.56 EUR");
assert_eq!(eur.format_with_locale(Locale::EnEu).unwrap(), "€1.234,56");
assert_eq!(format!("{}", eur.localized(Locale::EnEu).with_code()), "€1.234,56 EUR");

let parsed =
    Money::from_str_locale("€1.234,56", Currency::Iso(IsoCurrency::EUR), Locale::EnEu).unwrap();
assert_eq!(parsed.format(), "1234.56 EUR");
# }
```

Links
-----

- API docs: https://docs.rs/paft-money
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: https://github.com/paft-rs/paft/blob/main/LICENSE
