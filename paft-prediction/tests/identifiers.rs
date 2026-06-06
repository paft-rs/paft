use paft_prediction::{
    MAX_PREDICTION_ID_LEN, PredictionEventId, PredictionMarketId, PredictionOutcomeId,
    PredictionSeriesId, PredictionVenue,
};

#[test]
fn opaque_ids_accept_provider_native_shapes_and_preserve_case() {
    let kalshi_market = PredictionMarketId::new("  KXHIGHNY-24JAN01-T60  ").unwrap();
    assert_eq!(kalshi_market.as_str(), "KXHIGHNY-24JAN01-T60");

    let polymarket_condition =
        PredictionEventId::new("0x5EED579ff6763914d78a966c83473ba2485ac8910d0a0914").unwrap();
    assert_eq!(
        polymarket_condition.as_str(),
        "0x5EED579ff6763914d78a966c83473ba2485ac8910d0a0914"
    );

    let manifold_contract = PredictionMarketId::new("uX5aCaseSensitiveId").unwrap();
    assert_eq!(manifold_contract.as_str(), "uX5aCaseSensitiveId");

    let yes = PredictionOutcomeId::new("YES").unwrap();
    let no = PredictionOutcomeId::new("NO").unwrap();
    let clob_token = PredictionOutcomeId::new(
        "73470541315377973562501025254719659796416871135081220986683321361000395461644",
    )
    .unwrap();

    assert_eq!(yes.as_str(), "YES");
    assert_eq!(no.as_str(), "NO");
    assert!(clob_token.as_str().chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn opaque_ids_reject_empty_whitespace_control_and_embedded_whitespace() {
    assert!(PredictionEventId::new("").is_err());
    assert!(PredictionEventId::new("   ").is_err());
    assert!(PredictionEventId::new("event\nid").is_err());
    assert!(PredictionEventId::new("event id").is_err());
    assert!(PredictionEventId::new("event\tid").is_err());

    let too_long = "x".repeat(MAX_PREDICTION_ID_LEN + 1);
    assert!(PredictionSeriesId::new(&too_long).is_err());
}

#[test]
fn identifiers_serde_as_plain_strings_and_validate_on_input() {
    let id = PredictionOutcomeId::new("YES").unwrap();
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"YES\"");

    let round_trip: PredictionOutcomeId = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, id);

    assert!(serde_json::from_str::<PredictionOutcomeId>("\"bad id\"").is_err());
}

#[test]
fn venue_parses_known_values_and_preserves_unknown_values() {
    assert_eq!(
        "kalshi".parse::<PredictionVenue>().unwrap().as_str(),
        "KALSHI"
    );
    assert_eq!(
        "POLYMARKET".parse::<PredictionVenue>().unwrap().as_str(),
        "POLYMARKET"
    );
    assert_eq!(
        "Manifold".parse::<PredictionVenue>().unwrap().as_str(),
        "MANIFOLD"
    );

    let other = "FutureVenue-v2".parse::<PredictionVenue>().unwrap();
    assert_eq!(other.as_str(), "FutureVenue-v2");

    assert!(PredictionVenue::other("Kalshi").is_err());
    assert!("bad venue".parse::<PredictionVenue>().is_err());
}
