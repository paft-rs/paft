use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Required {
    #[serde(with = "paft_core::serde_helpers::ts_milliseconds")]
    ts: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Optional {
    #[serde(default, with = "paft_core::serde_helpers::ts_milliseconds_option")]
    ts: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Vector {
    #[serde(with = "paft_core::serde_helpers::ts_milliseconds_vec")]
    ts: Vec<DateTime<Utc>>,
}

#[test]
fn exact_milliseconds_round_trip_in_all_shapes() {
    for millis in [-1_001, -1_000, -999, -1, 0, 1, 999, 1_000, 1_001] {
        let ts = DateTime::from_timestamp_millis(millis).unwrap();
        let required = Required { ts };
        let json = serde_json::to_string(&required).unwrap();
        assert_eq!(json, format!(r#"{{"ts":{millis}}}"#));
        assert_eq!(serde_json::from_str::<Required>(&json).unwrap(), required);
        let optional = Optional { ts: Some(ts) };
        assert_eq!(serde_json::from_str::<Optional>(&json).unwrap(), optional);
        assert_eq!(serde_json::to_string(&optional).unwrap(), json);
        let vector = Vector { ts: vec![ts, ts] };
        let json = serde_json::to_string(&vector).unwrap();
        assert_eq!(json, format!(r#"{{"ts":[{millis},{millis}]}}"#));
        assert_eq!(serde_json::from_str::<Vector>(&json).unwrap(), vector);
    }
    assert_eq!(serde_json::from_str::<Optional>("{}").unwrap().ts, None);
    assert_eq!(
        serde_json::from_str::<Optional>(r#"{"ts":null}"#)
            .unwrap()
            .ts,
        None
    );
    assert_eq!(
        serde_json::to_string(&Optional { ts: None }).unwrap(),
        r#"{"ts":null}"#
    );
    assert_eq!(
        serde_json::to_string(&Vector { ts: vec![] }).unwrap(),
        r#"{"ts":[]}"#
    );
}

#[test]
fn unsupported_precision_and_leap_seconds_are_rejected_in_all_shapes() {
    for (secs, nanos) in [
        (0, 1),
        (0, 1_000_001),
        (0, 999_999_999),
        (-1, 1),
        (-1, 999_000_001),
        (-1, 999_999_999),
        (59, 1_000_000_000),
        (59, 1_001_000_000),
        (-1, 1_000_000_000),
    ] {
        let ts = DateTime::from_timestamp(secs, nanos).unwrap();
        for result in [
            serde_json::to_string(&Required { ts }),
            serde_json::to_string(&Optional { ts: Some(ts) }),
            serde_json::to_string(&Vector {
                ts: vec![DateTime::UNIX_EPOCH, ts],
            }),
        ] {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("cannot be preserved")
            );
        }
    }
}

#[test]
fn out_of_range_wire_timestamps_are_rejected() {
    assert!(serde_json::from_str::<Required>(r#"{"ts":9223372036854775807}"#).is_err());
    assert!(serde_json::from_str::<Optional>(r#"{"ts":9223372036854775807}"#).is_err());
    assert!(serde_json::from_str::<Vector>(r#"{"ts":[9223372036854775807]}"#).is_err());
}
