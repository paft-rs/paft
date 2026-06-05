paft-prediction
===============

Prediction-market identity and data models for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-prediction)](https://crates.io/crates/paft-prediction)
[![Docs.rs](https://docs.rs/paft-prediction/badge.svg)](https://docs.rs/paft-prediction)
[![Downloads](https://img.shields.io/crates/d/paft-prediction)](https://crates.io/crates/paft-prediction)

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = { version = "0.9.0", features = ["prediction"] }
```

Advanced (direct dependency, minimal features):

```toml
[dependencies]
paft-prediction = { version = "0.9.0", default-features = false }
```

With DataFrame integration:

```toml
[dependencies]
paft-prediction = { version = "0.9.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

What's inside
-------------

- `EventID` and `OutcomeID`: validated, serde-stable identifier newtypes. Event
  IDs normalize to canonical lowercase `0x...` hex; outcome IDs normalize
  surrounding whitespace and validate as ASCII digit strings up to 78 digits.
- `PredictionInstrument`: pairs an event ID with an outcome ID and exposes the
  stable `event_id/outcome_id` display form via `unique_key()`.
- `Market` and `Token`: plain prediction-market payload structs for a question
  and its tradeable outcomes. Collateral, minimum order size, and tick size use
  `paft-money` types.
- `PredictionError`: non-exhaustive validation error for identifier
  constructors and serde deserialization.

Features
--------

- `bigdecimal`: switch the shared money/price decimal backend from `rust_decimal` to `bigdecimal`
- `dataframe`: Polars integration for prediction types; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`

Quickstart
----------

The quickstart below uses the direct `paft-prediction` dependency. Facade users
can import the same types from `paft::prediction` or, with the `prediction`
feature enabled, from `paft::prelude`.

```rust
use paft_prediction::{EventID, OutcomeID, PredictionError, PredictionInstrument};

fn run() -> Result<(), PredictionError> {
    const RAW_EVENT: &str =
        "  0xABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890  ";
    const EVENT: &str =
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";

    let event = EventID::new(RAW_EVENT)?;
    assert_eq!(event.as_ref(), EVENT);

    // FromStr is wired through the same validation and normalization path.
    let outcome: OutcomeID = " 12345 ".parse()?;

    let instrument = PredictionInstrument::from_ids(event, outcome);
    assert_eq!(instrument.unique_key().as_ref(), format!("{EVENT}/12345"));
    assert_eq!(instrument.to_string(), instrument.unique_key());

    Ok(())
}

run().unwrap();
```

Prediction notes
----------------

- Prediction-market identity is intentionally separate from
  `paft_domain::Instrument`; use `PredictionInstrument` for tradeable outcomes.
- Outcome IDs are only assumed unique within an event, so `unique_key()` always
  includes both identifiers.

Links
-----

- API docs: [docs.rs/paft-prediction](https://docs.rs/paft-prediction)
- Workspace overview: [GitHub: workspace README](https://github.com/paft-rs/paft/blob/main/README.md)
- License: [LICENSE](../LICENSE)
