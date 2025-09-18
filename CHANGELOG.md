# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - Unreleased

### Breaking changes

- paft-core: `Money` operator overloads (`Add`, `Sub`, `Mul`, `Div`) are now disabled by default and gated behind the `panicking-money-ops` feature. Use `try_add`, `try_sub`, and `try_div` for safe arithmetic by default.
- paft-core: `Money::as_minor_units` now returns `Option<i128>` (was `Option<i64>`), and `Money::from_minor_units` now accepts `i128` (was `i64`).
- paft-core: Replaced `Money::{add,sub,div}` (Result-returning) methods with `Money::{try_add,try_sub,try_div}`.
- paft-core: `Currency` enum no longer includes several alt-crypto variants (e.g., BNB, ADA, SOL, XRP, DOT, DOGE, AVAX, LINK, LTC, MATIC, UNI). Use `Currency::Other("...")` and configure precision via `currency_utils`.
- paft-core: `Currency::decimal_places` and `Currency::minor_unit_scale` are no longer `const fn` due to runtime precision overrides.
- paft-market: Removed public `Switch` type and replaced internal toggles with `HistoryFlags` bitflags. `HistoryRequestBuilder::{new,include_prepost,include_actions,auto_adjust,keepna}` and `HistoryRequest::builder` are no longer `const fn`.

### Added

- paft-core: Feature flag `panicking-money-ops` to opt-in to panicking `Money` operators.
- paft (facade): Forwards `panicking-money-ops` to `paft-core` so users can enable it via `paft` features.
- paft-core: New `domain::currency_utils` module:
  - `normalize_currency_code`, `is_common_currency`, `describe_currency`
  - Minor-unit precision overrides: `currency_minor_units`, `set_currency_minor_units`, `clear_currency_minor_units`
- paft-core: `Money::{try_add,try_sub,try_div}` safe arithmetic helpers.
- paft-core: `Period` parsing supports day-first dates `DD-MM-YYYY` and case-insensitive quarter/year formats.
- paft-market: `HistoryFlags` bitflags for request behaviors (pre/post, actions, auto-adjust, keepna).
- Tooling: `just fmt` recipe.

### Changed

- paft-core: `Money::try_convert` now rounds to the target currency precision using `MidpointAwayFromZero`.
- paft-core: `Currency::decimal_places` consults runtime overrides for `Currency::Other` values; docs and parsing aliases improved.
- paft-market: History request builder defaults include `INCLUDE_ACTIONS | AUTO_ADJUST`; getters now read from bitflags.
- CI: Publish workflow uses `cargo --workspace publish`.
- Build: Workspace version bumped to `0.1.1`; `polars` updated to `0.51`.
- Docs: READMEs updated with badges and guidance for `panicking-money-ops`.

## [0.1.0] - 2025-09-16
- Initial public release.

[0.1.1]: https://github.com/paft-rs/paft/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/paft-rs/paft/releases/tag/v0.1.0
