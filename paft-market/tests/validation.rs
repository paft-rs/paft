use chrono::DateTime;
use paft_core::domain::AssetKind;
use paft_core::error::PaftError;
use paft_market::requests::{HistoryRequest, Range, SearchRequest};

#[test]
fn search_request_validation_empty_query_rejected() {
    let result = SearchRequest::new("");
    assert!(result.is_err());

    if result == Err(PaftError::EmptySearchQuery) {
        // expected
    } else {
        panic!("Expected EmptySearchQuery error for empty query");
    }
}

#[test]
fn search_request_validation_whitespace_only_query_rejected() {
    let result = SearchRequest::new("   \t\n  ");
    assert!(result.is_err());

    if result == Err(PaftError::EmptySearchQuery) {
        // expected
    } else {
        panic!("Expected EmptySearchQuery error for whitespace-only query");
    }
}

#[test]
fn search_request_validation_zero_limit_rejected() {
    let result = SearchRequest::builder("AAPL").limit(0).build();
    assert!(result.is_err());

    if let Err(PaftError::InvalidSearchLimit(limit)) = result {
        assert_eq!(limit, 0);
    } else {
        panic!("Expected InvalidSearchLimit error for zero limit");
    }
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

    if let Err(PaftError::InvalidPeriod { start, end }) = result {
        assert_eq!(start, 2000);
        assert_eq!(end, 1000);
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

    if let Err(PaftError::InvalidPeriod { start, end }) = result {
        assert_eq!(start, 1000);
        assert_eq!(end, 1000);
    } else {
        panic!("Expected InvalidPeriod error for equal start and end");
    }
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
