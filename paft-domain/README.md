# paft-domain

`paft-domain` contains the core domain modeling primitives used throughout the paft workspace. These types represent the high-level financial concepts that were previously part of `paft-core`.

## What’s inside?

- `Instrument` and supporting enums such as `AssetKind`
- `Exchange` with canonical codes and serde helpers
- `MarketState` session enumeration
- `Period` type for quarter/year/date based periods plus helpers
- `DomainError` for domain-specific error reporting

All string-backed enums leverage the macro helpers exported from `paft-core`, keeping code generation consistent across the workspace.

## Feature Flags

- `dataframe`: Enables implementations for the dataframe conversion traits re-exported from `paft-utils`.

## Relationship to other crates

- Depends on `paft-core` for shared macros and error infrastructure.
- Used by `paft-fundamentals`, `paft-market`, and the `paft` facade when the `domain` feature is enabled.
