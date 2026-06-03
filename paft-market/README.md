paft-market
===========

Market data models and request builders for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-market)](https://crates.io/crates/paft-market)
[![Docs.rs](https://docs.rs/paft-market/badge.svg)](https://docs.rs/paft-market)

- Unified market models: `Quote`, `Candle`, `HistoryResponse`, `OptionContract`, `OptionChain`, `OptionUpdate`, `NewsArticle`
- Snapshot-shaped market data (`Quote`, `OrderBook`) carries optional `as_of` timestamps for staleness checks
- Price-heavy market payloads carry `Currency` once per record and use
  contextual `PriceAmount` values; fractional-capable sizes and volumes use
  `QuantityAmount`
- History responses describe returned OHLC values with explicit
  `OhlcPriceBasis` / `PriceBasis` metadata
- Validated builders: `HistoryRequest`, `SearchRequest`; request parameter types: `NewsRequest`, `OptionExpirationsRequest`, `OptionChainRequest`
- Canonical, serde-stable string forms; optional DataFrame export
- Integrates with `paft-domain` and `paft-money`

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
assert_eq!(news.count.get(), 25);
```

Facade users can construct market payloads without importing companion crates
directly:

```rust
use paft::money::IsoCurrency;
use paft::prelude::*;

let instrument =
    Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity).unwrap();
let mut quote = Quote::new(instrument.clone(), Currency::Iso(IsoCurrency::USD));
quote.price = Some(PriceAmount::new(Decimal::from(19012) / Decimal::from(100)));
quote.day_volume = Some(QuantityAmount::from_decimal(Decimal::from(78_900_000)).unwrap());

let history = HistoryResponse {
    candles: vec![Candle::new(
        chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        Currency::Iso(IsoCurrency::USD),
        Ohlc::new(
            PriceAmount::new(Decimal::from(189)),
            PriceAmount::new(Decimal::from(191)),
            PriceAmount::new(Decimal::from(188)),
            PriceAmount::new(Decimal::from(190)),
        ),
    )],
    actions: vec![],
    price_basis: OhlcPriceBasis::uniform(PriceBasis::provider_latest_adjusted()),
    meta: None,
    provider: (),
};
assert_eq!(history.candles[0].ohlc.close.to_string(), "190");
```

Links
-----

- API docs: https://docs.rs/paft-market
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
