use chrono::{TimeZone, Utc};
use paft_aggregates::Snapshot;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Exchange, Instrument, MarketState};
use paft_money::{Currency, IsoCurrency, PriceAmount, QuantityAmount};
use pretty_assertions::assert_eq;
use std::str::FromStr;

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: &str) -> PriceAmount {
    PriceAmount::new(Decimal::from_str(value).unwrap())
}

fn quantity(value: &str) -> QuantityAmount {
    QuantityAmount::from_decimal(Decimal::from_str(value).unwrap()).unwrap()
}

#[test]
fn snapshot_roundtrip_minimal() {
    let snapshot = Snapshot {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        name: None,
        market_state: None,
        as_of: None,
        currency: usd(),
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
    let snapshot = Snapshot {
        instrument: Instrument::from_symbol_and_exchange(
            "MSFT",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )
        .unwrap(),
        name: Some("Microsoft Corporation".to_string()),
        market_state: Some(MarketState::Pre),
        as_of: Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap()),
        currency: usd(),
        last: Some(amount("430.01")),
        previous_close: Some(amount("429.50")),
        open: Some(amount("428.00")),
        day_high: Some(amount("432.22")),
        day_low: Some(amount("427.80")),
        volume: Some(quantity("25000000.5")),

        provider: (),
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["volume"], serde_json::json!("25000000.5"));
    let back: Snapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(back, snapshot);
}
