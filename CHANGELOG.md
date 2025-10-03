# Changelog

All notable changes to this project will be documented in this file.

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

[0.3.2]: https://github.com/paft-rs/paft/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/paft-rs/paft/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/paft-rs/paft/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/paft-rs/paft/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/paft-rs/paft/releases/tag/v0.1.0
