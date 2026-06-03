paft-domain
===========

Domain modeling primitives for the paft ecosystem: instruments, exchanges, periods, horizons, and market state.

[![Crates.io](https://img.shields.io/crates/v/paft-domain)](https://crates.io/crates/paft-domain)
[![Docs.rs](https://docs.rs/paft-domain/badge.svg)](https://docs.rs/paft-domain)

- Strongly-typed identifiers for securities (`Symbol`, `Figi`, `Isin`) with enforced validation
- `Instrument` with hierarchical identifiers for securities (FIGI → ISIN → Symbol@Exchange → Symbol)
- Canonical, serde-stable enums (`Exchange`, `AssetKind`, `MarketState`)
- `ReportingPeriod` parsing for fiscal/provider labels with a canonical wire format
- `CalendarPeriod` helpers for calendar year/quarter/date boundaries
- `Horizon` parsing for relative lookback windows such as `7d`, `1mo`, and `1y`

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.9.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-domain = { version = "0.9.0", default-features = false }
```

Enable DataFrame helpers as needed:

```toml
[dependencies]
paft-domain = { version = "0.9.0", default-features = false, features = ["dataframe"] }
```

Features
--------

- `tracing`: enable lightweight instrumentation on constructors and validators
- `dataframe`: enable DataFrame traits for Polars integration

Quickstart
----------

The quickstart below uses the direct `paft-domain` dependency shown above. If
you depend on the facade crate instead, import these types from `paft::domain`
or `paft::prelude`.

```rust
use paft_domain::{
    AssetKind, CalendarPeriod, Exchange, Figi, Horizon, Instrument, Isin, ReportingPeriod, Symbol,
};

// Minimal: instrument from symbol + exchange
let aapl = Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity)
    .unwrap();

// Globally-identified: build the flat struct directly to attach FIGI/ISIN
// (preferred over symbol when available).
let aapl_pro = Instrument {
    symbol: Symbol::new("AAPL").unwrap(),
    exchange: Some(Exchange::NASDAQ),
    figi: Some(Figi::new("BBG000B9XRY4").unwrap()),
    isin: Some(Isin::new("US0378331005").unwrap()),
    kind: AssetKind::Equity,
};
assert_eq!(aapl_pro.unique_key(), "EQUITY|FIGI|BBG000B9XRY4");
assert_eq!(aapl_pro.display_key(), "BBG000B9XRY4");

// ReportingPeriod constructors validate fiscal/provider labels.
let reported_q4 = ReportingPeriod::quarterly(2023, 4).unwrap();
assert_eq!(reported_q4.to_string(), "2023Q4");
assert!(ReportingPeriod::quarterly(2023, 5).is_err());

// CalendarPeriod is the type for calendar date-boundary logic.
let calendar_q4 = CalendarPeriod::quarterly(2023, 4).unwrap();
assert_eq!(calendar_q4.start_date().to_string(), "2023-10-01");
assert_eq!(calendar_q4.end_date().to_string(), "2023-12-31");

// Parsing keeps provider-friendly inputs available too.
let parsed = "2023-Q4".parse::<ReportingPeriod>().unwrap();
assert_eq!(parsed, reported_q4);

// Horizon parsing is separate from reporting period parsing.
let horizon = "3mo".parse::<Horizon>().unwrap();
assert_eq!(horizon.to_string(), "3mo");
```

Prediction markets
------------------

Prediction-market identity is intentionally outside `paft-domain`; use the separate `paft-prediction` crate:

```toml
[dependencies]
paft-domain = "0.9.0"
paft-prediction = "0.9.0"
```

```rust
use paft_prediction::PredictionInstrument;

// Create an instrument for a prediction market outcome
let pm = PredictionInstrument::new(
    "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de",
    "73470541315377973562501025254719659796416871135081220986683321361000395461644",
).unwrap();

// Unique key for prediction markets is event_id/outcome_id
let expected = concat!(
    "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de",
    "/",
    "73470541315377973562501025254719659796416871135081220986683321361000395461644",
);
assert_eq!(
    pm.unique_key(),
    expected,
);
```

Links
-----

- API docs: https://docs.rs/paft-domain
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
