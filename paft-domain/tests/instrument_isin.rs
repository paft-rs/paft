#[cfg(feature = "isin-validate")]
mod feature_enabled {
    use paft_domain::{
        AssetKind, DomainError, Instrument, instrument::is_valid_isin,
        instrument::normalize_isin_strict,
    };

    #[test]
    fn try_set_isin_accepts_clean_value() {
        let mut instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
        instrument.try_set_isin("US0378331005").expect("valid ISIN");
        assert_eq!(instrument.isin(), Some("US0378331005"));
        assert!(is_valid_isin("US0378331005"));
        assert_eq!(
            normalize_isin_strict("us0378331005").expect("valid normalization"),
            "US0378331005"
        );
    }

    #[test]
    fn try_set_isin_normalizes_loose_input() {
        let mut instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
        instrument
            .try_set_isin("us-037833-1005 ")
            .expect("normalized ISIN");
        assert_eq!(instrument.isin(), Some("US0378331005"));
        assert!(is_valid_isin("us-037833-1005 "));
    }

    #[test]
    fn try_set_isin_rejects_invalid_values() {
        let invalid_inputs = [
            "US037833100",   // too short after scrubbing
            "1234567890123", // too long
            "US0378331006",  // bad checksum
        ];

        for value in invalid_inputs {
            let mut instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
            let err = instrument.try_set_isin(value).expect_err("invalid ISIN");
            assert_eq!(
                err,
                DomainError::InvalidIsin {
                    value: value.to_string(),
                }
            );
            assert!(matches!(
                normalize_isin_strict(value),
                Err(DomainError::InvalidIsin { .. })
            ));
        }
    }

    #[test]
    fn set_isin_unchecked_bypasses_validation() {
        let mut instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
        instrument.set_isin_unchecked("raw-value");
        assert_eq!(instrument.isin(), Some("raw-value"));
    }
}

#[cfg(not(feature = "isin-validate"))]
mod feature_disabled {
    use paft_domain::{AssetKind, Instrument, instrument::is_valid_isin};

    #[test]
    fn try_set_isin_uppercases_and_trims() {
        let mut instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
        instrument
            .try_set_isin(" us-037833-1005 ")
            .expect("feature disabled accepts any value");
        assert_eq!(instrument.isin(), Some("US-037833-1005"));
    }

    #[test]
    fn try_set_isin_always_accepts_values() {
        let mut instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
        instrument
            .try_set_isin("invalid!!!")
            .expect("feature disabled accepts invalid forms");
        assert_eq!(instrument.isin(), Some("INVALID!!!"));
    }

    #[test]
    fn set_isin_unchecked_remains_raw() {
        let mut instrument = Instrument::from_symbol("AAPL", AssetKind::Equity);
        instrument.set_isin_unchecked(" raw-value ");
        assert_eq!(instrument.isin(), Some(" raw-value "));
    }

    #[test]
    fn is_valid_isin_checks_for_non_empty_scrubbed_content() {
        assert!(is_valid_isin("US0378331005"));
        assert!(is_valid_isin("us-037833-1005"));
        assert!(!is_valid_isin("   ---   "));
    }
}
