paft-market
===========

Market data models, request builders, and response types for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-market)](https://crates.io/crates/paft-market)
[![Docs.rs](https://docs.rs/paft-market/badge.svg)](https://docs.rs/paft-market)
[![Downloads](https://img.shields.io/crates/d/paft-market)](https://crates.io/crates/paft-market)

- Quotes, quote updates, order books, candles, downloads, options, news, and search responses
- Validated builders for `HistoryRequest` and `SearchRequest`
- Simple request parameter types for news and option expirations/chains
- Snapshot timestamps on `Quote` and `OrderBook` via optional `as_of`
- Contextual `PriceAmount`/`QuantityAmount` values with `Currency` stored once per market record
- Explicit `OhlcPriceBasis` / `PriceBasis` metadata for returned history prices
- Canonical, serde-stable string forms and optional DataFrame export

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
paft-market = { version = "0.10.0", default-features = false }
```

With DataFrame integration:

```toml
[dependencies]
paft-market = { version = "0.10.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.10.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

Features
--------

- `dataframe`: Polars integration for market types; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`
- `tracing`: enable lightweight instrumentation for request builders and search constructors

Quickstart
----------

```rust
use paft_market::{HistoryRequest, Interval, NewsRequest, NewsTab, Range, SearchRequest};
use std::num::NonZeroU32;

let history = HistoryRequest::try_from_range(Range::M1, Interval::D1).unwrap();
assert_eq!(history.range(), Some(Range::M1));
assert_eq!(history.interval(), Interval::D1);
assert!(history.include_actions());

let search = SearchRequest::builder(" AAPL ")
    .limit(10)
    .region("US")
    .build()
    .unwrap();
assert_eq!(search.query(), "AAPL");
assert_eq!(search.limit().unwrap().get(), 10);
assert_eq!(search.region(), Some("US"));

let news = NewsRequest {
    count: NonZeroU32::new(25).unwrap(),
    tab: NewsTab::News,
};
assert_eq!(news.count.get(), 25);
```

History time and action dates
-----------------------------

`Candle::ts` is the inclusive start of the actual aggregation window. It is
neither the first trade nor the bar's end or publication time. Fixed-duration
bars cover `[ts, end)`. A streaming candle keeps this start while it forms.
Daily/session and longer calendar bars use the actual calendar boundary or
session open; a date label must be resolved using known calendar rules.

[Databento OHLCV](https://databento.com/docs/schemas-and-data-formats/ohlcv)
already supplies the inclusive start. A provider's end-labeled bar needs a
verified window to recover its start. Boundary conversion must preserve the
underlying window: a UTC-day aggregate and a regular-session aggregate remain
different even when both are `Interval::D1`. Preserve the calendar/session rule
and, where needed, explicit end boundaries in generic provider metadata.
Neither an interval code nor an IANA timezone defines the trading schedule;
calendar windows must not be reduced to an assumed 24 hours.

`HistoryMeta::timezone` supplies authoritative IANA rules for local-calendar
interpretation and display. `utc_offset_seconds` is seconds east of UTC at the
earliest returned candle's start, regardless of row order; omit it if there
are no candles or that offset is unknown. It must agree with the timezone at
that instant. It is not a replacement for date-dependent timezone rules and
must not be extrapolated across the series when the timezone is unavailable.
Both fields describe UTC timestamps without shifting them.

For actions, `Dividend::date` and `CapitalGain::date` are the applicable ex-date;
`Split::date` is the first trading date on the new share basis. These are dates
in the history listing's market calendar. Providers can supply several distinct
dates, as illustrated by [Alpaca's announcement date definitions](https://docs.alpaca.markets/us/reference/getcorporateannouncements).
Adapters must not substitute declaration, record, payable, processing, or legal
effective dates unless the source establishes the required economic meaning.
An unavailable applicable date means the action cannot map to this type.

History period precision
------------------------

`TimeSpec::period` and `HistoryRequest` builders require non-leap-second endpoints
and `start < end` at full instant precision. A one-nanosecond period within one
millisecond remains nonempty through canonical JSON. Serialization validates
publicly constructed `TimeSpec::Period` values too. Errors retain the original
instants; `InvalidPeriodTimestamp` includes the shared timestamp failure reason.

For v0.10, period bounds and all market UTC-instant fields change from integer
milliseconds to canonical UTC ISO-8601-style strings. New readers accept strings
only. Migrate known legacy integer milliseconds explicitly without floating
point or epoch-unit inference. Calendar action dates remain dates. See the
[shared timestamp contract](../paft-core/README.md#utc-instants) for the grammar,
expanded years, retained millisecond adapters, and independently checked
DataFrame nanosecond range.

Market payload notes
--------------------

- Direct users constructing payloads usually import companion types from
  `paft-domain`, `paft-money`, and sometimes `paft-decimal`; facade users can
  import the same surface from `paft::prelude`.
- `Quote`, `QuoteUpdate`, `Candle`, `HistoryResponse`, `OptionContract`,
  `OptionChain`, and related aliases are standard shapes with no provider
  metadata. Use their `Generic*` forms when preserving provider fields.
- Provider metadata is flattened into JSON payloads and must avoid field-name
  collisions with paft fields. DataFrame export namespaces provider metadata
  under `provider.*`.
- DataFrame instrument fields expand to `instrument.security_key`,
  `instrument.listing_key`, legacy `instrument.key`, `instrument.display`,
  and the structured `symbol`, `exchange`, `figi`, `isin`, and `kind` columns
  under the same prefix. Choose `.security_key` for issue grouping or `.listing_key` for venue joins;
  missing keys cannot establish identity. `.display`
  is a readable label that can be shared by distinct instruments. Option keys,
  contracts, and updates use `underlying.*` and `contract_instrument.*`; option
  chains use list columns under `contracts.underlying.*` and
  `contracts.contract_instrument.*`. An absent contract instrument produces
  nulls. In v0.10.0 these columns replace the former display-only `instrument`,
  `underlying`, and `contract_instrument` strings.
- History responses contain observed candles only. Missing intervals are
  omitted; zero prices and carried-forward prices must not be invented as gap
  placeholders. Calendar-grid completion belongs to consumers, with missing
  slots and synthetic values represented separately. v0.10.0 removes
  `HistoryFlags::KEEP_MISSING` and both `keep_missing` helpers; the former bit
  `0b1000` is now rejected on ingestion.
- `HistoryResponse::validate` checks non-decreasing candle timestamps;
  `into_chronological` sorts caller-owned responses when provider data arrives
  out of order.

Links
-----

- API docs: https://docs.rs/paft-market
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
