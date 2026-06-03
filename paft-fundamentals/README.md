paft-fundamentals
=================

Fundamentals data models for the paft ecosystem: financial statements, analysis, holders, ESG, and key statistics.

[![Crates.io](https://img.shields.io/crates/v/paft-fundamentals)](https://crates.io/crates/paft-fundamentals)
[![Docs.rs](https://docs.rs/paft-fundamentals/badge.svg)](https://docs.rs/paft-fundamentals)

- Profiles: `CompanyProfile`, `FundProfile`
- Statements: `IncomeStatementRow`, `BalanceSheetRow`, `CashflowRow`
- Analysis: earnings, recommendations, price targets, trend/revision helper rows
- Statistics: `KeyStatistics`
- Holders: institutional, insiders
- ESG: scores, involvement, summary

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
paft-fundamentals = { version = "0.9.0", default-features = false }
```

Alternate decimal backend:

```toml
[dependencies]
paft-fundamentals = { version = "0.9.0", default-features = false, features = ["bigdecimal"] }
```

With DataFrame integration:

```toml
[dependencies]
paft-fundamentals = { version = "0.9.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

Features
--------

- `bigdecimal`: switch `Money`/`Price` and decimal-backed fields to `bigdecimal` by forwarding `paft-money`, `paft-decimal`, and `paft-utils`
- `dataframe`: Polars integration for dataframe-enabled row/leaf fundamentals types; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`
- `tracing`: enable lightweight instrumentation on parsing and helper constructors

Quickstart
----------

```rust
use paft_fundamentals::{CompanyProfile, Earnings, EarningsYear, Profile};

let earnings = Earnings { yearly: vec![EarningsYear { year: 2023, ..Default::default() }], ..Default::default() };
assert_eq!(earnings.yearly[0].year, 2023);

let profile = Profile::Company(CompanyProfile {
    name: "Example Corp".into(),
    sector: None,
    industry: None,
    website: None,
    address: None,
    summary: None,
    isin: None,
});
if let Profile::Company(c) = profile { assert_eq!(c.name, "Example Corp"); }
```

Links
-----

- API docs: https://docs.rs/paft-fundamentals
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
