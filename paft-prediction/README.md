paft-prediction
===============

Prediction-market data models for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-prediction)](https://crates.io/crates/paft-prediction)
[![Docs.rs](https://docs.rs/paft-prediction/badge.svg)](https://docs.rs/paft-prediction)

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = { version = "0.8.0", features = ["prediction"] }
```

Advanced (direct dependency, minimal features):

```toml
[dependencies]
paft-prediction = { version = "0.8.0", default-features = false }
```

What's inside
-------------

- `EventID`, `OutcomeID` — validated newtypes for the two prediction-market
  identifier classes. Hex event IDs are case- and whitespace-normalized at
  construction; outcome IDs are trimmed and validated as ASCII decimal
  integers up to 78 digits long.
- `PredictionInstrument` — pairs an `EventID` with an `OutcomeID` to identify
  a single tradeable outcome. Parallels `paft_domain::Instrument`.
- `Market`, `Token` — higher-level aggregates describing the question being
  predicted and the tokens used to bet on its outcomes. `Money` and `Price`
  fields reuse `paft_money` so they compose cleanly with the rest of the
  workspace.
- `PredictionError` — error type returned by identifier constructors.

Quickstart
----------

```rust
use paft_prediction::{EventID, OutcomeID, PredictionInstrument};

// Identifiers normalize on construction: lowercase hex, trimmed.
let a = EventID::new("  0xABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCD  ").unwrap();
let b = EventID::new("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd").unwrap();
assert_eq!(a, b);

// FromStr is wired via ::new, so .parse() works too.
let outcome: OutcomeID = "12345".parse().unwrap();

let instrument = PredictionInstrument::from_ids(a, outcome);
println!("{instrument}"); // displays the outcome id
```

Features
--------

- `dataframe`: derive Polars `ToDataFrame` impls for the prediction types
- `bigdecimal`: forwards to `paft-money` to switch the money backend from
  `rust_decimal` to `bigdecimal`

Links
-----

- API docs: [docs.rs/paft-prediction](https://docs.rs/paft-prediction)
- Workspace overview: [GitHub: workspace README](https://github.com/paft-rs/paft/blob/main/README.md)
- License: [GitHub: LICENSE](https://github.com/paft-rs/paft/blob/main/LICENSE)
