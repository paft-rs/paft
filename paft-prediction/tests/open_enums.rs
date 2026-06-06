use paft_prediction::{
    BinaryPayoutVector, BinarySettlement, LinkedBinaryRelation, OtherBinaryResolution,
    OtherClaimDescriptor, OtherEventStructure, OtherLinkedBinaryRelation,
    OtherPredictionMarketStatus, OutcomePayout, PredictionMarketStatus,
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
fn simple_open_enum_other_constructors_reject_modeled_codes() {
    assert!(OtherLinkedBinaryRelation::new("sum_to_one").is_err());
    assert!(OtherLinkedBinaryRelation::new("Negative_Risk_Conversion").is_err());
    assert!(OtherPredictionMarketStatus::new("open").is_err());
    assert!(OtherPredictionMarketStatus::new("Resolved").is_err());
    assert!(OtherBinaryResolution::new("yes").is_err());
    assert!(OtherBinaryResolution::new("payout_vector").is_err());
    assert!(OtherBinaryResolution::new("Void").is_err());
}

#[test]
fn binary_settlement_uses_structured_serde() {
    assert_eq!(
        serde_json::to_value(&BinarySettlement::Yes).unwrap(),
        serde_json::json!({ "kind": "yes" })
    );

    let yes_payout = 370_000_u64;
    let no_payout = 630_000_u64;
    let vector = BinarySettlement::PayoutVector(BinaryPayoutVector::new(
        OutcomePayout::from_micropayouts(yes_payout),
        OutcomePayout::from_micropayouts(no_payout),
    ));
    assert_eq!(
        serde_json::to_value(&vector).unwrap(),
        serde_json::json!({
            "kind": "payout_vector",
            "value": {
                "yes": yes_payout,
                "no": no_payout
            }
        })
    );
    let parsed: BinarySettlement = serde_json::from_value(serde_json::json!({
        "kind": "payout_vector",
        "value": {
            "yes": yes_payout,
            "no": no_payout
        }
    }))
    .unwrap();
    assert_eq!(parsed, vector);

    let other = BinarySettlement::Other(OtherBinaryResolution::new("provider_voided").unwrap());
    assert_eq!(
        serde_json::to_value(&other).unwrap(),
        serde_json::json!({
            "kind": "other",
            "value": "provider_voided"
        })
    );

    assert!(
        serde_json::from_value::<BinarySettlement>(
            serde_json::json!({ "kind": "yes", "value": { "yes": 1, "no": 0 } })
        )
        .is_err()
    );
}

#[test]
fn metadata_other_constructors_reject_modeled_codes() {
    assert!(OtherEventStructure::new("single_market").is_err());
    assert!(OtherEventStructure::new("ordered_buckets").is_err());
    assert!(OtherClaimDescriptor::new("text").is_err());
    assert!(OtherClaimDescriptor::new("Numeric_Range").is_err());
}
