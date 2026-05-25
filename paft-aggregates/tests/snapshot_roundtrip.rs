use chrono::{TimeZone, Utc};
use paft_aggregates::Snapshot;
use paft_domain::{AssetKind, Exchange, Instrument, MarketState};
use paft_money::IsoCurrency;
use paft_money::{Currency, Price};
use pretty_assertions::assert_eq;

#[test]
fn snapshot_roundtrip_minimal() {
    let snapshot = Snapshot {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: None,
        exchange: None,
        currency: None,
        market_state: None,
        as_of: None,
        last: None,
        previous_close: None,
        open: None,
        day_high: None,
        day_low: None,
        volume: None,

        provider: (),
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    let back: Snapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(back, snapshot);
}

#[test]
fn snapshot_roundtrip_full() {
    let usd = Currency::Iso(IsoCurrency::USD);
    let snapshot = Snapshot {
        instrument: Instrument::from_symbol("MSFT", AssetKind::Equity).unwrap(),
        name: Some("Microsoft Corporation".to_string()),
        exchange: Some(Exchange::NASDAQ),
        currency: Some(usd.clone()),
        market_state: Some(MarketState::Pre),
        as_of: Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap()),
        last: Some(Price::from_canonical_str("430.01", usd.clone()).unwrap()),
        previous_close: Some(Price::from_canonical_str("429.50", usd.clone()).unwrap()),
        open: Some(Price::from_canonical_str("428.00", usd.clone()).unwrap()),
        day_high: Some(Price::from_canonical_str("432.22", usd.clone()).unwrap()),
        day_low: Some(Price::from_canonical_str("427.80", usd).unwrap()),
        volume: Some(25_000_000),

        provider: (),
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    let back: Snapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(back, snapshot);
}
