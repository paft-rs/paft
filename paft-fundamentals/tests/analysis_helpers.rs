use paft_core::domain::{Currency, Money};
use paft_fundamentals::analysis::{
    EpsRevisions, EpsTrend, RecommendationSummary, RevisionPoint, TrendPoint,
};
use rust_decimal::Decimal;

#[test]
fn eps_trend_helpers() {
    let t = EpsTrend::new(
        Some(Money::new(Decimal::new(100, 2), Currency::USD)),
        vec![
            TrendPoint::new("7d", Money::new(Decimal::new(101, 2), Currency::USD)),
            TrendPoint::new("30d", Money::new(Decimal::new(98, 2), Currency::USD)),
        ],
    );
    assert_eq!(t.available_periods(), vec!["7d", "30d"]);
    assert!(t.find_by_period("7d").is_some());
    assert!(t.find_by_period("90d").is_none());
}

#[test]
fn eps_revisions_helpers() {
    let r = EpsRevisions::new(vec![
        RevisionPoint::new("7d", 3, 1),
        RevisionPoint::new("30d", 4, 6),
    ]);
    assert_eq!(r.available_periods(), vec!["7d", "30d"]);
    assert_eq!(r.total_up_revisions(), 7);
    assert_eq!(r.total_down_revisions(), 7);
    assert_eq!(r.net_revisions(), 0);
}

#[test]
fn recommendation_summary_defaults() {
    let s = RecommendationSummary::default();
    assert!(s.latest_period.is_none());
    assert!(s.mean.is_none());
}
