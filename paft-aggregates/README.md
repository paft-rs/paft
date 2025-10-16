paft-aggregates
===============

Aggregated snapshot and report models built on the paft primitives.

[![Crates.io](https://img.shields.io/crates/v/paft-aggregates)](https://crates.io/crates/paft-aggregates)
[![Docs.rs](https://docs.rs/paft-aggregates/badge.svg)](https://docs.rs/paft-aggregates)

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = "0.5.1"
```

Advanced (direct dependency, minimal features):

```toml
[dependencies]
paft-aggregates = { version = "0.5.1", default-features = false }
```

What’s inside
--------------

- `info`: `FastInfo`, `Info` — lightweight instrument snapshots (identity, prices, ranges, and timestamp)
- `reports`: `InfoReport`, `SearchReport`, `DownloadReport` — merge-friendly report envelopes with `warnings`
  - `DownloadReport` now wraps a per-symbol `DownloadResponse` (`history: { SYM: HistoryResponse }`)

Features
--------

- `bigdecimal`: change money backend from `rust_decimal` to `bigdecimal` via `paft-money`
- `panicking-money-ops`: forwards to `paft-money` to enable panicking arithmetic operators

Links
-----

- API docs: [docs.rs/paft-aggregates](https://docs.rs/paft-aggregates)
- Workspace overview: [GitHub: workspace README](https://github.com/paft-rs/paft/blob/main/README.md)
- License: [GitHub: LICENSE](https://github.com/paft-rs/paft/blob/main/LICENSE)
