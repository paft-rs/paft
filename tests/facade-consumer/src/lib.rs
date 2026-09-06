//! This fixture deliberately depends on no PAFT crate other than the facade.

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use paft::core::serde_helpers::{TimestampErrorKind, parse_timestamp, timestamp_nanos_exact};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Payload {
        #[serde(with = "paft::core::serde_helpers::ts_iso8601")]
        ts: DateTime<Utc>,
        #[serde(default, with = "paft::core::serde_helpers::ts_iso8601_option")]
        optional: Option<DateTime<Utc>>,
        #[serde(with = "paft::core::serde_helpers::ts_iso8601_vec")]
        list: Vec<DateTime<Utc>>,
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Legacy {
        #[serde(with = "paft::core::serde_helpers::ts_milliseconds")]
        ts: DateTime<Utc>,
        #[serde(default, with = "paft::core::serde_helpers::ts_milliseconds_option")]
        optional: Option<DateTime<Utc>>,
        #[serde(with = "paft::core::serde_helpers::ts_milliseconds_vec")]
        list: Vec<DateTime<Utc>>,
    }

    #[test]
    fn exact_timestamp_ingestion_and_explicit_legacy_adapters_are_reachable() {
        let ts = parse_timestamp("2022-10-19T23:28:22.061769Z").unwrap();
        assert_eq!(
            timestamp_nanos_exact(&ts).unwrap(),
            1_666_222_102_061_769_000
        );
        let value = Payload {
            ts,
            optional: Some(ts),
            list: vec![ts],
        };
        let wire = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<Payload>(&wire).unwrap(), value);
        assert_eq!(
            parse_timestamp("2026-01-01T00:00:00.1234567890Z")
                .unwrap_err()
                .kind,
            TimestampErrorKind::FractionalPrecision
        );
        let legacy: Legacy = serde_json::from_str(r#"{"ts":-1,"optional":0,"list":[1]}"#).unwrap();
        assert_eq!(legacy.ts, DateTime::from_timestamp_millis(-1).unwrap());
        assert_eq!(
            serde_json::from_str::<Legacy>(&serde_json::to_string(&legacy).unwrap()).unwrap(),
            legacy
        );
        assert!(serde_json::from_str::<Payload>(r#"{"ts":-1,"list":[]}"#).is_err());
    }
}
