use chrono::DateTime;
use paft::prelude::{HistoryRequest, Interval, Range, SearchRequest};

#[test]
fn test_encapsulation_prevents_invalid_construction() {
    // This test demonstrates that users can no longer create invalid instances
    // by bypassing the validation in the builders.

    // The following code would NOT compile because the fields are now private:
    /*
    let invalid_history = HistoryRequest {
        time_spec: TimeSpec::Range(Range::D1), // This would be invalid if we could also set a period
        interval: Interval::D1,
        include_prepost: false,
        include_actions: true,
        auto_adjust: true,
        keepna: false,
    };

    let invalid_search = SearchRequest {
        query: "".to_string(),
        kind: None,
        limit: None,
    };
    */

    // Instead, users must use the validated builders or constructors:

    // Valid construction through builder with validation
    let valid_history = HistoryRequest::builder()
        .range(Range::D1)
        .interval(Interval::D1)
        .build()
        .expect("Valid history request should succeed");

    // Accessing fields through getters still works
    assert_eq!(valid_history.range(), Some(Range::D1));
    assert_eq!(valid_history.period(), None);
    assert_eq!(valid_history.interval(), Interval::D1);

    // Valid construction through constructor with validation
    let valid_search = SearchRequest::new("AAPL").expect("Valid search request should succeed");

    // Accessing fields through getters still works
    assert_eq!(valid_search.query(), "AAPL");
    assert_eq!(valid_search.kind(), None);
    assert_eq!(valid_search.limit(), None);

    // Validation still prevents invalid states through builders
    let invalid_attempt = HistoryRequest::builder()
        .period(
            DateTime::from_timestamp(2000, 0).unwrap(),
            DateTime::from_timestamp(1000, 0).unwrap(),
        ) // start > end
        .build();
    assert!(
        invalid_attempt.is_err(),
        "Invalid period should be rejected"
    );

    let invalid_search_attempt = SearchRequest::new("");
    assert!(
        invalid_search_attempt.is_err(),
        "Empty query should be rejected"
    );
}

#[test]
fn test_serialization_still_works() {
    // Test that serialization/deserialization still works with private fields
    let history_request = HistoryRequest::builder()
        .range(Range::D1)
        .interval(Interval::D1)
        .include_prepost(true)
        .build()
        .unwrap();

    let json = serde_json::to_string(&history_request).unwrap();
    let deserialized: HistoryRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(history_request.range(), deserialized.range());
    assert_eq!(history_request.period(), deserialized.period());
    assert_eq!(history_request.interval(), deserialized.interval());
    assert_eq!(
        history_request.include_prepost(),
        deserialized.include_prepost()
    );

    let search_request = SearchRequest::builder("AAPL").limit(10).build().unwrap();

    let json = serde_json::to_string(&search_request).unwrap();
    let deserialized: SearchRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(search_request.query(), deserialized.query());
    assert_eq!(search_request.kind(), deserialized.kind());
    assert_eq!(search_request.limit(), deserialized.limit());
}
