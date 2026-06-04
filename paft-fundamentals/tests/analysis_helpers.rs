use paft_decimal::Decimal;
use paft_domain::Horizon;
use paft_fundamentals::{
    EarningsYear, EpsRevisions, EpsTrend, RecommendationSummary, RevisionPoint, TrendPoint,
};
use paft_money::{Currency, IsoCurrency, Price};
use std::str::FromStr;

#[test]
fn eps_trend_helpers() {
    let t = EpsTrend::new(
        Some(Price::new(
            Decimal::from_str("1.00").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )),
        vec![
            TrendPoint::try_new_str(
                "7d",
                Price::new(
                    Decimal::from_str("1.01").unwrap(),
                    Currency::Iso(IsoCurrency::USD),
                ),
            )
            .unwrap(),
            TrendPoint::try_new_str(
                "30d",
                Price::new(
                    Decimal::from_str("0.98").unwrap(),
                    Currency::Iso(IsoCurrency::USD),
                ),
            )
            .unwrap(),
        ],
    );
    assert_eq!(
        t.available_horizons()
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>(),
        vec!["7d", "30d"]
    );
    assert!(
        t.find_by_horizon(&"7d".parse::<Horizon>().expect("valid horizon"))
            .is_some()
    );
    assert!(
        t.find_by_horizon(&"90d".parse::<Horizon>().expect("valid horizon"))
            .is_none()
    );
    assert!(t.find_by_horizon_str("7d").expect("parsed").is_some());
    assert!(t.find_by_horizon_str("90d").expect("parsed").is_none());
}

#[test]
fn eps_revisions_helpers() {
    let r = EpsRevisions::new(vec![
        RevisionPoint::try_new_str("7d", 3, 1).unwrap(),
        RevisionPoint::try_new_str("30d", 4, 6).unwrap(),
    ]);
    assert_eq!(
        r.available_horizons()
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>(),
        vec!["7d", "30d"]
    );
    assert_eq!(
        r.find_by_horizon_str("7d")
            .expect("parsed")
            .expect("present")
            .net_revisions(),
        2
    );
    assert!(r.find_by_horizon_str("90d").expect("parsed").is_none());
}

#[test]
fn revision_point_revisions_do_not_wrap_large_counts() {
    let total = RevisionPoint::try_new_str("7d", u32::MAX, u32::MAX).unwrap();
    let upward = RevisionPoint::try_new_str("7d", u32::MAX, 0).unwrap();
    let downward = RevisionPoint::try_new_str("7d", 0, u32::MAX).unwrap();

    assert_eq!(total.total_revisions(), u64::from(u32::MAX) * 2);
    assert_eq!(upward.net_revisions(), i64::from(u32::MAX));
    assert_eq!(downward.net_revisions(), -i64::from(u32::MAX));
}

#[test]
fn recommendation_summary_defaults() {
    let s = RecommendationSummary::default();
    assert!(s.latest_period.is_none());
    assert!(s.mean.is_none());
}

#[test]
fn earnings_year_uses_validated_period_year() {
    let row = EarningsYear::new(2024).unwrap();
    assert_eq!(row.year.get(), 2024);

    let value = serde_json::to_value(&row).unwrap();
    assert_eq!(value["year"], serde_json::json!("2024"));

    let from_numeric: EarningsYear = serde_json::from_value(serde_json::json!({
        "year": 2024,
        "revenue": null,
        "earnings": null
    }))
    .unwrap();
    assert_eq!(from_numeric.year, row.year);

    assert!(EarningsYear::new(10_000).is_err());
    assert!(
        serde_json::from_value::<EarningsYear>(serde_json::json!({
            "year": 10000,
            "revenue": null,
            "earnings": null
        }))
        .is_err()
    );
}
