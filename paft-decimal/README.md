paft-decimal
============

Fixed-width decimal helpers for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-decimal)](https://crates.io/crates/paft-decimal)
[![Docs.rs](https://docs.rs/paft-decimal/badge.svg)](https://docs.rs/paft-decimal)
[![Downloads](https://img.shields.io/crates/d/paft-decimal)](https://crates.io/crates/paft-decimal)

- `Decimal` is an unconditional re-export of `rust_decimal::Decimal`
- Helpers for exact plain decimal parsing, canonical rendering,
  rounding, checked and exact arithmetic, and exact scaled-unit conversion
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
paft = "0.10.0"
```

Depend directly when you need helpers such as `parse_decimal`,
`try_to_scaled_units`, `from_minor_units`, or the serde adapters:

```toml
[dependencies]
paft-decimal = "0.10.0"
```

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

`parse_decimal` returns `Result<Decimal, DecimalParseError>`. `InvalidSyntax`
identifies malformed plain decimal text; `NotRepresentable` identifies a numeric
value that cannot fit PAFT exactly. Insignificant fractional trailing zeros
remain accepted, including beyond the scale limit. Nonzero digits are never
silently rounded away. Apply `round_dp_with_strategy`
explicitly when a representable value needs rounding.

Use PAFT's serde helpers for exact ingestion and canonical decimal strings.
Both adapters use the same exact parser and reject unrepresentable values;
native `Decimal` serde and upstream string adapters do not provide this contract:

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

Numeric contract
----------------

`Decimal` stores a 96-bit coefficient with a scale from 0 through 28. Magnitude
and fractional precision share that coefficient budget. For example,
`1000000000000.12` and `0.000000000000000001` fit exactly, while
`100000000000.000000000000000001` does not.

Exact ingestion preserves numeric value, not spelling: `1.2300` can serialize
as `"1.23"`. Native `Decimal::from_str`, `parse::<Decimal>()`, serde, and
arithmetic retain upstream semantics. PAFT cannot detect earlier precision loss
in an existing `Decimal`. Provider metadata uses the caller's serde policy.

`checked_add_exact`, `checked_sub_exact`, `checked_mul_exact`, and
`checked_div_exact` return `None` when the exact result cannot fit, including
underflow and nonterminating division such as `1 / 3`. Ordinary `checked_*`
helpers retain upstream precision rounding and are not exactness checks.
`round_dp_with_strategy` rounds explicitly. `Decimal128Mantissa` uses half-even
rounding when reducing scale; use `try_to_scaled_units` for exact integer units.

`try_to_scaled_units` uses checked `i128` coefficient scaling. The exact count
can exceed the decimal's 96-bit coefficient range as long as it fits `i128`.
Zero converts to zero at any target scale.

`try_from_scaled_units` and its panicking counterpart `from_minor_units` accept
numeric representability. They preserve the supplied coefficient and scale when
those fit; otherwise they remove trailing coefficient zeros and reduce the
scale together until the value fits. For example, `(10^29, 2)` represents
`10^27`, and `(10, 29)` represents `10^-28`; both are accepted exactly.
`(10^29 + 1, 2)` and `(1, 29)` fail because fitting them would lose nonzero
digits. Zero accepts any input scale, capped at 28 in storage.

v0.10.0 migration
-----------------

Remove `bigdecimal` from dependency feature lists. The feature and
arbitrary-precision storage have been removed; values outside PAFT's range must
be retained externally or rejected at the adapter boundary. BigDecimal
conversion helpers are deferred.

Update `parse_decimal` callers from `Option` to `Result`: use `?`, match
`DecimalParseError`, or call `.ok()` if the error reason is intentionally
unneeded. Existing `.unwrap()` and `.expect()` calls still compile.

Links
-----

- API docs: https://docs.rs/paft-decimal
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)

Exact arithmetic verification
-----------------------------

`cargo test -p paft-decimal --test rational_oracle` exercises the Rust addition,
subtraction, multiplication, and division helpers against 1,635 cases generated
independently with Python's arbitrary-precision `fractions.Fraction`. Every
operation includes both exact successes and representability failures, covering
signed boundaries, cancellation, coefficient reduction, underflow, and division.
No oracle arithmetic or arbitrary-precision dependency enters the library.

The committed fixture keeps Cargo tests self-contained. Regenerate it with
`python3 paft-decimal/tests/oracle/generate.py` from the workspace root, or use
`--check` to verify it without writing. Normal Cargo tests require no Python.
