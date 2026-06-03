use std::num::NonZeroU32;
use std::str::FromStr;

use paft_domain::{DomainError, Horizon, OtherHorizon};

const fn nonzero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

#[test]
fn horizon_modeled_values_use_compact_lowercase_codes() {
    let cases = [
        ("7d", Horizon::Days(nonzero(7)), "7d"),
        ("7D", Horizon::Days(nonzero(7)), "7d"),
        ("30days", Horizon::Days(nonzero(30)), "30d"),
        ("1mo", Horizon::Months(nonzero(1)), "1mo"),
        ("3MO", Horizon::Months(nonzero(3)), "3mo"),
        ("6months", Horizon::Months(nonzero(6)), "6mo"),
        ("1y", Horizon::Years(nonzero(1)), "1y"),
        ("2YEARS", Horizon::Years(nonzero(2)), "2y"),
    ];

    for (input, expected, canonical) in cases {
        let parsed = Horizon::from_str(input).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(parsed.to_string(), canonical);
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            format!("\"{canonical}\"")
        );
        assert_eq!(
            serde_json::from_str::<Horizon>(&format!("\"{input}\"")).unwrap(),
            expected
        );
    }
}

#[test]
fn horizon_constructors_reject_zero_counts() {
    assert_eq!(
        Horizon::days(0).unwrap_err(),
        DomainError::InvalidHorizonCount { count: 0 }
    );
    assert_eq!(
        Horizon::months(0).unwrap_err(),
        DomainError::InvalidHorizonCount { count: 0 }
    );
    assert_eq!(
        Horizon::years(0).unwrap_err(),
        DomainError::InvalidHorizonCount { count: 0 }
    );
    assert_eq!(
        "0d".parse::<Horizon>().unwrap_err(),
        DomainError::InvalidHorizonCount { count: 0 }
    );
}

#[test]
fn horizon_other_roundtrips_without_colliding_with_modeled_values() {
    let custom = Horizon::from_str("provider horizon").unwrap();
    assert_eq!(custom.to_string(), "PROVIDER_HORIZON");
    assert!(matches!(custom, Horizon::Other(_)));

    let minute_like = Horizon::from_str("1m").unwrap();
    assert_eq!(minute_like.to_string(), "1M");
    assert!(matches!(minute_like, Horizon::Other(_)));
}

#[test]
fn horizon_rejects_inputs_that_canonicalize_to_modeled_tokens() {
    for input in ["-1d", "+1d", "(7d)", "!3mo!", "1y-"] {
        assert_eq!(
            input.parse::<Horizon>().unwrap_err(),
            DomainError::InvalidHorizonFormat {
                format: input.to_string()
            }
        );
    }
}

#[test]
fn other_horizon_rejects_modeled_horizon_tokens() {
    assert!(OtherHorizon::new("7d").is_err());
    assert!(OtherHorizon::new("3mo").is_err());
    assert!(OtherHorizon::new("-1d").is_err());
    assert!(OtherHorizon::new("(1y)").is_err());
    assert_eq!(
        OtherHorizon::new("provider horizon").unwrap().as_ref(),
        "PROVIDER_HORIZON"
    );
}
