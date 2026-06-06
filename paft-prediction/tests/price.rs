use paft_prediction::{
    ContractQuantity, NonZeroContractQuantity, NonZeroOutcomePayout, OutcomePayout, OutcomePrice,
    PriceBand, PriceGrid, PriceTick,
};
use std::mem::size_of;

fn decimal(value: &str) -> paft_decimal::Decimal {
    paft_decimal::parse_decimal(value).unwrap()
}

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
fn outcome_price_parses_exact_decimal_values() {
    assert_eq!(
        OutcomePrice::from_canonical_str("0.41").unwrap().micros(),
        410_000
    );
    assert_eq!(
        OutcomePrice::from_canonical_str("0.999").unwrap().micros(),
        999_000
    );
    assert_eq!(
        OutcomePrice::from_canonical_str("0.0001").unwrap().micros(),
        100
    );
    assert_eq!(
        OutcomePrice::from_canonical_str("1.0000000")
            .unwrap()
            .micros(),
        OutcomePrice::SCALE
    );
    assert_eq!(
        OutcomePrice::from_decimal(decimal("0.41"))
            .unwrap()
            .micros(),
        410_000
    );
    assert_eq!(
        paft_decimal::to_canonical_string(
            &OutcomePrice::from_micros(410_000).unwrap().to_decimal()
        ),
        "0.41"
    );
}

#[test]
fn outcome_price_rejects_inexact_or_out_of_range_decimal_values() {
    assert!(OutcomePrice::from_canonical_str("0.0000001").is_err());
    assert!(OutcomePrice::from_canonical_str("-0.1").is_err());
    assert!(OutcomePrice::from_canonical_str("1.000001").is_err());
    assert!(OutcomePrice::from_canonical_str("1e-3").is_err());
    assert!(OutcomePrice::from_decimal(decimal("0.1234567")).is_err());
}

#[test]
fn price_tick_parses_exact_decimal_values() {
    assert_eq!(
        PriceTick::from_canonical_str("0.01").unwrap().micros(),
        10_000
    );
    assert_eq!(
        PriceTick::from_canonical_str("0.001").unwrap().micros(),
        1_000
    );
    assert_eq!(
        PriceTick::from_canonical_str("0.0001").unwrap().micros(),
        100
    );
    assert_eq!(
        PriceTick::from_decimal(decimal("0.01")).unwrap().micros(),
        10_000
    );
    assert_eq!(
        paft_decimal::to_canonical_string(&PriceTick::from_micros(100).unwrap().to_decimal()),
        "0.0001"
    );
}

#[test]
fn price_tick_rejects_zero_inexact_or_out_of_range_decimal_values() {
    assert!(PriceTick::from_canonical_str("0").is_err());
    assert!(PriceTick::from_canonical_str("0.0000000").is_err());
    assert!(PriceTick::from_canonical_str("0.0000001").is_err());
    assert!(PriceTick::from_canonical_str("-0.01").is_err());
    assert!(PriceTick::from_canonical_str("1.000001").is_err());
    assert!(PriceTick::from_canonical_str("1e-3").is_err());
    assert!(PriceTick::from_decimal(decimal("0.1234567")).is_err());
}

#[test]
fn fixed_point_display_uses_canonical_decimal_form() {
    assert_eq!(
        OutcomePrice::from_micros(410_000).unwrap().to_string(),
        "0.41"
    );
    assert_eq!(OutcomePrice::ONE.to_string(), "1");
    assert_eq!(PriceTick::from_micros(100).unwrap().to_string(), "0.0001");
    assert_eq!(
        ContractQuantity::from_microcontracts(2_000_000).to_string(),
        "2"
    );
    assert_eq!(
        ContractQuantity::from_microcontracts(219_217_767).to_string(),
        "219.217767"
    );
    assert_eq!(
        NonZeroContractQuantity::from_microcontracts(219_217_767)
            .unwrap()
            .to_string(),
        "219.217767"
    );
    assert_eq!(OutcomePayout::from_micropayouts(1_000_000).to_string(), "1");
    assert_eq!(
        NonZeroOutcomePayout::from_micropayouts(1_000_000)
            .unwrap()
            .to_string(),
        "1"
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
        PriceBand::new(
            OutcomePrice::ZERO,
            OutcomePrice::from_micros(100_000).unwrap(),
            PriceTick::from_micros(1_000).unwrap(),
        )
        .unwrap(),
        PriceBand::new(
            OutcomePrice::from_micros(100_100).unwrap(),
            OutcomePrice::ONE,
            PriceTick::from_micros(100).unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();

    assert!(grid.contains_price(OutcomePrice::from_micros(50_000).unwrap()));
    assert!(grid.contains_price(OutcomePrice::from_micros(100_100).unwrap()));
    assert!(!grid.contains_price(OutcomePrice::from_micros(100_150).unwrap()));
    assert_eq!(grid.bands().len(), 2);
    assert_eq!(grid.bands()[0].start(), OutcomePrice::ZERO);
    assert_eq!(grid.bands()[0].tick().micros(), 1_000);
    assert_eq!(grid.into_bands().len(), 2);
}

#[test]
fn price_band_constructor_and_deserialization_validate_invariants() {
    let descending = PriceBand::new(
        OutcomePrice::from_micros(100).unwrap(),
        OutcomePrice::ZERO,
        PriceTick::from_micros(10).unwrap(),
    );
    assert!(descending.is_err());

    let off_tick_endpoint = PriceBand::new(
        OutcomePrice::ZERO,
        OutcomePrice::from_micros(100).unwrap(),
        PriceTick::from_micros(30).unwrap(),
    );
    assert!(off_tick_endpoint.is_err());

    assert!(
        serde_json::from_str::<PriceBand>(r#"{ "start": 0, "end": 100, "tick": 30 }"#).is_err()
    );
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

    let off_tick_endpoint = r#"{
        "bands": [
            { "start": 0, "end": 100, "tick": 30 }
        ]
    }"#;
    assert!(serde_json::from_str::<PriceGrid>(off_tick_endpoint).is_err());

    let unknown_grid_field = r#"{
        "bands": [
            { "start": 0, "end": 100000, "tick": 1000 }
        ],
        "source": "ignored"
    }"#;
    assert!(serde_json::from_str::<PriceGrid>(unknown_grid_field).is_err());

    let unknown_band_field = r#"{
        "bands": [
            { "start": 0, "end": 100000, "tick": 1000, "label": "ignored" }
        ]
    }"#;
    assert!(serde_json::from_str::<PriceGrid>(unknown_band_field).is_err());

    let valid = r#"{
        "bands": [
            { "start": 0, "end": 100000, "tick": 1000 },
            { "start": 100001, "end": 200001, "tick": 1000 }
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

#[test]
fn outcome_payout_parses_exact_decimal_values() {
    assert_eq!(
        OutcomePayout::from_canonical_str("1")
            .unwrap()
            .micropayouts(),
        OutcomePayout::SCALE
    );
    assert_eq!(
        OutcomePayout::from_canonical_str("0.5")
            .unwrap()
            .micropayouts(),
        500_000
    );
    assert_eq!(
        OutcomePayout::from_decimal(decimal("2.25"))
            .unwrap()
            .micropayouts(),
        2_250_000
    );
    assert_eq!(
        paft_decimal::to_canonical_string(&OutcomePayout::from_micropayouts(500_000).to_decimal()),
        "0.5"
    );
}

#[test]
fn non_zero_outcome_payout_parses_exact_decimal_values() {
    let payout = NonZeroOutcomePayout::from_canonical_str("2.25").unwrap();

    assert_eq!(payout.micropayouts(), 2_250_000);
    assert_eq!(
        NonZeroOutcomePayout::from_decimal(decimal("1"))
            .unwrap()
            .micropayouts(),
        OutcomePayout::SCALE
    );
    assert_eq!(payout.to_payout().micropayouts(), 2_250_000);
    assert_eq!(
        paft_decimal::to_canonical_string(&payout.to_decimal()),
        "2.25"
    );
}

#[test]
fn non_zero_outcome_payout_rejects_zero_and_preserves_integer_serde() {
    assert!(NonZeroOutcomePayout::from_micropayouts(0).is_err());
    assert!(NonZeroOutcomePayout::from_payout(OutcomePayout::ZERO).is_err());
    assert!(NonZeroOutcomePayout::from_canonical_str("0").is_err());
    assert!(NonZeroOutcomePayout::from_canonical_str("0.0000000").is_err());
    assert!(serde_json::from_str::<NonZeroOutcomePayout>("0").is_err());

    let payout = NonZeroOutcomePayout::from_micropayouts(2_000_000).unwrap();
    assert_eq!(serde_json::to_string(&payout).unwrap(), "2000000");
    assert_eq!(
        serde_json::from_str::<NonZeroOutcomePayout>("2000000")
            .unwrap()
            .micropayouts(),
        2_000_000
    );
}

#[test]
fn outcome_payout_rejects_inexact_negative_or_overflowing_decimal_values() {
    assert!(OutcomePayout::from_canonical_str("1.0000001").is_err());
    assert!(OutcomePayout::from_canonical_str("-1").is_err());
    assert!(OutcomePayout::from_canonical_str("1e3").is_err());
    assert!(OutcomePayout::from_canonical_str("18446744073709.551616").is_err());
    assert!(OutcomePayout::from_decimal(decimal("1.0000001")).is_err());
}

#[test]
fn contract_quantity_parses_exact_decimal_values() {
    let quantity = ContractQuantity::from_canonical_str("219.217767").unwrap();
    assert_eq!(quantity.microcontracts(), 219_217_767);
    assert_eq!(
        ContractQuantity::from_decimal(decimal("2"))
            .unwrap()
            .microcontracts(),
        2_000_000
    );
    assert_eq!(
        paft_decimal::to_canonical_string(&quantity.to_decimal()),
        "219.217767"
    );
}

#[test]
fn contract_quantity_rejects_inexact_negative_or_overflowing_decimal_values() {
    assert!(ContractQuantity::from_canonical_str("219.2177671").is_err());
    assert!(ContractQuantity::from_canonical_str("-1").is_err());
    assert!(ContractQuantity::from_canonical_str("1e3").is_err());
    assert!(ContractQuantity::from_canonical_str("18446744073709.551616").is_err());
    assert!(ContractQuantity::from_decimal(decimal("1.0000001")).is_err());
}

#[test]
fn non_zero_contract_quantity_parses_exact_decimal_values() {
    let quantity = NonZeroContractQuantity::from_canonical_str("219.217767").unwrap();

    assert_eq!(quantity.microcontracts(), 219_217_767);
    assert_eq!(
        NonZeroContractQuantity::from_decimal(decimal("2"))
            .unwrap()
            .microcontracts(),
        2_000_000
    );
    assert_eq!(quantity.to_quantity().microcontracts(), 219_217_767);
    assert_eq!(
        paft_decimal::to_canonical_string(&quantity.to_decimal()),
        "219.217767"
    );
}

#[test]
fn non_zero_contract_quantity_rejects_zero_and_preserves_integer_serde() {
    assert!(NonZeroContractQuantity::from_microcontracts(0).is_err());
    assert!(NonZeroContractQuantity::from_quantity(ContractQuantity::ZERO).is_err());
    assert!(NonZeroContractQuantity::from_canonical_str("0").is_err());
    assert!(NonZeroContractQuantity::from_canonical_str("0.0000000").is_err());
    assert!(serde_json::from_str::<NonZeroContractQuantity>("0").is_err());

    let quantity = NonZeroContractQuantity::from_microcontracts(2_000_000).unwrap();
    assert_eq!(serde_json::to_string(&quantity).unwrap(), "2000000");
    assert_eq!(
        serde_json::from_str::<NonZeroContractQuantity>("2000000")
            .unwrap()
            .microcontracts(),
        2_000_000
    );
}

#[test]
fn optional_non_zero_contract_quantity_uses_non_zero_niche() {
    assert_eq!(
        size_of::<Option<NonZeroContractQuantity>>(),
        size_of::<u64>()
    );
}

#[test]
fn optional_non_zero_outcome_payout_uses_non_zero_niche() {
    assert_eq!(size_of::<Option<NonZeroOutcomePayout>>(), size_of::<u64>());
}
