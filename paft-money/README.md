paft-money
==========

Currency and money primitives for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-money)](https://crates.io/crates/paft-money)
[![Docs.rs](https://docs.rs/paft-money/badge.svg)](https://docs.rs/paft-money)

- `Currency` with ISO 4217 integration and extensible fallback
- Integrates with [`paft-decimal`](https://crates.io/crates/paft-decimal) for backend-agnostic decimal helpers
- `MoneyAmount` for high-precision values with optional currency hints
- `Money` with safe arithmetic and explicit conversions via `ExchangeRate`
- Runtime currency metadata overlays for non-ISO minor units (e.g., `XAU`, `XDR`)

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
```

Alternate decimal backend:

```toml
[dependencies]
paft-money = { version = "0.8.0", features = ["bigdecimal"] }
```

With DataFrame integration or panicking ops:

```toml
[dependencies]
paft-money = { version = "0.8.0", features = ["dataframe", "panicking-money-ops"] }
```

Features
--------

- `bigdecimal`: switch to arbitrary precision decimals
- `dataframe`: Polars integration (`ToDataFrame`/`ToDataFrameVec`)
- `panicking-money-ops`: opt-in operator overloading that panics on invalid operations
- `money-formatting`: locale-aware formatting and strict parsing for `Money`

Money layers
------------

Choose the level of structure you need:

- [`paft-decimal`](https://crates.io/crates/paft-decimal) exposes helpers such as `parse_decimal`, `from_minor_units`, `zero`, and `one`
- `MoneyAmount` keeps high-precision values with optional `Currency` hints and serde parity with `Decimal`
- `Money` attaches a currency, enforces metadata-driven rounding, and remains settlement-safe

```rust
use iso_currency::Currency as IsoCurrency;
use paft_decimal::{self as decimal, RoundingStrategy};
use paft_money::{Currency, Money, MoneyAmount, MoneyError};

fn run() -> Result<(), MoneyError> {
    let raw = decimal::from_minor_units(123_456, 4); // 12.3456
    let amount = MoneyAmount::new(raw);
    let shipping = MoneyAmount::from_str("1.25")?;
    let subtotal = amount.add(&shipping);
    let hinted = subtotal.with_currency_hint(Currency::Iso(IsoCurrency::USD));
    let money = hinted.to_money_with(
        Currency::Iso(IsoCurrency::USD),
        RoundingStrategy::MidpointAwayFromZero,
        None,
    )?;
    assert_eq!(money.format(), "13.60 USD");
    Ok(())
}

run().unwrap();
```

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
