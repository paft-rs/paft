use paft_domain::{IdentifierScheme, Instrument};

#[test]
fn deserialize_requires_valid_isin() {
    let json = r#"{
        "id": {
            "Security": {
                "symbol": "AAPL",
                "exchange": null,
                "figi": null,
                "isin": "US0378331006"
            }
        },
        "kind": "EQUITY"
    }"#;

    let err = serde_json::from_str::<Instrument>(json)
        .expect_err("invalid ISIN should be rejected during deserialize");
    assert!(err.to_string().contains("Invalid ISIN"));
}

#[test]
fn deserialize_normalizes_loose_isin() {
    let json = r#"{
        "id": {
            "Security": {
                "symbol": "AAPL",
                "exchange": null,
                "figi": null,
                "isin": "us-037833-1005 "
            }
        },
        "kind": "EQUITY"
    }"#;

    let instrument: Instrument = serde_json::from_str(json).expect("valid loose ISIN");
    match instrument.id() {
        IdentifierScheme::Security(sec) => {
            assert_eq!(sec.isin.as_ref().map(AsRef::as_ref), Some("US0378331005"));
        }
        IdentifierScheme::Prediction(_) => panic!("expected Security identifier"),
    }
}

#[test]
fn deserialize_rejects_empty_after_scrub() {
    let json = r#"{
        "id": {
            "Security": {
                "symbol": "AAPL",
                "exchange": null,
                "figi": null,
                "isin": "---"
            }
        },
        "kind": "EQUITY"
    }"#;

    let err =
        serde_json::from_str::<Instrument>(json).expect_err("empty after scrub should be rejected");
    assert!(err.to_string().contains("Invalid ISIN"));
}
