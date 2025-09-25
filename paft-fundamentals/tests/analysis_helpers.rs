use iso_currency::Currency as IsoCurrency;
use paft_fundamentals::analysis::{
    EpsRevisions, EpsTrend, RecommendationSummary, RevisionPoint, TrendPoint,
};
use paft_money::Currency;
use paft_money::Money;
use rust_decimal::Decimal;

#[test]
fn eps_trend_helpers() {
    let t = EpsTrend::new(
        Some(Money::new(
            Decimal::new(100, 2),
            Currency::Iso(IsoCurrency::USD),
        )),
        vec![
            TrendPoint::try_new_str(
                "7d",
                Money::new(Decimal::new(101, 2), Currency::Iso(IsoCurrency::USD)),
            )
            .unwrap(),
            TrendPoint::try_new_str(
                "30d",
                Money::new(Decimal::new(98, 2), Currency::Iso(IsoCurrency::USD)),
            )
            .unwrap(),
        ],
    );
    assert_eq!(
        t.available_periods()
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>(),
        vec!["7D", "30D"]
    );
    assert!(
        t.find_by_period(&"7d".parse().expect("valid period"))
            .is_some()
    );
    assert!(
        t.find_by_period(&"90d".parse().expect("valid period"))
            .is_none()
    );
    assert!(t.find_by_period_str("7d").expect("parsed").is_some());
    assert!(t.find_by_period_str("90d").expect("parsed").is_none());
}

#[test]
fn eps_revisions_helpers() {
    let r = EpsRevisions::new(vec![
        RevisionPoint::try_new_str("7d", 3, 1).unwrap(),
        RevisionPoint::try_new_str("30d", 4, 6).unwrap(),
    ]);
    assert_eq!(
        r.available_periods()
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>(),
        vec!["7D", "30D"]
    );
    assert_eq!(r.total_up_revisions(), 7);
    assert_eq!(r.total_down_revisions(), 7);
    assert_eq!(r.net_revisions(), 0);
    assert!(r.find_by_period_str("7d").expect("parsed").is_some());
    assert!(r.find_by_period_str("90d").expect("parsed").is_none());
}

#[test]
fn recommendation_summary_defaults() {
    let s = RecommendationSummary::default();
    assert!(s.latest_period.is_none());
    assert!(s.mean.is_none());
}
