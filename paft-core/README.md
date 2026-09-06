paft-core
=========

Shared error, enum, and serde building blocks for paft crates.

[![Crates.io](https://img.shields.io/crates/v/paft-core)](https://crates.io/crates/paft-core)
[![Docs.rs](https://docs.rs/paft-core/badge.svg)](https://docs.rs/paft-core)
[![Downloads](https://img.shields.io/crates/d/paft-core)](https://crates.io/crates/paft-core)

- Shared enum parsing error (`PaftError`)
- Canonical string enum macros (`string_enum_*`, `impl_display_via_code`)
- `other_string_code_type` for typed open-enum fallback codes
- Serde helpers for timestamp encodings

Install
-------

Most applications should depend on the facade crate:

```toml
[dependencies]
paft = "0.10.0"
```

Advanced (direct dependency, minimal):

```toml
[dependencies]
paft-core = { version = "0.10.0", default-features = false }
```

Use `paft-core` directly when defining paft-compatible crates or local enum
models that need the macro toolkit.

Features
--------

`paft-core` has no optional features. Serde support and timestamp helpers are
always available; DataFrame traits live in `paft-utils`.

Quickstart
----------

```rust
use paft_core::PaftError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Side {
    Buy,
    Sell,
}

paft_core::string_enum_closed_with_code!(
    Side,
    "Side",
    { "BUY" => Side::Buy, "SELL" => Side::Sell }
);
paft_core::impl_display_via_code!(Side);

assert_eq!(Side::Buy.code(), "BUY");
assert_eq!("sell".parse::<Side>().unwrap(), Side::Sell);
assert!(matches!("".parse::<Side>(), Err(PaftError::InvalidEnumValue { .. })));
```

Open enums with typed `Other`
-----------------------------

Use open enum macros for provider-facing concepts where upstreams can add new
tokens. The typed `OtherX` wrapper preserves unknown values while rejecting
tokens the enum already models. Unknown tokens are normalized into bounded
canonical strings.

```rust
use paft_core::PaftError;

paft_core::other_string_code_type!(
    /// Provider-specific venue not modeled by `Venue`.
    pub struct OtherVenue for Venue;
    type Error = PaftError;
    parse(input) => input.parse::<Venue>();
    invalid(input) => PaftError::InvalidEnumValue {
        enum_name: "Venue",
        value: input.to_string(),
    };
);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Venue {
    Nasdaq,
    Nyse,
    Other(OtherVenue),
}

paft_core::string_enum_with_code!(
    Venue, Other(OtherVenue), "Venue",
    {
        "NASDAQ" => Venue::Nasdaq,
        "NYSE" => Venue::Nyse
    },
    {
        "NASDAQ_GS" => Venue::Nasdaq,
        "NEW_YORK_STOCK_EXCHANGE" => Venue::Nyse
    }
);
paft_core::impl_display_via_code!(Venue);

assert_eq!("nasdaq-gs".parse::<Venue>().unwrap(), Venue::Nasdaq);
assert_eq!("dark pool".parse::<Venue>().unwrap().to_string(), "DARK_POOL");
assert!(OtherVenue::new("NASDAQ").is_err());
```

UTC instants
------------

`paft::core::serde_helpers` (or `paft_core::serde_helpers` for direct dependents)
exposes `parse_timestamp`, `TimestampError`, `TimestampErrorKind`,
`ts_iso8601`, `ts_iso8601_option`, and `ts_iso8601_vec`. Use the serde modules
with `#[serde(with = "paft::core::serde_helpers::ts_iso8601")]`, adding `default`
for optional fields that allow omission.

The canonical format is UTC ISO-8601-style text. Years `0000` through `9999`
follow RFC 3339 syntax; other years use Chrono-compatible signed expanded years:
`+` and five or six digits for positive years, `-` and four to six digits for
negative years. Negative years may have four-digit padding (`-0001`), but
expanded forms cannot have redundant leading zeros (`+010000`, `-00001`), and
negative zero is rejected. Chrono determines the supported calendar range.

Input requires padded date/time components, `T`/`t`, an optional fraction of
one to nine digits, and `Z`/`z` or `±HH:MM`. Whitespace, unpadded components,
`UTC`, colonless offsets, and excess fractional digits including trailing zeros
are rejected before Chrono validates calendar, clock, and offset arithmetic.
Leap seconds are explicitly rejected. Offset-induced UTC overflow is an error.

Output uses UTC `Z` and Chrono's `SecondsFormat::AutoSi`: the shortest exact
width among zero, three, six, and nine digits (`.1` → `.100`, `.1234` →
`.123400`, `.123400001` → `.123400001`). Canonicalization preserves the instant,
not its original offset, spelling, or declared source precision.

In memory and JSON, non-leap-second timestamps retain Chrono's supported range.
`timestamp_nanos_exact` independently checks export as signed i64 Unix
nanoseconds, from `1677-09-21T00:12:43.145224192Z` through
`2262-04-11T23:47:16.854775807Z`. PAFT's DataFrame columns use exactly
`Datetime(Nanoseconds, None)`, with physical counts denoting UTC and no timezone
annotation. Empty/all-null batches retain this schema. A supplied out-of-range
value errors rather than becoming null. Core's shared errors distinguish leap
seconds from export range failures and require no Polars dependency.

**v0.10 migration:** PAFT model JSON changes from integer milliseconds to strings
across all UTC-instant fields, including optional/list fields and history bounds.
Canonical deserializers reject integer and floating-point JSON and never infer
an epoch unit. Given a known legacy schema, read the signed integer exactly,
use `DateTime::from_timestamp_millis`, and serialize with the new adapter. Keep
calendar dates as dates. For application-owned legacy formats, the explicit
`ts_milliseconds`, `ts_milliseconds_option`, and `ts_milliseconds_vec` adapters
remain available and retain their exact-or-error millisecond policy.

Links
-----

- API docs: https://docs.rs/paft-core
- Workspace overview: https://github.com/paft-rs/paft/blob/main/README.md
- License: [LICENSE](../LICENSE)
