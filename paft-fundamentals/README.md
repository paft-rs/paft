paft-fundamentals
=================

Fundamentals data models for the paft ecosystem: financial statements, analysis, holders, ESG, and key statistics.

[![Crates.io](https://img.shields.io/crates/v/paft-fundamentals)](https://crates.io/crates/paft-fundamentals)
[![Docs.rs](https://docs.rs/paft-fundamentals/badge.svg)](https://docs.rs/paft-fundamentals)
[![Downloads](https://img.shields.io/crates/d/paft-fundamentals)](https://crates.io/crates/paft-fundamentals)

- Profiles: `CompanyProfile`, `FundProfile`
- Statements: `IncomeStatementRow`, `BalanceSheetRow`, `CashflowRow`, `Calendar`
- Analysis: earnings, recommendations, price targets, horizon-based trend/revision helper rows
- Statistics: `KeyStatistics`
- Holders: institutional, insiders
- ESG: scores, involvement, summary

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
paft-fundamentals = { version = "0.10.0", default-features = false }
```

With DataFrame integration:

```toml
[dependencies]
paft-fundamentals = { version = "0.10.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.10.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

Features
--------

- `dataframe`: Polars integration for dataframe-enabled row/leaf fundamentals types; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`
- `tracing`: enable lightweight instrumentation on parsing and helper constructors

Quickstart
----------

The quickstart below uses direct crate imports. Direct users should also add
the companion crates used by their constructors (`paft-decimal`, `paft-money`,
and sometimes `paft-domain`). Facade users can import through `paft::prelude`.

```rust
use paft_decimal::{Decimal, Ratio};
use paft_fundamentals::{
    CompanyProfile, Earnings, EarningsYear, EpsRevisions, EpsTrend, MajorHolder, Profile,
    RevisionPoint, TrendPoint,
};
use paft_money::{Currency, IsoCurrency, Price};

let earnings = Earnings {
    yearly: vec![EarningsYear::new(2023).unwrap()],
    ..Default::default()
};
assert_eq!(earnings.yearly[0].year.get(), 2023);

let usd = Currency::Iso(IsoCurrency::USD);
let eps_trend = EpsTrend::new(
    Some(Price::from_canonical_str("1.20", usd.clone()).unwrap()),
    vec![TrendPoint::try_new_str(
        "3mo",
        Price::from_canonical_str("1.05", usd).unwrap(),
    )
    .unwrap()],
);
assert!(eps_trend
    .find_by_horizon_str("3mo")
    .unwrap()
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

Statement measurement windows
-----------------------------

Every standalone row requires actual measurement context as well as its fiscal
`period` label. `IncomeStatementRow::window` and `CashflowRow::window` are
validated `StatementDuration` values with inclusive `start` and `end` dates
(`start <= end`). Values cover the start of the first day through the close of
the last day in the reporting entity's calendar. Standalone-quarter, cumulative
YTD, and trailing figures are all permitted, but all duration fields in a row,
including EPS and weighted-average shares, must share that exact window.
Adapters must split or normalize mixed windows before mapping.

For example, a Q2 cash-flow row covering `2024-04-01` through `2024-06-30` is
different from a Q2-labelled YTD row covering `2024-01-01` through `2024-06-30`.
The JSON context is `"window":{"start":"2024-04-01","end":"2024-06-30"}`.
`CashflowRow::end_cash_position` is the closing balance on the window's end date.

`BalanceSheetRow::as_of` is a `StatementInstant`, encoded as
`"as_of":{"date":"2024-06-30"}`. Every balance and share count is measured at
the close of that date, immediately before the next reporting day begins.
These are reporting-calendar boundaries; they imply neither UTC conversion nor
publication or revision time. The distinction follows the motivation behind
[instant and duration contexts in XBRL](https://www.xbrl.org/dates-in-xbrl/),
without adopting its wire representation.

Migration requires explicit context in Rust literals and JSON. Missing dates
are rejected; a fiscal label alone cannot safely reconstruct them. Context
objects reject unknown fields, while row payloads remain forward-compatible.
DataFrames retain `window.start`, `window.end`, or `as_of.date` as date columns.

Statement concepts
------------------

`BalanceSheetRow::current_debt` carries short-term borrowings plus the current
portion of long-term debt, excluding separately classified lease liabilities.
It is distinct from total current liabilities, which include non-debt
obligations.

`IncomeStatementRow::net_income_from_continuing_operations` carries after-tax
income or loss from continuing operations, including noncontrolling interests
and excluding discontinued operations. Continuing income attributable only to
the parent is a different measure. The existing `net_income` field retains its
meaning, and the two values are stored independently without reconciliation.

Both fields are `Option<Money>`: `None` means unavailable, and `Some` with a
zero amount means a reported zero. Negative continuing income is valid.
Provider adapters must map the accounting concept rather than infer it from a
provider's field name.

JSON payloads with explicit context that omit these optional fields deserialize
with `None`. Serialized
amounts retain their currency and captured minor-unit scale. DataFrame export
uses nullable `<field>.amount` decimal, `<field>.currency` string, and
`<field>.minor_units` unsigned-byte columns;
the continuing-income columns are appended to the income statement schema.
Code constructing a full `IncomeStatementRow` literal must supply the new
field, using `None` when unavailable.

Statement adapter sign guidance
-------------------------------

Income statement expense lines are signed amounts to subtract: charges are
positive, while net credits or reversals reducing an expense are negative.
Map the economic direction from the source's presentation; taking absolute
values would erase credits and reversals. The cash-flow convention of positive
inflows and negative outflows does not apply to income statement expenses.

`IncomeStatementRow::income_tax_expense` is a signed tax provision: **positive
for tax expense, negative for tax benefit (tax income)**. It includes current
and deferred tax recognized in profit or loss. IAS 12 likewise describes tax
expense or income as comprising current and deferred components.
([IAS 12, paragraphs 5–6](https://www.ifrs.org/content/dam/ifrs/publications/pdf-standards/english/2021/issued/part-a/ias-12-income-taxes.pdf#page=5))

For a simplified statement reporting pretax income of 100 USD, a tax benefit
of 20 USD, and net income of 120 USD, map the values as follows:

| PAFT field | Amount in USD |
| --- | ---: |
| `pretax_income` | 100 |
| `income_tax_expense` | -20 |
| `net_income` | 120 |

Subtracting the signed tax provision gives `100 - (-20) = 120` in this example.
Reported subtotals remain independent: PAFT does not recompute net income,
require this identity across provider statements, or reconcile differences in
scope or other adjustments. JSON and DataFrame export preserve the supplied
signed values. Adapters that previously took absolute values should remap the
original source data to recover the expense or benefit direction.

Links
-----

- API docs: https://docs.rs/paft-fundamentals
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
