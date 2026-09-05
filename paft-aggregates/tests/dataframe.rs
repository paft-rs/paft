#![cfg(feature = "dataframe")]
use chrono::{TimeZone, Utc};
use paft_aggregates::Snapshot;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Exchange, Instrument, MarketState};
use paft_money::{Currency, IsoCurrency, PriceAmount, QuantityAmount};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: i64) -> PriceAmount {
    PriceAmount::new(Decimal::from(value))
}

fn quantity(value: i64) -> QuantityAmount {
    QuantityAmount::from_decimal(Decimal::from(value)).unwrap()
}

#[test]
fn snapshot_to_dataframe() {
    let snapshot = Snapshot {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )
        .unwrap(),
        name: Some("Apple Inc.".to_string()),
        market_state: Some(MarketState::Regular),
        as_of: Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap()),
        currency: usd(),
        last: Some(amount(150)),
        previous_close: Some(amount(145)),
        open: Some(amount(148)),
        day_high: Some(amount(151)),
        day_low: Some(amount(147)),
        volume: Some(quantity(1_234_567)),

        provider: (),
    };

    let df = snapshot.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn snapshot_vec_to_dataframe() {
    let base = Snapshot {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )
        .unwrap(),
        name: Some("Apple Inc.".to_string()),
        market_state: Some(MarketState::Regular),
        as_of: Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap()),
        currency: usd(),
        last: Some(amount(150)),
        previous_close: Some(amount(145)),
        open: Some(amount(148)),
        day_high: Some(amount(151)),
        day_low: Some(amount(147)),
        volume: Some(quantity(1_000_000)),

        provider: (),
    };

    let snapshots = [
        base.clone(),
        Snapshot {
            name: Some("Alt".to_string()),
            ..base
        },
    ];
    let df = snapshots.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    let columns = df.get_column_names();
    assert!(columns.iter().any(|c| c.as_str() == "instrument.key"));
    assert!(columns.iter().any(|c| c.as_str() == "market_state"));
}

#[test]
fn snapshots_preserve_distinct_identities_with_the_same_display() {
    let instruments = [
        Instrument::from_symbol("BTC", AssetKind::Crypto).unwrap(),
        Instrument::from_symbol("BTC", AssetKind::Equity).unwrap(),
    ];
    let snapshots = instruments
        .each_ref()
        .map(|i| Snapshot::new(i.clone(), usd()));
    let df = snapshots.to_dataframe().unwrap();
    let keys = df.column("instrument.key").unwrap().str().unwrap();
    let labels = df.column("instrument.display").unwrap().str().unwrap();
    assert_ne!(keys.get(0), keys.get(1));
    for (row, instrument) in instruments.iter().enumerate() {
        assert_eq!(keys.get(row), Some(instrument.unique_key().as_str()));
        assert_eq!(labels.get(row), Some("BTC"));
        let single = snapshots[row].to_dataframe().unwrap();
        assert_eq!(single.schema(), df.schema());
        assert_eq!(
            single
                .column("instrument.key")
                .unwrap()
                .str()
                .unwrap()
                .get(0),
            keys.get(row)
        );
    }
    assert_eq!(Snapshot::empty_dataframe().unwrap().schema(), df.schema());
}
