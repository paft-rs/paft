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
fn event_id_rejects_wrong_length() {
    let result = EventID::new("0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085d");
    assert!(
        result.is_err(),
        "Event ID with wrong length should be rejected"
    );
}

#[test]
fn event_id_rejects_missing_0x_prefix() {
    let result = EventID::new("5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de");
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
fn event_id_display_and_as_ref_consistency() {
    let id_str = "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de";
    let event_id = EventID::new(id_str).unwrap();

    assert_eq!(event_id.as_ref(), id_str);
    assert_eq!(event_id.to_string(), id_str);
}

// OutcomeID tests

#[test]
fn token_id_accepts_valid_examples() {
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
fn token_id_accepts_single_digit() {
    let result = OutcomeID::new("0");
    assert!(result.is_ok(), "Single digit Outcome ID should be accepted");
}

#[test]
fn token_id_accepts_max_length() {
    let max_length_id = "1".repeat(78);
    let result = OutcomeID::new(&max_length_id);
    assert!(
        result.is_ok(),
        "Outcome ID at max length (78) should be accepted"
    );
}

#[test]
fn token_id_rejects_empty_string() {
    let result = OutcomeID::new("");
    assert!(result.is_err(), "Empty Outcome ID should be rejected");
}

#[test]
fn token_id_rejects_exceeds_max_length() {
    let too_long_id = "1".repeat(79);
    let result = OutcomeID::new(&too_long_id);
    assert!(
        result.is_err(),
        "Outcome ID exceeding max length should be rejected"
    );
}

#[test]
fn token_id_rejects_leading_plus() {
    let result = OutcomeID::new("+123");
    assert!(
        result.is_err(),
        "Outcome ID with leading + should be rejected"
    );
}

#[test]
fn token_id_rejects_leading_minus() {
    let result = OutcomeID::new("-123");
    assert!(
        result.is_err(),
        "Outcome ID with leading - should be rejected"
    );
}

#[test]
fn token_id_rejects_leading_whitespace() {
    let result = OutcomeID::new(" 123");
    assert!(
        result.is_err(),
        "Outcome ID with leading space should be rejected"
    );
}

#[test]
fn token_id_rejects_leading_tab() {
    let result = OutcomeID::new("\t123");
    assert!(
        result.is_err(),
        "Outcome ID with leading tab should be rejected"
    );
}

#[test]
fn token_id_rejects_non_digit_characters() {
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
fn token_id_rejects_trailing_whitespace() {
    let result = OutcomeID::new("123 ");
    assert!(
        result.is_err(),
        "Outcome ID with trailing space should be rejected"
    );
}

#[test]
fn token_id_rejects_embedded_whitespace() {
    let result = OutcomeID::new("123 456");
    assert!(
        result.is_err(),
        "Outcome ID with embedded space should be rejected"
    );
}

#[test]
fn token_id_display_and_as_ref_consistency() {
    let id_str = "12345";
    let outcome_id = OutcomeID::new(id_str).unwrap();

    assert_eq!(outcome_id.as_ref(), id_str);
    assert_eq!(outcome_id.to_string(), id_str);
}
