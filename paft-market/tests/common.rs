use iso_currency::Currency as IsoCurrency;
use paft_core::domain::{Currency, Exchange, MarketState, Money};
use paft_market::market::quote::Quote;
use rust_decimal::Decimal;

#[must_use]
pub fn build_quote() -> paft_market::market::quote::Quote {
    paft_market::market::quote::Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Money::new(
            Decimal::from(147),
            Currency::Iso(IsoCurrency::USD),
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    }
}

#[test]
fn quote_construction_smoke() {
    let quote = Quote {
        symbol: "AAPL".to_string(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(
            Decimal::from(150),
            Currency::Iso(IsoCurrency::USD),
        )),
        previous_close: Some(Money::new(
            Decimal::from(1475) / Decimal::from(10),
            Currency::Iso(IsoCurrency::USD),
        )),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };
    assert_eq!(quote.symbol, "AAPL");
}
