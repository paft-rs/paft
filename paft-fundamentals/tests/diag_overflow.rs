use paft_fundamentals::analysis::{RecommendationAction, RecommendationGrade};
use paft_fundamentals::holders::{InsiderPosition, TransactionType};
use paft_fundamentals::profile::FundKind;
use std::str::FromStr;

fn roundtrip<T: ToString + FromStr<Err = paft_core::error::PaftError>>(label: &str, token: &str) {
    eprintln!("starting {label} -> {token}");
    let parsed = T::from_str(token).unwrap();
    let display1 = parsed.to_string();
    eprintln!("{label} display1={display1}");
    let reparsed = T::from_str(&display1).unwrap();
    let display2 = reparsed.to_string();
    eprintln!("{label} display2={display2}");
    assert_eq!(display1, display2);
}

#[test]
fn find_overflow_point() {
    roundtrip::<RecommendationGrade>("RecommendationGrade", "HOLD");
    roundtrip::<RecommendationAction>("RecommendationAction", "UPGRADE");
    roundtrip::<TransactionType>("TransactionType", "BUY");
    roundtrip::<InsiderPosition>("InsiderPosition", "CEO");
    roundtrip::<FundKind>("FundKind", "ETF");
}
