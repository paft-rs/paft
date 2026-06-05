use paft_prediction::{
    BinaryMarketKey, BinaryOutcomeInstruments, OutcomeInstrument, PredictionEventKey,
    PredictionMarketKey,
};

#[test]
fn market_keys_are_venue_namespaced() {
    let kalshi = BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap();
    let other_venue = BinaryMarketKey::new("OTHER-VENUE", "KXHIGHNY-24JAN01-T60").unwrap();

    assert_ne!(kalshi.unique_key(), other_venue.unique_key());
    assert_eq!(kalshi.to_string(), "KALSHI:KXHIGHNY-24JAN01-T60");
    assert_eq!(kalshi.unique_key(), kalshi.to_market_key().unique_key());
}

#[test]
fn outcome_instrument_accepts_yes_no_and_clob_token_ids() {
    let kalshi_yes = OutcomeInstrument::new("KALSHI", "KXHIGHNY-24JAN01-T60", "YES").unwrap();
    assert_eq!(kalshi_yes.to_string(), "KALSHI:KXHIGHNY-24JAN01-T60/YES");

    let polymarket = OutcomeInstrument::new(
        "POLYMARKET",
        "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de",
        "73470541315377973562501025254719659796416871135081220986683321361000395461644",
    )
    .unwrap();

    assert_ne!(kalshi_yes.unique_key(), polymarket.unique_key());
    assert_eq!(polymarket.market_key().venue.as_str(), "POLYMARKET");
}

#[test]
fn event_and_market_keys_have_distinct_unique_key_roles() {
    let event = PredictionEventKey::new("POLYMARKET", "btc-price").unwrap();
    let market = PredictionMarketKey::new("POLYMARKET", "btc-price").unwrap();

    assert_ne!(event.unique_key(), market.unique_key());
}

#[test]
fn unique_keys_length_prefix_every_dynamic_component() {
    let first = OutcomeInstrument::new("A|market:1:B", "C", "D").unwrap();
    let second = OutcomeInstrument::new("A", "B|market:1:C", "D").unwrap();

    assert_ne!(first.unique_key(), second.unique_key());

    let first_event = PredictionEventKey::new("A|event:1:B", "C").unwrap();
    let second_event = PredictionEventKey::new("A", "B|event:1:C").unwrap();

    assert_ne!(first_event.unique_key(), second_event.unique_key());
}

#[test]
fn binary_market_key_builds_synthetic_yes_no_instruments() {
    let key = BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap();

    let yes = key.yes_instrument();
    let no = key.no_instrument();

    assert_eq!(yes.to_string(), "KALSHI:KXHIGHNY-24JAN01-T60/YES");
    assert_eq!(no.to_string(), "KALSHI:KXHIGHNY-24JAN01-T60/NO");
    assert_eq!(yes.market_key(), key.to_market_key());
    assert_ne!(yes.unique_key(), no.unique_key());
}

#[test]
fn binary_outcome_instruments_can_be_synthetic_for_market_key() {
    let key = BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap();
    let outcomes = BinaryOutcomeInstruments::synthetic_for_market(&key);

    assert_eq!(outcomes.yes.outcome_id.as_str(), "YES");
    assert_eq!(outcomes.no.outcome_id.as_str(), "NO");
}
