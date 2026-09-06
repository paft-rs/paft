use chrono::{DateTime, SecondsFormat, Utc};
use paft_core::serde_helpers::{TimestampErrorKind, parse_timestamp, timestamp_nanos_exact};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Required {
    #[serde(with = "paft_core::serde_helpers::ts_iso8601")]
    ts: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Optional {
    #[serde(default, with = "paft_core::serde_helpers::ts_iso8601_option")]
    ts: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct List {
    #[serde(with = "paft_core::serde_helpers::ts_iso8601_vec")]
    ts: Vec<DateTime<Utc>>,
}

fn canonical(input: &str) -> String {
    let value: Required = serde_json::from_value(json!({"ts":input})).unwrap();
    serde_json::to_value(value).unwrap()["ts"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn offset_and_fractional_canonicalization_preserve_the_instant() {
    for input in [
        "2026-09-06T20:00:00Z",
        "2026-09-06T22:00:00+02:00",
        "2026-09-06T14:30:00-05:30",
        "2026-09-06t20:00:00z",
    ] {
        assert_eq!(canonical(input), "2026-09-06T20:00:00Z");
    }
    assert_eq!(
        canonical("2026-01-01T12:00:00.123456-05:30"),
        "2026-01-01T17:30:00.123456Z"
    );
    for (input, output) in [
        ("", ""),
        (".0", ""),
        (".1", ".100"),
        (".12", ".120"),
        (".123", ".123"),
        (".1234", ".123400"),
        (".123400", ".123400"),
        (".123400500", ".123400500"),
        (".123400001", ".123400001"),
    ] {
        assert_eq!(
            canonical(&format!("2026-01-01T12:00:00{input}Z")),
            format!("2026-01-01T12:00:00{output}Z")
        );
    }
}

#[test]
fn lexical_policy_precedes_chronos_relaxed_parser() {
    for input in [
        "",
        "+",
        "2026-1-01T00:00:00Z",
        "2026-01-1T00:00:00Z",
        "2026-01-01T0:00:00Z",
        "2026-01-01 00:00:00Z",
        " 2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z ",
        "2026-01-01T00:00:00UTC",
        "2026-01-01T00:00:00+0200",
        "2026-01-01T00:00:00.Z",
        "2026-01-01T00:00:00",
        "+9999-01-01T00:00:00Z",
        "+010000-01-01T00:00:00Z",
        "10000-01-01T00:00:00Z",
        "-0000-01-01T00:00:00Z",
        "-00001-01-01T00:00:00Z",
        "-001-01-01T00:00:00Z",
        "２０２６-01-01T00:00:00Z",
    ] {
        assert_eq!(
            parse_timestamp(input).unwrap_err().kind,
            TimestampErrorKind::InvalidSyntax,
            "{input}"
        );
    }
    for fraction in ["1234567891", "1234567890", "0000000000"] {
        assert_eq!(
            parse_timestamp(&format!("2026-01-01T00:00:00.{fraction}Z"))
                .unwrap_err()
                .kind,
            TimestampErrorKind::FractionalPrecision
        );
    }
    for input in [
        "2026-02-29T00:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T00:00:00+25:00",
        "+999999-01-01T00:00:00Z",
    ] {
        assert!(
            matches!(
                parse_timestamp(input).unwrap_err().kind,
                TimestampErrorKind::InvalidDateTime(_)
            ),
            "{input}"
        );
    }
}

#[test]
fn strings_only_across_required_optional_and_list_adapters() {
    for number in [
        json!(0),
        json!(-1),
        json!(1.5),
        json!(1_640_995_200_123_i64),
    ] {
        assert!(serde_json::from_value::<Required>(json!({"ts":number})).is_err());
        assert!(serde_json::from_value::<Optional>(json!({"ts":number})).is_err());
        assert!(serde_json::from_value::<List>(json!({"ts":[number]})).is_err());
    }
    for wire in [json!({}), json!({"ts":null})] {
        assert_eq!(
            serde_json::from_value::<Optional>(wire).unwrap(),
            Optional { ts: None }
        );
    }
    for dates in [
        vec![],
        vec![
            DateTime::UNIX_EPOCH,
            DateTime::from_timestamp_nanos(-1),
            DateTime::from_timestamp_nanos(1),
        ],
    ] {
        let value = List { ts: dates };
        let wire = serde_json::to_value(&value).unwrap();
        assert_eq!(serde_json::from_value::<List>(wire).unwrap(), value);
    }
    let value = Optional {
        ts: Some(DateTime::from_timestamp_nanos(1)),
    };
    assert_eq!(
        serde_json::from_value::<Optional>(serde_json::to_value(&value).unwrap()).unwrap(),
        value
    );
}

#[test]
fn json_range_and_checked_nanosecond_range_are_independent() {
    for ts in [DateTime::<Utc>::MIN_UTC, DateTime::<Utc>::MAX_UTC] {
        let text = ts.to_rfc3339_opts(SecondsFormat::AutoSi, true);
        assert_eq!(parse_timestamp(&text).unwrap(), ts);
        assert_eq!(canonical(&text), text);
        assert_eq!(
            timestamp_nanos_exact(&ts).unwrap_err().kind,
            TimestampErrorKind::OutOfDataFrameRange
        );
    }
    for input in [
        "0000-01-01T00:00:00Z",
        "-0001-01-01T00:00:00Z",
        "-10000-01-01T00:00:00Z",
        "+12020-01-01T00:00:00Z",
        "+120200-01-01T00:00:00Z",
    ] {
        assert_eq!(canonical(input), input);
    }
    for (ts, overflow_offset, safe_offset) in [
        (DateTime::<Utc>::MIN_UTC, "+00:01", "-00:01"),
        (DateTime::<Utc>::MAX_UTC, "-00:01", "+00:01"),
    ] {
        let text = ts.to_rfc3339_opts(SecondsFormat::AutoSi, true);
        assert!(
            parse_timestamp(&format!("{}{overflow_offset}", text.trim_end_matches('Z'))).is_err()
        );
        assert!(parse_timestamp(&format!("{}{safe_offset}", text.trim_end_matches('Z'))).is_ok());
    }
    for nanos in [i64::MIN, -1, 0, 1, i64::MAX] {
        let ts = DateTime::from_timestamp_nanos(nanos);
        assert_eq!(timestamp_nanos_exact(&ts).unwrap(), nanos);
        assert_eq!(parse_timestamp(&canonical(&ts.to_rfc3339())).unwrap(), ts);
    }
}

#[test]
fn leap_seconds_are_rejected_on_both_boundaries_with_context() {
    for input in ["2016-12-31T23:59:60Z", "2017-01-01T01:59:60.1+02:00"] {
        let error = parse_timestamp(input).unwrap_err();
        assert_eq!(error.kind, TimestampErrorKind::LeapSecond);
        assert_eq!(error.timestamp, input);
    }
    let leap = DateTime::from_timestamp(59, 1_001_000_000).unwrap();
    assert_eq!(
        timestamp_nanos_exact(&leap).unwrap_err().kind,
        TimestampErrorKind::LeapSecond
    );
    assert!(serde_json::to_value(Required { ts: leap }).is_err());
    assert!(serde_json::to_value(Optional { ts: Some(leap) }).is_err());
    assert!(
        serde_json::to_value(List {
            ts: vec![DateTime::UNIX_EPOCH, leap]
        })
        .unwrap_err()
        .to_string()
        .contains("timestamps[1]")
    );
}
