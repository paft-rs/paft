use paft_money::{Currency, IsoCurrency};
use paft_prediction::{
    BinaryMarket, BinaryMarketKey, BinaryOutcomeInstruments, ClaimDescriptor, OutcomeInstrument,
    OutcomePayout, PredictionMarketStatus,
};

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
    );

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
        market.outcomes.yes.outcome_id.as_str(),
        "73470541315377973562501025254719659796416871135081220986683321361000395461644"
    );
    assert_eq!(
        market.outcomes.no.outcome_id.as_str(),
        "56393761733830483601097051857899348522495376869600726893014309766300892311293"
    );
    assert_eq!(market.outcomes.yes.market_key(), market.key.to_market_key());
    assert_eq!(market.outcomes.no.market_key(), market.key.to_market_key());
}
