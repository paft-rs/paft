use paft_domain::{DomainError, EventID, Figi, Isin, OutcomeID, Symbol};
use serde_json::{from_str, to_string};

#[test]
fn isin_normalizes_display_and_as_ref() {
    let isin = Isin::new("us0378331005").expect("valid ISIN");
    assert_eq!(isin.as_ref(), "US0378331005");
    assert_eq!(isin.to_string(), "US0378331005");
}

#[test]
fn isin_serde_roundtrip() {
    let isin = Isin::new("US0378331005").expect("valid ISIN");
    let json = to_string(&isin).expect("serialize");
    assert_eq!(json, "\"US0378331005\"");
    let back: Isin = from_str(&json).expect("deserialize");
    assert_eq!(back, isin);
}

#[test]
fn isin_rejects_bad_checksum() {
    let err = Isin::new("US0378331006").expect_err("checksum should fail");
    assert!(matches!(err, DomainError::InvalidIsin { .. }));
}

#[test]
fn isin_accepts_loose_input_with_scrubbing() {
    let isin = Isin::new(" us-037833-1005 \t").expect("valid loose input");
    assert_eq!(isin.as_ref(), "US0378331005");
}

#[test]
fn isin_rejects_non_isin_content() {
    let err = Isin::new(" invalid-value ").expect_err("non-isin values are rejected");
    assert!(matches!(err, DomainError::InvalidIsin { .. }));
}

#[test]
fn isin_rejects_empty_after_scrub() {
    let err = Isin::new(" --- ").expect_err("scrubbed empty should fail");
    assert!(matches!(err, DomainError::InvalidIsin { .. }));
}

#[test]
fn figi_uppercases_and_trims() {
    let figi = Figi::new(" bbg000b9xry4 ").expect("valid FIGI");
    assert_eq!(figi.as_ref(), "BBG000B9XRY4");
    assert_eq!(figi.to_string(), "BBG000B9XRY4");
}

#[test]
fn figi_serde_roundtrip() {
    let figi = Figi::new("BBG000B9XRY4").expect("valid FIGI");
    let json = to_string(&figi).expect("serialize");
    assert_eq!(json, "\"BBG000B9XRY4\"");
    let back: Figi = from_str(&json).expect("deserialize");
    assert_eq!(back, figi);
}

#[test]
fn figi_rejects_invalid_length() {
    let err = Figi::new("BBG000B9XRY").expect_err("length must be 12");
    assert!(matches!(err, DomainError::InvalidFigi { .. }));
}

#[test]
fn figi_rejects_non_alphanumeric() {
    let err = Figi::new("BBG000B9XRY!").expect_err("non-alphanumeric fails");
    assert!(matches!(err, DomainError::InvalidFigi { .. }));
}

#[test]
fn symbol_accepts_and_canonicalizes_variety() {
    let cases = [
        ("AAPL", "AAPL"),
        ("brk.b", "BRK.B"),
        ("rds/a", "RDS/A"),
        ("bac.pra", "BAC.PRA"),
        ("^gspc", "^GSPC"),
        ("es=f", "ES=F"),
        ("eurusd=x", "EURUSD=X"),
        ("btc-usd", "BTC-USD"),
        ("gs2c.de", "GS2C.DE"),
        ("vod.l", "VOD.L"),
        ("ry.to", "RY.TO"),
        ("0700.hk", "0700.HK"),
        ("7203.t", "7203.T"),
        ("600519.ss", "600519.SS"),
        ("aapl240118c00180000", "AAPL240118C00180000"),
    ];

    for (input, expected) in cases {
        let symbol = Symbol::new(input).unwrap();
        assert_eq!(symbol.as_str(), expected);
    }
}

#[test]
fn symbol_rejects_invalid_inputs() {
    let reject_cases = [
        "",
        "   ",
        "AAPL US",
        "AAPL\tUS",
        "AAPL\nUS",
        "\u{7f}AAPL",
        "AAPL\u{1f}",
    ];

    for input in reject_cases {
        let err = Symbol::new(input).expect_err("invalid symbol should be rejected");
        assert!(matches!(err, DomainError::InvalidSymbol { .. }));
    }

    let overlong = "a".repeat(65);
    let err = Symbol::new(&overlong).expect_err("overlong symbol should be rejected");
    assert!(matches!(err, DomainError::InvalidSymbol { .. }));
}

#[test]
fn symbol_equality_is_case_insensitive() {
    let lower = Symbol::new("aapl").unwrap();
    let upper = Symbol::new("AAPL").unwrap();
    assert_eq!(lower, upper);
}

#[test]
fn symbol_len_and_display() {
    let symbol = Symbol::new(" brk.b ").unwrap();
    assert_eq!(symbol.len(), 5);
    assert_eq!(symbol.to_string(), "BRK.B");
    assert!(!symbol.is_empty());
}

#[test]
fn symbol_serde_roundtrip() {
    let symbol = Symbol::new("ES=F").unwrap();
    let json = to_string(&symbol).unwrap();
    assert_eq!(json, "\"ES=F\"");
    let back: Symbol = from_str(&json).unwrap();
    assert_eq!(back, symbol);
}

#[test]
fn figi_rejects_bad_checksum() {
    let err = Figi::new("BBG000B9XRY5").expect_err("checksum should fail");
    assert!(matches!(err, DomainError::InvalidFigi { .. }));
}

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
