use iso_currency::Currency as IsoCurrency;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Exchange, Instrument, MarketState};
use paft_market::market::quote::Quote;
use paft_money::{Currency, Money};

#[must_use]
/// Builds a sample quote for testing.
///
/// # Panics
/// Panics if currency metadata is missing; tests ensure default metadata is available.
pub fn build_quote() -> paft_market::market::quote::Quote {
    paft_market::market::quote::Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(Decimal::from(147), Currency::Iso(IsoCurrency::USD)).unwrap(),
        ),
        day_volume: None,
        open: None,
        day_range_high: None,
        day_range_low: None,
        fifty_two_week_high: None,
        fifty_two_week_low: None,
        average_volume: None,
        market_cap: None,
        shares_outstanding: None,
        eps_ttm: None,
        pe_ttm: None,
        dividend_yield: None,
        ex_dividend_date: None,
        bid: None,
        ask: None,
        forward_dividend: None,
        forward_yield: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    }
}

#[test]
fn quote_construction_smoke() {
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(150), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(
                Decimal::from(1475) / Decimal::from(10),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
        ),
        day_volume: None,
        open: None,
        day_range_high: None,
        day_range_low: None,
        fifty_two_week_high: None,
        fifty_two_week_low: None,
        average_volume: None,
        market_cap: None,
        shares_outstanding: None,
        eps_ttm: None,
        pe_ttm: None,
        dividend_yield: None,
        ex_dividend_date: None,
        bid: None,
        ask: None,
        forward_dividend: None,
        forward_yield: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };
    assert_eq!(quote.instrument.unique_key().as_ref(), "AAPL");
}
