#[cfg(feature = "isin-validate")]
mod feature_enabled {
    use paft_domain::{AssetKind, DomainError, Instrument, Isin};

    #[test]
    fn try_set_isin_accepts_clean_value() {
        let mut instrument =
            Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid symbol");
        instrument.try_set_isin("US0378331005").expect("valid ISIN");
        assert_eq!(instrument.isin_str(), Some("US0378331005"));
        assert!(Isin::new("US0378331005").is_ok());
        let normalized = Isin::new("us0378331005").expect("valid normalization");
        assert_eq!(normalized.as_ref(), "US0378331005");
    }

    #[test]
    fn deserialize_requires_valid_isin() {
        let json = r#"{
            "figi": null,
            "isin": "US0378331006",
            "symbol": "AAPL",
            "exchange": null,
            "kind": "EQUITY"
        }"#;

        let err = serde_json::from_str::<Instrument>(json)
            .expect_err("invalid ISIN should be rejected during deserialize");
        assert!(err.to_string().contains("Invalid ISIN"));
    }

    #[test]
    fn deserialize_normalizes_loose_isin() {
        let json = r#"{
            "figi": null,
            "isin": "us-037833-1005 ",
            "symbol": "AAPL",
            "exchange": null,
            "kind": "EQUITY"
        }"#;

        let instrument: Instrument = serde_json::from_str(json).expect("valid loose ISIN");
        assert_eq!(instrument.isin_str(), Some("US0378331005"));
    }

    #[test]
    fn try_set_isin_normalizes_loose_input() {
        let mut instrument =
            Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid symbol");
        instrument
            .try_set_isin("us-037833-1005 ")
            .expect("normalized ISIN");
        assert_eq!(instrument.isin_str(), Some("US0378331005"));
        assert!(Isin::new("us-037833-1005 ").is_ok());
    }

    #[test]
    fn try_set_isin_rejects_invalid_values() {
        let invalid_inputs = [
            "US037833100",   // too short after scrubbing
            "1234567890123", // too long
            "US0378331006",  // bad checksum
        ];

        for value in invalid_inputs {
            let mut instrument =
                Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid symbol");
            let err = instrument.try_set_isin(value).expect_err("invalid ISIN");
            assert_eq!(
                err,
                DomainError::InvalidIsin {
                    value: value.to_string(),
                }
            );
            assert!(matches!(
                Isin::new(value),
                Err(DomainError::InvalidIsin { .. })
            ));
        }
    }
}

#[cfg(not(feature = "isin-validate"))]
mod feature_disabled {
    use paft_domain::{AssetKind, DomainError, Instrument, Isin};

    #[test]
    fn try_set_isin_scrubs_and_uppercases() {
        let mut instrument =
            Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid symbol");
        instrument
            .try_set_isin(" us-037833-1005 ")
            .expect("feature disabled scrubs separators and allows value");
        assert_eq!(instrument.isin_str(), Some("US0378331005"));
    }

    #[test]
    fn try_set_isin_accepts_non_empty_values() {
        let mut instrument =
            Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid symbol");
        instrument
            .try_set_isin("invalid!!!")
            .expect("feature disabled accepts non-empty forms");
        assert_eq!(instrument.isin_str(), Some("INVALID"));
    }

    #[test]
    fn try_set_isin_rejects_empty_after_scrub() {
        let mut instrument =
            Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid symbol");
        let err = instrument
            .try_set_isin("   ---   ")
            .expect_err("scrubbed empty strings are rejected");
        assert!(matches!(
            err,
            paft_domain::DomainError::InvalidIsin { value }
                if value == "   ---   "
        ));
    }

    #[test]
    fn isin_new_checks_for_non_empty_scrubbed_content() {
        assert!(Isin::new("US0378331005").is_ok());
        assert!(Isin::new("us-037833-1005").is_ok());
        assert!(matches!(
            Isin::new("   ---   "),
            Err(DomainError::InvalidIsin { .. })
        ));
    }

    #[test]
    fn deserialize_scrubs_and_uppercases() {
        let json = r#"{
            "figi": null,
            "isin": " us-037833-1005 ",
            "symbol": "AAPL",
            "exchange": null,
            "kind": "EQUITY"
        }"#;

        let instrument: Instrument =
            serde_json::from_str(json).expect("feature disabled normalization");
        assert_eq!(instrument.isin_str(), Some("US0378331005"));
    }

    #[test]
    fn deserialize_rejects_empty_after_scrub() {
        let json = r#"{
            "figi": null,
            "isin": "---",
            "symbol": "AAPL",
            "exchange": null,
            "kind": "EQUITY"
        }"#;

        let err = serde_json::from_str::<Instrument>(json)
            .expect_err("empty after scrub should be rejected");
        assert!(err.to_string().contains("Invalid ISIN"));
    }
}
