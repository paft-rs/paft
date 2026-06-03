# paft-decimal

Lightweight decimal facade used across the `paft` workspace. The crate wraps
`rust_decimal` by default and can switch to `bigdecimal` via the optional
`bigdecimal` feature, providing common helpers for parsing, rounding, and
canonical string rendering without depending on higher-level money types.

Install
-------

```toml
[dependencies]
paft-decimal = "0.9.0"
```

Use `bigdecimal` when you need arbitrary precision instead of the default
`rust_decimal` backend:

```toml
[dependencies]
paft-decimal = { version = "0.9.0", features = ["bigdecimal"] }
```

Quickstart
----------

```rust
use paft_decimal::{
    Decimal, NonNegativeDecimal, Ratio, parse_decimal, to_canonical_string,
};

let value = parse_decimal("00123.4500").unwrap();
assert_eq!(to_canonical_string(&value), "123.45");

let size = NonNegativeDecimal::new(Decimal::from(10)).unwrap();
assert_eq!(size.to_string(), "10");

let pct = Ratio::new(parse_decimal("0.135").unwrap()).unwrap();
assert_eq!(pct.to_string(), "0.135");
assert!(Ratio::new(parse_decimal("1.2").unwrap()).is_err());
```

Serde adapters
--------------

Use the serde helpers when a decimal-backed field must have the same JSON wire
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
