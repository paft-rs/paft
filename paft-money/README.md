paft-money
==========

Currency and money primitives for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-money)](https://crates.io/crates/paft-money)
[![Docs.rs](https://docs.rs/paft-money/badge.svg)](https://docs.rs/paft-money)
[![Downloads](https://img.shields.io/crates/d/paft-money)](https://crates.io/crates/paft-money)

- `Currency` with ISO 4217 integration, built-in non-ISO codes, and typed fallback codes
- `Money` for settled/payable amounts with captured minor-unit scale
- `Price` and `MonetaryAmount` for full-precision quoted values and exact totals
- `PriceAmount` and `QuantityAmount` for contextual market payload amounts
- Runtime metadata overlays for ISO-None codes such as `XAU`/`XDR` and custom currencies
- Optional locale-aware formatting, DataFrame export, tracing, and `bigdecimal` backend support

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.9.0"
```

Advanced (direct dependency, default backend):

```toml
[dependencies]
paft-money = "0.9.0"
paft-decimal = "0.9.0" # only needed when using decimal helpers directly
```

Alternate decimal backend:

```toml
[dependencies]
paft-money = { version = "0.9.0", features = ["bigdecimal"] }
```

With DataFrame integration:

```toml
[dependencies]
paft-money = { version = "0.9.0", features = ["dataframe"] }
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

With panicking ops:

```toml
[dependencies]
paft-money = { version = "0.9.0", features = ["panicking-money-ops"] }
```

With locale-aware formatting:

```toml
[dependencies]
paft-money = { version = "0.9.0", features = ["money-formatting"] }
```

Features
--------

- `bigdecimal`: switch to arbitrary precision decimals
- `dataframe`: Polars integration for money types; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`
- `money-formatting`: locale-aware formatting and strict parsing for `Money`
- `panicking-money-ops`: opt-in `Add`/`Sub`/`Mul`/`Div` implementations that panic on invalid operations
- `tracing`: enable lightweight instrumentation on constructors, parsers, currency metadata helpers, and money operations

Quickstart
----------

Choose the level of structure you need:

- `Money` carries a currency and enforces settlement minor units
- `Price` carries a currency and preserves provider quote precision
- `MonetaryAmount` carries a currency and preserves exact totals/intermediates until final settlement rounding
- `PriceAmount` carries only the decimal amount; attach a currency with `with_currency` when it needs to stand alone
- `QuantityAmount` carries a non-negative decimal quantity whose unit comes from the surrounding market record

```rust
use paft_decimal::{self as decimal, RoundingStrategy};
use paft_money::{
    Currency, IsoCurrency, MonetaryAmount, MoneyError, Price, PriceAmount, QuantityAmount,
};

fn run() -> Result<(), MoneyError> {
    let usd = Currency::Iso(IsoCurrency::USD);
    let quote = Price::from_canonical_str("1.3578", usd.clone())?;
    let contextual_quote = PriceAmount::new(decimal::from_minor_units(13578, 4));
    assert_eq!(contextual_quote.with_currency(usd.clone()), quote);

    let quantity = QuantityAmount::from_decimal(decimal::from_minor_units(250, 2)).unwrap();
    let exact_total = quote.try_total(&quantity)?;
    assert_eq!(quantity.to_string(), "2.5");
    let adjustment = MonetaryAmount::from_canonical_str("0.0049", usd)?;
    let subtotal = exact_total.try_add(&adjustment)?;
    let money = subtotal.to_money_with(
        RoundingStrategy::MidpointAwayFromZero,
        None,
    )?;
    assert_eq!(money.format(), "3.4 USD");

    let tax = Money::from_canonical_str("0.10", Currency::Iso(IsoCurrency::USD))?;
    let total = money.try_add(&tax)?;
    assert_eq!(total.format(), "3.5 USD");
    Ok(())
}

run().unwrap();
```

Money Scale
-----------

`Money` captures the resolved minor-unit scale when it is constructed and
serializes that scale with the amount and currency:

```json
{"amount":"12.34","currency":"USD","minor_units":2}
```

The `minor_units` field is the scale captured when the value was constructed,
and it participates in equality, hashing, `as_minor_units()`, and arithmetic
compatibility. Deserialization validates the amount against the serialized
scale. If current metadata exists for the currency and disagrees with the
serialized scale, the payload is rejected; if metadata is absent for a custom
or ISO-None currency, the serialized scale is enough to restore the captured
settlement semantics.

Currency Metadata
-----------------

For ISO codes without a prescribed minor-unit exponent, or for custom
currencies, register metadata before constructing settlement `Money`:

```rust
use paft_money::{Currency, Locale, Money, set_currency_metadata};

set_currency_metadata("XAU", "Gold", 3, "XAU", true, Locale::EnUs).unwrap();

let gold = Money::from_canonical_str("1.234", Currency::try_from_str("XAU").unwrap()).unwrap();
assert_eq!(gold.as_minor_units().unwrap(), 1234);
```

`set_currency_metadata` refuses to change an already-known scale. Use
`override_currency_metadata` only when a scale change is intentional; existing
`Money` values keep their captured scale.

Locale-aware formatting
-----------------------

When you enable `money-formatting`, localized output lives behind explicit APIs
so canonical `Display` remains stable as `"<amount> <CODE>"`.

```rust
use paft_money::{Currency, IsoCurrency, Locale, Money};

let eur = Money::from_canonical_str("1234.56", Currency::Iso(IsoCurrency::EUR)).unwrap();
assert_eq!(format!("{eur}"), "1234.56 EUR");
assert_eq!(eur.format_with_locale(Locale::EnEu).unwrap(), "€1.234,56");
assert_eq!(format!("{}", eur.localized(Locale::EnEu).with_code()), "€1.234,56 EUR");

let parsed =
    Money::from_str_locale("€1.234,56", Currency::Iso(IsoCurrency::EUR), Locale::EnEu).unwrap();
assert_eq!(parsed.format(), "1234.56 EUR");
```

Links
-----

- API docs: https://docs.rs/paft-money
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
