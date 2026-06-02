paft-money
==========

Currency and money primitives for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-money)](https://crates.io/crates/paft-money)
[![Docs.rs](https://docs.rs/paft-money/badge.svg)](https://docs.rs/paft-money)

- `Currency` with ISO 4217 integration and extensible fallback
- Integrates with [`paft-decimal`](https://crates.io/crates/paft-decimal) for backend-agnostic decimal helpers
- `Money` for settled/payable amounts with currency minor-unit enforcement
- `Price` for full-precision per-unit quotes
- `MonetaryAmount` for exact currency totals before settlement rounding
- Runtime currency metadata overlays for ISO codes without minor-unit exponents (e.g., `XAU`, `XDR`) and custom/non-ISO currencies

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.8.0"
```

Advanced (direct dependency, default backend):

```toml
[dependencies]
paft-money = "0.8.0"
paft-decimal = "0.8.0" # only needed if you use paft_decimal helpers directly
```

Alternate decimal backend:

```toml
[dependencies]
paft-money = { version = "0.8.0", features = ["bigdecimal"] }
```

With DataFrame integration:

```toml
[dependencies]
paft-money = { version = "0.8.0", features = ["dataframe"] }
paft-utils = { version = "0.8.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

With panicking ops:

```toml
[dependencies]
paft-money = { version = "0.8.0", features = ["panicking-money-ops"] }
```

Features
--------

- `bigdecimal`: switch to arbitrary precision decimals
- `dataframe`: Polars integration for paft-money types; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`
- `panicking-money-ops`: opt-in operator overloading that panics on invalid operations
- `money-formatting`: locale-aware formatting and strict parsing for `Money`
- `tracing`: enable lightweight instrumentation on constructors, parsers, currency metadata helpers, and money operations

Currency value types
--------------------

Choose the level of structure you need:

- [`paft-decimal`](https://crates.io/crates/paft-decimal) exposes helpers such as `parse_decimal`, `from_minor_units`, `zero`, and `one`
- `Money` carries a currency and enforces settlement minor units
- `Price` carries a currency and preserves provider quote precision
- `MonetaryAmount` carries a currency and preserves exact totals/intermediates until final settlement rounding

```rust
use paft_decimal::{self as decimal, RoundingStrategy};
use paft_money::{Currency, IsoCurrency, MonetaryAmount, MoneyError, Price};

fn run() -> Result<(), MoneyError> {
    let usd = Currency::Iso(IsoCurrency::USD);
    let quote = Price::from_canonical_str("1.3578", usd.clone())?;
    let exact_total = quote.try_total(&decimal::from_minor_units(250, 2))?;
    let adjustment = MonetaryAmount::from_canonical_str("0.0049", usd)?;
    let subtotal = exact_total.try_add(&adjustment)?;
    let money = subtotal.to_money_with(
        RoundingStrategy::MidpointAwayFromZero,
        None,
    )?;
    assert_eq!(money.format(), "3.4 USD");
    Ok(())
}

run().unwrap();
```

Quickstart
----------

```rust
use paft_money::{Currency, IsoCurrency, Money};

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
use paft_money::{Currency, IsoCurrency, Locale, Money};

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
- License: [LICENSE](../LICENSE)
