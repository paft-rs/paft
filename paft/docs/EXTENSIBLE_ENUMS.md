# The Extensible Enum Pattern in paft

Overview
--------

paft uses a consistent typed `Other` extensible enum pattern across provider-facing enums (`Currency`, `Exchange`, `AssetKind`, `Period`, `RecommendationGrade`, etc.). This embraces the reality that providers invent new tokens and aliases over time. Instead of failing on unknown values, these enums parse known canonical tokens and fall back to enum-specific unknown-code wrappers such as `OtherCurrency`, `OtherExchange`, and `OtherPeriod`.

Rules at a glance
-----------------

- Emission is a single canonical token per known variant (ASCII UPPERCASE, no spaces). Parsers accept a superset of aliases case‑insensitively.
- `Other(OtherX)` serializes and displays as its canonical string (no escape prefix).
- Public `OtherX::new(...)` constructors reject tokens that the enum parser maps to modeled variants or aliases.
- Unknown inputs normalize to uppercase with separators collapsed (via `paft_utils::canonicalize`).
- Serde round‑trips preserve identity for canonical variants and normalize unknowns consistently.

Implementation Pattern
----------------------

Use `paft-core` macros for consistent behavior:

```rust
use paft_core::{PaftError, impl_display_via_code, other_string_code_type, string_enum_with_code};
use std::str::FromStr;

paft_core::other_string_code_type!(
    /// Provider-specific example token not modeled by `ExampleEnum`.
    pub struct OtherExampleEnum for ExampleEnum;
    type Error = PaftError;
    parse(input) => ExampleEnum::from_str(input);
    invalid(input) => PaftError::InvalidEnumValue {
        enum_name: "ExampleEnum",
        value: input.to_string(),
    };
);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExampleEnum {
    Variant1,
    Variant2,
    Other(OtherExampleEnum),
}

paft_core::string_enum_with_code!(
    ExampleEnum, Other(OtherExampleEnum), "ExampleEnum",
    {
        "VARIANT1" => ExampleEnum::Variant1,
        "VARIANT2" => ExampleEnum::Variant2
    },
    {
        // Optional aliases a parser accepts, all mapped to canonical variants
        "ALT_NAME" => ExampleEnum::Variant1,
    }
);

paft_core::impl_display_via_code!(ExampleEnum);
```

Consumer Guidelines
-------------------

- Always handle `Other(OtherX)` in matches.
- Prefer canonical variants in your own code; use `Type::other(...)` or `OtherX::new(...)` only for values that are not modeled by that enum.
- Use `is_canonical()` where available to branch fast paths safely.

Examples
--------

Handling unknown values gracefully:

```rust
use paft_money::{Currency, IsoCurrency};

fn label(currency: Currency) -> String {
    match currency {
        Currency::Iso(IsoCurrency::USD) => "US Dollar".into(),
        Currency::Iso(IsoCurrency::EUR) => "Euro".into(),
        Currency::Other(code) => format!("Unknown currency: {}", code),
        _ => currency.to_string(),
    }
}
```

Normalizing provider strings to canonical variants:

```rust
use paft_money::{Currency, IsoCurrency};

fn normalize_currency(code: &str) -> Currency {
    match code.to_uppercase().as_ref() {
        "USD" | "US_DOLLAR" | "DOLLAR" => Currency::Iso(IsoCurrency::USD),
        "EUR" | "EURO" => Currency::Iso(IsoCurrency::EUR),
        "BTC" | "BITCOIN" | "XBT" => Currency::BTC,
        other => Currency::try_from_str(other)
            .unwrap_or_else(|_| Currency::other("UNKNOWN").expect("valid fallback code")),
    }
}
```

Why Canonical instead of String?
-------------------------------

The typed `OtherX` wrappers contain `Canonical`, which enforces lexical invariants (non‑empty, trimmed, uppercase ASCII with single underscores). The wrapper constructor adds the enum-specific invariant: a value cannot be `OtherX` if the owning enum parser recognizes it as a modeled variant or alias.

Trade‑offs
----------

- Benefits: graceful degradation, provider compatibility, future‑proofing, type safety for known variants.
- Costs: runtime string matching, explicit `Other` handling in consumers, careful API design to encourage convergence.

Migration Tips
--------------

1. Add `Other` handling to match statements.
2. Map common provider tokens to canonical variants.
3. Log unknowns to discover patterns for future canonicalization.
4. Prefer canonical variants inside your libraries and expose helpers to consumers.

See also
--------

- Best practices: ./BEST_PRACTICES.md
- Canonicalization utilities: `paft-utils::canonicalize`, `paft_utils::Canonical`
