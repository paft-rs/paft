//! Tests for `MarketState` and Period canonical behaviors.

use chrono::NaiveDate;
use paft_core::domain::{MarketState, Period};
use std::str::FromStr;

#[test]
fn market_state_helper_methods() {
    assert!(MarketState::Pre.is_trading());
    assert!(MarketState::Regular.is_trading());
    assert!(MarketState::Post.is_trading());
    assert!(!MarketState::Closed.is_trading());
    assert!(!MarketState::Halted.is_trading());

    assert_eq!(MarketState::Pre.full_name(), "Pre-market");
    assert_eq!(MarketState::Regular.full_name(), "Regular session");
    assert_eq!(MarketState::Post.full_name(), "Post-market");
}

struct PeriodCase {
    input: &'static str,
    expected: Period,
    canonical: &'static str,
}

#[test]
fn period_round_trips_display_fromstr_serde() {
    for case in period_cases() {
        let parsed = case.input.parse::<Period>().unwrap();
        assert_eq!(parsed, case.expected);

        let display = parsed.to_string();
        assert_eq!(display, case.canonical);

        let reparsed = Period::from_str(&display).unwrap();
        assert_eq!(reparsed, case.expected);

        let json = serde_json::to_string(&parsed).unwrap();
        assert_eq!(json, format!("\"{}\"", case.canonical));
        let back: Period = serde_json::from_str(&json).unwrap();
        assert_eq!(back, case.expected);
    }
}

#[test]
fn period_alias_inputs_normalize_to_canonical() {
    let aliases = [
        ("2023 q4", "2023Q4"),
        ("2023-Q4", "2023Q4"),
        ("FY2023", "2023"),
        ("Fiscal 2023", "2023"),
        ("12/31/2023", "2023-12-31"),
        ("31-12-2023", "2023-12-31"),
    ];

    for (alias, canonical) in aliases {
        let parsed = alias.parse::<Period>().unwrap();
        assert_eq!(parsed.to_string(), canonical);
    }
}

#[test]
fn period_invalid_matches_raise_error() {
    for invalid in ["2023Q0", "2023Q5", "2023-13-01", "2023/02/30"] {
        let err = invalid.parse::<Period>().unwrap_err();
        match err {
            paft_core::error::PaftError::InvalidPeriodFormat { format } => {
                assert_eq!(format, invalid);
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}

#[test]
fn period_other_values_uppercase() {
    let parsed = Period::try_from("custom range".to_string()).unwrap();
    assert_eq!(parsed.to_string(), "CUSTOM_RANGE");
    assert_eq!(parsed.to_string(), "CUSTOM_RANGE");

    let json = serde_json::to_string(&parsed).unwrap();
    assert_eq!(json, "\"CUSTOM_RANGE\"");
    let round_trip: Period = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, parsed);
}

fn period_cases() -> Vec<PeriodCase> {
    vec![
        PeriodCase {
            input: "2023Q4",
            expected: Period::Quarter {
                year: 2023,
                quarter: 4,
            },
            canonical: "2023Q4",
        },
        PeriodCase {
            input: "2023",
            expected: Period::Year { year: 2023 },
            canonical: "2023",
        },
        PeriodCase {
            input: "2023-12-31",
            expected: Period::Date(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
            canonical: "2023-12-31",
        },
    ]
}
