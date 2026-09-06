paft-domain
===========

Domain modeling primitives for the paft ecosystem: instruments, exchanges, periods, horizons, and market state.

[![Crates.io](https://img.shields.io/crates/v/paft-domain)](https://crates.io/crates/paft-domain)
[![Docs.rs](https://docs.rs/paft-domain/badge.svg)](https://docs.rs/paft-domain)
[![Downloads](https://img.shields.io/crates/d/paft-domain)](https://crates.io/crates/paft-domain)

- Validated identifiers for securities (`Symbol`, `Figi`, `Isin`)
- `Instrument` separates ISIN security identity from venue-specific listing identity
- Canonical, serde-stable open enums (`Exchange`, `AssetKind`, `MarketState`)
- `ReportingPeriod` parsing for fiscal/provider labels with a canonical wire format
- `CalendarPeriod` helpers for calendar year/quarter/date boundaries
- `Horizon` parsing for relative lookback windows such as `7d`, `1mo`, and `1y`

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.10.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-domain = { version = "0.10.0", default-features = false }
```

Enable DataFrame helpers as needed:

```toml
[dependencies]
paft-domain = { version = "0.10.0", default-features = false, features = ["dataframe"] }
```

Features
--------

- `tracing`: enable lightweight instrumentation on constructors and validators
- `dataframe`: enable DataFrame traits for Polars integration

Instrument DataFrame columns are `symbol`, `exchange`, `figi`, `isin`, `kind`,
`key`, `security_key`, `listing_key`, and `display`. Use `security_key()` for
cross-venue securities-issue grouping: it uses kind + ISIN and returns `None`
without an ISIN. Use `listing_key()` for venue observations: it requires an
exchange and uses kind + venue + venue FIGI, or symbol when FIGI is absent.
One issue can have multiple quotation lines on a venue, so ISIN does not replace
the symbol in a listing key. Missing keys export as null; they do not identify
one shared unknown entity. Exchange codes must describe consistent venue levels.

`figi` accepts only venue-level FIGIs (for example,
[AAPL on Nasdaq](https://www.openfigi.com/id/BBG000B9Y5X2)). Adapters must establish the level from
source metadata; checksum validation cannot distinguish venue, composite, or
share-class identifiers. See [OpenFIGI's documentation](https://www.openfigi.com/api/documentation).
PAFT does not resolve aliases or ticker reuse, and adding a FIGI changes a
symbol-based listing key. These helpers are not universal persistent identities.

The legacy `key` / `unique_key()` mixes identity levels and ignores exchange
when FIGI or ISIN is present. It remains for compatibility; use the explicit
keys for joins. `display` is a readable, non-unique label. Nested exports prefix
all columns, such as `instrument.listing_key`, and compute them from the current
public fields at export time. An absent instrument exports nulls throughout.

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
assert_eq!(aapl.listing_key().as_deref(), Some("LISTING|6:EQUITY|SYMBOL|4:AAPL|EXCHANGE|6:NASDAQ"));

// Attach global identifiers directly when provider data includes them.
let aapl_pro = Instrument {
    symbol: Symbol::new("AAPL").unwrap(),
    exchange: Some(Exchange::NASDAQ),
    figi: Some(Figi::new("BBG000B9Y5X2").unwrap()),
    isin: Some(Isin::new("US0378331005").unwrap()),
    kind: AssetKind::Equity,
};
assert_eq!(aapl_pro.security_key().as_deref(), Some("SECURITY|6:EQUITY|ISIN|US0378331005"));
assert_eq!(aapl_pro.display_key(), "BBG000B9Y5X2");

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
`paft-prediction` for `PredictionVenue`, role-specific prediction ids,
`PredictionEvent`, `BinaryMarket`, and `OutcomeInstrument`.

Links
-----

- API docs: https://docs.rs/paft-domain
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
