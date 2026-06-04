use chrono::NaiveDate;
use paft_domain::{
    CalendarPeriod, DomainError, MarketState, PeriodDate, PeriodYear, QuarterOfYear,
    ReportingPeriod,
};
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
    expected: ReportingPeriod,
    canonical: &'static str,
}

fn assert_invalid_period_format(input: &str) {
    let err = input.parse::<ReportingPeriod>().unwrap_err();
    match err {
        DomainError::InvalidPeriodFormat { format } => {
            assert_eq!(format, input);
        }
        other => panic!("unexpected error variant for {input:?}: {other:?}"),
    }
}

fn assert_period_roundtrips(input: &str) -> ReportingPeriod {
    let p: ReportingPeriod = input.parse().unwrap();
    let display = p.to_string();
    let display_reparsed: ReportingPeriod = display.parse().unwrap();
    assert_eq!(
        p, display_reparsed,
        "display round-trip failed for {input:?} ({display:?})"
    );

    let json = serde_json::to_string(&p).unwrap();
    let reparsed: ReportingPeriod = serde_json::from_str(&json).unwrap();
    assert_eq!(p, reparsed, "serde round-trip failed for {input:?}");
    p
}

fn date_period(year: i32, month: u32, day: u32) -> ReportingPeriod {
    ReportingPeriod::date(NaiveDate::from_ymd_opt(year, month, day).unwrap()).unwrap()
}

fn calendar_date_period(year: i32, month: u32, day: u32) -> CalendarPeriod {
    CalendarPeriod::date(NaiveDate::from_ymd_opt(year, month, day).unwrap()).unwrap()
}

#[test]
fn period_round_trips_display_fromstr_serde() {
    for case in period_cases() {
        let parsed = case.input.parse::<ReportingPeriod>().unwrap();
        assert_eq!(parsed, case.expected);

        let display = parsed.to_string();
        assert_eq!(display, case.canonical);

        let reparsed = ReportingPeriod::from_str(&display).unwrap();
        assert_eq!(reparsed, case.expected);

        let json = serde_json::to_string(&parsed).unwrap();
        assert_eq!(json, format!("\"{}\"", case.canonical));
        let back: ReportingPeriod = serde_json::from_str(&json).unwrap();
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
            .parse::<ReportingPeriod>()
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
        let err = invalid.parse::<ReportingPeriod>().unwrap_err();
        match err {
            DomainError::InvalidPeriodFormat { format } => {
                assert_eq!(format, invalid);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}

#[test]
fn period_rejects_inputs_that_canonicalize_to_invalid_structured_tokens() {
    for input in ["-2023Q5", "(2023Q5)", "!2023Q5!", "2023Q5!"] {
        assert_invalid_period_format(input);
    }
}

#[test]
fn period_rejects_inputs_that_canonicalize_to_modeled_tokens() {
    let cases = [
        "-2023Q4", "+2023Q4", "(2023Q4)", "!2023Q4!", "(2023)", "-2023", "+2023", "1234-",
    ];

    for input in cases {
        assert_invalid_period_format(input);
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
            .parse::<ReportingPeriod>()
            .unwrap_or_else(|e| panic!("expected {input:?} to parse, got {e:?}"));
        assert_eq!(parsed.to_string(), canonical, "input: {input:?}");
    }

    // "12-34-5678" matches the day-first byte-parser shape but month=34 is
    // invalid, so it must surface InvalidPeriodFormat (not fall through to Other).
    match "12-34-5678".parse::<ReportingPeriod>() {
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
        "0000Q1".parse::<ReportingPeriod>().unwrap(),
        ReportingPeriod::quarterly(0, 1).unwrap()
    );
    assert_eq!(
        "9999Q4".parse::<ReportingPeriod>().unwrap(),
        ReportingPeriod::quarterly(9999, 4).unwrap()
    );
    assert_eq!(
        "0000".parse::<ReportingPeriod>().unwrap(),
        ReportingPeriod::annual(0).unwrap()
    );
    assert_eq!(
        "9999".parse::<ReportingPeriod>().unwrap(),
        ReportingPeriod::annual(9999).unwrap()
    );
}

#[test]
fn period_low_years_emit_four_digit_canonical_codes() {
    let cases = [
        ("0000", ReportingPeriod::annual(0).unwrap()),
        ("0001", ReportingPeriod::annual(1).unwrap()),
        ("0012", ReportingPeriod::annual(12).unwrap()),
        ("0123", ReportingPeriod::annual(123).unwrap()),
        ("0000Q1", ReportingPeriod::quarterly(0, 1).unwrap()),
        ("0012Q4", ReportingPeriod::quarterly(12, 4).unwrap()),
    ];

    for (canonical, period) in cases {
        assert_eq!(period.to_string(), canonical);

        let display_round_trip: ReportingPeriod = period.to_string().parse().unwrap();
        assert_eq!(display_round_trip, period);

        let json = serde_json::to_string(&period).unwrap();
        let serde_round_trip: ReportingPeriod = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_round_trip, period);
    }
}

#[test]
fn period_year_serde_emits_canonical_strings_and_validates() {
    let year = PeriodYear::new(2024).unwrap();
    let json = serde_json::to_string(&year).unwrap();
    assert_eq!(json, "\"2024\"");

    let round_trip: PeriodYear = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, year);

    let low_year = PeriodYear::new(7).unwrap();
    assert_eq!(serde_json::to_string(&low_year).unwrap(), "\"0007\"");
    assert_eq!(
        serde_json::from_str::<PeriodYear>("\"0007\"").unwrap(),
        low_year
    );

    assert_eq!(serde_json::from_str::<PeriodYear>("7").unwrap(), low_year);
    assert!(serde_json::from_str::<PeriodYear>("\"7\"").is_err());
    assert!(serde_json::from_str::<PeriodYear>("\"10000\"").is_err());
    assert!(serde_json::from_str::<PeriodYear>("10000").is_err());
    assert!(serde_json::from_str::<PeriodYear>("-1").is_err());
}

#[test]
fn period_byte_parser_iso_does_not_swallow_us_or_dayfirst() {
    // A 4-digit-leading + '/'/'-' prefix that fails ISO must NOT cascade to US
    // or day-first (those start with 1-2 digits). Verify by checking inputs
    // that begin with 4 digits and the ISO branch ultimately fails.
    //
    let cases = [
        "1234-X", // length 6, ISO partial then non-digit → Other
        "1234/X", // length 6, ISO partial then non-digit → Other
    ];
    for input in cases {
        let parsed = input
            .parse::<ReportingPeriod>()
            .unwrap_or_else(|e| panic!("expected {input:?} to parse to Other, got {e:?}"));
        match parsed {
            ReportingPeriod::Other(_) => {}
            other => panic!("expected Other(_) for {input:?}, got {other:?}"),
        }
    }
}

#[test]
fn period_byte_parser_quarter_separator_whitespace_variants() {
    // The byte parser accepts a single `-`, no separator, or a *run* of ASCII
    // whitespace bytes between the year and the `Q` (matching `parse_year`'s
    // "Fiscal " handling). These all parse to the same Quarter.
    let inputs = [
        "2023Q4",
        "2023 Q4",
        "2023\tQ4",
        "2023-Q4",
        "2023q4",
        "2023-q4",
        // Multi-whitespace runs (regression: pre-3bb732b regex used `\s*`,
        // the byte scanner had regressed to a single whitespace byte).
        "2023  Q4",
        "2023 \t \t Q4",
        "2023\t\tq4",
    ];
    let expected = ReportingPeriod::quarterly(2023, 4).unwrap();
    for input in inputs {
        assert_eq!(
            input.parse::<ReportingPeriod>().unwrap(),
            expected,
            "input: {input:?}"
        );
    }
}

#[test]
fn period_other_round_trip_for_genuine_other_inputs() {
    // Inputs whose canonical form is *not* recognized by any structured
    // parser. They must parse to `Other` and round-trip cleanly via
    // `Display`/serde.
    //
    // `"2023-01-01extra"` is decided here as Other-with-no-collision: ISO
    // date parsing rejects it (length 15 > 10), and after canonicalization
    // (`"2023_01_01EXTRA"`) no structured parser accepts it either, so we
    // store it as Other and the canonical token round-trips.
    let inputs = [
        "2023-01-01extra",
        "ALPHA",
        "FY 2023",
        "2023-12/31",
        "x2023Q5",
    ];

    for input in inputs {
        let p = assert_period_roundtrips(input);
        assert!(
            matches!(p, ReportingPeriod::Other(_)),
            "expected Other for {input:?}, got {p:?}"
        );
    }
}

#[test]
fn period_partial_modeled_looking_provider_labels_remain_other() {
    let cases = [("2023-Q", "2023_Q"), ("FY", "FY")];

    for (input, canonical) in cases {
        let p = assert_period_roundtrips(input);
        assert_eq!(p.to_string(), canonical, "input: {input:?}");
        assert!(
            matches!(p, ReportingPeriod::Other(_)),
            "expected Other for {input:?}, got {p:?}"
        );
    }
}

#[test]
fn period_byte_parser_too_long_inputs_fall_through() {
    // Length > 10 cannot match any date format. They must fall through to Other.
    let parsed = "2023-12-31x".parse::<ReportingPeriod>().unwrap();
    assert!(matches!(parsed, ReportingPeriod::Other(_)));
}

#[test]
fn period_calendar_boundaries_are_explicit() {
    let date = calendar_date_period(2023, 5, 17);
    assert_eq!(
        date.start_date(),
        NaiveDate::from_ymd_opt(2023, 5, 17).unwrap()
    );
    assert_eq!(
        date.end_date(),
        NaiveDate::from_ymd_opt(2023, 5, 17).unwrap()
    );

    let quarter = CalendarPeriod::quarterly(2023, 2).unwrap();
    assert_eq!(
        quarter.start_date(),
        NaiveDate::from_ymd_opt(2023, 4, 1).unwrap()
    );
    assert_eq!(
        quarter.end_date(),
        NaiveDate::from_ymd_opt(2023, 6, 30).unwrap()
    );

    let year = CalendarPeriod::annual(2023).unwrap();
    assert_eq!(
        year.start_date(),
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
    );
    assert_eq!(
        year.end_date(),
        NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()
    );
}

#[test]
fn calendar_period_rejects_reporting_only_labels() {
    assert!("FY2023".parse::<CalendarPeriod>().is_err());
    assert!("custom range".parse::<CalendarPeriod>().is_err());

    let reporting = ReportingPeriod::other("custom range").unwrap();
    assert!(CalendarPeriod::try_from(reporting).is_err());

    let reporting = ReportingPeriod::quarterly(2023, 4).unwrap();
    assert_eq!(
        CalendarPeriod::try_from(reporting).unwrap(),
        CalendarPeriod::quarterly(2023, 4).unwrap()
    );
}

#[test]
fn period_boundaries_support_chronological_sort_keys() {
    let late_date = calendar_date_period(2099, 1, 1);
    let early_quarter = CalendarPeriod::quarterly(1900, 1).unwrap();

    assert!(early_quarter.start_date() < late_date.start_date());
    assert!(early_quarter.end_date() < late_date.end_date());
}

#[test]
fn period_constructors_reject_invalid_structured_components() {
    assert_eq!(
        ReportingPeriod::quarterly(2023, 5).unwrap_err(),
        DomainError::InvalidPeriodQuarter { quarter: 5 }
    );
    assert_eq!(
        QuarterOfYear::new(0).unwrap_err(),
        DomainError::InvalidPeriodQuarter { quarter: 0 }
    );
    assert_eq!(
        ReportingPeriod::annual(-1).unwrap_err(),
        DomainError::InvalidPeriodYear { year: -1 }
    );
    assert_eq!(
        ReportingPeriod::annual(10_000).unwrap_err(),
        DomainError::InvalidPeriodYear { year: 10_000 }
    );
    assert_eq!(
        PeriodYear::new(10_000).unwrap_err(),
        DomainError::InvalidPeriodYear { year: 10_000 }
    );
    assert_eq!(
        PeriodDate::new(NaiveDate::from_ymd_opt(10_000, 1, 1).unwrap()).unwrap_err(),
        DomainError::InvalidPeriodYear { year: 10_000 }
    );
    assert_eq!(
        ReportingPeriod::date(NaiveDate::from_ymd_opt(10_000, 1, 1).unwrap()).unwrap_err(),
        DomainError::InvalidPeriodYear { year: 10_000 }
    );
}

#[test]
fn period_helper_next_quarter() {
    // Date -> next quarter of its quarter bucket
    let d = calendar_date_period(2023, 3, 31);
    assert_eq!(
        d.next_quarter(),
        Some(CalendarPeriod::quarterly(2023, 2).unwrap())
    );

    // Quarter wrap
    let q = CalendarPeriod::quarterly(2023, 4).unwrap();
    assert_eq!(
        q.next_quarter(),
        Some(CalendarPeriod::quarterly(2024, 1).unwrap())
    );

    // Year -> Q1 of next year
    let y = CalendarPeriod::annual(2023).unwrap();
    assert_eq!(
        y.next_quarter(),
        Some(CalendarPeriod::quarterly(2024, 1).unwrap())
    );

    assert_eq!(
        CalendarPeriod::quarterly(9999, 4).unwrap().next_quarter(),
        None
    );
    assert_eq!(CalendarPeriod::annual(9999).unwrap().next_quarter(), None);
}

#[test]
fn period_helper_year_end() {
    let d = calendar_date_period(2023, 6, 15);
    let q = CalendarPeriod::quarterly(2023, 3).unwrap();
    let y = CalendarPeriod::annual(2023).unwrap();

    assert_eq!(d.year_end(), NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
    assert_eq!(q.year_end(), NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
    assert_eq!(y.year_end(), NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
}

#[test]
fn calendar_period_relationship_helpers() {
    let year = CalendarPeriod::annual(2023).unwrap();
    let same_year = CalendarPeriod::annual(2023).unwrap();
    let next_year = CalendarPeriod::annual(2024).unwrap();
    let q1 = CalendarPeriod::quarterly(2023, 1).unwrap();
    let q2 = CalendarPeriod::quarterly(2023, 2).unwrap();
    let q4 = CalendarPeriod::quarterly(2023, 4).unwrap();
    let jan_1 = calendar_date_period(2023, 1, 1);
    let mar_31 = calendar_date_period(2023, 3, 31);
    let apr_1 = calendar_date_period(2023, 4, 1);

    assert!(year.overlaps(&q1));
    assert!(q1.overlaps(&year));
    assert!(year.overlaps(&apr_1));
    assert!(q1.overlaps(&mar_31));
    assert!(!q1.overlaps(&q2));
    assert!(!q1.overlaps(&apr_1));
    assert!(!year.overlaps(&next_year));

    assert!(year.contains(&q4));
    assert!(year.contains(&jan_1));
    assert!(q1.contains(&mar_31));
    assert!(jan_1.contains(&jan_1));
    assert!(!q1.contains(&year));
    assert!(!jan_1.contains(&q1));

    assert!(year.is_same_exact_bucket_as(&same_year));
    assert!(jan_1.is_same_exact_bucket_as(&calendar_date_period(2023, 1, 1)));
    assert!(!year.is_same_exact_bucket_as(&q1));
    assert!(!year.is_same_exact_bucket_as(&jan_1));
    assert!(!q1.is_same_exact_bucket_as(&jan_1));
}

#[test]
fn period_other_values_uppercase() {
    let parsed = ReportingPeriod::try_from("custom range".to_string()).unwrap();
    assert_eq!(parsed.to_string(), "CUSTOM_RANGE");
    assert_eq!(parsed.to_string(), "CUSTOM_RANGE");

    let json = serde_json::to_string(&parsed).unwrap();
    assert_eq!(json, "\"CUSTOM_RANGE\"");
    let round_trip: ReportingPeriod = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, parsed);
}

fn period_cases() -> Vec<PeriodCase> {
    vec![
        PeriodCase {
            input: "2023Q4",
            expected: ReportingPeriod::quarterly(2023, 4).unwrap(),
            canonical: "2023Q4",
        },
        PeriodCase {
            input: "2023",
            expected: ReportingPeriod::annual(2023).unwrap(),
            canonical: "2023",
        },
        PeriodCase {
            input: "2023-12-31",
            expected: date_period(2023, 12, 31),
            canonical: "2023-12-31",
        },
    ]
}
