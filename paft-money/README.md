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
- Optional locale-aware formatting, DataFrame export, and tracing

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.10.0"
```

Direct dependency:

```toml
[dependencies]
paft-money = "0.10.0"
paft-decimal = "0.10.0" # only needed when using decimal helpers directly
```

With DataFrame integration:

```toml
[dependencies]
paft-money = { version = "0.10.0", features = ["dataframe"] }
paft-utils = { version = "0.10.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

With panicking ops:

```toml
[dependencies]
paft-money = { version = "0.10.0", features = ["panicking-money-ops"] }
```

With locale-aware formatting:

```toml
[dependencies]
paft-money = { version = "0.10.0", features = ["money-formatting"] }
```

Features
--------

`IsoCurrency` is re-exported from `iso_currency` `0.7`. Prefer that re-export
when constructing `Currency::Iso`; applications that depend directly on
`iso_currency` must use the compatible `0.7` line to share the same Rust type.

`PriceAmount::into_inner` and `QuantityAmount::into_inner` are const methods.
Decimal types and their capabilities are the same under every PAFT feature set.

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

Canonical string constructors and decimal serde fields require an exact
representation in `rust_decimal`. Values that exceed its precision are
rejected before currency-scale or domain validation. Insignificant fractional
trailing zeros remain accepted. `Money::new` and `round_dp_with_strategy` provide
explicit rounding for values already represented as decimals.

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

Precision and arithmetic
------------------------

PAFT uses a 96-bit decimal coefficient with scale 0 through 28; magnitude and
fractional detail share that coefficient budget. Canonical string constructors
report `MoneyError::InvalidDecimal` for syntax and `MoneyError::NotRepresentable`
for values outside PAFT's representation. Settlement-scale failures remain
separate errors. Native `Decimal` parsing and serde retain upstream semantics;
constructors taking an existing decimal cannot detect earlier precision loss.

All `Price` and `MonetaryAmount` arithmetic is exact-or-error, including
price-times-quantity totals. `1 / 3`, nonzero underflow, and other unrepresentable
results return `MoneyError::NotRepresentable`. A quoted USD price of `1.234567`
retains all six fractional digits; exact `Money` ingestion rejects that scale.

`Money` arithmetic first uses upstream checked decimal operations, which can
round to fit decimal precision, then rounds settlement amounts with
`MidpointAwayFromZero`. FX conversion uses the selected strategy for its final
rounding; the intermediate product may already have been rounded. Ratios and
exchange-rate inverses use upstream division precision without settlement
rounding. `to_money` and `Money::new` round explicitly; DataFrame scale reduction
uses half-even rounding. These paths do not promise an exact calculation or a
single rounding of an unlimited-precision intermediate.

For v0.10.0, remove the `bigdecimal` feature from dependencies and handle the new
representability errors. Arbitrary-precision storage and conversion helpers are
outside this release. See the [decimal contract](../paft-decimal/README.md).

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
serialized scale, the payload is rejected. When neither ISO nor registered
metadata supplies a scale, the serialized scale is enough to restore the
captured settlement semantics.

`as_minor_units()` scales the decimal coefficient exactly in `i128`. For
example, a USD amount of `1000000000000000000000000000` converts to
`100000000000000000000000000000` cents, even though that integer count exceeds
the decimal coefficient range. Counts that exceed `i128` return
`MoneyError::ConversionError`.

`Money::from_minor_units` accepts numeric representability: it removes trailing
coefficient zeros when necessary to fit the value without rounding. The stored
decimal may then have fewer fractional places than the currency, while the
captured `minor_units` remains the currency's exponent and the original count
round-trips exactly. `Price::from_scaled_units` and
`MonetaryAmount::from_scaled_units` use the same numeric conversion rule.
Currency scales remain capped at 18 decimal places.

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

Built-in non-ISO exponents describe native denominations, not venue quantity
increments or display preferences. `USDC`, `USDT`, `BNB`, and `AVAX` require
explicit metadata because their denominations depend on network or asset
variant. Their codes still parse; `Money` constructors that need a scale return
`MoneyError::MetadataNotFound` until registration. Metadata is process-wide and
keyed only by code, so use distinct application-defined codes when different
denominations must coexist:

```rust
use paft_money::{Currency, Locale, Money, set_currency_metadata};

set_currency_metadata("USDC_ETHEREUM", "USDC on Ethereum", 6, "USDC", true, Locale::EnUs).unwrap();
set_currency_metadata("USDC_STELLAR", "USDC on Stellar", 7, "USDC", true, Locale::EnUs).unwrap();

let unit = Money::from_minor_units(1, Currency::other("USDC_STELLAR").unwrap()).unwrap();
assert_eq!(unit.format(), "0.0000001 USDC_STELLAR");
```

In v0.10.0, LINK, UNI, and MATIC defaults change from 8 to 18 decimal places.
Old serialized `Money` with `minor_units: 8` conflicts with these corrected
defaults and fails deserialization. Migrate correct major-unit amounts without
changing their numeric value; if an original native integer count was decoded
with the wrong exponent, reconstruct from that source count instead. Values
with different captured scales remain incompatible for arithmetic. See the
[denomination audit and migration details](CURRENCY_DENOMINATIONS.md), including
primary sources for every retained non-ISO default.

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
