#![cfg(feature = "dataframe")]
use paft_prediction::PredictionInstrument;
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

const EVENT_A: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
const EVENT_B: &str = "0x2222222222222222222222222222222222222222222222222222222222222222";

#[test]
fn prediction_instrument_to_dataframe() {
    let instrument = PredictionInstrument::new(EVENT_A, "42").unwrap();
    let df = instrument.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn prediction_instruments_columnar_round_trips_string_cell_values() {
    // Multi-row case that exercises the iterator-based string column
    // construction (`from_iter_values`). Verifies the actual byte values land
    // in the DataFrame, not just that columns of the right names exist.
    let instruments = [
        PredictionInstrument::new(EVENT_A, "1").unwrap(),
        PredictionInstrument::new(EVENT_B, "9876543210").unwrap(),
    ];

    let df = instruments.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);

    let event_id = df.column("event_id").unwrap().str().unwrap();
    assert_eq!(event_id.get(0), Some(EVENT_A));
    assert_eq!(event_id.get(1), Some(EVENT_B));

    let outcome_id = df.column("outcome_id").unwrap().str().unwrap();
    assert_eq!(outcome_id.get(0), Some("1"));
    assert_eq!(outcome_id.get(1), Some("9876543210"));
}
