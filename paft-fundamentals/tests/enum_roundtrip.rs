use paft_fundamentals::analysis::{RecommendationAction, RecommendationGrade};
use paft_fundamentals::holders::{InsiderPosition, TransactionType};
use paft_fundamentals::profile::FundKind;
use std::str::FromStr;

fn assert_display_parse_display_idempotent<T>(token: &str)
where
    T: ToString + FromStr<Err = paft_core::error::PaftError> + PartialEq + Clone,
{
    let parsed = T::from_str(token).unwrap();
    let display1 = parsed.to_string();
    let reparsed = T::from_str(&display1).unwrap();
    let display2 = reparsed.to_string();
    assert_eq!(display1, display2);
}

#[test]
fn fundamentals_enums_idempotent_and_other_roundtrip() {
    // RecommendationGrade
    assert_display_parse_display_idempotent::<RecommendationGrade>("HOLD");
    let other_grade = RecommendationGrade::from_str("market perform").unwrap();
    assert_eq!(other_grade.to_string(), "HOLD"); // alias
    let other_grade2 = RecommendationGrade::from_str("custom-grade").unwrap();
    assert_eq!(other_grade2.to_string(), "CUSTOM_GRADE");

    // RecommendationAction
    assert_display_parse_display_idempotent::<RecommendationAction>("UPGRADE");
    let other_action = RecommendationAction::from_str("affirm").unwrap();
    assert_eq!(other_action.to_string(), "AFFIRM");

    // TransactionType
    assert_display_parse_display_idempotent::<TransactionType>("BUY");
    let other_tx = TransactionType::from_str("vesting").unwrap();
    assert_eq!(other_tx.to_string(), "VESTING");

    // InsiderPosition
    assert_display_parse_display_idempotent::<InsiderPosition>("CEO");
    let other_pos = InsiderPosition::from_str("chief_strategy_officer").unwrap();
    assert_eq!(other_pos.to_string(), "CHIEF_STRATEGY_OFFICER");

    // FundKind
    assert_display_parse_display_idempotent::<FundKind>("ETF");
    let other_fund = FundKind::from_str("interval_fund").unwrap();
    assert_eq!(other_fund.to_string(), "INTERVAL_FUND");
}

#[test]
fn rejects_inputs_that_canonicalize_to_empty_fundamentals_enums() {
    let empties = ["***", "__", "   "];

    for input in &empties {
        // RecommendationGrade
        let err = RecommendationGrade::from_str(input).unwrap_err();
        match err {
            paft_core::error::PaftError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "RecommendationGrade");
                assert_eq!(value, (*input).to_string());
            }
            other => panic!("unexpected error: {other}"),
        }

        // RecommendationAction
        let err = RecommendationAction::from_str(input).unwrap_err();
        match err {
            paft_core::error::PaftError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "RecommendationAction");
                assert_eq!(value, (*input).to_string());
            }
            other => panic!("unexpected error: {other}"),
        }

        // TransactionType
        let err = TransactionType::from_str(input).unwrap_err();
        match err {
            paft_core::error::PaftError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "TransactionType");
                assert_eq!(value, (*input).to_string());
            }
            other => panic!("unexpected error: {other}"),
        }

        // InsiderPosition
        let err = InsiderPosition::from_str(input).unwrap_err();
        match err {
            paft_core::error::PaftError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "InsiderPosition");
                assert_eq!(value, (*input).to_string());
            }
            other => panic!("unexpected error: {other}"),
        }

        // FundKind
        let err = FundKind::from_str(input).unwrap_err();
        match err {
            paft_core::error::PaftError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "FundKind");
                assert_eq!(value, (*input).to_string());
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
