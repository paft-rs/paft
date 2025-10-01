use paft_utils::{Canonical, CanonicalError, StringCode, canonicalize};
use std::borrow::Cow;

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

    // Empty string should also be rejected
    let err = Canonical::try_new("").unwrap_err();
    let CanonicalError::InvalidCanonicalToken { value } = err;
    assert_eq!(value, "");
}

#[test]
fn canonical_try_new_rejects_all_separator_inputs() {
    // All separators should be rejected
    let separator_inputs = vec![
        "***", "!!!", "@@@", "###", "$$$", "%%%", "^^^", "&&&", "***", "(((", ")))", "---", "+++",
        "===", "{{{", "}}}", "|||", "\\\\\\", ":::", ";;%", ",,,", "...", "???", "~~~", "```",
        "___", // multiple underscores
    ];

    for input in separator_inputs {
        let err = Canonical::try_new(input).unwrap_err();
        let CanonicalError::InvalidCanonicalToken { value } = err;
        assert_eq!(
            value, input,
            "Failed to reject all-separator input: {input}"
        );
    }
}

#[test]
fn canonical_try_new_rejects_all_non_ascii_inputs() {
    // All non-ASCII inputs should be rejected
    let non_ascii_inputs = vec![
        "€€€",
        "¥¥¥",
        "£££",
        "¢¢¢",
        "αβγ",
        "你好",
        "Здравствуй",
        "مرحبا",
        "Γεια σας",
        "שלום",
        "नमस्ते",
        "こんにちは",
        "안녕하세요",
        "🚀🚀🚀",
        "🎉🎉🎉",
        "💯💯💯",
        "ééé", // accented characters
        "ñññ",
        "üüü",
        "ççç",
    ];

    for input in non_ascii_inputs {
        let err = Canonical::try_new(input).unwrap_err();
        let CanonicalError::InvalidCanonicalToken { value } = err;
        assert_eq!(
            value, input,
            "Failed to reject all-non-ASCII input: {input}"
        );
    }
}

#[test]
fn canonical_try_new_accepts_valid_tokens() {
    let canonical = Canonical::try_new("foo bar").expect("valid token");
    assert_eq!(canonical.as_str(), "FOO_BAR");
    assert_eq!(canonical.to_string(), "FOO_BAR");
}

#[test]
fn canonical_roundtrip_property() {
    // Test round-trip property: Canonical::try_new(s).map(|c| c.as_str().to_string()) == canonicalize(s)
    // for all valid inputs that produce non-empty canonical forms
    let test_cases = vec![
        "USD",
        "usd",
        "FOO_BAR",
        "foo bar",
        "123",
        "1 2 3",
        "S&P 500",
        "Pre-Market",
        "multiple   spaces",
        "  leading spaces  ",
        "trailing spaces   ",
        "  both  ",
        "a1B2c3",
        "TEST123",
        "TEST_123",
        "TEST_123_TEST",
    ];

    for input in test_cases {
        let canonical_result = Canonical::try_new(input);

        // If Canonical creation succeeds, the result should equal canonicalize result
        if let Ok(canonical) = canonical_result {
            let canonical_str = canonical.as_str();
            let canonicalize_result = canonicalize(input);

            assert_eq!(
                canonical_str,
                canonicalize_result.as_ref(),
                "Round-trip failed for input: {input:?}"
            );
        }
        // If Canonical creation fails, canonicalize should produce empty string
        else {
            let canonicalize_result = canonicalize(input);
            assert!(
                canonicalize_result.as_ref().is_empty(),
                "Canonical creation failed but canonicalize produced non-empty result for input: {input:?}"
            );
        }
    }
}

#[test]
fn canonicalize_idempotent_property() {
    // Test idempotence: canonicalize(canonicalize(s)) == canonicalize(s)
    // and that canonical results are marked as borrowed
    let test_cases = vec![
        "",
        "USD",
        "usd",
        "FOO_BAR",
        "foo bar",
        "123",
        "1 2 3",
        "S&P 500",
        "Pre-Market",
        "multiple   spaces",
        "  leading spaces  ",
        "trailing spaces   ",
        "  both  ",
        "a1B2c3",
        "TEST123",
        "TEST_123",
        "TEST_123_TEST",
        "_A",
        "A_",
        "_FOO_",
        "FOO__BAR",
        "***",
        "€€€",
        "🚀🚀🚀",
        "ééé",
        "你好",
    ];

    for input in test_cases {
        let c1 = canonicalize(input);

        // If the result is non-empty, it should be canonical
        if !c1.is_empty() {
            // Verify the result matches the expected canonical pattern
            let result_str = c1.as_ref();
            assert!(
                result_str
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'),
                "canonicalize result should only contain A-Z, 0-9, and underscores: {result_str:?}"
            );
            assert!(
                !result_str.starts_with('_') && !result_str.ends_with('_'),
                "canonicalize result should not start or end with underscore: {result_str:?}"
            );

            // Second canonicalization should return borrowed if input was already canonical
            let c2 = canonicalize(c1.as_ref());
            // Check if input was already canonical by checking if canonicalize(input) == input
            if c1.as_ref() == input {
                match c2 {
                    std::borrow::Cow::Borrowed(_) => {} // Expected for already canonical
                    std::borrow::Cow::Owned(_) => {
                        panic!("Expected borrowed result for already canonical input: {input:?}")
                    }
                }
            }
        }

        // Test idempotence: canonicalize(canonicalize(s)) == canonicalize(s)
        let c2 = canonicalize(c1.as_ref());
        assert_eq!(
            c1, c2,
            "canonicalize is not idempotent for input: {input:?}"
        );
    }
}

#[test]
fn canonicalize_separator_collapse_edge_cases() {
    // Test that leading/trailing/sequences collapse correctly
    let test_cases = vec![
        ("__us d - - e__", "US_D_E"),
        ("   ", ""),          // all spaces should become empty
        ("\t\t\t", ""),       // all tabs should become empty
        ("\n\n\n", ""),       // all newlines should become empty
        ("___", ""),          // multiple underscores should become empty
        ("a___b", "A_B"),     // multiple underscores between valid chars
        ("_a_b_", "A_B"),     // leading and trailing underscores
        ("a_b__c", "A_B_C"),  // double underscore in middle
        ("a__b__c", "A_B_C"), // multiple double underscores
        ("a_b_c_", "A_B_C"),  // trailing underscore
        ("_a_b_c", "A_B_C"),  // leading underscore
    ];

    for (input, expected) in test_cases {
        let result = canonicalize(input);
        assert_eq!(
            result.as_ref(),
            expected,
            "Failed for input: {:?} -> expected: {:?}, got: {:?}",
            input,
            expected,
            result.as_ref()
        );
    }
}

#[test]
fn canonicalize_and_canonical_try_new_are_consistent() {
    // Test that canonicalize and Canonical::try_new are consistent
    // Canonical::try_new(s) succeeds iff canonicalize(s) is non-empty
    let test_cases = vec![
        "",         // empty
        "USD",      // valid ASCII
        "usd",      // valid ASCII lowercase
        "FOO_BAR",  // valid with underscores
        "foo bar",  // valid with spaces
        "123",      // valid digits
        "1 2 3",    // valid with spaces
        "***",      // all separators
        "€€€",      // all non-ASCII
        "S&P 500",  // mixed valid/invalid
        "_A",       // leading underscore
        "A_",       // trailing underscore
        "_FOO_",    // leading and trailing underscores
        "FOO__BAR", // double underscore
        "a1B2c3",   // mixed case
        "TEST123",  // mixed alphanumeric
    ];

    for input in test_cases {
        let canonical_result = Canonical::try_new(input);
        let canonicalize_result = canonicalize(input);
        let canonicalize_is_empty = canonicalize_result.as_ref().is_empty();

        match canonical_result {
            Ok(_) => {
                assert!(
                    !canonicalize_is_empty,
                    "Canonical::try_new succeeded but canonicalize produced empty for: {input:?}"
                );
            }
            Err(_) => {
                assert!(
                    canonicalize_is_empty,
                    "Canonical::try_new failed but canonicalize produced non-empty for: {input:?}"
                );
            }
        }
    }
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

#[test]
fn canonicalize_fast_path_for_already_canonical_strings() {
    // Test that already canonical strings return borrowed references
    let canonical_input = "USD";
    let result = canonicalize(canonical_input);

    // The result should be borrowed (not owned)
    match result {
        std::borrow::Cow::Borrowed(s) => assert_eq!(s, canonical_input),
        std::borrow::Cow::Owned(_) => {
            panic!("Expected borrowed result for already canonical input")
        }
    }
}

#[test]
fn canonicalize_normalizes_non_canonical_strings() {
    // Test that non-canonical strings get normalized and return owned strings
    let non_canonical_input = "usd";
    let result = canonicalize(non_canonical_input);

    // The result should be owned since normalization occurred
    match result {
        std::borrow::Cow::Owned(s) => assert_eq!(s, "USD"),
        std::borrow::Cow::Borrowed(_) => panic!("Expected owned result for non-canonical input"),
    }
}

#[test]
fn canonicalize_correctly_handles_leading_underscores() {
    // Test cases that should be normalized (owned results)
    assert_eq!(canonicalize("_A"), "A");
    assert_eq!(canonicalize("A_"), "A");
    assert_eq!(canonicalize("_FOO_"), "FOO");
    assert_eq!(canonicalize("_"), "");

    // Test that these produce owned results (not borrowed)
    match canonicalize("_A") {
        std::borrow::Cow::Owned(s) => assert_eq!(s, "A"),
        std::borrow::Cow::Borrowed(_) => {
            panic!("Expected owned result for input with leading underscore")
        }
    }

    match canonicalize("A_") {
        std::borrow::Cow::Owned(s) => assert_eq!(s, "A"),
        std::borrow::Cow::Borrowed(_) => {
            panic!("Expected owned result for input with trailing underscore")
        }
    }
}

#[test]
fn canonicalize_additional_edge_cases() {
    // Multiple spaces and mixed separators
    assert_eq!(canonicalize(" usd  eur "), "USD_EUR");

    // Multiple underscores and trimming
    assert_eq!(canonicalize("__X__"), "X");
    assert_eq!(canonicalize("x__y"), "X_Y");

    // Already canonical should be borrowed
    assert_eq!(canonicalize("A_B"), "A_B");
    match canonicalize("A_B") {
        std::borrow::Cow::Borrowed(s) => assert_eq!(s, "A_B"),
        std::borrow::Cow::Owned(_) => {
            panic!("Expected borrowed result for already canonical input")
        }
    }

    // Non-ASCII characters are treated as separators
    assert_eq!(canonicalize("€USD—¥"), "USD");
}

#[test]
fn canonicalize_is_idempotent() {
    let test_cases = vec![
        "",
        "USD",
        "usd",
        "FOO_BAR",
        "foo bar",
        "123",
        "1 2 3",
        "_A",
        "A_",
        "_FOO_",
        "€USD—¥",
        "x__y",
        "__X__",
        " usd  eur ",
        "!@#$%",
        "a1B2c3",
        "_a1B2c3_",
    ];

    for input in test_cases {
        let once = canonicalize(input);
        let twice = canonicalize(&once);

        // Should be equal
        assert_eq!(
            once, twice,
            "canonicalize is not idempotent for input: {input:?}"
        );

        // The borrowing behavior can legitimately change between calls
        // (e.g., "usd" -> "USD" (owned), then "USD" -> "USD" (borrowed))
        // Just ensure the string values are the same
    }
}

#[test]
fn canonicalize_result_is_canonical() {
    let test_cases = vec![
        "USD",
        "usd",
        "FOO_BAR",
        "foo bar",
        "123",
        "1 2 3",
        "_A",
        "A_",
        "_FOO_",
        "€USD—¥",
        "x__y",
        "__X__",
        " usd  eur ",
        "!@#$%",
        "a1B2c3",
        "_a1B2c3_",
    ];

    for input in test_cases {
        let once = canonicalize(input);
        let twice = canonicalize(&once);

        // canonicalize should be idempotent: canonicalize(canonicalize(x)) == canonicalize(x)
        assert_eq!(
            once, twice,
            "canonicalize is not idempotent for input: {input:?}"
        );

        // Since it's idempotent, the result must be canonical (for non-empty results)
        let result_str = once.as_ref();
        if !result_str.is_empty() {
            // Test that already-canonical strings return borrowed results
            if input
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                && !input.starts_with('_')
                && !input.ends_with('_')
                && !input.contains("__")
            {
                match &once {
                    Cow::Borrowed(_) => {} // Expected for already canonical
                    Cow::Owned(_) => {
                        panic!("Already canonical input {input:?} should return borrowed")
                    }
                }
            }
        }
    }
}

#[test]
fn pathological_separators_collapse_to_single_underscore_and_trim() {
    let test_cases = vec![
        ("!@#$", ""),
        ("a!@#$b", "A_B"),
        ("a!@#$", "A"),
        ("!@#$b", "B"),
        ("a!@#$b!@#$c", "A_B_C"),
        ("___a___", "A"),
        ("a___b", "A_B"),
        ("a___", "A"),
        ("___", ""),
        ("a!@#$_b!@#$", "A_B"),
        ("!@#$a!@#$", "A"),
        ("a!@#$b_", "A_B"),
        ("_a!@#$b", "A_B"),
        ("_a!@#$b_", "A_B"),
    ];

    for (input, expected) in test_cases {
        let result = canonicalize(input);
        assert_eq!(
            result.as_ref(),
            expected,
            "Failed for input: {:?} -> expected: {:?}, got: {:?}",
            input,
            expected,
            result.as_ref()
        );
    }
}

#[test]
fn canonicalize_sanity_properties() {
    let test_cases = vec![
        "",
        "USD",
        "usd",
        "FOO_BAR",
        "foo bar",
        "123",
        "1 2 3",
        "_A",
        "A_",
        "_FOO_",
        "€USD—¥",
        "x__y",
        "__X__",
        " usd  eur ",
        "!@#$%",
        "a1B2c3",
        "_a1B2c3_",
    ];

    for input in test_cases {
        let result = canonicalize(input);

        // Idempotency: canonicalize(canonicalize(x).as_ref()) == canonicalize(x)
        let double_result = canonicalize(result.as_ref());
        assert_eq!(result, double_result, "Not idempotent for input: {input:?}");

        // Fast-path correctness: is_canonical(x) ⇒ canonicalize(x).is_borrowed()
        // (except empty → still Owned empty)
        if !result.as_ref().is_empty() {
            let expected_borrowed = input
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                && !input.starts_with('_')
                && !input.ends_with('_')
                && !input.contains("__");
            match &result {
                Cow::Borrowed(_) => {
                    assert!(expected_borrowed, "Unexpected borrow for input: {input:?}");
                }
                Cow::Owned(_) => {
                    assert!(!expected_borrowed, "Unexpected owned for input: {input:?}");
                }
            }
        }

        // Round-trip Canonical: Canonical::try_new(x) succeeds iff !canonicalize(x).is_empty()
        match Canonical::try_new(input) {
            Ok(_) => assert!(
                !result.as_ref().is_empty(),
                "Canonical::try_new succeeded but canonicalize produced empty for: {input:?}"
            ),
            Err(_) => assert!(
                result.as_ref().is_empty(),
                "Canonical::try_new failed but canonicalize produced non-empty for: {input:?}"
            ),
        }
    }
}

#[test]
fn canonicalize_handles_mixed_canonical_and_non_canonical() {
    let inputs_and_expected = vec![
        ("USD", "USD"),         // Already canonical - should be borrowed
        ("usd", "USD"),         // Needs normalization - should be owned
        ("FOO_BAR", "FOO_BAR"), // Already canonical - should be borrowed
        ("foo bar", "FOO_BAR"), // Needs normalization - should be owned
        ("123", "123"),         // Already canonical - should be borrowed
        ("1 2 3", "1_2_3"),     // Needs normalization - should be owned
        ("_A", "A"),            // Leading underscore - should be owned
        ("A_", "A"),            // Trailing underscore - should be owned
        ("_FOO_", "FOO"),       // Leading and trailing underscores - should be owned
    ];

    for (input, expected) in inputs_and_expected {
        let result = canonicalize(input);
        assert_eq!(result.as_ref(), expected);

        // Check if borrowing behavior is correct
        if input == expected {
            // Input is already canonical, should be borrowed
            match result {
                std::borrow::Cow::Borrowed(s) => assert_eq!(s, input),
                std::borrow::Cow::Owned(_) => {
                    panic!("Expected borrowed result for canonical input: {input}")
                }
            }
        } else {
            // Input needs normalization, should be owned
            match result {
                std::borrow::Cow::Owned(s) => assert_eq!(s, expected),
                std::borrow::Cow::Borrowed(_) => {
                    panic!("Expected owned result for non-canonical input: {input}")
                }
            }
        }
    }
}
