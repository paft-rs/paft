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
        ("2023\tq4", "2023Q4"),
        ("2023q4", "2023Q4"),
        ("FY2023", "2023"),
        ("fy2023", "2023"),
        ("Fy2023", "2023"),
        ("Fiscal 2023", "2023"),
        ("FISCAL 2023", "2023"),
        ("fiscal\t2023", "2023"),
        ("Fiscal  2023", "2023"),
        ("12/31/2023", "2023-12-31"),
        ("1/1/2023", "2023-01-01"),
        ("31-12-2023", "2023-12-31"),
        ("1-1-2023", "2023-01-01"),
        ("2023/12/31", "2023-12-31"),
        ("2023-1-1", "2023-01-01"),
        ("2023/1/1", "2023-01-01"),
    ];

    for (alias, canonical) in aliases {
        let parsed = alias
            .parse::<Period>()
            .unwrap_or_else(|e| panic!("expected {alias:?} to parse, got {e:?}"));
        assert_eq!(parsed.to_string(), canonical, "input: {alias:?}");
    }
}

#[test]
fn period_invalid_matches_raise_error() {
    let invalids = [
        // quarter out of [1,4]
        "2023Q0",
        "2023Q5",
        "2023Q12",
        "2023Q99",
        "2023Q12345", // overflow path: quarter > 9 inside the loop
        "2023-Q0",
        "2023 Q5",
        // ISO date with invalid month/day
        "2023-13-01",
        "2023/02/30",
        "2023-00-01",
        "2023-01-00",
        "2023-1-32",
        "2023/2/29", // 2023 is not a leap year
        // US date with invalid month/day
        "13/01/2023",
        "12/32/2023",
        "00/01/2023",
        "01/00/2023",
        // day-first date with invalid month/day
        "32-12-2023",
        "01-13-2023",
        "30-02-2023",
        "00-01-2023",
        "01-00-2023",
    ];
    for invalid in invalids {
        let err = invalid.parse::<Period>().unwrap_err();
        match err {
            DomainError::InvalidPeriodFormat { format } => {
                assert_eq!(format, invalid);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}

#[test]
fn period_byte_parser_falls_through_to_other() {
    // Inputs that look quarterly/year/date-ish but fail every structural check.
    // Each must canonicalize to Other(...) instead of erroring.
    let cases = [
        ("ALPHA", "ALPHA"),
        ("custom range", "CUSTOM_RANGE"),
        // Length too short for any structured format.
        ("2023Q", "2023Q"),
        ("12345", "12345"),
        // Year-ish but not 4 digits / not FY / not Fiscal+ws.
        ("FISCAL2023", "FISCAL2023"),
        ("FY 2023", "FY_2023"), // FY without joined digits has no regex/byte match
        ("Fiscal X 2023", "FISCAL_X_2023"),
        ("FYY2023", "FYY2023"),
        // Date-ish but wrong shape.
        ("2023-12/31", "2023_12_31"), // mixed ISO separators
        ("12-34-5678", "12_34_5678"), // day-first shape but month 34 → InvalidPeriodFormat? see below
        ("2023QQ4", "2023QQ4"),       // double Q
    ];
    // Most of the above produce Other(...), except "12-34-5678" which structurally
    // matches day-first but has invalid month — that one is checked below.
    for (input, canonical) in cases {
        if input == "12-34-5678" {
            continue;
        }
        let parsed = input
            .parse::<Period>()
            .unwrap_or_else(|e| panic!("expected {input:?} to parse, got {e:?}"));
        assert_eq!(parsed.to_string(), canonical, "input: {input:?}");
    }

    // "12-34-5678" matches the day-first byte-parser shape but month=34 is
    // invalid, so it must surface InvalidPeriodFormat (not fall through to Other).
    match "12-34-5678".parse::<Period>() {
        Err(DomainError::InvalidPeriodFormat { format }) => {
            assert_eq!(format, "12-34-5678");
        }
        other => panic!("expected InvalidPeriodFormat for 12-34-5678, got {other:?}"),
    }
}

#[test]
fn period_byte_parser_year_boundaries() {
    // Year 0 and year 9999 are accepted by the byte parser; chrono accepts them
    // too. Quarter boundaries 1 and 4 are inclusive.
    assert_eq!(
        "0000Q1".parse::<Period>().unwrap(),
        Period::Quarter {
            year: 0,
            quarter: 1
        }
    );
    assert_eq!(
        "9999Q4".parse::<Period>().unwrap(),
        Period::Quarter {
            year: 9999,
            quarter: 4
        }
    );
    assert_eq!("0000".parse::<Period>().unwrap(), Period::Year { year: 0 });
    assert_eq!(
        "9999".parse::<Period>().unwrap(),
        Period::Year { year: 9999 }
    );
}

#[test]
fn period_byte_parser_iso_does_not_swallow_us_or_dayfirst() {
    // A 4-digit-leading + '/'/'-' prefix that fails ISO must NOT cascade to US
    // or day-first (those start with 1-2 digits). Verify by checking inputs
    // that begin with 4 digits and the ISO branch ultimately fails.
    let cases = [
        "1234-",  // length 5, too short for any format → Other
        "1234-X", // length 6, ISO partial then non-digit → Other
        "1234/X", // length 6, ISO partial then non-digit → Other
    ];
    for input in cases {
        let parsed = input
            .parse::<Period>()
            .unwrap_or_else(|e| panic!("expected {input:?} to parse to Other, got {e:?}"));
        match parsed {
            Period::Other(_) => {}
            other => panic!("expected Other(_) for {input:?}, got {other:?}"),
        }
    }
}

#[test]
fn period_byte_parser_quarter_separator_whitespace_variants() {
    // The byte parser accepts ASCII whitespace as the optional separator
    // (space and tab in particular). These all parse to the same Quarter.
    let inputs = [
        "2023Q4", "2023 Q4", "2023\tQ4", "2023-Q4", "2023q4", "2023-q4",
    ];
    let expected = Period::Quarter {
        year: 2023,
        quarter: 4,
    };
    for input in inputs {
        assert_eq!(
            input.parse::<Period>().unwrap(),
            expected,
            "input: {input:?}"
        );
    }
}

#[test]
fn period_byte_parser_too_long_inputs_fall_through() {
    // Length > 10 cannot match any date format. They must fall through to Other.
    let parsed = "2023-12-31x".parse::<Period>().unwrap();
    assert!(matches!(parsed, Period::Other(_)));
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
