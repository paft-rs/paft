use chrono::{TimeZone, Utc};
use paft_fundamentals::statements::Calendar;
use serde_json::{from_str, json, to_string};

#[test]
fn calendar_ts_seconds_serde() {
    let c = Calendar {
        earnings_dates: vec![
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap(),
        ],
        ex_dividend_date: Some(Utc.with_ymd_and_hms(2024, 2, 15, 0, 0, 0).unwrap()),
        dividend_payment_date: None,
    };

    let s = to_string(&c).unwrap();
    let v: serde_json::Value = from_str(&s).unwrap();
    assert_eq!(v["earnings_dates"], json!([1704067200, 1711929600]));
    assert_eq!(v["ex_dividend_date"], json!(1707955200));
    assert!(v["dividend_payment_date"].is_null());

    let back: Calendar = from_str(&s).unwrap();
    assert_eq!(back, c);
}
