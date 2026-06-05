# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.9.0] - 2026-06-02

### Added

- Decimal/facade: added constrained decimal newtypes
  `NonNegativeDecimal`, `PositiveDecimal`, and `Ratio`, plus
  `DecimalConstraintError`.
- Decimal: added `serde::canonical_str` and `serde::option_canonical_str`
  helpers for backend-stable decimal string wire formats.
- Money/facade: added `PriceAmount`, a transparent contextual price-domain
  amount for values whose currency is supplied by an enclosing market record.
- Money/facade: added `QuantityAmount`, a transparent non-negative decimal
  quantity amount for provider-agnostic market sizes and volumes.
- Money: added `override_currency_metadata` for explicitly replacing a
  registered currency scale.
- Domain/facade: added `CalendarPeriod` for calendar year/quarter/date
  boundary helpers such as `start_date`, `end_date`, `overlaps`, `contains`,
  and `is_same_exact_bucket_as`.
- Domain/facade: added standalone serde support for `PeriodDate` and
  `QuarterOfYear`, matching `PeriodYear` as validated public period
  components.
- Domain/facade: added `Horizon` and `OtherHorizon` for relative lookback
  windows such as `7d`, `1mo`, and `1y`.
- Market/facade: added `Ohlc` plus OHLC price-basis modeling types
  `OhlcPriceBasis`, `PriceBasis`, `AdjustmentAnchor`, `AdjustmentMethod`,
  `CorporateActionAdjustmentCause`, and `CorporateActionAdjustmentCauses`.
- Market: added `OptionExpirationsResponse::new_sorted` and
  `GenericHistoryResponse::is_chronologically_ordered` helpers for callers that
  need explicit date/order normalization or validation.
- Market: added advisory-invariant helpers for response boundaries:
  `GenericHistoryResponse::{validate, into_chronological}`,
  `GenericOrderBook::{is_sorted, sort_levels}`, and
  `OptionExpirationsResponse::is_sorted_unique`.
- Fundamentals/facade: added `DataFrame` export support for `EsgSummary`,
  including nested `scores.*` and `involvement.*` columns.
- Market: added `TimeSpec::{range, period, validate}` so standalone time
  specifications have an explicit period-validation boundary.
- Market/facade: added `FromStr` parsers for closed request enums `Range`,
  `Interval`, `NewsTab`, and `OptionSide`.
- Facade: `paft::Error` now converts from `DecimalConstraintError`, allowing
  constrained decimal constructors to compose with `paft::prelude::Result`.
- Facade: `paft::prelude` now re-exports `IsoCurrency` for common
  `Currency::Iso(IsoCurrency::...)` construction.
- Docs/examples: added a no-metadata v0.9 ergonomics example and refreshed
  crate READMEs for contextual amounts, price basis, horizons, and constrained
  decimals.
- Docs: clarified the wire-compatibility policy: tagged provider/data payloads
  remain forward-compatible, serde-flattened provider metadata collisions are
  unsupported JSON key names rather than universally detected errors, and
  dataframe exports namespace provider metadata under `provider.*` columns.
- Prediction/facade: added provider-neutral prediction-market identity,
  metadata, fixed-point price/quantity, quote, order-book, and trade-history
  types, including `PredictionVenue`, role-specific opaque ids,
  `PredictionEvent`, `BinaryMarket`, `OutcomeInstrument`, `OutcomePrice`,
  `ContractQuantity`, `NonZeroContractQuantity`, `PriceGrid`, and canonical
  YES-view `BinaryOrderBook`.
- Prediction/facade: added `BinaryOutcomeInstruments` and
  `BinaryMarketKey::{synthetic_yes_instrument, synthetic_no_instrument}` so
  binary markets expose tradable YES/NO outcome instruments directly while
  still supporting synthetic `YES`/`NO` ids for venues that do not issue
  separate instrument ids.
- Prediction/facade: added `PredictionQuoteLevel` for top-of-book quote levels
  whose displayed quantity may be unavailable.
- Prediction: added exact decimal/string conversion helpers for
  `OutcomePrice` and `ContractQuantity`, plus canonical decimal `Display`
  implementations for `OutcomePrice`, `PriceTick`, `ContractQuantity`, and
  `OutcomePayout`.
- Prediction/facade: added `NonZeroContractQuantity` for quantity surfaces
  where zero is not semantically valid.

### Changed

- Workspace: version bumped to `0.9.0`.
- Prediction/facade: replaced the Polymarket-shaped `EventId`/`OutcomeId`,
  `PredictionInstrument`, `Market`, and `Token` surface with venue-namespaced
  `PredictionEventId`, `PredictionMarketId`, `PredictionOutcomeId`,
  `OutcomeInstrument`, `PredictionEvent`, and market-shape metadata types.
- Prediction: `GenericBinaryMarket` now requires `BinaryOutcomeInstruments`,
  keeping Polymarket-style CLOB token/asset ids in the provider-agnostic
  binary market payload instead of provider metadata.
- Prediction: `LinkedBinaryRelation`, `PredictionMarketStatus`, and
  `BinaryResolution` now use paft-style open string parsing/serde, so unknown
  strings deserialize into `Other(...)` and serialize back as strings.
- Prediction: prediction event, market, and outcome `unique_key()` values now
  length-prefix every dynamic component, including venue, and `BinaryMarketKey`
  now emits the same market identity string as `PredictionMarketKey`.
- Prediction: `PriceGrid` and `NumericRange` deserialization now validates
  constructor invariants instead of accepting invalid wire payloads.
- Prediction: `BinaryQuote` now uses optional-quantity `PredictionQuoteLevel`
  values for best bid/ask, while order-book depth continues to use
  quantity-required `PredictionBookLevel` values.
- Prediction: `BinaryOutcomeInstruments` now validates that YES and NO
  instruments belong to the same venue/market and have distinct outcome ids,
  including during deserialization.
- Prediction: book levels, present quote quantities, trades, and market
  minimum order quantities now use `NonZeroContractQuantity` instead of
  zero-capable `ContractQuantity`.
- Prediction: `PredictionBookLevel` fields are ordered to keep the common depth
  level representation compact on current targets.
- Prediction: `PriceGrid` validation now rejects bands whose inclusive end
  endpoint is not reachable from the start by whole tick increments.
- Docs: consolidated the workspace and crate READMEs around crate-local usage,
  standardized crate badges for Crates.io, docs.rs, and downloads, and removed
  duplicated install/API guidance from the root README.
- Market/fundamentals: universally constrained non-amount fields now use
  dedicated newtypes: option implied volatility uses `NonNegativeDecimal`,
  holder fractions use `Ratio`, and news request counts use `NonZeroU32`.
- Market: documented `Action::Split` ratio direction as new shares per old
  shares, so a 4-for-1 split is `numerator = 4`, `denominator = 1`.
- Market/fundamentals: date-only financial concepts now use `NaiveDate` and
  serialize as `YYYY-MM-DD` instead of Unix milliseconds. This includes
  corporate action dates (`Action` now uses `date` instead of `ts`), dividend
  calendar dates, holder/reporting dates, insider transaction dates, and
  `ShareCount::date`.
- Fundamentals: `InsiderTransaction::url` is now `Option<String>` so missing
  filing URLs can be represented without sentinel strings.
- Money: scalar arithmetic helpers now borrow decimal operands:
  `Money::{try_mul, try_div}`, `MonetaryAmount::{try_mul, try_div}`,
  `Price::{try_mul, try_div}`, and `Price::try_total_decimal`.
- Money: `Price::try_total` now accepts `&QuantityAmount`, keeping normal
  price-times-quantity totals on the non-negative quantity path. Use
  `Price::try_total_decimal` for signed/raw decimal quantity semantics.
- Money: `set_currency_metadata` now preserves an already registered
  minor-unit scale; callers must use `override_currency_metadata` for
  intentional scale changes.
- Money: `override_currency_metadata` now rejects `minor_units` values that
  conflict with an ISO-defined exponent, keeping public metadata aligned with
  `Currency::decimal_places()`.
- Money: `Currency::full_name()` now uses registered metadata for all non-ISO
  currencies, including modeled variants such as `BTC`, `ETH`, and `XMR`; ISO
  currency names remain sourced from ISO 4217.
- Money: locale grouping specifications now use non-zero chunk sizes
  internally, making invalid zero-width grouping patterns unrepresentable in
  formatter/parser state.
- Decimal/money/market/fundamentals: decimal-backed serde fields now serialize
  through canonical strings from `paft-decimal`, independent of the active
  decimal backend.
- Money: JSON now includes the captured `minor_units` scale, and deserialization
  requires it so `Money` wire values are self-contained with respect to
  equality, hashing, minor-unit conversion, and same-currency arithmetic
  compatibility.
- Decimal/money/utils: backend-specific decimal cloning, checked arithmetic,
  precision limits, and decimal128 mantissa encoding now live behind
  `paft-decimal` APIs so downstream crates do not infer the active decimal
  backend from their own local feature flags.
- Money/facade: `PriceAmount` and `QuantityAmount` no longer implement `Copy`
  under the default local feature set. They remain `Clone`, matching the
  backend-agnostic ownership contract of `paft_decimal::Decimal`.
- Market history: `HistoryResponse` now exposes `price_basis:
  OhlcPriceBasis`, describing the returned OHLC price basis as either uniform
  `PriceBasis` metadata or per-field open/high/low/close bases.
- Market/fundamentals: `Action` and `Profile` now use flat tagged serde shapes
  with a `kind` discriminator instead of externally tagged enum objects.
- Market: standalone `TimeSpec` deserialization now rejects `Period` values
  whose `start >= end`, matching `HistoryRequest` validation.
- Market/aggregates: high-cardinality price records now carry denomination once
  at the containing record and store contextual `PriceAmount` values for
  candles, order-book levels, quotes, quote updates, snapshots, and option
  quote fields. Option contracts/updates carry an explicit premium `currency`
  so quote fields do not inherit the strike currency implicitly.
- Market/facade: `OptionContractKey` now stores optional
  `contract_instrument` identity as part of equality and hashing so known
  listed contracts with the same economic terms can remain distinct.
- Market/aggregates: book-level sizes and provider-agnostic volume fields now
  use contextual `QuantityAmount` values so fractional crypto, FX,
  commodities, and base/quote-volume feeds can be represented without rounding
  or metadata side channels.
- Market/aggregates: generic payload containers now derive `Eq` conditionally,
  so standard no-metadata aliases such as `Quote`, `HistoryResponse`, and
  `Snapshot` implement `Eq` while metadata payloads that only implement
  `PartialEq` remain usable.
- Market: nested generic metadata containers now use separate type parameters
  for container-level and child-level provider payloads, so response metadata,
  row metadata, and leaf metadata no longer have to share one Rust type.
- Domain/facade: `Instrument::unique_key()` now emits a kind-aware,
  source-namespaced identity key; new `Instrument::display_key()` preserves the
  compact FIGI/ISIN/SYMBOL@EXCHANGE/SYMBOL display chain.
- Domain/facade: `Instrument::unique_key()` now returns `String` instead of
  `Cow<'_, str>` because the namespaced identity key is always a synthetic
  composite.
- Domain: `Isin` and `Figi` now use inline `SmolStr` storage instead of heap
  `String` storage for their fixed 12-byte identifier codes.
- Domain/facade: `Period` was split into `ReportingPeriod` for fiscal/provider
  labels and `CalendarPeriod` for date-boundary logic. Structured period
  variants now store validated `PeriodYear`, `QuarterOfYear`, and
  `PeriodDate` components.
- Domain/facade: split the ambiguous `CalendarPeriod::is_same_bucket_as`
  relationship helper into `overlaps`, `contains`, and
  `is_same_exact_bucket_as`.
- Fundamentals/facade: `EarningsYear::year` now uses the validated
  `PeriodYear` newtype instead of raw `i32`.
- Domain/fundamentals/facade: `PeriodYear` serde now emits canonical
  four-digit strings, including nested uses such as `EarningsYear::year`, while
  deserialization still accepts integer years and normalizes them on output.
- Fundamentals/facade: EPS trend and revision historical points now use
  `Horizon` for lookback windows instead of overloading `ReportingPeriod`.
- Domain/money/fundamentals/facade: extensible enum `Other` variants now use
  enum-specific unknown-code wrappers (`OtherCurrency`, `OtherExchange`,
  `OtherAssetKind`, `OtherMarketState`, `OtherPeriod`,
  `OtherRecommendationGrade`, `OtherRecommendationAction`,
  `OtherTransactionType`, `OtherInsiderPosition`, and `OtherFundKind`) instead
  of raw `Canonical` payloads.
- Domain/facade: `MarketState` is now extensible via
  `MarketState::Other(OtherMarketState)`, so provider-specific session states
  round-trip instead of failing as invalid enum values.
- Domain/fundamentals/facade: enum parsing APIs now expose crate-level domain
  errors instead of `paft_core::PaftError`; domain enums use `DomainError`, and
  fundamentals enums use the new `FundamentalsError` surfaced by `paft::Error`.
- Docs: standalone guide material was folded into the workspace and crate
  READMEs so crate-local API guidance stays next to the code it documents.

### Fixed

- Facade docs: gated the feature-dependent quickstart doctest so
  `cargo test -p paft --no-default-features --doc` does not compile imports that
  require `domain` and `market`.
- Facade examples: aligned the README example list with the Cargo example
  targets, including `extensible_enums` and `nested_metadata_propagation`.
- Docs: corrected the facade README identifier serde wording to describe the
  manual plain-string implementations instead of `#[serde(transparent)]`.
- Docs: corrected snapshot, history-period, and search-response comments to
  avoid overstating guarantees.
- Core/domain/money/fundamentals: public enum-specific `Other*` wrappers now
  implement serde directly and validate deserialized strings through their
  checked constructors, so modeled codes remain rejected.
- Feature matrix: `paft-aggregates/dataframe` now explicitly enables
  Polars datetime support for `Snapshot::as_of`, and the `v09_ergonomics`
  example declares the facade features it imports.
- Decimal: `checked_div` now rejects zero divisors under the `bigdecimal`
  backend, `parse_decimal` validates a backend-stable plain decimal grammar,
  and canonical decimal rendering normalizes signed zero to `0`.
- Decimal: `parse_decimal` now rejects duplicate explicit sign prefixes such as
  `+-1` and `++1` instead of accepting them after leading-plus normalization.
- Dataframe: `Decimal128Encode` now rejects target scales above Polars decimal
  precision and uses checked exponentiation when rescaling mantissas.
- Market requests: `HistoryRequest` and `SearchRequest` JSON deserialization
  now rejects unknown top-level fields instead of silently ignoring request
  typos.
- Market requests/responses: `NewsRequest`, `OptionExpirationsRequest`,
  `OptionChainRequest`, and `HistoryMeta` JSON deserialization now rejects
  unknown fields, keeping semantic wire shapes aligned with the strict request
  policy.
- Market requests: `SearchRequest` now trims accepted `lang`/`region` values
  and rejects empty or whitespace-only values through both builder and serde
  construction paths.
- Market/facade: `SearchRequest` now stores result limits as
  `Option<std::num::NonZeroU32>` and validates builder/deserialized limits from
  `u32`, avoiding platform-dependent `usize` in serialized request models.
- Market/facade: `HistoryValidationError` now converts into `paft::Error`, so
  `HistoryResponse::validate()?` composes with `paft::Result`.
- Market requests: `HistoryFlags` now serializes as an explicit `u8` bitset,
  and deserialization rejects unknown flag bits instead of retaining unmodeled
  request behavior.
- Money: localized formatting now rejects fraction digit requests above the
  active decimal backend precision instead of attempting unbounded zero padding.
- Money: localized parsing now delegates to `Money::new_exact`, encoding its
  no-implicit-rounding contract at construction.
- Money: `PriceAmount::into_inner` and `QuantityAmount::into_inner` are now
  `const fn` under the default decimal backend, matching constrained decimal
  accessor behavior.
- Money: `Money` and `ExchangeRate` JSON deserialization now rejects unknown
  top-level fields instead of silently ignoring stale wire payloads.
- Money: existing `Money` values now capture their resolved minor-unit scale,
  so later custom metadata changes or clears cannot reinterpret
  `as_minor_units()` or same-currency arithmetic.
- Money: deserialization now validates the serialized `minor_units` scale and
  rejects payloads when currently registered currency metadata conflicts,
  instead of recomputing the scale from process-local metadata and silently
  changing the value identity.
- Money: `as_minor_units()` now rejects non-integral scaled decimals before
  converting to `i128`.
- Decimal/money: constrained decimal and contextual amount `Display`
  implementations now emit canonical decimal strings without gratuitous
  trailing zeroes, matching serde and hash behavior.
- Prediction: `Other*` metadata-code constructors now reject codes already
  modeled by their owning enum instead of allowing ambiguous `Other` values.
- Prediction: `NumericRange` now rejects descending finite ranges and empty
  zero-width finite intervals.
- Decimal feature matrix: crates that import `paft_decimal::Decimal` now compile
  correctly when another package in the dependency graph enables
  `paft-decimal/bigdecimal` without enabling each downstream crate's local
  `bigdecimal` feature.
- Domain: structured `ReportingPeriod` values can no longer expose invalid public
  states such as quarter 5 or date/period years outside `0..=9999`, and low
  years now emit four-digit canonical codes so display/serde round trips
  preserve identity.
- Domain: `Horizon` and `ReportingPeriod` parsing now rejects malformed inputs whose
  canonical fallback would become a modeled token, such as `-1d` or
  `-2023Q4`, instead of accepting them as valid structured values.
- Domain docs/tests: clarified that partial modeled-looking provider labels
  such as `FY`, `2023-Q`, and `7 d` remain extensible `Other` values unless
  they match a supported parser shape or would canonicalize to a modeled token.
- Domain/money/fundamentals: string enum parsers now reject malformed inputs
  whose canonicalized form would resolve to a modeled value, such as `$USD`,
  `---NYSE`, or `CLOSED!`.
- Money: `Currency` parsing and metadata registration now require valid token
  boundaries for every code, so malformed metadata-known open currencies such
  as `$DOGE` no longer normalize to `DOGE`.
- Fundamentals: `Profile` JSON deserialization no longer rejects unknown
  payload fields, matching the workspace's open data-model serde policy.
- Docs/market: documented the wire compatibility policy: requests,
  configuration, and semantic invariant-bearing tagged shapes are strict,
  while provider/data payloads are forward-compatible by default. `Action`
  JSON now follows that policy by ignoring unknown payload fields.
- Prediction/money docs: public `OutcomeId` and `from_scaled_units`
  messages/docs now match trimming and decimal-backend behavior.
- Domain/money/fundamentals: manually constructed extensible enum `Other`
  payloads can no longer use tokens already modeled by the owning enum,
  preserving serde identity for values created through public constructors.
- Utils/core/domain/money/fundamentals: canonical `Other` enum tokens now reject
  canonical forms longer than `MAX_CANONICAL_TOKEN_LEN` (256 bytes), preventing
  unbounded unknown-token storage and round-tripping from untrusted inputs.

### Breaking Changes

- Fundamentals/facade: canonical string output for
  `RecommendationAction::Initiate` changed from `INIT` to `INITIATE`, and
  `InsiderPosition::VicePresident` changed from `VP` to `VICE_PRESIDENT`; the old
  short forms remain accepted aliases.
- Domain/fundamentals/facade: enum `FromStr`, `TryFrom<String>`,
  `try_from_str`, and `other` constructors now return crate-level errors:
  domain enum APIs return `DomainError`, fundamentals enum APIs return
  `FundamentalsError`, and facade callers can compose fundamentals parse
  failures through `paft::Error`.
- Domain/facade: `MarketState` no longer implements `Copy`, unknown
  `MarketState` tokens now parse as `MarketState::Other(OtherMarketState)`, and
  `MarketState::code()` is no longer `const`. `MarketState::full_name()` now
  returns `Cow<'static, str>` to display unknown provider states.
- Market/facade: `BookLevel::size` now uses `Option<QuantityAmount>`;
  `OptionContract::implied_volatility` and `OptionUpdate::implied_volatility`
  now use `Option<NonNegativeDecimal>`. `NewsRequest::count` now uses
  `std::num::NonZeroU32`.
- Market/facade: `SearchRequestBuilder::limit` now accepts `u32`,
  `SearchRequest::limit()` returns `Option<std::num::NonZeroU32>`, and
  `MarketError::InvalidSearchLimit` carries `u32` instead of `usize`.
- Fundamentals/facade: `MajorHolder::value` now uses `Ratio`, and
  `InstitutionalHolder::pct_held` now uses `Option<Ratio>`.
- Fundamentals/facade: `InsiderTransaction` struct literals and consumers must
  handle `url: Option<String>` instead of `url: String`.
- Money/facade: callers of non-panicking scalar arithmetic helpers must pass
  `&Decimal` instead of `Decimal`.
- Money/facade: callers of `Price::try_total` must pass `&QuantityAmount`
  instead of `&Decimal`; use `Price::try_total_decimal` for signed/raw decimal
  quantities.
- Money/facade: `set_currency_metadata` no longer changes `minor_units` for a
  code with a known scale; use `override_currency_metadata` for explicit
  replacement.
- Domain/money/fundamentals/facade: extensible enum constructors, parsers, and
  serde now reject unknown `Other` tokens whose canonical form exceeds 256 bytes.
- Decimal/money/market/fundamentals: decimal-backed JSON fields now emit
  canonical strings without gratuitous trailing zeroes, so values such as
  `"12.340"` serialize as `"12.34"` regardless of backend.
- Market/facade: `HistoryResponse::adjusted: bool` was replaced by
  `HistoryResponse::price_basis: OhlcPriceBasis`; update struct literals and
  JSON payloads to describe returned OHLC values with `PriceBasis` variants
  such as `Raw`, `ProviderAdjusted`, `CorporateActionAdjusted`, or
  `ContractRollAdjusted`; known corporate-action adjustments carry a non-empty
  `CorporateActionAdjustmentCauses` set aligned with the modeled `Action`
  classes.
- Market/facade: history request adjustment preference APIs were renamed from
  `HistoryFlags::AUTO_ADJUST`, `HistoryRequestBuilder::auto_adjust`, and
  `HistoryRequest::auto_adjust` to `HistoryFlags::PREFER_ADJUSTED_PRICES`,
  `HistoryRequestBuilder::prefer_adjusted_prices`, and
  `HistoryRequest::prefer_adjusted_prices`.
- Market/facade: history request missing-slot APIs were renamed from
  `HistoryFlags::KEEPNA`, `HistoryRequestBuilder::keepna`, and
  `HistoryRequest::keepna` to `HistoryFlags::KEEP_MISSING`,
  `HistoryRequestBuilder::keep_missing`, and `HistoryRequest::keep_missing`.
  The serialized flag bit is unchanged.
- Market/facade: nested metadata containers gained independent child metadata
  type parameters. `GenericHistoryResponse<M>`, `GenericOptionChain<M>`,
  `GenericDownloadEntry<M>`, `GenericDownloadResponse<M>`,
  `GenericCandleUpdate<M>`, `GenericOrderBook<M>`, `GenericQuote<M>`, and
  `GenericSearchResponse<M>` now apply `M` to the outer/container provider
  payload while nested rows or leaves default to `()`. Use the additional type
  parameters when child metadata is needed, such as
  `GenericHistoryResponse<ResponseMeta, CandleMeta>`.
- Market/fundamentals: `Action` and `Profile` JSON moved from externally tagged
  enum objects to flat tagged payloads with `kind`; fund profiles now put the
  fund type in `fund_kind` so it does not collide with the discriminator.
- Domain/facade: `Period` was replaced by `ReportingPeriod` for
  fiscal/provider labels and `CalendarPeriod` for calendar boundary logic.
  `ReportingPeriod::Quarter { year, quarter }`,
  `ReportingPeriod::Year { year }`, and `ReportingPeriod::Date(date)` now
  require validated component newtypes instead of raw integers or `NaiveDate`.
  Existing reporting literals should move to
  `ReportingPeriod::quarterly(year, quarter)?`,
  `ReportingPeriod::annual(year)?`, or `ReportingPeriod::date(date)?`.
- Fundamentals/facade: `EarningsYear::year` now uses `PeriodYear`; construct
  with `EarningsYear::new(year)?` or `PeriodYear::new(year)?` in struct
  literals. `EarningsYear` no longer implements `Default`.
- Market/facade: `Candle` now has `currency: Currency` and flattened
  `ohlc: Ohlc` `PriceAmount` values instead of independent `Price` fields for
  `open`, `high`, `low`, and `close`; `close_unadj` is now
  `Option<PriceAmount>`.
- Market/facade: `OrderBook`, `Quote`, `QuoteUpdate`, and aggregate `Snapshot`
  now require a record-level `currency`; their contained price fields use
  `PriceAmount`.
- Market/facade: `Candle::volume`, `Quote::day_volume`,
  `QuoteUpdate::volume`, and `Snapshot::volume` now use
  `Option<QuantityAmount>` instead of `Option<u64>`. `QuoteUpdate::volume` is
  a cumulative provider-defined session/window snapshot, not a per-update
  delta.
- Market/facade: option contract/update quote fields (`price`, `bid`, `ask`,
  and `last_price`) now use `PriceAmount`, and option contracts/updates now
  require `currency: Currency` for those premium amounts.
  `OptionContractKey::strike` remains a standalone `Price`.
- Market/facade: `contract_instrument` moved from `OptionContract` and
  `OptionUpdate` into `OptionContractKey`; JSON remains flattened at the
  contract/update object level, but Rust callers should use
  `contract.key.contract_instrument` and
  `OptionContractKey::with_contract_instrument(...)`.
- Domain/facade: `Instrument::unique_key()` no longer returns bare FIGI, ISIN,
  `SYMBOL@EXCHANGE`, or `SYMBOL` strings. It now includes asset kind and
  identifier source, e.g. `EQUITY|SYMBOL|4:AAPL` or
  `CRYPTO|SYMBOL|3:BTC`, so symbol-equivalent instruments from different asset
  classes cannot collide. Use `Instrument::display_key()` when the old compact
  display format is desired.
- Domain/fundamentals/facade: removed `Default` from `Symbol`, `Exchange`,
  `AssetKind`, and `FundKind` because their old defaults (`DEFAULT`, `NASDAQ`,
  `EQUITY`, and `ETF`) looked like real financial identity data.
- Domain/facade: `ReportingPeriod` no longer exposes calendar boundary helpers
  or `Ord`/`PartialOrd`; callers must choose `CalendarPeriod` for calendar
  boundaries or a provider-specific structural sort key for fiscal labels.
- Fundamentals/facade: `TrendPoint::period` and `RevisionPoint::period` were
  renamed to `horizon` and now use `Horizon`; helper methods were renamed from
  `find_by_period*`/`available_periods` to
  `find_by_horizon*`/`available_horizons`.
- Fundamentals/facade: removed
  `EpsRevisions::{total_up_revisions, total_down_revisions, net_revisions}`;
  revision horizons are overlapping lookbacks, not disjoint buckets. Select a
  concrete `RevisionPoint` before using `RevisionPoint::total_revisions` or
  `RevisionPoint::net_revisions`.
- Domain/money/fundamentals/facade: public `Other(Canonical)` enum payloads were
  replaced by typed wrappers. Use `Type::other("TOKEN")?`,
  `OtherType::new("TOKEN")?`, or the existing `FromStr`/serde parsers instead
  of constructing `Type::Other(Canonical::try_new(...).unwrap())` directly.

## [0.8.0] - 2026-05-27

This release is audited against the `v0.7.1` tag. It is a breaking API and
wire-format update across the workspace.

### Added

- New crate `paft-decimal` exposing `Decimal`, `RoundingStrategy`, parsing,
  rounding, scaled-unit construction, and canonical rendering helpers shared by
  money, market, fundamentals, and dataframe code.
- New crate `paft-prediction` exposing validated `EventId` and `OutcomeId`
  newtypes, `PredictionInstrument`, `Market`, `Token`, and `PredictionError`.
  The facade exposes these behind the new `prediction` feature.
- Money: added full-precision `Price` and `MonetaryAmount` value types. `Money`
  remains the settlement-oriented type that enforces currency minor units.
- Money: added `Money::new_exact`, `Money::try_div_money`, `Hash for Money`,
  `ExchangeRate::try_inverse`, `Currency::USDC`, `Currency::USDT`,
  `CurrencyMetadata` exports, and `Div<Money> -> Decimal` under
  `panicking-money-ops`.
- Market: added provider-metadata generic payloads with flattened
  `provider: M` fields for quotes, book levels/order books, candles/history,
  option contracts/updates/chains, news articles, search/download responses, and
  aggregate snapshots. Public aliases such as `Quote`, `Candle`, and `Snapshot`
  remain `M = ()`; their `Generic*` forms are exported from crate roots and the
  `paft` facade/prelude.
- Market: added `BookLevel`, `OrderBook`, `CandleUpdate`, `OptionSide`,
  `OptionContractKey`, and `OptionUpdate`.
- Market history: added intraday `Range` variants, second/minute/hour
  `Interval` variants, `Range::code`, `Interval::code`,
  `Interval::seconds`, and expanded `Interval::minutes`.
- Fundamentals: added `KeyStatistics` for valuation, trailing EPS, dividend,
  52-week range, volume, and beta metrics.
- Dataframe: added `Decimal128Encode` support for decimal backends and
  dataframe support for provider metadata and prediction types.

### Changed

- Workspace: version bumped to `0.8.0`, MSRV set to Rust `1.90`, workspace
  clippy lints were enabled, `polars` was bumped to `0.53`, `rust_decimal` to
  `1.42`, `bitflags` to `2.11`, and the old monolithic `df-derive` dependency
  was replaced by `df-derive-core`/`df-derive-macros` `0.3.1`.
- Facade: `paft::market`, `paft::fundamentals`, and `paft::prelude` now export
  the broader release surface directly, including decimal, price, provider
  metadata, option update, order book, key statistics, request builders/flags,
  money precision limits, and prediction types.
- Domain/utils: `Symbol` and `Canonical` now use `SmolStr`; typical short
  tokens avoid heap allocation and longer clones share storage.
- Domain: `AssetKind` is extensible via `AssetKind::Other(Canonical)`, and
  `Exchange::full_name()`/`AssetKind::full_name()` now return
  `Cow<'static, str>`.
- Domain: `Figi` now validates the Bloomberg FIGI structure in addition to the
  checksum, `Isin` uses strict `isin::parse`, and `Symbol` rejects non-ASCII
  input.
- Domain: `Period` parsing no longer depends on `regex`; documented quarter,
  fiscal year, ISO date, US date, and day-first date shapes are retained.
- Market, aggregates, and fundamentals: per-unit quoted values now use `Price`
  instead of `Money`, and analytic ratios/percentages that were `f64` now use
  `Decimal`.
- Market, aggregates, and fundamentals: numeric `DateTime<Utc>` serde formats
  now use Unix milliseconds in affected payloads, including candles, quote
  updates, order books, actions, options, news, snapshots, holders, calendar
  fields, share counts, and analyst upgrade/downgrade rows.
- Market: `Range`, `Interval`, `TimeSpec`, and `NewsTab` now use explicit compact
  wire codes. `TimeSpec` serializes as a tagged shape with millisecond period
  timestamps.
- Market and aggregate payloads now favor instrument-level identity over
  duplicated top-level `symbol`, `exchange`, `kind`, and `currency` fields.
- Dataframe: paft re-exports the shared `df-derive-core` `ToDataFrame`,
  `Columnar`, and `ToDataFrameVec` traits. The schema signature now returns
  owned column names, and `Columnar` works from references.

### Breaking Changes

- Domain: `Instrument` already stored `symbol`, optional `exchange`, optional
  `figi`, optional `isin`, and `kind` in `v0.7.1`. The break in `0.8.0` is API
  access: those fields are now public, `Instrument::new(Symbol, AssetKind)` and
  `Instrument::from_figi(...)` were added, and the old accessor/mutator methods
  (`symbol()`, `exchange()`, `figi()`, `isin()`, `kind()`, `has_*`,
  `try_set_*`, `try_with_*`) plus `try_new(...)` were removed.
- Domain: `AssetKind` no longer implements `Copy`; `AssetKind::code()` and
  `SearchRequestBuilder::kind()` are no longer `const`, and
  `SearchRequest::kind()` returns `Option<&AssetKind>`.
- Domain: `Exchange::full_name()` and `AssetKind::full_name()` now return
  `Cow<'static, str>` instead of `&str`.
- Aggregates: `FastInfo` and `Info` were removed. Use `Snapshot` for
  instant-in-time market data and the `paft-fundamentals` types for analyst,
  ESG, profile, key statistics, and statement data.
- Money: `Decimal` and `RoundingStrategy` are no longer public through
  `paft_money`; import them from `paft_decimal` or from the facade root.
- Money: `Currency` no longer implements `Default`. `Money::from_canonical_str`
  now rejects over-precise amounts instead of rounding them, and serde
  deserialization rejects over-precise amounts instead of accepting values that
  bypassed constructor validation. Use `Money::new` when explicit rounding is
  intended.
- Money, market, and fundamentals model split: quote, candle, book-level,
  option strike, option bid/ask/last, EPS, price target, dividend-per-share,
  and 52-week per-unit fields now use `Price`; settled totals remain `Money`.
- Market quotes/search/downloads/snapshots: public payloads now carry
  `Instrument` identity and optional `provider: M` metadata. Struct literals
  must account for fields such as `instrument`, `provider: ()`, `bid`, `ask`,
  `as_of`, and renamed `Quote::name`.
- Market history wire formats changed: `Range`/`Interval` serialize as compact
  codes such as `"1d"` and `"1mo"` instead of Rust variant names, and
  `TimeSpec` uses the tagged shape `{ "kind": "range", ... }` or
  `{ "kind": "period", ... }`.
- Market options: `OptionContract` now flattens an `OptionContractKey`,
  `contract_symbol` became optional `contract_instrument`, `in_the_money` is
  `Option<bool>`, `OptionChain` uses `contracts` plus `calls()`/`puts()`, and
  option requests use `underlying: Instrument` instead of `symbol: Symbol`.
- Market actions: `Action` serde now uses tagged snake_case JSON with a
  `"kind"` discriminator instead of externally tagged Rust variant names.
  `Action::Split` numerator and denominator fields are now `NonZeroU32`
  instead of `u32`; serde deserialization rejects zero split ratio components.
- Market, aggregates, and fundamentals timestamps that were serialized as Unix
  seconds now serialize as Unix milliseconds in the affected structs.
- Market payload aliases no longer implement `Eq`, and refactored generic
  payloads do not derive `Hash`; compare or hash explicit keys instead.
- Fundamentals: `Profile` serde now uses tagged snake_case JSON with a `"kind"`
  discriminator instead of externally tagged Rust variant names; `Profile::Fund`
  payloads use `"fund_kind"` for the fund type to avoid colliding with the
  profile discriminator.
- Fundamentals: analyst, ESG, holder, profile, and statement structs changed
  several field types (`Money` to `Price`, `f64` to `Decimal`, seconds to
  milliseconds) and added new optional statement fields. Revision helper totals
  now return `u64`/`i64`.
- Facade features: `aggregates` no longer enables `market` and `fundamentals`;
  `full` now includes `prediction`.
- Dataframe: `df-derive` trait identity moved to `df-derive-core`; handwritten
  impls must update `schema()` to `Vec<(String, DataType)>` and implement the
  reference-based `Columnar` API. `Instrument` dataframe column order now
  follows the public struct order: `symbol`, `exchange`, `figi`, `isin`, `kind`.

### Fixed

- Market: `SearchRequest` now stores the trimmed query it validates, so inputs
  such as `" AAPL "` are normalized to `"AAPL"` instead of preserving outer
  whitespace.
- Money: `ExchangeRate::inverse` now uses checked decimal division so callers
  can choose `ExchangeRate::try_inverse` to receive `MoneyError::ConversionError`
  instead of risking backend overflow in a panicking path.
- `paft-core` doctests now declare the dev dependency needed by macro-internal
  paths, and the `paft-money` doctest expectation now matches canonical decimal
  rendering without trailing zero padding.

### Migration notes

- Use direct `Instrument` fields (`instrument.symbol`, `instrument.exchange`,
  `instrument.figi`, `instrument.isin`, `instrument.kind`) in place of the
  removed accessors. The v0.7.1 identity model was already security fields plus
  `AssetKind`; there is no identifier enum migration for v0.8.0.
- Import decimal helpers from `paft_decimal` or from the facade root:
  `use paft::{Decimal, RoundingStrategy};`.
- Use `Money` for settled/payable amounts, `Price` for quoted per-unit values,
  and `MonetaryAmount` for full-precision currency totals before settlement.
- Prefer constructors such as `Quote::new(instrument)`,
  `OrderBook::new(instrument)`, `Candle::new(...)`,
  `OptionContractKey::new(...)`, and `Snapshot::new(instrument)`; if using
  struct literals, add `provider: ()` for the standard no-metadata case.
- Replace quote/search/snapshot top-level identity reads with `instrument` field
  access, for example `quote.instrument.symbol` and `quote.name`.
- Construct split actions with `std::num::NonZeroU32`, for example
  `NonZeroU32::new(2).unwrap()`.
- Replace `OptionChain { calls, puts }` field access with `chain.calls()` and
  `chain.puts()` over `contracts`.
- Multiply stored numeric timestamps by `1000` when migrating 0.7.x JSON that
  used Unix seconds into v0.8.0 millisecond wire formats.
- Update history and news JSON to the new compact codes (`"1d"`, `"1mo"`,
  `"NEWS"`, etc.) and the tagged `TimeSpec` shape.
- For yfinance-rs, see `../yfinance-rs/MIGRATION-paft-0.8.md`. For borsa,
  see `../borsa-workspace/MIGRATION-paft-0.8.md`.

### Documentation

- Updated README files and examples for public `Instrument` fields, provider
  metadata, prediction models, facade exports, and the `Money` / `Price` /
  `MonetaryAmount` split.

## [0.7.1] - 2025-10-31

### Added

- Domain: `paft_domain::identifiers::Symbol` now implements `PartialOrd` and `Ord`, enabling use in sorted collections and ordered comparisons.

## [0.7.0] - 2025-10-28

### Breaking Change

- Market quotes: added `day_volume` to `paft_market::market::quote::Quote` and `volume` to
  `paft_market::market::quote::QuoteUpdate` to surface intraday volumes.
- Market search: `SearchRequest` gains optional `lang` and `region` parameters to control
  language of results and region scoping. Builder now supports `.lang("en")` and
  `.region("US")`; getters `lang()`/`region()` return `Option<&str>`.
- Aggregates: `paft_aggregates::info::FastInfo` adds a new `volume: Option<u64>` field.
- Aggregates: `paft_aggregates::info::Info` adds new fields `price_target: Option<PriceTarget>`,
  `recommendation_summary: Option<RecommendationSummary>`, and `esg_scores: Option<EsgScores>`.

## [0.6.0] - 2025-10-21

### Breaking Change

- Market: `paft_market::responses::download::DownloadResponse` JSON shape changed from `{ "history": {SYM: HistoryResponse} }` to `{ "entries": [{ instrument, history }] }`, keyed by full `Instrument` identity (supports dual-listed symbols). Migrate symbol lookups by iterating `iter_by_symbol()` and collecting as needed.
- Aggregates: Removed the report envelopes (`InfoReport`, `SearchReport`, `DownloadReport`) from `paft-aggregates`.

### Added

- `DownloadResponse::iter()` and `iter_by_symbol()` helpers for zero-copy traversal of entries.

### Migration notes

- Market downloads: iterate entries via `iter()` or `iter_by_symbol()`; construct maps as needed.

### Documentation

- Documented the `tracing` feature flag in the workspace and `paft` READMEs, including scope and zero-cost when disabled.
- Added `borsa` to the Projects Using paft section
- Updated `paft-aggregates/README.md` and workspace READMEs to reflect removal of report envelopes (`InfoReport`, `SearchReport`, `DownloadReport`) and to describe `paft-aggregates` as snapshots only (`FastInfo`, `Info`).

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

[Unreleased]: https://github.com/paft-rs/paft/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/paft-rs/paft/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/paft-rs/paft/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/paft-rs/paft/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/paft-rs/paft/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/paft-rs/paft/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/paft-rs/paft/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/paft-rs/paft/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/paft-rs/paft/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/paft-rs/paft/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/paft-rs/paft/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/paft-rs/paft/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/paft-rs/paft/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/paft-rs/paft/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/paft-rs/paft/releases/tag/v0.1.0
