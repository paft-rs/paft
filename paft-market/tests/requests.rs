use paft_domain::AssetKind;
use paft_market::{
    HistoryRequest, Interval, MarketError, NewsRequest, NewsTab, Range, SearchRequest, TimeSpec,
};
use std::num::NonZeroU32;

#[test]
fn search_request_serialization() {
    let request = SearchRequest::builder("AAPL")
        .kind(AssetKind::Equity)
        .limit(10)
        .build()
        .unwrap();
    assert_eq!(request.limit(), NonZeroU32::new(10));

    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["limit"], serde_json::json!(10));

    let deserialized: SearchRequest = serde_json::from_value(value).unwrap();
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
fn news_request_serialization() {
    let request = NewsRequest {
        count: NonZeroU32::new(25).unwrap(),
        tab: NewsTab::PressReleases,
    };

    let value = serde_json::to_value(request).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "count": 25,
            "tab": "PRESS_RELEASES"
        })
    );

    let deserialized: NewsRequest = serde_json::from_value(value).unwrap();
    assert_eq!(request, deserialized);
    assert_eq!(NewsRequest::default().tab, NewsTab::News);
}

#[test]
fn news_request_rejects_zero_count() {
    let value = serde_json::json!({
        "count": 0,
        "tab": "NEWS"
    });

    assert!(serde_json::from_value::<NewsRequest>(value).is_err());
}

#[test]
fn news_request_deserialization_unknown_field_rejected() {
    let mut value = serde_json::to_value(NewsRequest::default()).unwrap();
    value
        .as_object_mut()
        .expect("news request serializes as an object")
        .insert("coutn".to_owned(), serde_json::json!(25));

    assert!(serde_json::from_value::<NewsRequest>(value).is_err());
}

#[test]
fn news_tab_serialization() {
    let tabs = [
        (NewsTab::News, "NEWS"),
        (NewsTab::All, "ALL"),
        (NewsTab::PressReleases, "PRESS_RELEASES"),
    ];

    for (tab, code) in tabs {
        assert_eq!(tab.code(), code);
        assert_eq!(tab.to_string(), code);
        assert_eq!(code.parse::<NewsTab>().unwrap(), tab);
        assert_eq!(serde_json::to_value(tab).unwrap(), serde_json::json!(code));

        let deserialized: NewsTab = serde_json::from_value(serde_json::json!(code)).unwrap();
        assert_eq!(tab, deserialized);
    }
}

#[test]
fn news_tab_from_str_rejects_unknown_code() {
    let err = "PRESS".parse::<NewsTab>().unwrap_err();

    assert!(matches!(
        err,
        MarketError::InvalidEnumValue {
            enum_name: "NewsTab",
            value,
        } if value == "PRESS"
    ));
}

#[test]
fn history_request_serialization() {
    let request = HistoryRequest::builder()
        .range(Range::D1)
        .interval(Interval::D1)
        .include_prepost(true)
        .include_actions(true)
        .prefer_adjusted_prices(false)
        .build()
        .unwrap();

    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(
        value["time_spec"],
        serde_json::json!({
            "kind": "range",
            "range": "1d"
        })
    );
    assert_eq!(value["interval"], serde_json::json!("1d"));
    assert_eq!(value["flags"], serde_json::json!(3));

    let deserialized: HistoryRequest = serde_json::from_value(value).unwrap();
    assert_eq!(request, deserialized);
}

#[test]
fn time_spec_range_uses_explicit_kind_wire_shape() {
    let time_spec = TimeSpec::range(Range::M6);

    let value = serde_json::to_value(&time_spec).unwrap();

    assert_eq!(
        value,
        serde_json::json!({
            "kind": "range",
            "range": "6mo"
        })
    );

    let deserialized: TimeSpec = serde_json::from_value(value).unwrap();
    assert_eq!(time_spec, deserialized);
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
        .prefer_adjusted_prices(true)
        .keep_missing(true)
        .build()
        .unwrap();

    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(
        value["time_spec"],
        serde_json::json!({
            "kind": "period",
            "start": 1_000_000,
            "end": 2_000_000
        })
    );

    let deserialized: HistoryRequest = serde_json::from_value(value).unwrap();
    assert_eq!(request, deserialized);
    assert!(deserialized.keep_missing());
}

#[test]
fn time_spec_period_uses_epoch_millisecond_wire_shape() {
    use chrono::DateTime;

    let time_spec = TimeSpec::period(
        DateTime::from_timestamp(1_716_595_200, 123_000_000).unwrap(),
        DateTime::from_timestamp(1_719_187_200, 456_000_000).unwrap(),
    )
    .unwrap();

    let value = serde_json::to_value(&time_spec).unwrap();

    assert_eq!(
        value,
        serde_json::json!({
            "kind": "period",
            "start": 1_716_595_200_123_i64,
            "end": 1_719_187_200_456_i64
        })
    );

    let deserialized: TimeSpec = serde_json::from_value(value).unwrap();
    assert_eq!(time_spec, deserialized);
}

#[test]
fn time_spec_period_constructor_rejects_invalid_bounds() {
    use chrono::DateTime;

    let start = DateTime::from_timestamp(2_000, 0).unwrap();
    let end = DateTime::from_timestamp(1_000, 0).unwrap();

    let err = TimeSpec::period(start, end).unwrap_err();
    assert!(matches!(
        err,
        paft_market::MarketError::InvalidPeriod {
            start: 2_000_000,
            end: 1_000_000,
        }
    ));
}

#[test]
fn time_spec_validate_detects_directly_constructed_invalid_period() {
    use chrono::DateTime;

    let time_spec = TimeSpec::Period {
        start: DateTime::from_timestamp(1_000, 0).unwrap(),
        end: DateTime::from_timestamp(1_000, 0).unwrap(),
    };

    assert!(time_spec.validate().is_err());
}

#[test]
fn time_spec_deserialization_rejects_invalid_period() {
    let value = serde_json::json!({
        "kind": "period",
        "start": 2_000_000,
        "end": 1_000_000
    });

    assert!(serde_json::from_value::<TimeSpec>(value).is_err());
}

#[test]
fn interval_serialization() {
    let intervals = [
        (Interval::I1s, "1s"),
        (Interval::I2s, "2s"),
        (Interval::I3s, "3s"),
        (Interval::I5s, "5s"),
        (Interval::I6s, "6s"),
        (Interval::I10s, "10s"),
        (Interval::I15s, "15s"),
        (Interval::I30s, "30s"),
        (Interval::I90s, "90s"),
        (Interval::I1m, "1m"),
        (Interval::I2m, "2m"),
        (Interval::I3m, "3m"),
        (Interval::I5m, "5m"),
        (Interval::I6m, "6m"),
        (Interval::I10m, "10m"),
        (Interval::I15m, "15m"),
        (Interval::I30m, "30m"),
        (Interval::I90m, "90m"),
        (Interval::I1h, "1h"),
        (Interval::I2h, "2h"),
        (Interval::I3h, "3h"),
        (Interval::I4h, "4h"),
        (Interval::I6h, "6h"),
        (Interval::I8h, "8h"),
        (Interval::I12h, "12h"),
        (Interval::D1, "1d"),
        (Interval::D5, "5d"),
        (Interval::W1, "1wk"),
        (Interval::M1, "1mo"),
        (Interval::M3, "3mo"),
        (Interval::M6, "6mo"),
        (Interval::Y1, "1y"),
        (Interval::Y2, "2y"),
        (Interval::Y5, "5y"),
        (Interval::Y10, "10y"),
    ];

    for (interval, code) in intervals {
        assert_eq!(interval.code(), code);
        assert_eq!(interval.to_string(), code);
        assert_eq!(code.parse::<Interval>().unwrap(), interval);
        assert_eq!(
            serde_json::to_value(interval).unwrap(),
            serde_json::json!(code)
        );

        let deserialized: Interval = serde_json::from_value(serde_json::json!(code)).unwrap();
        assert_eq!(interval, deserialized);
    }
}

#[test]
fn interval_from_str_rejects_unknown_code() {
    let err = "1week".parse::<Interval>().unwrap_err();

    assert!(matches!(
        err,
        MarketError::InvalidEnumValue {
            enum_name: "Interval",
            value,
        } if value == "1week"
    ));
}

#[test]
fn range_serialization() {
    let ranges = [
        (Range::I1m, "1m"),
        (Range::I2m, "2m"),
        (Range::I5m, "5m"),
        (Range::I10m, "10m"),
        (Range::I15m, "15m"),
        (Range::I30m, "30m"),
        (Range::I1h, "1h"),
        (Range::I4h, "4h"),
        (Range::I6h, "6h"),
        (Range::I8h, "8h"),
        (Range::I12h, "12h"),
        (Range::D1, "1d"),
        (Range::D5, "5d"),
        (Range::M1, "1mo"),
        (Range::M3, "3mo"),
        (Range::M6, "6mo"),
        (Range::Y1, "1y"),
        (Range::Y2, "2y"),
        (Range::Y5, "5y"),
        (Range::Y10, "10y"),
        (Range::Ytd, "ytd"),
        (Range::Max, "max"),
    ];

    for (range, code) in ranges {
        assert_eq!(range.code(), code);
        assert_eq!(range.to_string(), code);
        assert_eq!(code.parse::<Range>().unwrap(), range);
        assert_eq!(
            serde_json::to_value(range).unwrap(),
            serde_json::json!(code)
        );

        let deserialized: Range = serde_json::from_value(serde_json::json!(code)).unwrap();
        assert_eq!(range, deserialized);
    }
}

#[test]
fn range_from_str_rejects_unknown_code() {
    let err = "all".parse::<Range>().unwrap_err();

    assert!(matches!(
        err,
        MarketError::InvalidEnumValue {
            enum_name: "Range",
            value,
        } if value == "all"
    ));
}
