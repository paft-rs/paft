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
paft = "0.9.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-market = { version = "0.9.0", default-features = false }
```

Alternate decimal backend:

```toml
[dependencies]
paft-market = { version = "0.9.0", default-features = false, features = ["bigdecimal"] }
```

With DataFrame integration:

```toml
[dependencies]
paft-market = { version = "0.9.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

Features
--------

- `bigdecimal`: switch the shared decimal backend from `rust_decimal` to `bigdecimal`
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
- `HistoryResponse::validate` checks non-decreasing candle timestamps;
  `into_chronological` sorts caller-owned responses when provider data arrives
  out of order.

Links
-----

- API docs: https://docs.rs/paft-market
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
