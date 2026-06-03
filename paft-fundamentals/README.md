paft-fundamentals
=================

Fundamentals data models for the paft ecosystem: financial statements, analysis, holders, ESG, and key statistics.

[![Crates.io](https://img.shields.io/crates/v/paft-fundamentals)](https://crates.io/crates/paft-fundamentals)
[![Docs.rs](https://docs.rs/paft-fundamentals/badge.svg)](https://docs.rs/paft-fundamentals)

- Profiles: `CompanyProfile`, `FundProfile`
- Statements: `IncomeStatementRow`, `BalanceSheetRow`, `CashflowRow`
- Analysis: earnings, recommendations, price targets, horizon-based trend/revision helper rows
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

When constructing rows directly, fundamentals types usually compose with
`paft-domain`, `paft-money`, and `paft-decimal` primitives.

```rust
use paft_decimal::{Decimal, Ratio};
use paft_domain::Horizon;
use paft_fundamentals::{
    CompanyProfile, Earnings, EarningsYear, EpsRevisions, EpsTrend, MajorHolder, Profile,
    RevisionPoint, TrendPoint,
};
use paft_money::{Currency, IsoCurrency, Price};

let earnings = Earnings { yearly: vec![EarningsYear { year: 2023, ..Default::default() }], ..Default::default() };
assert_eq!(earnings.yearly[0].year, 2023);

let usd = Currency::Iso(IsoCurrency::USD);
let eps_trend = EpsTrend::new(
    Some(Price::from_canonical_str("1.20", usd.clone()).unwrap()),
    vec![TrendPoint::try_new_str(
        "3mo",
        Price::from_canonical_str("1.05", usd).unwrap(),
    ).unwrap()],
);
assert!(eps_trend
    .find_by_horizon(&Horizon::months(3).unwrap())
    .is_some());

let revisions = EpsRevisions::new(vec![RevisionPoint::try_new_str("30d", 4, 1).unwrap()]);
assert_eq!(
    revisions
        .find_by_horizon_str("30d")
        .unwrap()
        .unwrap()
        .net_revisions(),
    3
);

let holder = MajorHolder {
    category: "% held by insiders".into(),
    value: Ratio::new(Decimal::from(135) / Decimal::from(1000)).unwrap(),
};
assert_eq!(holder.value.to_string(), "0.135");

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

`Profile` serializes as a flat tagged shape with `kind`; fund profiles use
`fund_kind` for the fund type so it cannot collide with the discriminator.

Links
-----

- API docs: https://docs.rs/paft-fundamentals
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
