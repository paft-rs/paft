paft-fundamentals
=================

Fundamentals data models for the paft ecosystem: financial statements, analysis, holders, and ESG.

[![Crates.io](https://img.shields.io/crates/v/paft-fundamentals)](https://crates.io/crates/paft-fundamentals)
[![Docs.rs](https://docs.rs/paft-fundamentals/badge.svg)](https://docs.rs/paft-fundamentals)

- Profiles: `CompanyProfile`, `FundProfile`
- Statements: `IncomeStatementRow`, `BalanceSheetRow`, `CashflowRow`
- Analysis: earnings, recommendations, price targets
- Holders: institutional, insiders
- ESG: scores, involvement, summary

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.7.2"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-fundamentals = { version = "0.7.2", default-features = false }
```

Alternate decimal backend:

```toml
[dependencies]
paft-fundamentals = { version = "0.7.2", default-features = false, features = ["bigdecimal"] }
```

With DataFrame integration:

```toml
[dependencies]
paft-fundamentals = { version = "0.7.2", default-features = false, features = ["dataframe"] }
```

Features
--------

- `bigdecimal`: change money backend from `rust_decimal` to `bigdecimal` via `paft-money`
- `dataframe`: Polars integration (`ToDataFrame`/`ToDataFrameVec`)

Quickstart
----------

```rust
use paft_fundamentals::{Earnings, EarningsYear, Profile, CompanyProfile};

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
- License: https://github.com/paft-rs/paft/blob/main/LICENSE
