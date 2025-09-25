use paft_utils::{Canonical, CanonicalError, StringCode, canonicalize};

#[test]
fn canonicalize_applies_normalization_rules() {
    assert_eq!(canonicalize("usd"), "USD");
    assert_eq!(canonicalize("Pre-Market"), "PRE_MARKET");
    assert_eq!(canonicalize("S&P 500"), "S_P_500");
    assert_eq!(canonicalize("  multiple   spaces  "), "MULTIPLE_SPACES");
}

#[test]
fn canonicalize_collapses_and_trims_underscores() {
    assert_eq!(canonicalize("__foo--bar__"), "FOO_BAR");
    assert_eq!(canonicalize("!@#"), "");
}

#[test]
fn canonical_try_new_rejects_empty_tokens() {
    let err = Canonical::try_new("***").unwrap_err();
    let CanonicalError::InvalidCanonicalToken { value } = err;
    assert_eq!(value, "***");
}

#[test]
fn canonical_try_new_accepts_valid_tokens() {
    let canonical = Canonical::try_new("foo bar").expect("valid token");
    assert_eq!(canonical.as_str(), "FOO_BAR");
    assert_eq!(canonical.to_string(), "FOO_BAR");
}

#[test]
fn canonical_from_str_delegates_to_try_new() {
    let canonical: Canonical = "  other  value  ".parse().expect("valid token");
    assert_eq!(canonical.as_ref(), "OTHER_VALUE");
}

#[test]
fn string_code_default_is_canonical() {
    #[derive(Debug)]
    struct Dummy;

    impl StringCode for Dummy {
        fn code(&self) -> &'static str {
            "DUMMY"
        }
    }

    let value = Dummy;
    assert!(value.is_canonical());
    assert_eq!(value.code(), "DUMMY");
}
