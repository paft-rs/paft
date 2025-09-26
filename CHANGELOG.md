# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2025-09-25

### Highlights

- Split money/currency and utilities into dedicated crates for clearer boundaries and optionality:
  - `paft-money`: `Money`, `Currency`, `ExchangeRate`, errors, and currency helpers
  - `paft-utils`: canonical string utilities and dataframe traits
- Facade updated to expose a `money` module and to re-export dataframe traits from `paft-utils`.
- Currency now backed by `iso_currency` for ISO 4217; combined with `rust_decimal` in `paft-money` for a robust money type that supports fiat, crypto, and provider-specific codes.
- Most users can keep using the `paft` facade; advanced users can depend on `paft-money` directly for a smaller, focused dependency graph.

### Breaking changes

- Moved from `paft-core` to `paft-money`:
  - `Money`, `Currency`, `ExchangeRate`, `MoneyError`, `MinorUnitError`
  - Helpers: `try_normalize_currency_code`, `currency_metadata`, `set_currency_metadata`, `clear_currency_metadata`
- Removed `paft-core::domain::string_canonical` (moved to `paft-utils`; re-exported via `paft_core::domain::{Canonical, canonicalize, StringCode}`).
- DataFrame traits moved to `paft-utils`:
  - Use `paft_utils::dataframe::{ToDataFrame, ToDataFrameVec}` (also re-exported via `paft_core::dataframe`).
- `paft` facade now exposes `paft::money::{Currency, Money, ExchangeRate, ...}`; prelude routes these through the new module.
- `Currency` ISO support now uses `iso_currency::Currency` for its ISO variant: match as `Currency::Iso(IsoCurrency::USD)` instead of any prior ad-hoc ISO representation.
- Currency parse errors now originate from `paft-money` (`paft_money::MoneyParseError`), not `paft-core`.

### Added

- New crates: `paft-money`, `paft-utils` (added to workspace members).
- `AssetKind` and `MarketState` now implement `Copy`.
- `SearchRequest::kind()` is `const` and returns by value.
- ISO 4217 integration via `iso_currency` across currency parsing, display names, and exponents.

### Changed

- Workspace: Version bumped to 0.3.0; `df-derive` updated to 0.1.1.
- DataFrame feature wiring: `paft-core`'s `dataframe` feature now depends on `paft-utils/dataframe`; all crates import traits from `paft_utils::dataframe`.
- `paft` facade: new `money` namespace; dataframe re-exports now come from `paft-utils`.
- README: examples and docs reference `paft_money::{Money, Currency}`.
- paft-money: ISO-None → metadata overlay. `Currency::decimal_places()` now:
  - Uses the ISO exponent when present.
  - If ISO is silent (e.g., `XAU`, `XDR`), looks up the metadata registry by ISO code.
  - If metadata exists, uses that scale; otherwise returns `MoneyError::MetadataNotFound`.
  - Removed `MoneyError::IsoExponentUnavailable`.
  - Register overlays via `set_currency_metadata("XAU", "Gold", N)`.
- Features: `panicking-money-ops` now forwards to `paft-money`.

### Migration notes

- Replace imports:
  - `paft_core::domain::{Money, Currency, ExchangeRate, MoneyError, MinorUnitError, try_normalize_currency_code, currency_metadata, set_currency_metadata, clear_currency_metadata}` → `paft_money::{...}` (or `paft::money::{...}` via facade)
  - `paft_core::dataframe::{ToDataFrame, ToDataFrameVec}` → `paft_utils::dataframe::{...}` (or `paft::core::dataframe::{...}` via facade)
  - `paft_core::domain::string_canonical::Canonical` → `paft_core::domain::Canonical` or `paft_utils::Canonical`
- If you use the facade prelude, most downstream code continues to compile; prefer `paft::prelude::{Currency, Money}`.
- Where you previously cloned `AssetKind`, you can now copy it.
- Pattern matching on ISO currencies should use `Currency::Iso(IsoCurrency::XXX)`.
- For metals/funds (ISO-None), register a domain-appropriate scale:
  - Example defaults: `XAU` 3 or 6 dp; `XDR` 6 dp; `XAG`/`XPT` often 3 dp.
  - Use `set_currency_metadata("CODE", "Name", decimals)`; absence will yield `MetadataNotFound`.
- If you handle parse errors for currencies, update matches to `paft_money::MoneyParseError` variants.

## [0.2.0] - 2025-09-19

### Highlights

- Unified canonical string handling across enums; serde/display now emit a single canonical token per variant.
- Money safety tightened: panicking operators opt-in via feature; new try_* APIs by default.
- History/search requests moved to a dedicated MarketError; history builder uses bitflags.
- Period now uses ISO dates and richer parsing; fundamentals adopt Period in models.
- Facade (`paft`) reorganized into namespaces with clearer prelude exports.

### Breaking changes

- paft-core / paft-fundamentals: Canonical token normalization for enum string forms is unified across the workspace. All Display/serde strings are produced by a single canonicalizer: uppercase ASCII; each contiguous run of non‑alphanumeric chars becomes a single `_`; leading/trailing `_` trimmed. Examples: `PRE-MARKET` → `PRE_MARKET`, `10% owner` → `10_OWNER`, `S&P 500` → `S_P_500`.
- paft-core / paft-fundamentals: Removed infallible `From<String>` for extensible enums; use `try_from_str` or `TryFrom<String>`. Empty/whitespace inputs are rejected.
- paft-core / paft-fundamentals: Serialization emits exactly the enum canonical `code()` for all variants (including `Other`). Deserialization routes through `try_from_str` (aliases first; unknowns normalize to `Other(UPPERCASE)`), removing any prior escape prefixes.
- paft-core: Money operator overloads (`Add`, `Sub`, `Mul`, `Div`) are disabled by default; enable with `panicking-money-ops`. Use `try_add`, `try_sub`, `try_div` by default.
- paft-core: Money::as_minor_units returns `Option<i128>` (was `Option<i64>`); Money::from_minor_units accepts `i128` and returns `Result<Self, MoneyError>`.
- paft-core: Replaced `Money::{add, sub, div}` with `Money::{try_add, try_sub, try_div}`.
- paft-core: Currency no longer contains several alt‑crypto variants (BNB, ADA, SOL, XRP, DOT, DOGE, AVAX, LINK, LTC, MATIC, UNI). Use `Currency::Other("…")` and configure precision via `currency_utils`.
- paft-core: `Currency::decimal_places()` and `minor_unit_scale()` are no longer `const fn` and can consult runtime overrides; `minor_unit_scale()` now returns `Result<i64, MoneyError>`.
- paft-core: Period::Date stores `NaiveDate` and always serializes as `YYYY-MM-DD` (ISO). Previously used timestamp seconds via `chrono::serde::ts_seconds`.
- paft-market: Request validation errors moved to `MarketError`. `HistoryRequestBuilder::{new, include_prepost, include_actions, auto_adjust, keepna}` and `HistoryRequest::builder` are no longer `const fn`.
- paft-market: History request toggles replaced by `HistoryFlags` bitflags.
- paft: Default features changed to `["market", "fundamentals"]`; facade exports reorganized under `core`, `market`, `fundamentals` modules. Prelude updated accordingly.

### Added

- paft-core: `panicking-money-ops` feature to opt‑in to `Money` operator overloading with panics on mismatches.
- paft (facade): Forwards `panicking-money-ops` to `paft-core` to enable via `paft` Cargo feature.
- paft-core: `domain::currency_utils` module:
  - Precision limits (`MAX_DECIMAL_PRECISION`, `MAX_MINOR_UNIT_DECIMALS`).
  - Minor‑unit overrides: `currency_minor_units`, `set_currency_minor_units`, `clear_currency_minor_units` with error `MinorUnitError`.
  - `try_normalize_currency_code` helper.
- paft-core: String canonicalization utilities in `domain::string_canonical` with `Canonical` wrapper and `canonicalize()` function.
- paft-core: `Money::{try_add, try_sub, try_div}` safe arithmetic helpers; `Money::try_convert` rounds using target currency precision.
- paft-core: Enum macro toolkit exported via `domain` (`string_enum*`, `impl_display_via_code`) for workspace enums.
- paft-core: `Exchange::full_name()`, `MarketState::full_name()`, `AssetKind::full_name()`, `Currency::full_name()` human labels.
- paft-core: `Exchange::is_european_exchange()` geography helper; `Instrument::unique_key()` docs clarify legacy `symbol@exchange` format.
- paft-core: Period parser accepts `FY2023`, `2023-Q4`, US dates `MM/DD/YYYY`, and day‑first `DD-MM-YYYY`.
- paft-fundamentals: Models now use `Period` where appropriate (`TrendPoint.period`, `RevisionPoint.period`, `NetSharePurchaseActivity.period`), plus helpers: `try_new_str`, `find_by_period`, `find_by_period_str`, `available_periods`.
- paft-market: `HistoryFlags` (bitflags) and exported `MarketError` type.
- Tooling: `just fmt` recipe; CI publish workflow improvements.

### Changed

- Workspace: Version bumped to 0.2.0; `polars` updated to 0.51.
- paft-core: `Money::try_convert` now rounds with `MidpointAwayFromZero` to the target currency scale.
- paft-core: `Currency::decimal_places()` consults runtime overrides for `Currency::Other` codes; built‑in overrides added for common cryptos/stablecoins in `currency_utils`.
- paft-market: History builder defaults now enable `INCLUDE_ACTIONS | AUTO_ADJUST`; getters read from bitflags.
- paft: Facade re‑exports consolidated under namespaces; prelude flattened for ergonomics and now includes `MarketError` and `DownloadResponse`.
- Docs: READMEs updated with badges, canonical code guidance, `panicking-money-ops` usage, and human‑label helpers.

### Migration notes

- Replace `From<String>` usages with `Type::try_from(s)` or `Type::try_from_str(&s)`. Audit for empty/whitespace inputs.
- If you previously matched on string forms, use `enum.code()` or `Display` for canonical tokens; use `full_name()` for UI text.
- Update `Money` arithmetic: replace `add/sub/div` with `try_add/try_sub/try_div`. If you require operators, enable `features = ["panicking-money-ops"]` and ensure invariants.
- Currency precision for removed crypto variants: use `Currency::Other("…")` plus `set_currency_minor_units("CODE", decimals)`.
- Period wire format is now `YYYY-MM-DD`. Adjust any consumers expecting epoch seconds.
- History/Search: switch error handling to `paft_market::MarketError` (and via facade `paft::market::MarketError` or `paft::prelude::MarketError`).

## [0.1.0] - 2025-09-16

- Initial public release.

[0.3.0]: https://github.com/paft-rs/paft/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/paft-rs/paft/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/paft-rs/paft/releases/tag/v0.1.0
