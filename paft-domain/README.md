paft-domain
===========

Domain modeling primitives for the paft ecosystem: instruments, exchanges, periods, horizons, and market state.

[![Crates.io](https://img.shields.io/crates/v/paft-domain)](https://crates.io/crates/paft-domain)
[![Docs.rs](https://docs.rs/paft-domain/badge.svg)](https://docs.rs/paft-domain)
[![Downloads](https://img.shields.io/crates/d/paft-domain)](https://crates.io/crates/paft-domain)

- Validated identifiers for securities (`Symbol`, `Figi`, `Isin`)
- `Instrument` identity precedence: FIGI, then ISIN, then symbol plus exchange, then symbol
- Canonical, serde-stable open enums (`Exchange`, `AssetKind`, `MarketState`)
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
assert_eq!(aapl.display_key(), "AAPL@NASDAQ");
assert_eq!(aapl.unique_key(), "EQUITY|SYMBOL|4:AAPL|EXCHANGE|NASDAQ");

// Attach global identifiers directly when provider data includes them.
let aapl_pro = Instrument {
    symbol: Symbol::new("AAPL").unwrap(),
    exchange: Some(Exchange::NASDAQ),
    figi: Some(Figi::new("BBG000B9XRY4").unwrap()),
    isin: Some(Isin::new("US0378331005").unwrap()),
    kind: AssetKind::Equity,
};
assert_eq!(aapl_pro.unique_key(), "EQUITY|FIGI|BBG000B9XRY4");
assert_eq!(aapl_pro.display_key(), "BBG000B9XRY4");

// ReportingPeriod models reporting/fiscal labels; constructors validate components.
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

Prediction-market identity is intentionally outside `paft-domain`. Use
`paft-prediction` for `PredictionInstrument`, `EventID`, and `OutcomeID`.

Links
-----

- API docs: https://docs.rs/paft-domain
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
