use paft_decimal::Decimal;
use paft_domain::{AssetKind, Exchange, Instrument, MarketState};
use paft_market::market::quote::Quote;
use paft_money::{Currency, IsoCurrency, PriceAmount};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: impl Into<Decimal>) -> PriceAmount {
    PriceAmount::new(value.into())
}

#[must_use]
/// Builds a sample quote for testing.
///
/// # Panics
/// Panics if currency metadata is missing; tests ensure default metadata is available.
pub fn build_quote() -> paft_market::market::quote::Quote {
    paft_market::market::quote::Quote {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )
        .unwrap(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(147)),
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    }
}

#[test]
fn quote_construction_smoke() {
    let quote = Quote {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )
        .unwrap(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(PriceAmount::new(Decimal::from(1475) / Decimal::from(10))),
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };
    assert_eq!(
        quote.instrument.unique_key().as_ref(),
        "EQUITY|SYMBOL|4:AAPL|EXCHANGE|NASDAQ"
    );
}
