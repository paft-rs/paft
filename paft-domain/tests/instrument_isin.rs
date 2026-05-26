use paft_domain::Instrument;

#[test]
fn deserialize_requires_valid_isin() {
    let json = r#"{
        "symbol": "AAPL",
        "exchange": null,
        "figi": null,
        "isin": "US0378331006",
        "kind": "EQUITY"
    }"#;

    let err = serde_json::from_str::<Instrument>(json)
        .expect_err("invalid ISIN should be rejected during deserialize");
    assert!(err.to_string().contains("Invalid ISIN"));
}

#[test]
fn deserialize_normalizes_trimmed_isin() {
    let json = r#"{
        "symbol": "AAPL",
        "exchange": null,
        "figi": null,
        "isin": "us0378331005 ",
        "kind": "EQUITY"
    }"#;

    let instrument: Instrument = serde_json::from_str(json).expect("valid normalized ISIN");
    assert_eq!(
        instrument.isin.as_ref().map(AsRef::as_ref),
        Some("US0378331005")
    );
}

#[test]
fn deserialize_rejects_formatted_isin() {
    let json = r#"{
        "symbol": "AAPL",
        "exchange": null,
        "figi": null,
        "isin": "US-037833-1005",
        "kind": "EQUITY"
    }"#;

    let err =
        serde_json::from_str::<Instrument>(json).expect_err("formatted ISIN should be rejected");
    assert!(err.to_string().contains("Invalid ISIN"));
}
