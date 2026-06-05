use paft_prediction::{
    BinaryResolution, LinkedBinaryRelation, OtherBinaryResolution, OtherClaimDescriptor,
    OtherEventStructure, OtherLinkedBinaryRelation, OtherPredictionMarketStatus,
    PredictionMarketStatus,
};

#[test]
fn prediction_market_status_uses_open_string_serde() {
    assert_eq!(
        serde_json::to_string(&PredictionMarketStatus::Open).unwrap(),
        "\"open\""
    );

    let unknown: PredictionMarketStatus = serde_json::from_str("\"foo_state\"").unwrap();
    match unknown {
        PredictionMarketStatus::Other(value) => assert_eq!(value.as_str(), "foo_state"),
        other => panic!("expected Other status, got {other:?}"),
    }

    let other =
        PredictionMarketStatus::Other(OtherPredictionMarketStatus::new("provider_paused").unwrap());
    assert_eq!(
        serde_json::to_string(&other).unwrap(),
        "\"provider_paused\""
    );
}

#[test]
fn linked_binary_relation_uses_open_string_serde() {
    assert_eq!(
        serde_json::to_string(&LinkedBinaryRelation::NegativeRiskConversion).unwrap(),
        "\"negative_risk_conversion\""
    );

    let unknown: LinkedBinaryRelation = serde_json::from_str("\"provider_relation\"").unwrap();
    match unknown {
        LinkedBinaryRelation::Other(value) => {
            assert_eq!(value.as_str(), "provider_relation");
        }
        other => panic!("expected Other relation, got {other:?}"),
    }
}

#[test]
fn binary_resolution_uses_open_string_serde() {
    assert_eq!(
        serde_json::to_string(&BinaryResolution::Yes).unwrap(),
        "\"yes\""
    );

    let unknown: BinaryResolution = serde_json::from_str("\"provider_voided\"").unwrap();
    match unknown {
        BinaryResolution::Other(value) => assert_eq!(value.as_str(), "provider_voided"),
        other => panic!("expected Other resolution, got {other:?}"),
    }
}

#[test]
fn simple_open_enum_other_constructors_reject_modeled_codes() {
    assert!(OtherLinkedBinaryRelation::new("sum_to_one").is_err());
    assert!(OtherLinkedBinaryRelation::new("Negative_Risk_Conversion").is_err());
    assert!(OtherPredictionMarketStatus::new("open").is_err());
    assert!(OtherPredictionMarketStatus::new("Resolved").is_err());
    assert!(OtherBinaryResolution::new("yes").is_err());
    assert!(OtherBinaryResolution::new("Void").is_err());
}

#[test]
fn metadata_other_constructors_reject_modeled_codes() {
    assert!(OtherEventStructure::new("single_market").is_err());
    assert!(OtherEventStructure::new("ordered_buckets").is_err());
    assert!(OtherClaimDescriptor::new("text").is_err());
    assert!(OtherClaimDescriptor::new("Numeric_Range").is_err());
}
