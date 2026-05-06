use paft_prediction::{EventID, OutcomeID};

// EventID tests

#[test]
fn event_id_accepts_valid_examples() {
    let valid_ids = [
        "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de",
        "0x8901bf367fcb32b406b54e8deb1bcb3320fdc4a994bd7f0a7a1fe72956dc1c9a",
    ];

    for id in valid_ids {
        let result = EventID::new(id);
        assert!(result.is_ok(), "Valid Event ID should be accepted: {id}");
    }
}

#[test]
fn event_id_rejects_empty_string() {
    let result = EventID::new("");
    assert!(result.is_err());
}

#[test]
fn event_id_rejects_whitespace_only_string() {
    let result = EventID::new("   ");
    assert!(result.is_err(), "Whitespace-only EventID should be rejected");
}

#[test]
fn event_id_rejects_wrong_length() {
    let result = EventID::new("0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085d");
    assert!(
        result.is_err(),
        "Event ID with wrong length should be rejected"
    );
}

#[test]
fn event_id_rejects_missing_0x_prefix() {
    let result = EventID::new("5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085dee");
    assert!(
        result.is_err(),
        "Event ID without 0x prefix should be rejected"
    );
}

#[test]
fn event_id_rejects_invalid_hex_characters() {
    let result = EventID::new("0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085dg");
    assert!(
        result.is_err(),
        "Event ID with invalid hex character should be rejected"
    );
}

#[test]
fn event_id_accepts_uppercase_hex() {
    let result = EventID::new("0x5EED579FF6763914D78A966C83473BA2485AC8910D0A0914EEF6D9FCB33085DE");
    assert!(
        result.is_ok(),
        "Event ID with uppercase hex should be accepted"
    );
}

#[test]
fn event_id_accepts_mixed_case_hex() {
    let result = EventID::new("0x5eed579fF6763914D78a966c83473bA2485ac8910d0a0914eef6D9fcb33085de");
    assert!(
        result.is_ok(),
        "Event ID with mixed case hex should be accepted"
    );
}

#[test]
fn event_id_normalizes_case_and_whitespace() {
    let padded_upper = format!(
        "  0x{}  ",
        "5EED579FF6763914D78A966C83473BA2485AC8910D0A0914EEF6D9FCB33085DE"
    );
    let canonical = "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de";

    let from_padded = EventID::new(&padded_upper).unwrap();
    let from_canonical = EventID::new(canonical).unwrap();

    assert_eq!(from_padded, from_canonical);
    assert_eq!(from_padded.as_ref(), canonical);
}

#[test]
fn event_id_normalizes_uppercase_0x_prefix() {
    // 0X (uppercase X) should be normalized to lowercase 0x.
    let upper_x = "0X5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de";
    let lower_x = "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de";
    assert_eq!(EventID::new(upper_x).unwrap(), EventID::new(lower_x).unwrap());
}

#[test]
fn event_id_rejects_embedded_control_characters() {
    // An embedded newline is rejected outright. (A *trailing* newline would be
    // stripped by trim and then fail the length/hex check, which is also fine,
    // but here we want to exercise the explicit control-char rejection.)
    let with_embedded_newline =
        "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d\n9fcb33085de";
    assert!(
        EventID::new(with_embedded_newline).is_err(),
        "EventID with an embedded newline should be rejected"
    );

    // NUL is a control character that is *not* whitespace, so trim leaves it
    // in place.
    let with_nul = "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085d\0";
    assert!(
        EventID::new(with_nul).is_err(),
        "EventID with a NUL control character should be rejected"
    );
}

#[test]
fn event_id_with_trailing_newline_fails() {
    // Spec example: the trailing newline is stripped by trim(), but the
    // result is too short to be a valid event id, so construction still fails.
    let result = EventID::new("0xabc\n");
    assert!(
        result.is_err(),
        "EventID with trailing newline + short body should be rejected"
    );
}

#[test]
fn event_id_from_str_parses_canonical_form() {
    let canonical = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    let parsed: EventID = canonical.parse().unwrap();
    assert_eq!(parsed.as_ref(), canonical);
}

#[test]
fn event_id_deserialize_normalizes() {
    let mixed = "0xABCdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    let original = EventID::new(mixed).unwrap();
    let json = serde_json::to_string(&original).unwrap();

    // The serialized form is the normalized lowercase value.
    assert_eq!(json, format!("\"{}\"", original.as_ref()));

    // Round-trip preserves equality...
    let round_trip: EventID = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, original);

    // ...and deserializing the *raw* mixed-case JSON also normalizes.
    let raw_json = format!("\"{mixed}\"");
    let from_raw: EventID = serde_json::from_str(&raw_json).unwrap();
    assert_eq!(from_raw, original);
}

#[test]
fn event_id_display_and_as_ref_consistency() {
    let id_str = "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de";
    let event_id = EventID::new(id_str).unwrap();

    assert_eq!(event_id.as_ref(), id_str);
    assert_eq!(event_id.to_string(), id_str);
}

// OutcomeID tests

#[test]
fn outcome_id_accepts_valid_examples() {
    let valid_ids = [
        "73470541315377973562501025254719659796416871135081220986683321361000395461644",
        "56393761733830483601097051857899348522495376869600726893014309766300892311293",
        "63099584499166723696938912801834245359789884653808158281242893092101276172908",
        "30276400766909644348018771740725995990021159099338826772350018698410972351366",
    ];

    for id in valid_ids {
        let result = OutcomeID::new(id);
        assert!(result.is_ok(), "Valid Outcome ID should be accepted: {id}");
        assert_eq!(result.unwrap().as_ref(), id);
    }
}

#[test]
fn outcome_id_accepts_single_digit() {
    let result = OutcomeID::new("0");
    assert!(result.is_ok(), "Single digit Outcome ID should be accepted");
}

#[test]
fn outcome_id_accepts_max_length() {
    let max_length_id = "1".repeat(78);
    let result = OutcomeID::new(&max_length_id);
    assert!(
        result.is_ok(),
        "Outcome ID at max length (78) should be accepted"
    );
}

#[test]
fn outcome_id_rejects_empty_string() {
    let result = OutcomeID::new("");
    assert!(result.is_err(), "Empty Outcome ID should be rejected");
}

#[test]
fn outcome_id_rejects_whitespace_only_string() {
    let result = OutcomeID::new("   ");
    assert!(
        result.is_err(),
        "Whitespace-only Outcome ID should be rejected"
    );
}

#[test]
fn outcome_id_rejects_exceeds_max_length() {
    let too_long_id = "1".repeat(79);
    let result = OutcomeID::new(&too_long_id);
    assert!(
        result.is_err(),
        "Outcome ID exceeding max length should be rejected"
    );
}

#[test]
fn outcome_id_rejects_leading_plus() {
    let result = OutcomeID::new("+123");
    assert!(
        result.is_err(),
        "Outcome ID with leading + should be rejected"
    );
}

#[test]
fn outcome_id_rejects_leading_minus() {
    let result = OutcomeID::new("-123");
    assert!(
        result.is_err(),
        "Outcome ID with leading - should be rejected"
    );
}

#[test]
fn outcome_id_normalizes_surrounding_whitespace() {
    let padded = "  12345  ";
    let canonical = "12345";
    assert_eq!(
        OutcomeID::new(padded).unwrap(),
        OutcomeID::new(canonical).unwrap()
    );
    assert_eq!(OutcomeID::new(padded).unwrap().as_ref(), canonical);
}

#[test]
fn outcome_id_rejects_non_digit_characters() {
    let invalid_ids = vec!["12a34", "123.456", "123e5", "123_456", "123-456"];

    for id in invalid_ids {
        let result = OutcomeID::new(id);
        assert!(
            result.is_err(),
            "Outcome ID with non-digit character should be rejected: {id}"
        );
    }
}

#[test]
fn outcome_id_rejects_embedded_whitespace() {
    let result = OutcomeID::new("123 456");
    assert!(
        result.is_err(),
        "Outcome ID with embedded space should be rejected"
    );
}

#[test]
fn outcome_id_rejects_embedded_control_characters() {
    let embedded_newline = "123\n456";
    assert!(
        OutcomeID::new(embedded_newline).is_err(),
        "OutcomeID with embedded newline should be rejected"
    );

    // NUL is not whitespace, so trim does not strip it.
    let with_nul = "12345\0";
    assert!(
        OutcomeID::new(with_nul).is_err(),
        "OutcomeID with NUL control character should be rejected"
    );
}

#[test]
fn outcome_id_from_str_parses_canonical_form() {
    let parsed: OutcomeID = "12345".parse().unwrap();
    assert_eq!(parsed.as_ref(), "12345");
}

#[test]
fn outcome_id_deserialize_normalizes() {
    let padded = "  12345  ";
    let original = OutcomeID::new(padded).unwrap();
    let json = serde_json::to_string(&original).unwrap();

    // Serialized form is the trimmed canonical value.
    assert_eq!(json, "\"12345\"");

    // Round-trip preserves equality...
    let round_trip: OutcomeID = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, original);

    // ...and deserializing the *raw* padded JSON also normalizes.
    let raw_json = format!("\"{padded}\"");
    let from_raw: OutcomeID = serde_json::from_str(&raw_json).unwrap();
    assert_eq!(from_raw, original);
}

#[test]
fn outcome_id_display_and_as_ref_consistency() {
    let id_str = "12345";
    let outcome_id = OutcomeID::new(id_str).unwrap();

    assert_eq!(outcome_id.as_ref(), id_str);
    assert_eq!(outcome_id.to_string(), id_str);
}
