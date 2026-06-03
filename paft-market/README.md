paft-market
===========

Market data models and request builders for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-market)](https://crates.io/crates/paft-market)
[![Docs.rs](https://docs.rs/paft-market/badge.svg)](https://docs.rs/paft-market)

- Unified market models: `Quote`, `Candle`, `HistoryResponse`, `OptionContract`, `OptionChain`, `OptionUpdate`, `NewsArticle`
- Snapshot-shaped market data (`Quote`, `OrderBook`) carries optional `as_of` timestamps for staleness checks
- Validated builders: `HistoryRequest`, `SearchRequest`; request parameter types: `NewsRequest`, `OptionExpirationsRequest`, `OptionChainRequest`
- Canonical, serde-stable string forms; optional DataFrame export
- Integrates with `paft-domain` and `paft-money`

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.8.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-market = { version = "0.8.0", default-features = false }
```

Alternate decimal backend:

```toml
[dependencies]
paft-market = { version = "0.8.0", default-features = false, features = ["bigdecimal"] }
```

With DataFrame integration:

```toml
[dependencies]
paft-market = { version = "0.8.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.8.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

Features
--------

- `bigdecimal`: switch the shared decimal backend used by `paft-money` prices and exposed `paft_decimal::Decimal` fields from `rust_decimal` to `bigdecimal`
- `dataframe`: Polars integration for market types; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`
- `tracing`: enable lightweight instrumentation for request builders and search constructors

Quickstart
----------

```rust
use paft_market::{HistoryRequest, Interval, NewsRequest, NewsTab, Range, SearchRequest};
use std::num::NonZeroU32;

// 1 month of daily candles
let req = HistoryRequest::try_from_range(Range::M1, Interval::D1).unwrap();
assert_eq!(req.interval(), Interval::D1);

// Validated instrument search
let search = SearchRequest::new("AAPL").unwrap();
assert_eq!(search.query(), "AAPL");

// News request parameters
let news = NewsRequest {
    count: NonZeroU32::new(25).unwrap(),
    tab: NewsTab::News,
};
assert_eq!(news.count, 25);
```

Links
-----

- API docs: https://docs.rs/paft-market
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
