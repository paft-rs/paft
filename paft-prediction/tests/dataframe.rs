#![cfg(feature = "dataframe")]

use paft_prediction::OutcomeInstrument;
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

#[test]
fn outcome_instrument_to_dataframe() {
    let instrument = OutcomeInstrument::new("KALSHI", "KXHIGHNY-24JAN01-T60", "YES").unwrap();
    let df = instrument.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn outcome_instruments_columnar_round_trips_string_cell_values() {
    let instruments = [
        OutcomeInstrument::new("KALSHI", "KXHIGHNY-24JAN01-T60", "YES").unwrap(),
        OutcomeInstrument::new(
            "POLYMARKET",
            "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de",
            "73470541315377973562501025254719659796416871135081220986683321361000395461644",
        )
        .unwrap(),
    ];

    let df = instruments.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);

    let venue = df.column("venue").unwrap().str().unwrap();
    assert_eq!(venue.get(0), Some("KALSHI"));
    assert_eq!(venue.get(1), Some("POLYMARKET"));

    let market_id = df.column("market_id").unwrap().str().unwrap();
    assert_eq!(market_id.get(0), Some("KXHIGHNY-24JAN01-T60"));

    let outcome_id = df.column("outcome_id").unwrap().str().unwrap();
    assert_eq!(outcome_id.get(0), Some("YES"));
}
