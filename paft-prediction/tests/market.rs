use paft_money::{Currency, IsoCurrency};
use paft_prediction::{
    BinaryMarket, BinaryMarketKey, BinaryOutcomeInstruments, ClaimDescriptor, EventStructure,
    NonZeroContractQuantity, OutcomeInstrument, OutcomePayout, PredictionEventKey,
    PredictionMarketStatus, PredictionSeriesKey,
};
use paft_prediction::{NumericBound, NumericRange, PredictionEvent};

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
        OutcomePayout::ONE,
    );

    assert_eq!(
        market.outcomes.yes().outcome_id.as_str(),
        "73470541315377973562501025254719659796416871135081220986683321361000395461644"
    );
    assert_eq!(
        market.outcomes.no().outcome_id.as_str(),
        "56393761733830483601097051857899348522495376869600726893014309766300892311293"
    );
    assert_eq!(
        market.outcomes.yes().market_key(),
        market.key.to_market_key()
    );
    assert_eq!(
        market.outcomes.no().market_key(),
        market.key.to_market_key()
    );
}

#[test]
fn numeric_range_constructor_validates_interval() {
    let valid_single_point = NumericRange::new(
        NumericBound::Included(10.into()),
        NumericBound::Included(10.into()),
        Some("USD".to_string()),
    );
    assert!(valid_single_point.is_ok());

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
        OutcomePayout::ONE,
    );
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
        OutcomePayout::ONE,
    );
    market.event_key = Some(PredictionEventKey::new("KALSHI", "KXHIGHNY-24JAN01").unwrap());

    let value = serde_json::to_value(&market).unwrap();

    assert!(value.get("event_id").is_none());
    assert_eq!(value["event_key"]["venue"], "KALSHI");
    assert_eq!(value["event_key"]["event_id"], "KXHIGHNY-24JAN01");
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
