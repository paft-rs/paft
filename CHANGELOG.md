# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.6.0] - 2025-10-21

### Breaking Change

- Market: `paft_market::responses::download::DownloadResponse` JSON shape changed from `{ "history": {SYM: HistoryResponse} }` to `{ "entries": [{ instrument, history }] }`, keyed by full `Instrument` identity (supports dual-listed symbols). Migrate symbol lookups by iterating `iter_by_symbol()` and collecting as needed.
- Aggregates: `paft_aggregates::DownloadReport.response` continues to wrap `DownloadResponse`; update consumers to traverse entries instead of indexing a map.

### Added

- `DownloadResponse::iter()` and `iter_by_symbol()` helpers for zero-copy traversal of entries.

### Documentation

- Documented the `tracing` feature flag in the workspace and `paft` READMEs, including scope and zero-cost when disabled.
- Added `borsa` to the Projects Using paft section
- Updated `paft-aggregates/README.md` to describe `DownloadResponse` as an entry list and recommend `iter_by_symbol()` for symbol-centric access.

## [0.5.2] - 2025-10-19

### Added

- Optional, feature-gated `tracing` instrumentation across crates:
  - Domain: identifier constructors, `Period::from_str`, `Exchange::try_from_str`, instrument helpers
  - Money: constructors, arithmetic ops, localized parser, `currency_utils::set_currency_metadata`
  - Market: `HistoryRequestBuilder::build`, `SearchRequestBuilder::build`, `SearchRequest::new`
  - Fundamentals: enum `try_from_str` methods, `TrendPoint::try_new_str`, `RevisionPoint::try_new_str`
- Workspace-level `tracing` feature wiring (no default subscriber)

### Documentation

- Root README now includes an "Observability (tracing)" section.

## [0.5.1] - 2025-10-17

### Added

- DataFrame integration now covers additional core models behind `feature = "dataframe"`:
  - Market search results (`paft_market::responses::search::SearchResult`).
  - Fundamentals ESG types (`paft_fundamentals::esg::EsgScores`, `EsgInvolvement`) and profiles (`paft_fundamentals::profile::Profile`).
  - Market corporate actions (`paft_market::market::action::Action`).
  - Domain instruments (`paft_domain::instrument::Instrument`) and aggregate snapshots (`paft_aggregates::info::{FastInfo, Info}`).
- Added targeted tests in the market, fundamentals, domain, aggregates, and money crates to ensure the new conversions (and previously derived ones) round-trip into Polars `DataFrame`s.

### Changed

- Trimmed unused dependencies and tightened workspace dependency management to reduce compile times and the feature surface:
  - Promoted `isin` to a workspace-managed dependency.
  - Removed unused deps like `chrono-tz`, `regex`, and dev-only `polars` where no longer needed.
  - Pruned redundant cross-crate links (e.g., `paft-aggregates` no longer depends on `paft-fundamentals`).

- Simplified the feature graph around DataFrame integration:
  - Removed the `paft-core` `dataframe` feature and its re-exports.
  - DataFrame support now remains via `paft-utils` and the domain/market/fundamentals crates behind `feature = "dataframe"`.
  - The `paft` facade's `dataframe` feature continues to work and forwards to those crates.

### Tooling

- `just`: reordered `test-full`/`lint-full` steps to run facade-critical checks first for faster feedback, then full powerset.

### Compatibility

- No public API changes for the `paft` facade; typical users are unaffected.
- Direct consumers of internal crates who imported `paft_core::dataframe::*` should import from `paft_utils::dataframe::*` (or via crates that enable `dataframe`). No migration is required when using the `paft` facade.

## [0.5.0] - 2025-10-16

This release tightens identifier validation across the entire workspace and introduces a canonical `Symbol` type so downstream crates receive normalized, provider-agnostic instrument identifiers. Market download payloads were also reshaped to make per-symbol adjustments explicit.

### Highlights

- Canonical `paft_domain::Symbol` replaces raw strings for instrument symbols and propagates through market, aggregate, and facade APIs.
- `Isin` and `Figi` now always validate checksums—no feature flags required—improving correctness at the serde boundary.
- Market downloads surface complete per-symbol `HistoryResponse` values, eliminating ambiguity around adjustment flags.

### Breaking Changes

- Market downloads: `paft_market::DownloadResponse` now wraps per-symbol `HistoryResponse`.
  - Old shape: `{ "series": {SYM: [Candle, ...]}, "meta": {SYM: HistoryMeta?}, "actions": {SYM: [Action, ...]}, "adjusted": bool }`.
  - New shape: `{ "history": {SYM: HistoryResponse} }`.
  - Rationale: `adjusted` can legitimately differ by symbol; a scoped `HistoryResponse` captures the exact outcome.

- Aggregates: `paft-aggregates::DownloadReport` now wraps `paft_market::DownloadResponse` as `response`.
  - Removed the old `history: Option<HistoryResponse>` field and legacy JSON shape `{ "history": { ... }, "warnings": [...] }`.
  - New JSON shape: `{ "response": { "history": {SYMBOL: HistoryResponse} }, "warnings": [...] }`.
  - Update consumers to access per-symbol data via `report.response.unwrap().history.get("SYM")` (or pattern match safely).

- Identifier validation is unconditional.
  - Removed the `isin-validate`, `figi-validate`, and `ident-validate` Cargo features from `paft` and `paft-domain`.
  - The `isin` crate is now a required dependency.

- `paft_domain::Symbol` newtype replaces raw strings for instrument symbols.
  - Canonicalization trims, uppercases ASCII letters, forbids whitespace/control chars, enforces 1–64 byte length, and preserves punctuation/numerics verbatim.
  - Implements `Display`, `FromStr`, `TryFrom<String>`, `AsRef<str>`, serde (transparent), and helpers `as_str()/len()/is_empty()`.
  - Equality/hash operate on the canonical string, guaranteeing `"aapl" == "AAPL"` post-normalization.

- `paft_domain::DomainError` gains `InvalidSymbol { value: String }` and is now marked `#[non_exhaustive]`.
  - Match arms over `DomainError` must include a wildcard (or otherwise account for future variants) to compile.

- `paft_domain::Instrument` now stores `Symbol`.
  - `symbol()` returns `&Symbol`; new `symbol_str()` helper exposes `&str`.
  - `Instrument::from_symbol`/`from_symbol_and_exchange` return `Result<Self, DomainError>` to propagate symbol validation.
  - `Instrument::try_new` validates the incoming symbol while continuing to accept optional FIGI/ISIN.
  - `unique_key()` still emits the canonical symbol string, so downstream formatting remains unchanged.

### Other Changes

- Workspace symbol usages migrated to `Symbol` (serde wire shapes remain strings because the type is `#[serde(transparent)]`).
  - Market requests/responses: `OptionExpirationsRequest.symbol`, `OptionChainRequest.symbol`, `Quote.symbol`, `QuoteUpdate.symbol`, `OptionContract.contract_symbol`, `SearchResult.symbol`, and `DownloadResponse.history: HashMap<Symbol, HistoryResponse>`.
  - Aggregates: `FastInfo.symbol`, `Info.symbol`, `InfoReport.symbol` now use `Symbol`.
  - Tests/examples/README updated to construct validated symbols via `Symbol::new(...)` and handle `Result` from the adjusted constructors.
  - All affected structs retain `#[cfg_attr(feature = "dataframe", df_derive(as_string))]` to keep DataFrame output identical.

- Facade (`paft`) re-exports `Symbol` through the `domain` module and `prelude`, ensuring downstream crates pick up the new type by default.

## [0.4.0] - 2025-10-11

### Added

- New feature flag `money-formatting` introducing locale-aware formatting and strict parsing for `Money`.
- New `Locale` enum plus a `LocalizedMoney` builder (via `Money::localized(locale)`) for opt-in localized `Display`.
- Currency metadata can store `symbol`, `symbol_first`, and `default_locale` (used only when `money-formatting` is enabled).
- New explicit APIs: `Money::format_with_locale`, `Money::from_str_locale`, and `Money::from_default_locale_str`.

- New crate `paft-aggregates` with summary and reporting types:
  - `FastInfo`, `Info` (instrument snapshot models)
  - `InfoReport`, `SearchReport`, `DownloadReport` (lightweight report containers)

- Market options module:
  - Types: `OptionGreeks`, `OptionContract`, `OptionChain`
  - Requests: `OptionExpirationsRequest`, `OptionChainRequest`
  - Response: `OptionExpirationsResponse`
  - DataFrame integration for options types when the `dataframe` feature is enabled

### Changed (Breaking)

- `set_currency_metadata` signature changed to require symbol, symbol_first, and default_locale (breaking change for direct callers).
- Market history: moved `unadjusted_close` from `HistoryResponse` to `Candle.close_unadj` for better data organization.
- Removed `Money::from_str` and replaced it with the new, more explicit `Money::from_canonical_str`
- When `money-formatting` is enabled:
  - `MoneyError` gains extra variants for format/parse failures (`InvalidAmountFormat`, `InvalidGrouping`, `MismatchedCurrencyAffix`, `ScaleTooLarge`, `UnsupportedLocale`).
  - `CurrencyMetadata` stores `symbol`, `symbol_first`, and `default_locale`; consequently `set_currency_metadata` accepts these extra fields.
  - These changes are feature‑gated and do not affect existing users unless the feature is explicitly enabled.

### Changed

- Added `#![forbid(unsafe_code)]` to all crates to prevent unsafe code usage across the workspace.
- DataFrame wiring: consolidated conditional Polars imports across crates for more consistent `dataframe` behavior.
- Polars optimization: reduced enabled Polars features to speed up compile times and cut binary size:
  - Workspace pins `polars` with `default-features = false`.
  - `paft-market` enables only `dtype-datetime` and `dtype-decimal` when `dataframe` is on.
  - `paft-money` enables only `dtype-decimal` when `dataframe` is on.
- Decimal backend selection in `paft-money` simplified:
  - `rust_decimal` is now the implicit default (the explicit `rust_decimal` feature was removed).
  - To use `bigdecimal`, enable the `bigdecimal` feature.
  - Enabling `bigdecimal` currently still pulls `rust_decimal` transitively; the previous compile error from enabling both backends is gone.

### Facade

- `paft` exposes a `money-formatting` feature that forwards to `paft-money/money-formatting` and re-exports the new APIs.
- `paft` re-exports market options models and requests/responses from `paft-market`:
  - `OptionGreeks`, `OptionContract`, `OptionChain`
  - `OptionExpirationsRequest`, `OptionChainRequest`, `OptionExpirationsResponse`
- New `aggregates` feature on the facade forwards to `paft-aggregates` and re-exports: `FastInfo`, `Info`, `InfoReport`, `SearchReport`, `DownloadReport` (disabled by default).

### Migration notes

- Update all calls to `set_currency_metadata(code, name, units)` to `set_currency_metadata(code, name, units, symbol, symbol_first, default_locale)`.
- Use `Money::from_canonical_str` over `Money::from_str` going forward.
- To adopt localization, enable `money-formatting` and:
  - Update calls to `set_currency_metadata(code, name, units)` to include `symbol`, `symbol_first`, and `default_locale`.
  - Use `Money::format_with_locale` or `Money::localized(locale)` for rendering, and `Money::from_str_locale` (or `from_default_locale_str`) for parsing.

- Cargo features: remove any explicit `paft-money/rust_decimal` feature flag; `rust_decimal` is now implicit. To use `bigdecimal`, enable `paft-money/bigdecimal`. Enabling `bigdecimal` is sufficient; there is no longer a separate `rust_decimal` feature to toggle.

### Fixed

- FIGI checksum validation corrected (parity and computation) in `paft-domain`; examples/tests updated accordingly.
- Updated `paft` unified error wiring to reference the `paft-money::MoneyError` consistently.
- Documentation for `MAX_MINOR_UNIT_DECIMALS` now reflects the actual i64 precision limit.

### Tooling

- CI now uses the workspace `justfile` with install actions for tools; reduced custom setup.
- `just`: simplified recipes by removing explicit decimal-backend toggles (now relies on implicit `rust_decimal`); CI/test matrix is smaller and easier to maintain.

## [0.3.2] - 2025-10-3

### Fixed

- docs.rs: `paft-money` lib.rs now builds reliably on docs.rs.

## [0.3.1] - 2025-10-3

### Highlights

- No functional changes; documentation-only release.

### Changed

- READMEs: refined wording and link formatting; examples now prefer the `paft` facade and reference version 0.3.1.

### Fixed

- docs.rs: `paft-money` documentation should now build reliably on docs.rs (added appropriate `cfg(docsrs)`/doc attribute handling and cleaned up doc comments).

### Tooling

- just: added a documentation generation command to mirror the docs.rs build locally.

## [0.3.0] - 2025-10-2

### Highlights

- Unified error handling across the facade: new `paft::Error` enum and `paft::Result<T>` alias
  aggregate errors from `paft-core`, `paft-domain` (feature = "domain"), `paft-market`
  (feature = "market"), `paft-money` (`MoneyError`, `MoneyParseError`), and `paft-utils`
  (`CanonicalError`).
- Split money/currency and shared utilities into dedicated crates for clearer boundaries and optionality:
  - `paft-money`: `Money`, `Currency`, `ExchangeRate`, errors, and currency helpers
  - `paft-utils`: canonical string utilities and dataframe traits
  - `paft-domain`: core domain types (`Exchange`, `Instrument`, `MarketState`, `Period`) and macro re-exports
- Facade (`paft`) adds a `money` module and re-exports dataframe traits; domain types are behind `feature = "domain"`.
- Currency now backed by `iso_currency` for ISO 4217; combined with `rust_decimal` (default) or `bigdecimal` (opt-in) in `paft-money` for a robust money type supporting fiat, crypto, and provider-specific codes.
- Most users can keep using the `paft` facade; advanced users can depend on `paft-money`/`paft-utils` directly for a smaller dependency graph.
- Feature-gated ISIN validation and normalization in `paft-domain` (forwarded by the facade via `feature = "isin-validate"`). `Instrument::try_new` replaces `new` to surface validation errors where enabled.

### Breaking changes

- Types moved from `paft-core`:
  - To `paft-money`: `Money`, `Currency`, `ExchangeRate`, `MoneyError`, `MinorUnitError`, and helpers `try_normalize_currency_code`, `currency_metadata`, `set_currency_metadata`, `clear_currency_metadata`.
  - To `paft-utils`: canonical string utilities (`Canonical`, `StringCode`, `canonicalize`).
  - To `paft-domain`: surface domain enums/structs and macro re-exports.
- `string_canonical` is no longer under `paft-core::domain`; import from `paft-utils` (or via `paft-domain`/facade).
- DataFrame traits now live in `paft-utils`; `paft-core` re-exports them under `paft_core::dataframe` when the `dataframe` feature is enabled.
- The `paft` facade now exposes `paft::money::{Currency, Money, ExchangeRate, ...}` and re-exports `IsoCurrency`.
- Currency parse errors now originate from `paft-money` (`paft_money::MoneyParseError`).
- `paft-domain::Instrument::new(...)` is replaced by `Instrument::try_new(...) -> Result<Instrument, DomainError>`.
- `Instrument::try_new` signature change: the `figi` parameter is now `Option<&str>` (was `Option<String>`). Internally identifiers are stored as typed newtypes (`Figi`/`Isin`), not `String`.

- Facade prelude no longer exports individual error types (`PaftError`, `DomainError`,
  `MarketError`, `MoneyError`, `CanonicalError`). Prefer the unified `paft::Error`, or import
  specific errors from their namespaces (e.g., `paft::market::MarketError`).

### Added

- New crates: `paft-money`, `paft-utils`, `paft-domain` (workspace members).
- `AssetKind` and `MarketState` now implement `Copy`.
- `SearchRequest::kind()` is `const` and returns by value.
- ISO 4217 integration via `iso_currency` across currency parsing, display names, and exponents.

- Optional ISIN validation in `paft-domain` behind `feature = "isin-validate"`; validation is provided by the new optional dependency `isin`.
- Facade (`paft`) forwards `isin-validate` to `paft-domain` so you can enable it at the top level.
- New re-exported identifier newtypes: `paft-domain::identifiers::{Isin, Figi}` with optional checksum validation and transparent serde support.
- Facade (`paft`) forwards a new `figi-validate` feature to `paft-domain` for consistent FIGI validation across the stack.
- New error variants: `DomainError::InvalidIsin` and `DomainError::InvalidFigi`.
- ISIN parsing now flows entirely through the `Isin` newtype; legacy `instrument` helper functions were removed in favor of the constructor-based API.
- New `Instrument` APIs: `try_set_isin(&str) -> Result<(), DomainError>`, `try_with_isin(&str) -> Result<Self, DomainError>`, `try_set_figi(&str) -> Result<(), DomainError>`, and `try_with_figi(&str) -> Result<Self, DomainError>`.

- Facade (`paft`): new `error` module with `Error` enum and `Result<T>` alias.
- Facade (`paft`): added direct dependency on `thiserror` to derive the unified error type.

### Changed

- Workspace version bumped to 0.3.0; `df-derive` updated to 0.1.1.
- DataFrame feature wiring: `paft-core`'s `dataframe` feature now depends on `paft-utils/dataframe`; crates import traits from `paft_utils::dataframe`.
- `paft` facade: new `money` namespace; dataframe re-exports now come from `paft-utils`.
- `paft-domain::Exchange` canonical tokens clarified: `Exchange::BSE` now maps to Bombay (`"BSE"`), `Exchange::PSE` to the Philippines (`"PSE"`), dedicated `Exchange::PSE_CZ` / `Exchange::BSE_HU` variants cover Prague/Budapest, and legacy `BSE_IND` / `PSE_PH` canonical strings plus the Prague `"PSE"` alias were removed.
- `paft-money` policy for ISO currencies without an exponent:
  - Use the ISO exponent when present.
  - If ISO is silent (e.g., `XAU`, `XDR`), consult the metadata registry by ISO code.
  - If metadata exists, use that scale; otherwise return `MoneyError::MetadataNotFound`.
  - Register overlays via `set_currency_metadata("XAU", "Gold", N)`.
- Feature forwarding: `panicking-money-ops` is provided by `paft-money` and forwarded by the facade.

- ISIN normalization: inputs are always scrubbed to uppercase ASCII alphanumerics and must not be empty. With `isin-validate` enabled, the cleaned value is additionally validated using the `isin` crate. Invalid inputs return `DomainError::InvalidIsin` from `try_new`/`try_set_isin`.
- ISIN-aware deserialization: `Instrument` now normalizes/validates the optional `isin` field during `Deserialize`, ensuring the `isin-validate` feature applies to incoming JSON as well.
- Docs and examples updated to use `Instrument::try_new(...).expect("valid instrument")` where appropriate.
- `Instrument` now stores typed identifiers (`Option<Figi>` / `Option<Isin>`), and profile structs (`CompanyProfile`, `FundProfile`) adopt `Option<Isin>`.

- Facade (`paft`): moved unified error definitions to `paft/src/error.rs`; `lib.rs` re-exports
  `Error` and `Result`. Prelude exports updated to remove individual error types, encouraging
  `paft::{Error, Result}`.

### Migration notes

- Replace imports:
  - `paft_core::{Money, Currency, ExchangeRate, MoneyError, MinorUnitError, try_normalize_currency_code, currency_metadata, set_currency_metadata, clear_currency_metadata}` → `paft_money::{...}` (or `paft::money::{...}` via facade)
  - DataFrame traits: `paft_core::dataframe::{ToDataFrame, ToDataFrameVec}` → `paft_utils::dataframe::{...}` (or `paft::core::dataframe::{...}` via facade)
  - Canonical strings: `paft_core::domain::string_canonical::*` → `paft_utils::*` (or `paft::domain::{Canonical, canonicalize, StringCode}`)
- If you use the facade prelude, most downstream code continues to compile; prefer `paft::prelude::{Currency, Money}`.
- Where you previously cloned `AssetKind`, you can now copy it.
- Pattern match ISO currencies as `Currency::Iso(IsoCurrency::XXX)`.
- For metals/funds (ISO-None), register a domain-appropriate scale; absence yields `MetadataNotFound`.
- If you handle parse errors for currencies, update matches to `paft_money::MoneyParseError` variants.

- Prefer `use paft::{Error, Result};` across your application. The `?` operator will automatically
  convert from `paft_core::PaftError`, `paft_domain::DomainError` (with `feature = "domain"`),
  `paft_market::MarketError` (with `feature = "market"`), `paft_money::{MoneyError, MoneyParseError}`,
  and `paft_utils::CanonicalError` into `paft::Error` via `From`.
- If you need to match on a specific error type, import it from its namespace (e.g.,
  `use paft::market::MarketError;`). The facade prelude no longer exports individual error types.
- If you previously imported `paft::prelude::MarketError`, update imports to
  `paft::market::MarketError` or pattern-match against `paft::Error`.

- Replace `Instrument::new(...)` with `Instrument::try_new(...)`. Handle the `Result` with `?`, `expect`, or a match. Example: `let inst = Instrument::try_new("AAPL", AssetKind::Equity, Some(figi), Some("US0378331005"), Some(Exchange::NASDAQ))?;`.
- Update call sites passing a FIGI: use borrowed strings, e.g., replace `Some("BBG000B9XRY4".to_string())` with `Some("BBG000B9XRY4")`.
- Enable `features = ["isin-validate"]` on `paft` or `paft-domain` to require checksum validation. Without it, values are still scrubbed to uppercase ASCII alphanumerics and must be non-empty, but no checksum is enforced.
- Identifiers are now strongly typed: `Instrument::figi()` / `Instrument::isin()` return `Option<&Figi>` / `Option<&Isin>`. Use `figi_str()` / `isin_str()` (or `map(AsRef::as_ref)`) when you need `&str` slices.
- Construct identifiers with `Figi::new(...)` / `Isin::new(...)` (or the new `Instrument::try_set_*` / `try_with_*` helpers). Profile structs (`CompanyProfile`, `FundProfile`) now expect `Option<Isin>`.
- If you match on `DomainError`, add cases for `InvalidIsin` and `InvalidFigi` when using the new typed identifiers.

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

[0.5.2]: https://github.com/paft-rs/paft/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/paft-rs/paft/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/paft-rs/paft/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/paft-rs/paft/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/paft-rs/paft/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/paft-rs/paft/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/paft-rs/paft/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/paft-rs/paft/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/paft-rs/paft/releases/tag/v0.1.0
