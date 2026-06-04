use paft_fundamentals::FundamentalsError;
use paft_fundamentals::analysis::{
    OtherRecommendationAction, OtherRecommendationGrade, RecommendationAction, RecommendationGrade,
};
use paft_fundamentals::holders::{
    InsiderPosition, OtherInsiderPosition, OtherTransactionType, TransactionType,
};
use paft_fundamentals::profile::{FundKind, OtherFundKind};
use paft_utils::MAX_CANONICAL_TOKEN_LEN;
use std::str::FromStr;

fn assert_display_parse_display_idempotent<T>(token: &str)
where
    T: ToString + FromStr<Err = FundamentalsError> + PartialEq + Clone,
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
            FundamentalsError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "RecommendationGrade");
                assert_eq!(value, (*input).to_string());
            }
            _ => panic!("expected InvalidEnumValue"),
        }

        // RecommendationAction
        let err = RecommendationAction::from_str(input).unwrap_err();
        match err {
            FundamentalsError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "RecommendationAction");
                assert_eq!(value, (*input).to_string());
            }
            _ => panic!("expected InvalidEnumValue"),
        }

        // TransactionType
        let err = TransactionType::from_str(input).unwrap_err();
        match err {
            FundamentalsError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "TransactionType");
                assert_eq!(value, (*input).to_string());
            }
            _ => panic!("expected InvalidEnumValue"),
        }

        // InsiderPosition
        let err = InsiderPosition::from_str(input).unwrap_err();
        match err {
            FundamentalsError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "InsiderPosition");
                assert_eq!(value, (*input).to_string());
            }
            _ => panic!("expected InvalidEnumValue"),
        }

        // FundKind
        let err = FundKind::from_str(input).unwrap_err();
        match err {
            FundamentalsError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "FundKind");
                assert_eq!(value, (*input).to_string());
            }
            _ => panic!("expected InvalidEnumValue"),
        }
    }
}

#[test]
fn other_wrappers_reject_modeled_fundamentals_tokens() {
    assert!(OtherRecommendationGrade::new("BUY").is_err());
    assert!(OtherRecommendationGrade::new("market perform").is_err());
    assert_eq!(
        OtherRecommendationGrade::new("custom grade")
            .unwrap()
            .as_ref(),
        "CUSTOM_GRADE"
    );

    assert!(OtherRecommendationAction::new("UPGRADE").is_err());
    assert!(OtherRecommendationAction::new("up").is_err());
    assert_eq!(
        OtherRecommendationAction::new("affirm").unwrap().as_ref(),
        "AFFIRM"
    );

    assert!(OtherTransactionType::new("BUY").is_err());
    assert!(OtherTransactionType::new("purchase").is_err());
    assert_eq!(
        OtherTransactionType::new("vesting").unwrap().as_ref(),
        "VESTING"
    );

    assert!(OtherInsiderPosition::new("CEO").is_err());
    assert!(OtherInsiderPosition::new("chief executive officer").is_err());
    assert_eq!(
        OtherInsiderPosition::new("chief strategy officer")
            .unwrap()
            .as_ref(),
        "CHIEF_STRATEGY_OFFICER"
    );

    assert!(OtherFundKind::new("ETF").is_err());
    assert!(OtherFundKind::new("exchange traded fund").is_err());
    assert_eq!(
        OtherFundKind::new("interval fund").unwrap().as_ref(),
        "INTERVAL_FUND"
    );
}

#[test]
fn other_wrappers_reject_overlong_fundamentals_tokens() {
    let input = "x".repeat(MAX_CANONICAL_TOKEN_LEN + 1);

    assert!(OtherRecommendationGrade::new(&input).is_err());
    assert!(RecommendationGrade::from_str(&input).is_err());

    assert!(OtherRecommendationAction::new(&input).is_err());
    assert!(RecommendationAction::from_str(&input).is_err());

    assert!(OtherTransactionType::new(&input).is_err());
    assert!(TransactionType::from_str(&input).is_err());

    assert!(OtherInsiderPosition::new(&input).is_err());
    assert!(InsiderPosition::from_str(&input).is_err());

    assert!(OtherFundKind::new(&input).is_err());
    assert!(FundKind::from_str(&input).is_err());

    let json = serde_json::to_string(&input).unwrap();
    assert!(serde_json::from_str::<OtherRecommendationGrade>(&json).is_err());
    assert!(serde_json::from_str::<RecommendationGrade>(&json).is_err());
}

#[test]
fn other_wrappers_serde_uses_checked_constructors_for_fundamentals_tokens() {
    let grade = OtherRecommendationGrade::new("custom grade").unwrap();
    assert_eq!(serde_json::to_string(&grade).unwrap(), "\"CUSTOM_GRADE\"");
    let grade: OtherRecommendationGrade = serde_json::from_str("\"custom grade\"").unwrap();
    assert_eq!(grade.as_ref(), "CUSTOM_GRADE");
    assert!(serde_json::from_str::<OtherRecommendationGrade>("\"BUY\"").is_err());
    assert!(serde_json::from_str::<OtherRecommendationGrade>("\"market perform\"").is_err());

    let action = OtherRecommendationAction::new("affirm").unwrap();
    assert_eq!(serde_json::to_string(&action).unwrap(), "\"AFFIRM\"");
    let action: OtherRecommendationAction = serde_json::from_str("\"affirm\"").unwrap();
    assert_eq!(action.as_ref(), "AFFIRM");
    assert!(serde_json::from_str::<OtherRecommendationAction>("\"UPGRADE\"").is_err());
    assert!(serde_json::from_str::<OtherRecommendationAction>("\"up\"").is_err());

    let transaction_type = OtherTransactionType::new("vesting").unwrap();
    assert_eq!(
        serde_json::to_string(&transaction_type).unwrap(),
        "\"VESTING\""
    );
    let transaction_type: OtherTransactionType = serde_json::from_str("\"vesting\"").unwrap();
    assert_eq!(transaction_type.as_ref(), "VESTING");
    assert!(serde_json::from_str::<OtherTransactionType>("\"BUY\"").is_err());
    assert!(serde_json::from_str::<OtherTransactionType>("\"purchase\"").is_err());

    let position = OtherInsiderPosition::new("chief strategy officer").unwrap();
    assert_eq!(
        serde_json::to_string(&position).unwrap(),
        "\"CHIEF_STRATEGY_OFFICER\""
    );
    let position: OtherInsiderPosition =
        serde_json::from_str("\"chief strategy officer\"").unwrap();
    assert_eq!(position.as_ref(), "CHIEF_STRATEGY_OFFICER");
    assert!(serde_json::from_str::<OtherInsiderPosition>("\"CEO\"").is_err());
    assert!(serde_json::from_str::<OtherInsiderPosition>("\"chief executive officer\"").is_err());

    let fund_kind = OtherFundKind::new("interval fund").unwrap();
    assert_eq!(
        serde_json::to_string(&fund_kind).unwrap(),
        "\"INTERVAL_FUND\""
    );
    let fund_kind: OtherFundKind = serde_json::from_str("\"interval fund\"").unwrap();
    assert_eq!(fund_kind.as_ref(), "INTERVAL_FUND");
    assert!(serde_json::from_str::<OtherFundKind>("\"ETF\"").is_err());
    assert!(serde_json::from_str::<OtherFundKind>("\"exchange traded fund\"").is_err());
}
