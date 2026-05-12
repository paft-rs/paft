//! Integration tests for the `string_enum_*` macro family in `paft-core`.
//!
//! These tests define throwaway enums locally and exercise the macro-generated
//! impls directly so we cover the public surface (`code()`, `Display`,
//! `FromStr`, `TryFrom<String>`, `From<T> for String`, serde, `is_canonical`)
//! independently of any consumer crate.
//!
//! Several tests double as macro-hygiene checks: the macros must expand to
//! `$crate::__utils::...` paths so that consumer crates do not need
//! `paft_utils` in scope under any particular name.

#![allow(clippy::items_after_statements)]

use paft_core::__utils::{Canonical, StringCode};
use paft_core::PaftError;
use std::str::FromStr;

// ---------- Closed enum ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Side {
    Buy,
    Sell,
}

paft_core::string_enum_closed_with_code!(
    Side,
    "Side",
    { "BUY" => Side::Buy, "SELL" => Side::Sell },
    { "B" => Side::Buy, "S" => Side::Sell }
);
paft_core::impl_display_via_code!(Side);

#[test]
fn closed_code_returns_canonical_token() {
    assert_eq!(Side::Buy.code(), "BUY");
    assert_eq!(Side::Sell.code(), "SELL");
}

#[test]
fn closed_display_matches_code() {
    assert_eq!(Side::Buy.to_string(), "BUY");
    assert_eq!(format!("{}", Side::Sell), "SELL");
}

#[test]
fn closed_from_str_round_trip() {
    let parsed: Side = "buy".parse().unwrap();
    assert_eq!(parsed, Side::Buy);
    assert_eq!(parsed.to_string().parse::<Side>().unwrap(), parsed);

    // Whitespace and case are normalised.
    assert_eq!("  Sell  ".parse::<Side>().unwrap(), Side::Sell);
}

#[test]
fn closed_aliases_resolve_to_canonical_variants() {
    assert_eq!("b".parse::<Side>().unwrap(), Side::Buy);
    assert_eq!("S".parse::<Side>().unwrap(), Side::Sell);
}

#[test]
fn closed_try_from_string_and_into_string() {
    let parsed = Side::try_from(String::from("sell")).unwrap();
    assert_eq!(parsed, Side::Sell);

    let s: String = Side::Buy.into();
    assert_eq!(s, "BUY");
}

#[test]
fn closed_unknown_value_is_rejected() {
    let err = "hold".parse::<Side>().unwrap_err();
    assert!(matches!(
        err,
        PaftError::InvalidEnumValue { enum_name, value }
            if enum_name == "Side" && value == "hold"
    ));
}

#[test]
fn closed_serde_round_trip_json() {
    let json = serde_json::to_string(&Side::Sell).unwrap();
    assert_eq!(json, "\"SELL\"");
    let back: Side = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Side::Sell);

    // Provider strings (case-insensitive) deserialise too.
    let from_lower: Side = serde_json::from_str("\"buy\"").unwrap();
    assert_eq!(from_lower, Side::Buy);
}

#[test]
fn closed_code_is_const_fn() {
    // Compiles only if `Side::code` is `const fn` — the macro promises this
    // for closed enums (the open-enum variant cannot be const because it
    // dereferences a `Canonical` payload).
    const BUY: &str = Side::Buy.code();
    const SELL: &str = Side::Sell.code();
    assert_eq!(BUY, "BUY");
    assert_eq!(SELL, "SELL");
}

#[test]
fn closed_string_code_trait_impl() {
    // Generic over `StringCode` to confirm the macro wired the trait impl.
    fn code_of<T: StringCode>(t: &T) -> &str {
        t.code()
    }
    assert_eq!(code_of(&Side::Buy), "BUY");
    assert!(StringCode::is_canonical(&Side::Buy));
}

// ---------- Open enum (extensible) ----------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Venue {
    Nasdaq,
    Nyse,
    Other(Canonical),
}

paft_core::string_enum_with_code!(
    Venue,
    Other,
    "Venue",
    { "NASDAQ" => Venue::Nasdaq, "NYSE" => Venue::Nyse },
    { "BIG_BOARD" => Venue::Nyse }
);
paft_core::impl_display_via_code!(Venue);

#[test]
fn open_canonical_variants_round_trip() {
    assert_eq!(Venue::Nasdaq.code(), "NASDAQ");
    assert_eq!(Venue::Nyse.to_string(), "NYSE");

    let parsed: Venue = "nyse".parse().unwrap();
    assert_eq!(parsed, Venue::Nyse);
    assert!(parsed.is_canonical());
}

#[test]
fn open_alias_resolves_to_canonical_variant() {
    let parsed: Venue = "Big Board".parse().unwrap();
    assert_eq!(parsed, Venue::Nyse);
    assert!(parsed.is_canonical());
}

#[test]
fn open_unknown_input_becomes_other_with_canonical_token() {
    let parsed: Venue = "tsxv".parse().unwrap();
    let Venue::Other(canon) = parsed.clone() else {
        panic!("expected Other for unknown input, got {parsed:?}");
    };
    assert_eq!(canon.as_str(), "TSXV");
    assert_eq!(parsed.to_string(), "TSXV");
    assert!(!parsed.is_canonical());
}

#[test]
fn open_two_equivalent_unknown_inputs_normalise_to_same_other() {
    let a: Venue = "foo bar".parse().unwrap();
    let b: Venue = "foo_bar".parse().unwrap();
    let c: Venue = "  FOO-BAR  ".parse().unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a.to_string(), "FOO_BAR");
}

#[test]
fn open_serde_round_trip_json_canonical_and_other() {
    // Canonical variant
    let json = serde_json::to_string(&Venue::Nasdaq).unwrap();
    assert_eq!(json, "\"NASDAQ\"");
    let back: Venue = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Venue::Nasdaq);

    // Other variant
    let other: Venue = "moex".parse().unwrap();
    let json = serde_json::to_string(&other).unwrap();
    assert_eq!(json, "\"MOEX\"");
    let back: Venue = serde_json::from_str(&json).unwrap();
    assert_eq!(back, other);

    // Non-canonical input deserialises into a normalised Other.
    let from_provider: Venue = serde_json::from_str("\"Some Venue\"").unwrap();
    assert_eq!(from_provider.to_string(), "SOME_VENUE");
}

#[test]
fn open_empty_input_is_rejected() {
    let err = "".parse::<Venue>().unwrap_err();
    assert!(matches!(
        err,
        PaftError::InvalidEnumValue { enum_name, value }
            if enum_name == "Venue" && value.is_empty()
    ));

    let err = "   ".parse::<Venue>().unwrap_err();
    assert!(matches!(
        err,
        PaftError::InvalidEnumValue { enum_name, value }
            if enum_name == "Venue" && value == "   "
    ));
}

#[test]
fn open_input_canonicalising_to_empty_is_rejected_not_other() {
    // "!!!" canonicalises to "" → must be rejected via Canonical::try_new,
    // never produce Other(Canonical("")).
    let err = "!!!".parse::<Venue>().unwrap_err();
    assert!(matches!(
        err,
        PaftError::InvalidEnumValue { enum_name, value }
            if enum_name == "Venue" && value == "!!!"
    ));

    let err = "---".parse::<Venue>().unwrap_err();
    assert!(matches!(
        err,
        PaftError::InvalidEnumValue { enum_name, value }
            if enum_name == "Venue" && value == "---"
    ));
}

#[test]
fn open_try_from_string_and_into_string() {
    let v = Venue::try_from(String::from("nasdaq")).unwrap();
    assert_eq!(v, Venue::Nasdaq);

    let s: String = Venue::Nyse.into();
    assert_eq!(s, "NYSE");

    let other = Venue::try_from(String::from("custom venue")).unwrap();
    let s: String = other.into();
    assert_eq!(s, "CUSTOM_VENUE");
}

#[test]
fn open_string_code_trait_reports_canonical_correctly() {
    fn is_canon<T: StringCode>(t: &T) -> bool {
        t.is_canonical()
    }
    assert!(is_canon(&Venue::Nasdaq));
    let other: Venue = "weird".parse().unwrap();
    assert!(!is_canon(&other));
}

// ---------- Macro hygiene smoke test ----------
//
// This module deliberately does NOT bring `paft_utils` into scope — neither
// directly nor via `use paft_core::__utils::*`. The enum definition reaches
// `Canonical` through the fully qualified `paft_core::__utils::Canonical`
// re-export, and the macro invocation must work without anything else being
// in scope. Compilation and passing assertions prove that the macro bodies
// resolve all helpers via `$crate::__utils::...` as intended.
mod hygiene_no_paft_utils_in_scope {
    use paft_core::PaftError;
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Color {
        Red,
        Other(paft_core::__utils::Canonical),
    }

    paft_core::string_enum_with_code!(
        Color,
        Other,
        "Color",
        { "RED" => Color::Red }
    );
    paft_core::impl_display_via_code!(Color);

    #[test]
    fn macro_resolves_paft_utils_via_crate_re_export() {
        assert_eq!(Color::Red.code(), "RED");
        assert_eq!(Color::Red.to_string(), "RED");

        let parsed: Color = "  red  ".parse().unwrap();
        assert_eq!(parsed, Color::Red);

        let unknown: Color = "Forest Green".parse().unwrap();
        let Color::Other(c) = unknown.clone() else {
            panic!("expected Other for unknown input");
        };
        assert_eq!(c.as_str(), "FOREST_GREEN");
        assert_eq!(unknown.to_string(), "FOREST_GREEN");

        // Empty-after-canonicalisation must still be rejected even with the
        // re-exported `Canonical` path inside the macro expansion.
        let err = "...".parse::<Color>().unwrap_err();
        assert!(matches!(
            err,
            PaftError::InvalidEnumValue { enum_name, value }
                if enum_name == "Color" && value == "..."
        ));
    }
}
