#![cfg(feature = "dataframe")]
use chrono::{TimeZone, Utc};
use paft_aggregates::Snapshot;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Exchange, Instrument, MarketState};
use paft_money::{Currency, IsoCurrency, Price};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

fn usd(amount: i64) -> Price {
    Price::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD))
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
        last: Some(usd(150)),
        previous_close: Some(usd(145)),
        open: Some(usd(148)),
        day_high: Some(usd(151)),
        day_low: Some(usd(147)),
        volume: Some(1_234_567),

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
        last: Some(usd(150)),
        previous_close: Some(usd(145)),
        open: Some(usd(148)),
        day_high: Some(usd(151)),
        day_low: Some(usd(147)),
        volume: Some(1_000_000),

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
    assert!(columns.iter().any(|c| c.as_str() == "instrument"));
    assert!(columns.iter().any(|c| c.as_str() == "market_state"));
}
