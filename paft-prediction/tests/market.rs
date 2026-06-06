use paft_money::{Currency, IsoCurrency};
use paft_prediction::{
    BinaryMarket, BinaryMarketKey, BinaryOutcomeInstruments, BinaryPayoutVector, BinarySettlement,
    ClaimDescriptor, EventStructure, MultiOutcomeMarket, NonZeroContractQuantity,
    NonZeroOutcomePayout, OutcomeDescriptor, OutcomeInstrument, OutcomePayout, PredictionError,
    PredictionEventKey, PredictionMarket, PredictionMarketKey, PredictionMarketStatus,
    PredictionOutcomeId, PredictionSeriesKey,
};
use paft_prediction::{NumericBound, NumericRange, PredictionEvent};

fn outcome(market_id: &str, outcome_id: &str, label: &str) -> OutcomeDescriptor {
    OutcomeDescriptor {
        instrument: OutcomeInstrument::new("MANIFOLD", market_id, outcome_id).unwrap(),
        label: label.to_string(),
    }
}

fn binary_market(market_id: &str) -> BinaryMarket {
    let key = BinaryMarketKey::new("KALSHI", market_id).unwrap();
    BinaryMarket::new(
        key.clone(),
        BinaryOutcomeInstruments::synthetic_for_market(&key),
        format!("Market {market_id}"),
        ClaimDescriptor::Text {
            description: format!("Claim {market_id}"),
        },
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        NonZeroOutcomePayout::ONE,
    )
    .unwrap()
}

#[test]
fn binary_market_carries_required_polymarket_outcome_instruments() {
    let key = BinaryMarketKey::new(
        "POLYMARKET",
        "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de",
    )
    .unwrap();
    let outcomes = BinaryOutcomeInstruments::new(
        OutcomeInstrument::new(
            "POLYMARKET",
            key.market_id.as_str(),
            "73470541315377973562501025254719659796416871135081220986683321361000395461644",
        )
        .unwrap(),
        OutcomeInstrument::new(
            "POLYMARKET",
            key.market_id.as_str(),
            "56393761733830483601097051857899348522495376869600726893014309766300892311293",
        )
        .unwrap(),
    )
    .unwrap();

    let market = BinaryMarket::new(
        key,
        outcomes,
        "Will BTC close above 100k?".to_string(),
        ClaimDescriptor::Text {
            description: "BTC closes above 100k by expiry.".to_string(),
        },
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        NonZeroOutcomePayout::ONE,
    )
    .unwrap();

    assert_eq!(
        market.outcomes().yes().outcome_id.as_str(),
        "73470541315377973562501025254719659796416871135081220986683321361000395461644"
    );
    assert_eq!(
        market.outcomes().no().outcome_id.as_str(),
        "56393761733830483601097051857899348522495376869600726893014309766300892311293"
    );
    assert_eq!(
        market.outcomes().yes().market_key(),
        market.key().to_market_key()
    );
    assert_eq!(
        market.outcomes().no().market_key(),
        market.key().to_market_key()
    );
}

#[test]
fn numeric_range_constructor_validates_interval() {
    let valid_single_point = NumericRange::new(
        NumericBound::Included(10.into()),
        NumericBound::Included(10.into()),
        Some("USD".to_string()),
    );
    let valid_single_point = valid_single_point.unwrap();
    assert!(matches!(
        valid_single_point.lower(),
        NumericBound::Included(_)
    ));
    assert!(matches!(
        valid_single_point.upper(),
        NumericBound::Included(_)
    ));
    assert_eq!(valid_single_point.unit(), Some("USD"));
    let (_, _, unit) = valid_single_point.into_parts();
    assert_eq!(unit.as_deref(), Some("USD"));

    let descending = NumericRange::new(
        NumericBound::Included(11.into()),
        NumericBound::Included(10.into()),
        None,
    );
    assert!(descending.is_err());

    let empty_zero_width = NumericRange::new(
        NumericBound::Included(10.into()),
        NumericBound::Excluded(10.into()),
        None,
    );
    assert!(empty_zero_width.is_err());
}

#[test]
fn binary_market_deserialization_rejects_zero_min_order_quantity() {
    let key = BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap();
    let mut market = BinaryMarket::new(
        key.clone(),
        BinaryOutcomeInstruments::synthetic_for_market(&key),
        "Will NYC temperature exceed 60F?".to_string(),
        ClaimDescriptor::Text {
            description: "NYC temperature exceeds 60F at expiry.".to_string(),
        },
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        NonZeroOutcomePayout::ONE,
    )
    .unwrap();
    market.min_order_quantity = Some(NonZeroContractQuantity::ONE);

    let mut value = serde_json::to_value(&market).unwrap();
    value["min_order_quantity"] = serde_json::json!(0);

    assert!(serde_json::from_value::<BinaryMarket>(value).is_err());
}

#[test]
fn binary_market_uses_full_event_key_reference() {
    let key = BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap();
    let mut market = BinaryMarket::new(
        key.clone(),
        BinaryOutcomeInstruments::synthetic_for_market(&key),
        "Will NYC temperature exceed 60F?".to_string(),
        ClaimDescriptor::Text {
            description: "NYC temperature exceeds 60F at expiry.".to_string(),
        },
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        NonZeroOutcomePayout::ONE,
    )
    .unwrap();
    market.event_key = Some(PredictionEventKey::new("KALSHI", "KXHIGHNY-24JAN01").unwrap());

    let value = serde_json::to_value(&market).unwrap();

    assert!(value.get("event_id").is_none());
    assert_eq!(value["event_key"]["venue"], "KALSHI");
    assert_eq!(value["event_key"]["event_id"], "KXHIGHNY-24JAN01");
}

#[test]
fn binary_market_derives_settlement_payout_vectors() {
    let key = BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap();
    let mut market = BinaryMarket::new(
        key.clone(),
        BinaryOutcomeInstruments::synthetic_for_market(&key),
        "Will NYC temperature exceed 60F?".to_string(),
        ClaimDescriptor::Text {
            description: "NYC temperature exceeds 60F at expiry.".to_string(),
        },
        PredictionMarketStatus::Resolved,
        Currency::Iso(IsoCurrency::USD),
        NonZeroOutcomePayout::ONE,
    )
    .unwrap();

    market.settlement = Some(BinarySettlement::Yes);
    let payouts = market.resolved_payouts().unwrap();
    assert_eq!(payouts.yes, OutcomePayout::ONE);
    assert_eq!(payouts.no, OutcomePayout::ZERO);

    market.settlement = Some(BinarySettlement::No);
    let payouts = market.resolved_payouts().unwrap();
    assert_eq!(payouts.yes, OutcomePayout::ZERO);
    assert_eq!(payouts.no, OutcomePayout::ONE);

    market.settlement = Some(BinarySettlement::PayoutVector(BinaryPayoutVector::new(
        OutcomePayout::from_micropayouts(370_000),
        OutcomePayout::from_micropayouts(630_000),
    )));
    let payouts = market.resolved_payouts().unwrap();
    assert_eq!(payouts.yes.micropayouts(), 370_000);
    assert_eq!(payouts.no.micropayouts(), 630_000);

    market.settlement = Some(BinarySettlement::Void);
    assert_eq!(market.resolved_payouts(), None);
}

#[test]
fn binary_market_constructor_rejects_outcomes_for_different_market_key() {
    let key = BinaryMarketKey::new("KALSHI", "MARKET_A").unwrap();
    let other_key = BinaryMarketKey::new("KALSHI", "MARKET_B").unwrap();
    let outcomes = BinaryOutcomeInstruments::synthetic_for_market(&other_key);

    let err = BinaryMarket::new(
        key,
        outcomes,
        "Will MARKET_A resolve YES?".to_string(),
        ClaimDescriptor::Text {
            description: "MARKET_A resolves YES.".to_string(),
        },
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        NonZeroOutcomePayout::ONE,
    )
    .unwrap_err();

    assert!(matches!(
        err,
        PredictionError::MismatchedBinaryMarketOutcomes(_)
    ));
}

#[test]
fn binary_market_deserialization_rejects_outcomes_for_different_market_key() {
    let json = r#"{
        "key": {
            "venue": "KALSHI",
            "market_id": "MARKET_A"
        },
        "outcomes": {
            "yes": {
                "venue": "KALSHI",
                "market_id": "MARKET_B",
                "outcome_id": "YES"
            },
            "no": {
                "venue": "KALSHI",
                "market_id": "MARKET_B",
                "outcome_id": "NO"
            }
        },
        "event_key": null,
        "title": "Will MARKET_A resolve YES?",
        "yes_label": null,
        "no_label": null,
        "claim": {
            "kind": "text",
            "description": "MARKET_A resolves YES."
        },
        "status": "open",
        "collateral_currency": {
            "iso": "USD"
        },
        "winning_payout": 1000000,
        "price_grid": null,
        "min_order_quantity": null,
        "open_time": null,
        "close_time": null,
        "settlement_time": null,
        "settlement": null
    }"#;

    assert!(serde_json::from_str::<BinaryMarket>(json).is_err());
}

#[test]
fn multi_outcome_market_constructor_validates_outcomes() {
    let key = PredictionMarketKey::new("MANIFOLD", "contract-1").unwrap();
    let too_few = MultiOutcomeMarket::new(
        key.clone(),
        "Which team wins?".to_string(),
        vec![outcome("contract-1", "A", "Team A")],
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        OutcomePayout::ONE,
    )
    .unwrap_err();
    assert!(matches!(
        too_few,
        PredictionError::TooFewMultiOutcomeMarketOutcomes { count: 1 }
    ));

    let mismatched = MultiOutcomeMarket::new(
        key.clone(),
        "Which team wins?".to_string(),
        vec![
            outcome("contract-1", "A", "Team A"),
            outcome("other-contract", "B", "Team B"),
        ],
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        OutcomePayout::ONE,
    )
    .unwrap_err();
    assert!(matches!(
        mismatched,
        PredictionError::MismatchedMultiOutcomeMarketOutcome(_)
    ));

    let duplicate = MultiOutcomeMarket::new(
        key,
        "Which team wins?".to_string(),
        vec![
            outcome("contract-1", "A", "Team A"),
            outcome("contract-1", "A", "Team A again"),
        ],
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        OutcomePayout::ONE,
    )
    .unwrap_err();
    assert!(matches!(
        duplicate,
        PredictionError::DuplicateMultiOutcomeMarketOutcome { .. }
    ));
}

#[test]
fn multi_outcome_market_resolution_must_reference_listed_outcome() {
    let key = PredictionMarketKey::new("MANIFOLD", "contract-1").unwrap();
    let mut market = MultiOutcomeMarket::new(
        key.clone(),
        "Which team wins?".to_string(),
        vec![
            outcome("contract-1", "A", "Team A"),
            outcome("contract-1", "B", "Team B"),
        ],
        PredictionMarketStatus::Open,
        Currency::Iso(IsoCurrency::USD),
        OutcomePayout::ONE,
    )
    .unwrap();

    let err = market
        .set_resolution(Some(PredictionOutcomeId::new("C").unwrap()))
        .unwrap_err();
    assert!(matches!(
        err,
        PredictionError::InvalidMultiOutcomeMarketResolution { .. }
    ));

    market
        .set_resolution(Some(PredictionOutcomeId::new("B").unwrap()))
        .unwrap();
    assert_eq!(market.key(), &key);
    assert_eq!(market.outcomes().len(), 2);
    assert_eq!(market.resolution().unwrap().as_str(), "B");
}

#[test]
fn multi_outcome_market_deserialization_validates_outcomes_and_resolution() {
    let mismatched = r#"{
        "key": {
            "venue": "MANIFOLD",
            "market_id": "contract-1"
        },
        "event_key": null,
        "title": "Which team wins?",
        "outcomes": [
            {
                "instrument": {
                    "venue": "MANIFOLD",
                    "market_id": "contract-1",
                    "outcome_id": "A"
                },
                "label": "Team A"
            },
            {
                "instrument": {
                    "venue": "MANIFOLD",
                    "market_id": "other-contract",
                    "outcome_id": "B"
                },
                "label": "Team B"
            }
        ],
        "status": "open",
        "collateral_currency": {
            "iso": "USD"
        },
        "unit_payout": 1000000,
        "price_grid": null,
        "min_order_quantity": null,
        "open_time": null,
        "close_time": null,
        "settlement_time": null,
        "resolution": null
    }"#;
    assert!(serde_json::from_str::<MultiOutcomeMarket>(mismatched).is_err());

    let invalid_resolution = r#"{
        "key": {
            "venue": "MANIFOLD",
            "market_id": "contract-1"
        },
        "event_key": null,
        "title": "Which team wins?",
        "outcomes": [
            {
                "instrument": {
                    "venue": "MANIFOLD",
                    "market_id": "contract-1",
                    "outcome_id": "A"
                },
                "label": "Team A"
            },
            {
                "instrument": {
                    "venue": "MANIFOLD",
                    "market_id": "contract-1",
                    "outcome_id": "B"
                },
                "label": "Team B"
            }
        ],
        "status": "open",
        "collateral_currency": {
            "iso": "USD"
        },
        "unit_payout": 1000000,
        "price_grid": null,
        "min_order_quantity": null,
        "open_time": null,
        "close_time": null,
        "settlement_time": null,
        "resolution": "C"
    }"#;
    assert!(serde_json::from_str::<MultiOutcomeMarket>(invalid_resolution).is_err());
}

#[test]
fn prediction_event_uses_full_series_key_reference() {
    let key = PredictionEventKey::new("KALSHI", "KXHIGHNY-24JAN01").unwrap();
    let mut event = PredictionEvent::new(
        key,
        "High temperature in NYC on Jan 1, 2024".to_string(),
        EventStructure::OrderedBuckets { exhaustive: true },
    );
    event.series_key = Some(PredictionSeriesKey::new("KALSHI", "KXHIGHNY").unwrap());

    let value = serde_json::to_value(&event).unwrap();

    assert!(value.get("series_id").is_none());
    assert_eq!(value["series_key"]["venue"], "KALSHI");
    assert_eq!(value["series_key"]["series_id"], "KXHIGHNY");
}

#[test]
fn prediction_event_structure_validation_is_explicit() {
    let mut single = PredictionEvent::new(
        PredictionEventKey::new("KALSHI", "SINGLE").unwrap(),
        "Single market event".to_string(),
        EventStructure::SingleMarket,
    );
    assert!(matches!(
        single.validate_structure(),
        Err(PredictionError::InvalidEventStructure {
            structure: "single_market",
            market_count: 0,
            ..
        })
    ));

    single
        .markets
        .push(PredictionMarket::Binary(binary_market("MKT1")));
    assert!(single.validate_structure().is_ok());

    single
        .markets
        .push(PredictionMarket::Binary(binary_market("MKT2")));
    assert!(matches!(
        single.validate_structure(),
        Err(PredictionError::InvalidEventStructure {
            structure: "single_market",
            market_count: 2,
            ..
        })
    ));

    let mut buckets = PredictionEvent::new(
        PredictionEventKey::new("KALSHI", "BUCKETS").unwrap(),
        "Bucket event".to_string(),
        EventStructure::OrderedBuckets { exhaustive: true },
    );
    buckets
        .markets
        .push(PredictionMarket::Binary(binary_market("BUCKET1")));
    assert!(matches!(
        buckets.validate_structure(),
        Err(PredictionError::InvalidEventStructure {
            structure: "ordered_buckets",
            market_count: 1,
            ..
        })
    ));

    buckets
        .markets
        .push(PredictionMarket::Binary(binary_market("BUCKET2")));
    assert!(buckets.validate_structure().is_ok());
}

#[test]
fn numeric_range_deserialization_validates_interval() {
    let descending = r#"{
        "lower": { "included": "11" },
        "upper": { "included": "10" },
        "unit": "USD"
    }"#;
    assert!(serde_json::from_str::<NumericRange>(descending).is_err());

    let empty_zero_width = r#"{
        "lower": { "excluded": "10" },
        "upper": { "included": "10" },
        "unit": null
    }"#;
    assert!(serde_json::from_str::<NumericRange>(empty_zero_width).is_err());

    let valid = r#"{
        "lower": { "included": "10" },
        "upper": { "excluded": "11" },
        "unit": "USD"
    }"#;
    assert!(serde_json::from_str::<NumericRange>(valid).is_ok());
}

#[test]
fn semantic_metadata_deserialization_rejects_unknown_fields() {
    let event_structure = r#"{
        "kind": "mutually_exclusive",
        "exhaustive": true,
        "provider_hint": "ignored"
    }"#;
    assert!(serde_json::from_str::<EventStructure>(event_structure).is_err());

    let claim = r#"{
        "kind": "text",
        "description": "BTC closes above 100k.",
        "resolution_source": "ignored"
    }"#;
    assert!(serde_json::from_str::<ClaimDescriptor>(claim).is_err());

    let range = r#"{
        "lower": { "included": "10" },
        "upper": { "excluded": "11" },
        "unit": "USD",
        "precision": "ignored"
    }"#;
    assert!(serde_json::from_str::<NumericRange>(range).is_err());
}
