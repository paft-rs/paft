use paft_core::domain::AssetKind;
use paft_market::requests::history::{HistoryRequest, Interval, Range};
use paft_market::requests::search::SearchRequest;

#[test]
fn search_request_serialization() {
    let request = SearchRequest::builder("AAPL")
        .kind(AssetKind::Equity)
        .limit(10)
        .build()
        .unwrap();

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: SearchRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request, deserialized);
}

#[test]
fn search_request_minimal() {
    let request = SearchRequest::new("AAPL").unwrap();

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: SearchRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request, deserialized);
}

#[test]
fn history_request_serialization() {
    let request = HistoryRequest::builder()
        .range(Range::D1)
        .interval(Interval::D1)
        .include_prepost(true)
        .include_actions(true)
        .auto_adjust(false)
        .build()
        .unwrap();

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: HistoryRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request, deserialized);
}

#[test]
fn history_request_with_period() {
    use chrono::DateTime;

    let request = HistoryRequest::builder()
        .period(
            DateTime::from_timestamp(1000, 0).unwrap(),
            DateTime::from_timestamp(2000, 0).unwrap(),
        )
        .interval(Interval::D1)
        .include_prepost(false)
        .include_actions(false)
        .auto_adjust(true)
        .keepna(true)
        .build()
        .unwrap();

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: HistoryRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request, deserialized);
}

#[test]
fn interval_serialization() {
    let intervals = [
        Interval::I1m,
        Interval::I5m,
        Interval::I15m,
        Interval::I30m,
        Interval::I1h,
        Interval::D1,
        Interval::W1,
        Interval::M1,
    ];

    for interval in intervals {
        let json = serde_json::to_string(&interval).unwrap();
        let deserialized: Interval = serde_json::from_str(&json).unwrap();
        assert_eq!(interval, deserialized);
    }
}

#[test]
fn range_serialization() {
    let ranges = [
        Range::D1,
        Range::D5,
        Range::M1,
        Range::M3,
        Range::M6,
        Range::Y1,
        Range::Y2,
        Range::Y5,
        Range::Y10,
        Range::Ytd,
        Range::Max,
    ];

    for range in ranges {
        let json = serde_json::to_string(&range).unwrap();
        let deserialized: Range = serde_json::from_str(&json).unwrap();
        assert_eq!(range, deserialized);
    }
}
