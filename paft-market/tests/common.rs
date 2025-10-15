use iso_currency::Currency as IsoCurrency;
use paft_domain::{Exchange, MarketState, Symbol};
use paft_market::market::quote::Quote;
use paft_money::{Currency, Decimal, Money};

#[must_use]
/// Builds a sample quote for testing.
///
/// # Panics
/// Panics if currency metadata is missing; tests ensure default metadata is available.
pub fn build_quote() -> paft_market::market::quote::Quote {
    paft_market::market::quote::Quote {
        symbol: Symbol::new("AAPL").unwrap(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(Decimal::from(147), Currency::Iso(IsoCurrency::USD)).unwrap(),
        ),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    }
}

#[test]
fn quote_construction_smoke() {
    let quote = Quote {
        symbol: Symbol::new("AAPL").unwrap(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(
                Decimal::from(1475) / Decimal::from(10),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
        ),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };
    assert_eq!(quote.symbol.as_str(), "AAPL");
}
