use paft_domain::{DomainError, Figi, Isin, Symbol};
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
