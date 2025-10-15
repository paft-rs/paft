// This test requires both domain and market features
#![cfg(all(feature = "domain", feature = "market"))]

use chrono::DateTime;
use chrono_tz::Tz;
use iso_currency::Currency as IsoCurrency;
use paft::market::MarketError;
use paft::prelude::{
    Action, AssetKind, Candle, Currency, Exchange, HistoryMeta, HistoryRequest, HistoryResponse,
    Instrument, Interval, MarketState, Money, Quote, QuoteUpdate, Range, SearchRequest, Symbol,
};
use paft_money::Decimal;
use std::str::FromStr;

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
    let instrument = Instrument::try_new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4"),
        None,
        Some(Exchange::NASDAQ),
    )
    .expect("valid instrument");
    assert_eq!(instrument.symbol().as_str(), "AAPL");
    assert_eq!(instrument.kind(), &AssetKind::Equity);

    // 3. Create a history request
    let _history_req = HistoryRequest::builder()
        .range(Range::D1)
        .interval(Interval::D1)
        .include_actions(true)
        .auto_adjust(true)
        .build()
        .unwrap();

    // 4. Create sample history data
    let candle = Candle {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        open: Money::new(
            Decimal::from_str("100.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        high: Money::new(
            Decimal::from_str("110.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        low: Money::new(
            Decimal::from_str("95.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        close: Money::new(
            Decimal::from_str("105.0").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
        close_unadj: None,
        volume: Some(1_000_000),
    };

    let action = Action::Dividend {
        ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        amount: Money::new(
            Decimal::from_str("0.5").unwrap(),
            Currency::Iso(IsoCurrency::USD),
        )
        .unwrap(),
    };

    let meta = HistoryMeta {
        timezone: Some("America/New_York".parse::<Tz>().unwrap()),
        utc_offset_seconds: Some(-18000),
    };

    let history_response = HistoryResponse {
        candles: vec![candle],
        actions: vec![action],
        adjusted: true,
        meta: Some(meta),
    };

    // 5. Create a quote
    let quote = Quote {
        symbol: Symbol::new("AAPL").unwrap(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(Money::new(Decimal::from(105), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap(),
        ),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
    };

    // 6. Create a quote update
    let quote_update = QuoteUpdate {
        symbol: Symbol::new("AAPL").unwrap(),
        price: Some(Money::new(Decimal::from(106), Currency::Iso(IsoCurrency::USD)).unwrap()),
        previous_close: Some(
            Money::new(Decimal::from(100), Currency::Iso(IsoCurrency::USD)).unwrap(),
        ),
        ts: DateTime::from_timestamp(1_640_995_260, 0).unwrap(),
    };

    // Verify all data is consistent
    assert_eq!(quote.symbol.as_str(), instrument.symbol().as_str());
    assert_eq!(quote_update.symbol.as_str(), instrument.symbol().as_str());
    assert_eq!(history_response.candles[0].close, quote.price.unwrap());
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
        assert_eq!(start, 2000);
        assert_eq!(end, 1000);
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
        .auto_adjust(true)
        .build()
        .unwrap();

    let history_json = serde_json::to_string(&history_req).unwrap();
    let deserialized_history: HistoryRequest = serde_json::from_str(&history_json).unwrap();
    assert_eq!(history_req, deserialized_history);

    // 3. Create and serialize a quote
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
        let instrument = Instrument::try_new(
            "TEST",
            asset_kind,
            Some("BBG000B9XRY4"),
            None,
            Some(Exchange::try_from_str("TEST").unwrap()),
        )
        .expect("valid instrument for asset kind");
        assert_eq!(instrument.kind(), &asset_kind);

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
            amount: Money::new(
                Decimal::from_str("0.5").unwrap(),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
        },
        Action::Split {
            ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            numerator: 2,
            denominator: 1,
        },
        Action::CapitalGain {
            ts: DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
            gain: Money::new(
                Decimal::from_str("1.0").unwrap(),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
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
            adjusted: false,
            meta: None,
        };

        let json = serde_json::to_string(&history_response).unwrap();
        let deserialized: HistoryResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(history_response, deserialized);
    }
}
