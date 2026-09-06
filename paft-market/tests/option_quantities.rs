use chrono::NaiveDate;
use paft_decimal::{PositiveDecimal, checked_div_exact, parse_decimal};
use paft_domain::{AssetKind, Instrument};
use paft_market::{OptionContract, OptionContractKey, OptionSide};
use paft_money::{Currency, IsoCurrency, Price, QuantityAmount};
use serde::Deserialize;
use serde_json::value::RawValue;

#[derive(Deserialize)]
struct Summary {
    instrument_name: String,
    base_currency: String,
    volume: Box<RawValue>,
    open_interest: Box<RawValue>,
}

fn summary() -> Summary {
    serde_json::from_str(include_str!("fixtures/deribit_option_quantities.json")).unwrap()
}

// Test adapter: parse the original numeric token without an f64 intermediate,
// then convert base units to contracts only when the exact ratio is supported.
fn contracts(raw: &RawValue, size: Option<&PositiveDecimal>) -> Option<QuantityAmount> {
    let amount = parse_decimal(raw.get()).ok()?;
    let normalized = checked_div_exact(&amount, size?.as_decimal())?;
    QuantityAmount::from_decimal(normalized).ok()
}

fn mapped_contract() -> OptionContract {
    let source = summary();
    let currency = Currency::Iso(IsoCurrency::USD);
    let key = OptionContractKey::new(
        Instrument::from_symbol(&source.base_currency, AssetKind::Crypto).unwrap(),
        OptionSide::Put,
        Price::from_canonical_str("140", currency.clone()).unwrap(),
        NaiveDate::from_ymd_opt(2019, 2, 22).unwrap(),
    )
    .with_contract_instrument(
        Instrument::from_symbol(&source.instrument_name, AssetKind::Option).unwrap(),
    );
    let mut contract = OptionContract::new(key, currency);
    // Explicit mapping context: one underlying unit per contract.
    let size = PositiveDecimal::new(parse_decimal("1").unwrap()).unwrap();
    contract.volume = Some(contracts(&source.volume, Some(&size)).unwrap());
    contract.open_interest = Some(contracts(&source.open_interest, Some(&size)).unwrap());
    contract
}

#[test]
fn fractional_provider_quantities_normalize_exactly() {
    let contract = mapped_contract();
    let wire = serde_json::to_value(&contract).unwrap();
    for field in ["volume", "open_interest"] {
        assert_eq!(wire[field], "0.55");
        let mut invalid = wire.clone();
        invalid[field] = serde_json::json!("-0.55");
        assert!(serde_json::from_value::<OptionContract>(invalid).is_err());
    }
    assert_eq!(
        serde_json::from_value::<OptionContract>(wire).unwrap(),
        contract
    );

    let source = summary();
    for raw in [&source.volume, &source.open_interest] {
        let size = PositiveDecimal::new(parse_decimal("0.1").unwrap()).unwrap();
        assert_eq!(
            contracts(raw, Some(&size)).unwrap().as_decimal(),
            &parse_decimal("5.5").unwrap()
        );
        assert!(contracts(raw, None).is_none());
        let size = PositiveDecimal::new(parse_decimal("3").unwrap()).unwrap();
        assert!(contracts(raw, Some(&size)).is_none());
    }
}

#[cfg(feature = "dataframe")]
#[test]
fn fractional_option_dataframe_preserves_quantity_zero_and_missing() {
    use paft_utils::dataframe::{Columnar, ToDataFrame};
    use polars::prelude::DataType;

    let fractional = mapped_contract();
    let mut zero = fractional.clone();
    zero.volume = Some(QuantityAmount::from_decimal(parse_decimal("0").unwrap()).unwrap());
    zero.open_interest = zero.volume.clone();
    let mut missing = fractional.clone();
    missing.volume = None;
    missing.open_interest = None;
    let rows = [fractional, zero, missing];
    let frame = OptionContract::columnar_to_dataframe(&rows).unwrap();
    assert_eq!(
        frame.schema(),
        OptionContract::empty_dataframe().unwrap().schema()
    );
    for field in ["volume.amount", "open_interest.amount"] {
        let column = frame.column(field).unwrap();
        assert_eq!(column.dtype(), &DataType::Decimal(38, 10));
        assert_eq!(
            (0..3)
                .map(|row| column.decimal().unwrap().physical().get(row))
                .collect::<Vec<_>>(),
            [Some(5_500_000_000), Some(0), None]
        );
    }
    for (i, row) in rows.iter().enumerate() {
        assert!(
            frame
                .slice(i64::try_from(i).unwrap(), 1)
                .equals_missing(&row.to_dataframe().unwrap())
        );
    }
}
