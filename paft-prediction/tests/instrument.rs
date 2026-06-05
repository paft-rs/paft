use paft_prediction::{
    BinaryMarketKey, OutcomeInstrument, PredictionEventKey, PredictionMarketKey,
};

#[test]
fn market_keys_are_venue_namespaced() {
    let kalshi = BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60").unwrap();
    let other_venue = BinaryMarketKey::new("OTHER-VENUE", "KXHIGHNY-24JAN01-T60").unwrap();

    assert_ne!(kalshi.unique_key(), other_venue.unique_key());
    assert_eq!(kalshi.to_string(), "KALSHI:KXHIGHNY-24JAN01-T60");
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
