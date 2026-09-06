use chrono::{NaiveDate, TimeZone, Utc};
use paft_fundamentals::statements::Calendar;
use serde_json::{from_str, json, to_string};

const fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

#[test]
fn calendar_serializes_instants_and_dates_with_domain_precision() {
    let c = Calendar {
        earnings_dates: vec![
            Utc.timestamp_opt(1_704_067_200, 123_000_000).unwrap(),
            Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap(),
        ],
        ex_dividend_date: Some(date(2024, 2, 15)),
        dividend_payment_date: None,
    };

    let s = to_string(&c).unwrap();
    let v: serde_json::Value = from_str(&s).unwrap();
    assert_eq!(
        v["earnings_dates"],
        json!(["2024-01-01T00:00:00.123Z", "2024-04-01T00:00:00Z"])
    );
    assert_eq!(v["ex_dividend_date"], json!("2024-02-15"));
    assert!(v["dividend_payment_date"].is_null());

    let back: Calendar = from_str(&s).unwrap();
    assert_eq!(back, c);
}
