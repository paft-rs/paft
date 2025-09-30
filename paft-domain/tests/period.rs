use chrono::NaiveDate;
use paft_domain::{DomainError, MarketState, Period};
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
            DomainError::InvalidPeriodFormat { format } => {
                assert_eq!(format, invalid);
            }
            DomainError::InvalidExchangeValue { .. } => unreachable!(),
            DomainError::InvalidIsin { .. } => unreachable!(),
        }
    }
}

#[test]
fn period_ordering_is_stable_and_chronological() {
    use std::cmp::Ordering::*;
    let d1 = Period::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
    let d2 = Period::Date(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
    let q1 = Period::Quarter {
        year: 2023,
        quarter: 1,
    };
    let q2 = Period::Quarter {
        year: 2023,
        quarter: 2,
    };
    let y = Period::Year { year: 2023 };
    let o = Period::try_from("ALPHA".to_string()).unwrap();

    // Variant precedence: Date < Quarter < Year < Other
    assert_eq!(d1.cmp(&q1), Less);
    assert_eq!(q1.cmp(&y), Less);
    assert_eq!(y.cmp(&o), Less);

    // Intra-variant chronology
    assert_eq!(d1.cmp(&d2), Less);
    assert_eq!(q1.cmp(&q2), Less);
    assert_eq!(Period::Year { year: 2022 }.cmp(&y), Less);
}

#[test]
fn period_helper_next_quarter() {
    // Date -> next quarter of its quarter bucket
    let d = Period::Date(NaiveDate::from_ymd_opt(2023, 3, 31).unwrap());
    assert_eq!(
        d.next_quarter(),
        Some(Period::Quarter {
            year: 2023,
            quarter: 2
        })
    );

    // Quarter wrap
    let q = Period::Quarter {
        year: 2023,
        quarter: 4,
    };
    assert_eq!(
        q.next_quarter(),
        Some(Period::Quarter {
            year: 2024,
            quarter: 1
        })
    );

    // Year -> Q1 of next year
    let y = Period::Year { year: 2023 };
    assert_eq!(
        y.next_quarter(),
        Some(Period::Quarter {
            year: 2024,
            quarter: 1
        })
    );
}

#[test]
fn period_helper_year_end() {
    let d = Period::Date(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap());
    let q = Period::Quarter {
        year: 2023,
        quarter: 3,
    };
    let y = Period::Year { year: 2023 };

    assert_eq!(
        d.year_end(),
        Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap())
    );
    assert_eq!(
        q.year_end(),
        Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap())
    );
    assert_eq!(
        y.year_end(),
        Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap())
    );
}

#[test]
fn period_helper_is_same_bucket_as() {
    // Year bucket
    let y = Period::Year { year: 2023 };
    let d = Period::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
    let q = Period::Quarter {
        year: 2023,
        quarter: 2,
    };
    assert!(y.is_same_bucket_as(&Period::Year { year: 2023 }));
    assert!(y.is_same_bucket_as(&d));
    assert!(y.is_same_bucket_as(&q));
    assert!(!y.is_same_bucket_as(&Period::Year { year: 2022 }));

    // Quarter bucket
    let d_q2 = Period::Date(NaiveDate::from_ymd_opt(2023, 4, 1).unwrap());
    let q2 = Period::Quarter {
        year: 2023,
        quarter: 2,
    };
    assert!(q2.is_same_bucket_as(&d_q2));
    assert!(q2.is_same_bucket_as(&Period::Quarter {
        year: 2023,
        quarter: 2
    }));
    assert!(!q2.is_same_bucket_as(&Period::Quarter {
        year: 2023,
        quarter: 3
    }));

    // Date exact
    let d1 = Period::Date(NaiveDate::from_ymd_opt(2023, 7, 4).unwrap());
    let d2 = Period::Date(NaiveDate::from_ymd_opt(2023, 7, 4).unwrap());
    let d3 = Period::Date(NaiveDate::from_ymd_opt(2023, 7, 5).unwrap());
    assert!(d1.is_same_bucket_as(&d2));
    assert!(!d1.is_same_bucket_as(&d3));
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
