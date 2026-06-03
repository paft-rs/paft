// This test requires both domain and market features
#![cfg(all(feature = "domain", feature = "market"))]

use chrono::DateTime;
use chrono_tz::Tz;
use iso_currency::Currency as IsoCurrency;
use paft::market::MarketError;
use paft::prelude::{
    Action, AssetKind, Candle, Currency, Exchange, HistoryMeta, HistoryRequest, HistoryResponse,
    Instrument, Interval, MarketState, Ohlc, OhlcPriceBasis, Price, PriceAmount, PriceBasis, Quote,
    QuoteUpdate, Range, SearchRequest,
};
use paft_decimal::Decimal;
use std::num::NonZeroU32;
use std::str::FromStr;

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: impl Into<Decimal>) -> PriceAmount {
    PriceAmount::new(value.into())
}

fn ohlc(open: &str, high: &str, low: &str, close: &str) -> Ohlc {
    Ohlc::new(
        amount(Decimal::from_str(open).unwrap()),
        amount(Decimal::from_str(high).unwrap()),
        amount(Decimal::from_str(low).unwrap()),
        amount(Decimal::from_str(close).unwrap()),
    )
}

#[cfg(all(feature = "domain", feature = "market"))]
#[test]
fn end_to_end_workflow() {
    // Test a complete workflow from search to history to quote updates

    // 1. Create a search request
    let _search_req = SearchRequest::builder("AAPL")
        .kind(AssetKind::Equity)
        .limit(10)
        .build()
        .unwrap();

    // 2. Create an instrument from search results
    let instrument =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity)
            .expect("valid instrument");
    let inst_symbol = instrument.symbol.as_str();
    assert_eq!(inst_symbol, "AAPL");
    assert_eq!(instrument.kind, AssetKind::Equity);

    // 3. Create a history request
    let _history_req = HistoryRequest::builder()
        .range(Range::D1)
        .interval(Interval::D1)
        .include_actions(true)
        .prefer_adjusted_prices(true)
        .build()
        .unwrap();

    // 4. Create sample history data
    let candle = Candle {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        currency: usd(),
        ohlc: ohlc("100.0", "110.0", "95.0", "105.0"),
        close_unadj: None,
        volume: Some(1_000_000),

        provider: (),
    };

    let action = Action::Dividend {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        amount: Price::new(Decimal::from_str("0.5").unwrap(), usd()),
    };

    let meta = HistoryMeta {
        timezone: Some("America/New_York".parse::<Tz>().unwrap()),
        utc_offset_seconds: Some(-18000),
    };

    let history_response = HistoryResponse {
        candles: vec![candle],
        actions: vec![action],
        price_basis: OhlcPriceBasis::uniform(PriceBasis::provider_latest_adjusted()),
        meta: Some(meta),
        provider: (),
    };

    // 5. Create a quote
    let quote = Quote {
        instrument: instrument.clone(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(105)),
        previous_close: Some(amount(100)),
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    // 6. Create a quote update
    let quote_update = QuoteUpdate {
        instrument: instrument.clone(),
        currency: usd(),
        price: Some(amount(106)),
        previous_close: Some(amount(100)),
        volume: None,
        ts: DateTime::from_timestamp(1_640_995_260, 0).unwrap(),

        provider: (),
    };

    // Verify all data is consistent
    let inst_symbol = instrument.symbol.as_str();
    assert_eq!(quote.instrument.symbol.as_str(), inst_symbol);
    assert_eq!(quote_update.instrument.symbol.as_str(), inst_symbol);
    assert_eq!(history_response.candles[0].ohlc.close, quote.price.unwrap());
}

#[cfg(all(feature = "domain", feature = "market"))]
#[test]
fn error_handling_workflow() {
    // Test error handling in a realistic workflow

    // 1. Invalid search request
    let result = SearchRequest::builder("").limit(0).build();
    assert!(result.is_err());

    if result == Err(MarketError::EmptySearchQuery) {
        // The validation correctly identifies empty search query
    } else {
        panic!("Expected EmptySearchQuery error, got: {result:?}");
    }

    // 2. Invalid history request - test invalid period
    let result = HistoryRequest::builder()
        .period(
            DateTime::from_timestamp(2000, 0).unwrap(),
            DateTime::from_timestamp(1000, 0).unwrap(),
        ) // Invalid period: start >= end
        .build();
    assert!(result.is_err());

    if let Err(MarketError::InvalidPeriod { start, end }) = result {
        assert_eq!(start, 2_000_000);
        assert_eq!(end, 1_000_000);
    } else {
        panic!("Expected InvalidPeriod error, got: {result:?}");
    }

    // 3. Test that builder prevents invalid states by design
    // The builder replaces range with period (enum approach ensures mutual exclusivity at compile time)
    let result = HistoryRequest::builder()
        .range(Range::D1)
        .period(
            DateTime::from_timestamp(1000, 0).unwrap(),
            DateTime::from_timestamp(2000, 0).unwrap(),
        ) // This replaces the range with a period
        .build();

    assert!(result.is_ok());
    let req = result.unwrap();
    assert_eq!(req.range(), None);
    let (s, e) = req.period().unwrap();
    assert_eq!(s.timestamp(), 1000);
    assert_eq!(e.timestamp(), 2000);
}

#[cfg(all(feature = "domain", feature = "market"))]
#[test]
fn serialization_workflow() {
    // Test serialization in a realistic workflow

    // 1. Create and serialize a search request
    let search_req = SearchRequest::builder("AAPL")
        .kind(AssetKind::Equity)
        .build()
        .unwrap();
    let search_json = serde_json::to_string(&search_req).unwrap();
    let deserialized_search: SearchRequest = serde_json::from_str(&search_json).unwrap();
    assert_eq!(search_req, deserialized_search);

    // 2. Create and serialize a history request
    let history_req = HistoryRequest::builder()
        .range(Range::D1)
        .interval(Interval::D1)
        .include_actions(true)
        .prefer_adjusted_prices(true)
        .build()
        .unwrap();

    let history_json = serde_json::to_string(&history_req).unwrap();
    let deserialized_history: HistoryRequest = serde_json::from_str(&history_json).unwrap();
    assert_eq!(history_req, deserialized_history);

    // 3. Create and serialize a quote
    let quote = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(amount(150)),
        previous_close: Some(amount(Decimal::from(1475) / Decimal::from(10))),
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    let quote_json = serde_json::to_string(&quote).unwrap();
    let deserialized_quote: Quote = serde_json::from_str(&quote_json).unwrap();
    assert_eq!(quote, deserialized_quote);
}

#[cfg(all(feature = "domain", feature = "market"))]
#[test]
fn asset_kind_workflow() {
    // Test AssetKind usage in a realistic workflow

    let asset_kinds = [
        AssetKind::Equity,
        AssetKind::Crypto,
        AssetKind::Fund,
        AssetKind::Index,
        AssetKind::Forex,
        AssetKind::Bond,
        AssetKind::Commodity,
        AssetKind::Option,
        AssetKind::Future,
        AssetKind::REIT,
        AssetKind::Warrant,
        AssetKind::Convertible,
        AssetKind::NFT,
        AssetKind::PerpetualFuture,
        AssetKind::LeveragedToken,
        AssetKind::LPToken,
        AssetKind::LST,
        AssetKind::RWA,
    ];

    // Test that each asset kind can be used in an instrument
    for asset_kind in asset_kinds {
        let instrument = Instrument::from_symbol_and_exchange(
            "TEST",
            Exchange::try_from_str("TEST").unwrap(),
            asset_kind.clone(),
        )
        .expect("valid instrument for asset kind");
        assert_eq!(instrument.kind, asset_kind);

        // Test serialization
        let json = serde_json::to_string(&instrument).unwrap();
        let deserialized: Instrument = serde_json::from_str(&json).unwrap();
        assert_eq!(instrument, deserialized);
    }
}

#[cfg(all(feature = "domain", feature = "market"))]
#[test]
fn interval_and_range_workflow() {
    // Test Interval and Range usage in a realistic workflow

    let intervals = [
        Interval::I1m,
        Interval::I5m,
        Interval::I15m,
        Interval::I30m,
        Interval::I1h,
        Interval::D1,
        Interval::W1,
        Interval::M1,
    ];

    let ranges = [
        Range::D1,
        Range::D5,
        Range::M1,
        Range::M3,
        Range::M6,
        Range::Y1,
        Range::Y2,
        Range::Y5,
        Range::Y10,
        Range::Ytd,
        Range::Max,
    ];

    // Test that each interval can be used in a history request
    for interval in intervals {
        let history_req = HistoryRequest::builder()
            .range(Range::D1)
            .interval(interval)
            .build()
            .unwrap();

        // Test serialization
        let json = serde_json::to_string(&history_req).unwrap();
        let deserialized: HistoryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(history_req, deserialized);
    }

    // Test that each range can be used in a history request
    for range in ranges {
        let history_req = HistoryRequest::builder()
            .range(range)
            .interval(Interval::D1)
            .build()
            .unwrap();

        // Test serialization
        let json = serde_json::to_string(&history_req).unwrap();
        let deserialized: HistoryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(history_req, deserialized);
    }
}

#[cfg(all(feature = "domain", feature = "market"))]
#[test]
fn action_types_workflow() {
    // Test all action types in a realistic workflow

    let actions = [
        Action::Dividend {
            ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            amount: Price::new(Decimal::from_str("0.5").unwrap(), usd()),
        },
        Action::Split {
            ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            numerator: NonZeroU32::new(2).unwrap(),
            denominator: NonZeroU32::new(1).unwrap(),
        },
        Action::CapitalGain {
            ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            gain: Price::new(Decimal::from_str("1.0").unwrap(), usd()),
        },
    ];

    for action in actions {
        // Test serialization
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);

        // Test that action can be used in history response
        let history_response = HistoryResponse {
            candles: vec![],
            actions: vec![action],
            price_basis: OhlcPriceBasis::raw(),
            meta: None,
            provider: (),
        };

        let json = serde_json::to_string(&history_response).unwrap();
        let deserialized: HistoryResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(history_response, deserialized);
    }
}
