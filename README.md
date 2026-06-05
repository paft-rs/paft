# paft: Provider Agnostic Financial Types

[![Crates.io](https://img.shields.io/crates/v/paft)](https://crates.io/crates/paft)
[![Docs.rs](https://docs.rs/paft/badge.svg)](https://docs.rs/paft)
[![CI](https://github.com/paft-rs/paft/actions/workflows/ci.yml/badge.svg)](https://github.com/paft-rs/paft/actions/workflows/ci.yml)
[![Downloads](https://img.shields.io/crates/d/paft)](https://crates.io/crates/paft)
[![License](https://img.shields.io/crates/l/paft)](LICENSE)

**Building a provider-neutral foundation for financial data in Rust.**

> New to paft? Start with the [facade crate README](paft/README.md) for
> installation, feature flags, examples, and API-level usage. This root README
> explains the workspace, design contract, and provider integration model.

## Vision

Financial data providers expose different wire formats, field names, request
models, enum tokens, and completeness guarantees. That fragmentation makes it
harder to switch providers, combine multiple sources, share analysis code, and
build reusable Rust libraries over financial data.

`paft` models reusable financial concepts, not provider wire shapes. It gives
provider adapters, applications, storage layers, analysis libraries, and
visualization tools a common set of data values without forcing every provider
behind one artificial client API.

The goal is an ecosystem where:

1. Provider crates keep their own authentication, endpoints, rate limits, and
   request flow, but return paft-compatible output values.
2. Application developers can write analysis code over stable financial types
   instead of each provider's proprietary payloads.
3. Library authors can build on a shared foundation for identifiers, money,
   market data, fundamentals, prediction markets, and DataFrame export.
4. Unknown provider values and provider-specific extras remain representable
   without weakening the canonical paft model.

## The Dream

A provider adapter should be free to look like itself. A Yahoo-style client, a
brokerage data client, and a prediction-market client can all expose different
methods, pagination, auth, and rate-limit behavior. The interoperability point
is the value boundary: once they hand you a `Quote`, `HistoryResponse`,
`Money`, `Instrument`, or `Profile`, the rest of your code can stop caring
which upstream API produced it.

That is the narrow promise of paft: not a universal financial API, but a stable
language of financial data types that provider crates and downstream tools can
agree on.

## Workspace Crates

The workspace is on the v0.9.0 line. The `paft` facade enables domain, market,
and fundamentals types by default; aggregate snapshots, prediction-market
models, DataFrame export, tracing, formatting, and backend choices are opt-in.

| Crate README | Role |
| --- | --- |
| [`paft`](paft/README.md) | Facade crate for applications that want one dependency, common re-exports, forwarded features, and runnable examples. |
| [`paft-domain`](paft-domain/README.md) | Instruments, exchanges, asset kinds, market state, reporting/calendar periods, horizons, and validated security identifiers. |
| [`paft-money`](paft-money/README.md) | Currency, money, price, quantity, settlement scale, runtime currency metadata, and optional localized formatting. |
| [`paft-decimal`](paft-decimal/README.md) | Backend-agnostic decimal alias and helpers, constrained decimal newtypes, canonical string serde, and decimal128 support. |
| [`paft-market`](paft-market/README.md) | Quotes, candles, history, order books, options, news, search, downloads, and validated market request builders. |
| [`paft-fundamentals`](paft-fundamentals/README.md) | Profiles, statements, analysis rows, holders, ESG, key statistics, and related helper models. |
| [`paft-aggregates`](paft-aggregates/README.md) | Instant-in-time instrument snapshots with optional provider metadata. |
| [`paft-prediction`](paft-prediction/README.md) | Prediction-market identifiers, instruments, markets, and tokens. |
| [`paft-utils`](paft-utils/README.md) | Canonical string tokens, open-enum support utilities, and optional Polars DataFrame traits. |
| [`paft-core`](paft-core/README.md) | Shared error, enum, display, and serde macro building blocks for paft crates and compatible adapters. |

## Architecture

`paft` separates provider access from data semantics:

```text
Applications, analysis libraries, storage, visualization
        ^
        | consume standardized paft values
        |
Provider adapters and client crates
        ^
        | fetch, authenticate, rate-limit, and normalize
        |
Financial data provider APIs
```

The ecosystem layers are:

- `paft`: facade for normal application use.
- Domain model crates: `paft-domain`, `paft-money`, `paft-decimal`,
  `paft-market`, `paft-fundamentals`, `paft-aggregates`, and
  `paft-prediction`.
- Infrastructure crates: `paft-core` and `paft-utils`.
- Provider crates outside this workspace: API clients that convert provider
  wire data into paft values.

This structure lets applications and libraries share analysis code over paft
types while providers keep their specialized APIs.

## Provider-Agnostic Contract

`paft` types represent concepts, not "whatever this provider happened to
return". Required fields are the valid minimum for the concept. If a provider
record cannot supply those fields, the adapter should fail the conversion or
keep the data in provider-specific metadata instead of manufacturing an
invalid paft value.

Provider-specific extras belong in generic metadata. Standard type aliases use
no metadata where that is the common case, and `Generic*` forms preserve
provider fields when adapters need them.

Provider-facing enums are open where upstreams can add new tokens. Known
provider aliases should map to canonical variants. Truly unknown values should
round-trip through typed `Other` wrappers, not through ad hoc strings.

## Provider Integration Guidance

Provider crates should keep efficient internal wire types and add an explicit
conversion layer into paft values at the public boundary. Public APIs can still
look like the provider: provider-specific method names, pagination, auth,
regional behavior, and rate-limit policies are not hidden by paft.

When mapping provider data:

- Map aliases to canonical variants whenever paft models the concept.
- Use `Type::other(..)` or typed `OtherType::new(..)` only for genuinely
  unknown provider tokens; those constructors reject values that already parse
  to modeled variants or aliases.
- Log unknown `Other` values in adapter code so future canonical mappings can
  be added with evidence.
- Test both directions of every mapping: known provider aliases should become
  canonical variants, and unknown provider tokens should round-trip.
- Do not flatten incomplete provider records into paft concepts. Missing
  required conceptual fields are conversion errors, not optional details.
- Prefix or nest provider metadata fields when there is any chance of colliding
  with paft-owned JSON fields.

## Wire And Metadata Policy

Requests, configuration, and semantic metadata shapes are strict when silently
dropping fields could change meaning. Invariant-bearing request/config types
use builders and manual or shadow deserialization where needed. Plain data bags
can remain public structs when no invariant is enforced.

Provider and data payload structs are forward-compatible by default: unmodeled
JSON fields are ignored unless validation requires rejecting them. A tagged
payload is not strict solely because it has a `kind` discriminator.

Serde-flattened provider metadata shares the owning JSON namespace, so
colliding JSON field names are unsupported rather than universally detected.
Prefer provider-specific prefixes or nested metadata objects. DataFrame export
uses a separate namespace: provider metadata columns are emitted under
`provider.*`, while explicitly flattened DataFrame fields validate duplicate
output column names.

## Feature Families

Exact feature names and install snippets live in the crate READMEs. At the
workspace level, the optional feature families are:

- DataFrame export through Polars and shared `ToDataFrame` traits.
- `bigdecimal` as an alternate decimal backend to the default
  `rust_decimal` backend.
- Locale-aware money formatting and strict parsing.
- Opt-in panicking `Money` arithmetic operators for controlled code paths;
  safe `try_*` methods remain the default recommendation.
- Feature-gated `tracing` spans in selected constructors, validators, parsers,
  and money operations. No subscriber is bundled.
- Optional aggregate snapshot and prediction-market model crates through the
  facade.

## Scope

`paft` currently focuses on reusable financial data values: market data,
fundamentals, aggregate snapshots, prediction-market payloads, identifiers,
money, decimal handling, serde, and optional DataFrame export.

It does not model trading execution or portfolio infrastructure. Orders,
trades, fills, positions, accounts, balances, portfolio accounting, risk
metrics, strategy models, and backtest result types should be built by
specialized crates or applications on top of paft data.

## Examples And Docs

- [Facade README](paft/README.md): installation, feature flags, quickstart, and
  common examples.
- [Facade examples](paft/examples/): runnable examples for v0.9 ergonomics,
  provider metadata, metadata DataFrame export, and extensible enums.
- [docs.rs/paft](https://docs.rs/paft): facade API documentation.
- Crate-specific docs and links are listed in [Workspace Crates](#workspace-crates).

## Projects Using paft

The following open-source projects use paft types in their public APIs:

- [yfinance-rs](https://github.com/gramistella/yfinance-rs): ergonomic Yahoo
  Finance client.
- [borsa](https://github.com/borsaorg/borsa): high-level asynchronous API for
  fetching market and financial data from multiple sources.

Want to add your project? Open a PR to include it here.

## Contributing

Contributions are welcome for core models, provider adapters, documentation,
examples, tests, performance, and ergonomics.

Start with [CONTRIBUTING.md](CONTRIBUTING.md) and
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), then open a
[GitHub issue](https://github.com/paft-rs/paft/issues) or start a
[GitHub discussion](https://github.com/paft-rs/paft/discussions).

## License

MIT License. See [LICENSE](LICENSE) for details.
