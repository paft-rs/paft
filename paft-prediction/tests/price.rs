use paft_prediction::{
    ContractQuantity, OutcomePayout, OutcomePrice, PriceBand, PriceGrid, PriceTick,
};

#[test]
fn outcome_price_uses_micro_unit_scale_and_complements() {
    let price = OutcomePrice::from_micros(250_000).unwrap();
    assert_eq!(price.micros(), 250_000);
    assert_eq!(price.complement().micros(), 750_000);
    assert_eq!(
        OutcomePrice::from_micros(333_333)
            .unwrap()
            .midpoint(OutcomePrice::from_micros(666_667).unwrap())
            .micros(),
        500_000
    );
}

#[test]
fn outcome_price_and_tick_validate_through_serde() {
    assert!(OutcomePrice::from_micros(1_000_001).is_err());
    assert!(serde_json::from_str::<OutcomePrice>("1000001").is_err());

    assert!(PriceTick::from_micros(0).is_err());
    assert!(PriceTick::from_micros(1_000_001).is_err());
    assert!(serde_json::from_str::<PriceTick>("0").is_err());

    let tick: PriceTick = serde_json::from_str("100").unwrap();
    assert_eq!(tick.micros(), 100);
}

#[test]
fn price_grid_validates_piecewise_ticks() {
    let grid = PriceGrid::new(vec![
        PriceBand {
            start: OutcomePrice::ZERO,
            end: OutcomePrice::from_micros(100_000).unwrap(),
            tick: PriceTick::from_micros(1_000).unwrap(),
        },
        PriceBand {
            start: OutcomePrice::from_micros(100_001).unwrap(),
            end: OutcomePrice::ONE,
            tick: PriceTick::from_micros(100).unwrap(),
        },
    ])
    .unwrap();

    assert!(grid.contains_price(OutcomePrice::from_micros(50_000).unwrap()));
    assert!(grid.contains_price(OutcomePrice::from_micros(100_101).unwrap()));
    assert!(!grid.contains_price(OutcomePrice::from_micros(100_150).unwrap()));
}

#[test]
fn price_grid_deserialization_validates_invariants() {
    let empty = r#"{"bands":[]}"#;
    assert!(serde_json::from_str::<PriceGrid>(empty).is_err());

    let descending_band = r#"{
        "bands": [
            { "start": 100000, "end": 0, "tick": 1000 }
        ]
    }"#;
    assert!(serde_json::from_str::<PriceGrid>(descending_band).is_err());

    let overlapping = r#"{
        "bands": [
            { "start": 0, "end": 100000, "tick": 1000 },
            { "start": 100000, "end": 200000, "tick": 1000 }
        ]
    }"#;
    assert!(serde_json::from_str::<PriceGrid>(overlapping).is_err());

    let valid = r#"{
        "bands": [
            { "start": 0, "end": 100000, "tick": 1000 },
            { "start": 100001, "end": 200000, "tick": 1000 }
        ]
    }"#;
    assert!(serde_json::from_str::<PriceGrid>(valid).is_ok());
}

#[test]
fn quantity_and_payout_are_transparent_fixed_integers() {
    let quantity = ContractQuantity::from_microcontracts(219_217_767);
    let payout = OutcomePayout::from_micropayouts(1_000_000);

    assert_eq!(quantity.microcontracts(), 219_217_767);
    assert_eq!(payout.micropayouts(), OutcomePayout::SCALE);
    assert!(!quantity.is_zero());
}
