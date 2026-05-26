use paft_prediction::PredictionInstrument;

const EVENT_A: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
const EVENT_B: &str = "0x2222222222222222222222222222222222222222222222222222222222222222";
const OUTCOME: &str = "42";

#[test]
fn unique_key_uses_event_and_outcome_ids() {
    let instrument = PredictionInstrument::new(EVENT_A, OUTCOME).unwrap();
    let expected = format!("{EVENT_A}/{OUTCOME}");

    assert_eq!(instrument.unique_key().as_ref(), expected.as_str());
}

#[test]
fn same_outcome_id_in_different_events_gets_distinct_unique_keys() {
    let first = PredictionInstrument::new(EVENT_A, OUTCOME).unwrap();
    let second = PredictionInstrument::new(EVENT_B, OUTCOME).unwrap();

    assert_ne!(first.unique_key(), second.unique_key());
}

#[test]
fn display_uses_unique_key() {
    let instrument = PredictionInstrument::new(EVENT_A, OUTCOME).unwrap();

    assert_eq!(instrument.to_string(), instrument.unique_key());
}
