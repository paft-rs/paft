use paft_decimal::{Decimal, parse_decimal};
use paft_domain::ReportingPeriod;
use paft_fundamentals::{
    InsiderPosition, InsiderRosterHolder, InsiderTransaction, NetSharePurchaseActivity,
    TransactionType,
};
use paft_money::QuantityAmount;
use serde_json::{Value, json};

fn quantity(text: &str) -> QuantityAmount {
    QuantityAmount::from_decimal(parse_decimal(text).unwrap()).unwrap()
}

// Test-only mapping of the attributed filing, not a general Form 4 adapter.
fn reported_holdings() -> (InsiderTransaction, InsiderRosterHolder) {
    let raw: Value =
        serde_json::from_str(include_str!("fixtures/factset_insider_2020.json")).unwrap();
    assert_eq!(raw["transaction_code"], "J");
    assert_eq!(raw["acquired_or_disposed"], "A");
    assert_eq!(raw["ownership_form"], "D");
    assert_eq!(raw["filing_date"], "2020-03-03");
    assert_eq!(raw["transaction_date"], "2020-02-28");
    let transaction_date = raw["transaction_date"].as_str().unwrap().parse().unwrap();
    let position = InsiderPosition::other("SVP_CHRO").unwrap();
    let name = raw["reporting_person"].as_str().unwrap().to_owned();
    let transaction = InsiderTransaction {
        insider: name.clone(),
        position: position.clone(),
        // The filing's footnote identifies J as an ESPP acquisition.
        transaction_type: TransactionType::Buy,
        shares: Some(quantity(raw["shares_acquired"].as_str().unwrap())),
        value: None,
        transaction_date,
        url: Some(raw["source"].as_str().unwrap().to_owned()),
    };
    let ownership = InsiderRosterHolder {
        name,
        position,
        most_recent_transaction: TransactionType::Buy,
        latest_transaction_date: transaction_date,
        shares_owned_directly: Some(quantity(
            &raw["shares_owned_afterward"]
                .as_str()
                .unwrap()
                .replace(',', ""),
        )),
        position_direct_date: raw["filing_date"].as_str().unwrap().parse().unwrap(),
    };
    (transaction, ownership)
}

fn activity() -> NetSharePurchaseActivity {
    // Synthetic independently reported aggregates: the net is deliberately not
    // recomputed from the supplied buy/sell values or ownership balance.
    NetSharePurchaseActivity {
        period: ReportingPeriod::quarterly(2020, 1).unwrap(),
        buy_shares: Some(quantity("38.8938")),
        buy_count: Some(1),
        sell_shares: Some(quantity("40")),
        sell_count: Some(1),
        net_shares: Some(parse_decimal("-1.2").unwrap()),
        net_count: Some(0),
        total_insider_shares: Some(quantity("2048.8938")),
        net_percent_insider_shares: None,
    }
}

#[test]
fn filing_quantities_survive_canonical_json() {
    let (transaction, ownership) = reported_holdings();
    let wire = serde_json::to_value(&transaction).unwrap();
    assert_eq!(wire["shares"], "38.8938");
    assert_eq!(
        serde_json::from_value::<InsiderTransaction>(wire).unwrap(),
        transaction
    );
    let wire = serde_json::to_value(&ownership).unwrap();
    assert_eq!(wire["shares_owned_directly"], "2048.8938");
    assert_eq!(
        serde_json::from_value::<InsiderRosterHolder>(wire).unwrap(),
        ownership
    );

    let activity = activity();
    let wire = serde_json::to_value(&activity).unwrap();
    assert_eq!(wire["buy_shares"], "38.8938");
    assert_eq!(wire["sell_shares"], "40");
    assert_eq!(wire["net_shares"], "-1.2");
    assert_eq!(wire["total_insider_shares"], "2048.8938");
    assert_eq!(wire["buy_count"], 1);
    assert_eq!(wire["sell_count"], 1);
    assert_eq!(wire["net_count"], 0);
    assert_eq!(
        serde_json::from_value::<NetSharePurchaseActivity>(wire).unwrap(),
        activity
    );
}

#[test]
fn missing_zero_and_negative_quantities_have_distinct_contracts() {
    let base = serde_json::to_value(activity()).unwrap();
    for field in [
        "buy_shares",
        "sell_shares",
        "total_insider_shares",
        "net_shares",
    ] {
        for amount in [Value::Null, json!("0")] {
            let mut wire = base.clone();
            wire[field] = amount.clone();
            let row: NetSharePurchaseActivity = serde_json::from_value(wire).unwrap();
            assert_eq!(serde_json::to_value(row).unwrap()[field], amount);
        }
        let mut wire = base.clone();
        wire[field] = json!("-0.0001");
        assert_eq!(
            serde_json::from_value::<NetSharePurchaseActivity>(wire).is_ok(),
            field == "net_shares"
        );
    }
    let (transaction, ownership) = reported_holdings();
    let mut wire = serde_json::to_value(transaction).unwrap();
    wire["shares"] = json!("-0.1");
    assert!(serde_json::from_value::<InsiderTransaction>(wire).is_err());
    let mut wire = serde_json::to_value(ownership).unwrap();
    wire["shares_owned_directly"] = json!("-0.1");
    assert!(serde_json::from_value::<InsiderRosterHolder>(wire).is_err());
}

#[test]
fn legacy_integers_migrate_without_floating_point() {
    let legacy: Value = serde_json::from_str(
        r#"{"shares":18446744073709551615,"net_shares":-9223372036854775808}"#,
    )
    .unwrap();
    let shares =
        QuantityAmount::from_decimal(Decimal::from(legacy["shares"].as_u64().unwrap())).unwrap();
    let net = Decimal::from(legacy["net_shares"].as_i64().unwrap());
    let (mut transaction, _) = reported_holdings();
    transaction.shares = Some(shares);
    assert_eq!(
        serde_json::to_value(transaction).unwrap()["shares"],
        "18446744073709551615"
    );
    let mut row = activity();
    row.net_shares = Some(net);
    assert_eq!(
        serde_json::to_value(row).unwrap()["net_shares"],
        "-9223372036854775808"
    );
}

#[cfg(feature = "dataframe")]
mod dataframe {
    use super::*;
    use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
    use polars::prelude::{AnyValue, DataType};

    #[derive(df_derive_macros::ToDataFrame)]
    struct Filing {
        transaction: InsiderTransaction,
        ownership: InsiderRosterHolder,
    }

    fn check<T: ToDataFrame + Columnar>(rows: &[T], field: &str, expected: &[Option<i128>]) {
        let batch = rows.to_dataframe().unwrap();
        assert_eq!(batch.schema(), T::empty_dataframe().unwrap().schema());
        assert_eq!(
            batch.column(field).unwrap().dtype(),
            &DataType::Decimal(38, 10)
        );
        let refs: Vec<_> = rows.iter().collect();
        let borrowed = T::columnar_from_refs(&refs).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let expected =
                expected[i].map_or(AnyValue::Null, |value| AnyValue::Decimal(value, 38, 10));
            for df in [&batch, &borrowed, &row.to_dataframe().unwrap()] {
                let index = if df.height() == 1 { 0 } else { i };
                assert_eq!(df.column(field).unwrap().get(index).unwrap(), expected);
            }
        }
    }

    #[test]
    fn filing_and_activity_values_preserve_fractions_zero_and_absence() {
        let (transaction, ownership) = reported_holdings();
        let transactions = [Some(quantity("38.8938")), Some(quantity("0")), None].map(|shares| {
            InsiderTransaction {
                shares,
                ..transaction.clone()
            }
        });
        check(
            &transactions,
            "shares.amount",
            &[Some(388_938_000_000), Some(0), None],
        );
        let owners =
            [Some(quantity("2048.8938")), Some(quantity("0")), None].map(|shares_owned_directly| {
                InsiderRosterHolder {
                    shares_owned_directly,
                    ..ownership.clone()
                }
            });
        check(
            &owners,
            "shares_owned_directly.amount",
            &[Some(20_488_938_000_000), Some(0), None],
        );
        let rows = [
            Some(parse_decimal("-1.2").unwrap()),
            Some(Decimal::ZERO),
            None,
        ]
        .map(|net_shares| NetSharePurchaseActivity {
            net_shares,
            ..activity()
        });
        check(&rows, "net_shares", &[Some(-12_000_000_000), Some(0), None]);
        check(&rows, "buy_shares.amount", &[Some(388_938_000_000); 3]);
        check(&rows, "sell_shares.amount", &[Some(400_000_000_000); 3]);
        check(
            &rows,
            "total_insider_shares.amount",
            &[Some(20_488_938_000_000); 3],
        );
        let df = rows.to_dataframe().unwrap();
        assert_eq!(df.column("buy_count").unwrap().dtype(), &DataType::UInt64);
        assert_eq!(df.column("sell_count").unwrap().dtype(), &DataType::UInt64);
        assert_eq!(df.column("net_count").unwrap().dtype(), &DataType::Int64);

        let nested = Filing {
            transaction,
            ownership,
        }
        .to_dataframe()
        .unwrap();
        assert_eq!(
            nested
                .column("transaction.shares.amount")
                .unwrap()
                .get(0)
                .unwrap(),
            AnyValue::Decimal(388_938_000_000, 38, 10)
        );
        assert_eq!(
            nested
                .column("ownership.shares_owned_directly.amount")
                .unwrap()
                .get(0)
                .unwrap(),
            AnyValue::Decimal(20_488_938_000_000, 38, 10)
        );
    }
}
