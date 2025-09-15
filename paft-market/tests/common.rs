use paft_core::domain::{Currency, Exchange, MarketState, Money};
use paft_market::market::quote::Quote;
use rust_decimal::Decimal;

#[test]
fn quote_construction_smoke() {
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::USD)),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::USD,
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };
    assert_eq!(quote.symbol, "AAPL");
}
