use chrono::DateTime;
use paft_domain::AssetKind;
use paft_market::MarketError;
use paft_market::requests::{HistoryRequest, Interval, Range, SearchRequest};
use std::num::NonZeroU32;

#[test]
fn search_request_validation_empty_query_rejected() {
    let result = SearchRequest::new("");
    assert!(result.is_err());

    if result == Err(MarketError::EmptySearchQuery) {
        // expected
    } else {
        panic!("Expected EmptySearchQuery error for empty query");
    }
}

#[test]
fn search_request_validation_whitespace_only_query_rejected() {
    let result = SearchRequest::new("   \t\n  ");
    assert!(result.is_err());

    if result == Err(MarketError::EmptySearchQuery) {
        // expected
    } else {
        panic!("Expected EmptySearchQuery error for whitespace-only query");
    }
}

#[test]
fn search_request_validation_trims_query_before_storing() {
    let request = SearchRequest::new(" AAPL ").unwrap();

    assert_eq!(request.query(), "AAPL");
}

#[test]
fn search_request_validation_trims_locale_fields_before_storing() {
    let request = SearchRequest::builder("AAPL")
        .lang(" en ")
        .region(" US ")
        .build()
        .unwrap();

    assert_eq!(request.lang(), Some("en"));
    assert_eq!(request.region(), Some("US"));
}

#[test]
fn search_request_validation_zero_limit_rejected() {
    let result = SearchRequest::builder("AAPL").limit(0).build();
    assert!(result.is_err());

    if let Err(MarketError::InvalidSearchLimit(limit)) = result {
        assert_eq!(limit, 0);
    } else {
        panic!("Expected InvalidSearchLimit error for zero limit");
    }
}

#[test]
fn search_request_validation_empty_lang_rejected() {
    let result = SearchRequest::builder("AAPL").lang(" \t\n ").build();

    assert_eq!(
        result,
        Err(MarketError::EmptySearchLocaleField { field: "lang" })
    );
}

#[test]
fn search_request_validation_empty_region_rejected() {
    let result = SearchRequest::builder("AAPL").region("").build();

    assert_eq!(
        result,
        Err(MarketError::EmptySearchLocaleField { field: "region" })
    );
}

#[test]
fn search_request_deserialization_empty_query_rejected() {
    let result = serde_json::from_str::<SearchRequest>(
        r#"{"query":"","kind":null,"limit":10,"lang":null,"region":null}"#,
    );

    assert!(result.is_err());
}

#[test]
fn search_request_deserialization_zero_limit_rejected() {
    let result = serde_json::from_str::<SearchRequest>(
        r#"{"query":"AAPL","kind":null,"limit":0,"lang":null,"region":null}"#,
    );

    assert!(result.is_err());
}

#[test]
fn search_request_deserialization_empty_lang_rejected() {
    let result = serde_json::from_str::<SearchRequest>(
        r#"{"query":"AAPL","kind":null,"limit":10,"lang":" ","region":null}"#,
    );

    assert!(result.is_err());
}

#[test]
fn search_request_deserialization_empty_region_rejected() {
    let result = serde_json::from_str::<SearchRequest>(
        r#"{"query":"AAPL","kind":null,"limit":10,"lang":null,"region":""}"#,
    );

    assert!(result.is_err());
}

#[test]
fn search_request_deserialization_unknown_field_rejected() {
    let result = serde_json::from_str::<SearchRequest>(
        r#"{"query":"AAPL","kind":null,"limit":10,"lang":null,"region":null,"limti":5}"#,
    );

    assert!(result.is_err());
}

#[test]
fn search_request_validation_valid_request_passes() {
    let result = SearchRequest::builder("AAPL")
        .kind(AssetKind::Equity)
        .limit(10)
        .build();
    assert!(result.is_ok());
}

#[test]
fn search_request_validation_no_limit_passes() {
    let result = SearchRequest::new("AAPL");
    assert!(result.is_ok());
}

#[test]
fn search_request_validation_positive_limit_passes() {
    let result = SearchRequest::builder("AAPL").limit(1).build();
    assert!(result.is_ok());
}

#[test]
fn search_request_validation_accepts_u32_limit_boundary() {
    let request = SearchRequest::builder("AAPL")
        .limit(u32::MAX)
        .build()
        .unwrap();

    assert_eq!(request.limit(), NonZeroU32::new(u32::MAX));
}

#[test]
fn history_request_validation_range_and_period_mutually_exclusive() {
    let result = HistoryRequest::builder()
        .range(Range::D1)
        .period(
            DateTime::from_timestamp(1000, 0).unwrap(),
            DateTime::from_timestamp(2000, 0).unwrap(),
        )
        .build();

    assert!(result.is_ok());
    let req = result.unwrap();
    assert_eq!(req.range(), None);
    let (s, e) = req.period().unwrap();
    assert_eq!(s.timestamp(), 1000);
    assert_eq!(e.timestamp(), 2000);
}

#[test]
fn history_request_validation_period_start_ge_end_rejected() {
    let result = HistoryRequest::builder()
        .period(
            DateTime::from_timestamp(2000, 0).unwrap(),
            DateTime::from_timestamp(1000, 0).unwrap(),
        )
        .build();
    assert!(result.is_err());

    if let Err(MarketError::InvalidPeriod { start, end }) = result {
        assert_eq!(start, 2_000_000);
        assert_eq!(end, 1_000_000);
    } else {
        panic!("Expected InvalidPeriod error for invalid period");
    }
}

#[test]
fn history_request_validation_period_start_eq_end_rejected() {
    let result = HistoryRequest::builder()
        .period(
            DateTime::from_timestamp(1000, 0).unwrap(),
            DateTime::from_timestamp(1000, 0).unwrap(),
        )
        .build();
    assert!(result.is_err());

    if let Err(MarketError::InvalidPeriod { start, end }) = result {
        assert_eq!(start, 1_000_000);
        assert_eq!(end, 1_000_000);
    } else {
        panic!("Expected InvalidPeriod error for equal start and end");
    }
}

#[test]
fn history_request_deserialization_period_start_ge_end_rejected() {
    let invalid = serde_json::json!({
        "time_spec": {
            "kind": "period",
            "start": 2_000_000,
            "end": 1_000_000
        },
        "interval": Interval::D1,
        "flags": 6
    });

    let result = serde_json::from_value::<HistoryRequest>(invalid);

    assert!(result.is_err());
}

#[test]
fn history_request_deserialization_unknown_field_rejected() {
    let invalid = serde_json::json!({
        "time_spec": {
            "kind": "range",
            "range": "1d"
        },
        "interval": Interval::D1,
        "flags": 6,
        "interavl": "1d"
    });

    let result = serde_json::from_value::<HistoryRequest>(invalid);

    assert!(result.is_err());
}

#[test]
fn history_request_validation_valid_range_passes() {
    let result = HistoryRequest::builder().range(Range::D1).build();
    assert!(result.is_ok());
}

#[test]
fn history_request_validation_valid_period_passes() {
    let result = HistoryRequest::builder()
        .period(
            DateTime::from_timestamp(1000, 0).unwrap(),
            DateTime::from_timestamp(2000, 0).unwrap(),
        )
        .build();
    assert!(result.is_ok());
}

#[test]
fn history_request_validation_neither_range_nor_period_passes() {
    let result = HistoryRequest::builder().build();
    assert!(result.is_ok());
}
