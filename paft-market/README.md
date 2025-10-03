paft-market
===========

Market data models and request builders for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-market)](https://crates.io/crates/paft-market)
[![Docs.rs](https://docs.rs/paft-market/badge.svg)](https://docs.rs/paft-market)

- Unified market models: `Quote`, `Candle`, `HistoryResponse`, `OptionChain`, `NewsArticle`
- Validated builders: `HistoryRequest`, `SearchRequest`
- Canonical, serde-stable string forms; optional DataFrame export
- Integrates with `paft-domain` and `paft-money`

Install
-------

```toml
[dependencies]
paft-market = "0.3.0"
```

Features
--------

- `rust-decimal` (default) | `bigdecimal`: money backend via `paft-money`
- `dataframe`: Polars integration (`ToDataFrame`/`ToDataFrameVec`)

Quickstart
----------

```rust
use paft_market::{HistoryRequest, Interval, Range, SearchRequest};

// 1 month of daily candles
let req = HistoryRequest::try_from_range(Range::M1, Interval::D1).unwrap();
assert_eq!(req.interval(), Interval::D1);

// Validated instrument search
let search = SearchRequest::new("AAPL").unwrap();
assert_eq!(search.query(), "AAPL");
```

Links
-----

- API docs: https://docs.rs/paft-market
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: https://github.com/paft-rs/paft/blob/main/LICENSE
