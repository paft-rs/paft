use chrono::DateTime;
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Instrument};
use paft_market::{FieldUpdate, QuoteUpdate};
use paft_money::{Currency, IsoCurrency, PriceAmount, QuantityAmount};
use serde_json::{Value, json};

fn base() -> QuoteUpdate {
    QuoteUpdate::new(
        Instrument::from_symbol("TEST", AssetKind::Equity).unwrap(),
        Currency::Iso(IsoCurrency::USD),
        DateTime::from_timestamp_nanos(1),
    )
}

fn sequence() -> Vec<QuoteUpdate> {
    // Synthetic source patches with an explicitly known patch contract.
    // These are not a migration rule for ambiguous historical nulls.
    [Some(json!("5")), None, Some(Value::Null), Some(json!("0"))]
        .into_iter()
        .map(|value| {
            let mut wire = serde_json::to_value(base()).unwrap();
            if let Some(value) = value {
                for field in ["price", "previous_close", "volume"] {
                    wire[field] = value.clone();
                }
            }
            serde_json::from_value(wire).unwrap()
        })
        .collect()
}

#[test]
fn quote_patch_sequence_preserves_unchanged_clear_and_present_zero() {
    let mut price = Some(PriceAmount::new(100.into()));
    let mut previous_close = price.clone();
    let mut volume = Some(QuantityAmount::from_decimal(100.into()).unwrap());
    let context = base();
    assert_eq!(context.price, FieldUpdate::Unchanged);
    assert_eq!(context.previous_close, FieldUpdate::Unchanged);
    assert_eq!(context.volume, FieldUpdate::Unchanged);
    context.price.apply_to(&mut price);
    assert_eq!(price, Some(PriceAmount::new(100.into())));

    for (row, expected) in sequence()
        .into_iter()
        .zip([Some(5), Some(5), None, Some(0)])
    {
        // This consumer establishes context before applying individual fields.
        assert_eq!(row.instrument, context.instrument);
        assert_eq!(row.currency, context.currency);
        let wire = serde_json::to_value(&row).unwrap();
        for field in ["price", "previous_close", "volume"] {
            match &row.price {
                FieldUpdate::Unchanged => assert!(wire.get(field).is_none()),
                FieldUpdate::Clear => assert_eq!(wire[field], Value::Null),
                FieldUpdate::Set(value) => assert_eq!(wire[field], value.as_decimal().to_string()),
            }
        }
        let decoded: QuoteUpdate = serde_json::from_value(wire).unwrap();
        assert_eq!(decoded, row);
        decoded.price.apply_to(&mut price);
        decoded.previous_close.apply_to(&mut previous_close);
        decoded.volume.apply_to(&mut volume);
        let expected = expected.map(Decimal::from);
        assert_eq!(price.as_ref().map(|v| *v.as_decimal()), expected);
        assert_eq!(previous_close.as_ref().map(|v| *v.as_decimal()), expected);
        assert_eq!(volume.as_ref().map(|v| *v.as_decimal()), expected);
    }
}

#[cfg(feature = "dataframe")]
mod dataframe {
    use super::*;
    use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
    use polars::prelude::{AnyValue, DataFrame, DataType};

    #[derive(df_derive_macros::ToDataFrame)]
    struct Nested {
        update: QuoteUpdate,
    }

    fn replay(frames: &[DataFrame], prefix: &str) {
        for field in ["price", "previous_close", "volume"] {
            let mut state = Some(Decimal::from(100));
            for (df, expected) in frames.iter().zip([Some(5), Some(5), None, Some(0)]) {
                let op = df.column(&format!("{prefix}{field}.operation")).unwrap();
                let value = df.column(&format!("{prefix}{field}.value")).unwrap();
                assert_eq!(op.dtype(), &DataType::String);
                assert_eq!(value.dtype(), &DataType::Decimal(38, 10));
                match op.str().unwrap().get(0).unwrap() {
                    "UNCHANGED" => assert_eq!(value.get(0).unwrap(), AnyValue::Null),
                    "CLEAR" => {
                        assert_eq!(value.get(0).unwrap(), AnyValue::Null);
                        state = None;
                    }
                    "SET" => {
                        let AnyValue::Decimal(coefficient, 38, 10) = value.get(0).unwrap() else {
                            panic!("SET must retain its decimal value")
                        };
                        state = Some(Decimal::from_i128_with_scale(coefficient, 10));
                    }
                    op => panic!("unexpected operation {op}"),
                }
                assert_eq!(state, expected.map(Decimal::from));
            }
        }
    }

    #[test]
    fn dataframe_operations_reconstruct_the_same_state_on_every_path() {
        let rows = sequence();
        let batch = rows.to_dataframe().unwrap();
        assert_eq!(
            batch.schema(),
            QuoteUpdate::empty_dataframe().unwrap().schema()
        );
        let refs: Vec<_> = rows.iter().collect();
        let borrowed = QuoteUpdate::columnar_from_refs(&refs).unwrap();
        for df in [&batch, &borrowed] {
            let operations = df.column("volume.operation").unwrap().str().unwrap();
            for (i, op) in ["SET", "UNCHANGED", "CLEAR", "SET"].into_iter().enumerate() {
                assert_eq!(operations.get(i), Some(op));
            }
            assert_eq!(
                df.column("volume.value").unwrap().get(0).unwrap(),
                AnyValue::Decimal(50_000_000_000, 38, 10)
            );
            replay(&(0..4).map(|i| df.slice(i, 1)).collect::<Vec<_>>(), "");
        }
        replay(
            &rows
                .iter()
                .map(|row| row.to_dataframe().unwrap())
                .collect::<Vec<_>>(),
            "",
        );
        replay(
            &rows
                .into_iter()
                .map(|update| Nested { update }.to_dataframe().unwrap())
                .collect::<Vec<_>>(),
            "update.",
        );
    }
}
