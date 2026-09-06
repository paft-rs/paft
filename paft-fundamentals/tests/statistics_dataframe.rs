#![cfg(feature = "dataframe")]

use paft_fundamentals::KeyStatistics;
use paft_money::{Currency, IsoCurrency, MonetaryAmount, QuantityAmount};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};
use polars::prelude::{AnyValue, DataType};

#[derive(df_derive_macros::ToDataFrame)]
struct StatisticsContext {
    statistics: KeyStatistics,
}

#[test]
fn average_volume_preserves_fractional_zero_and_missing_values_in_dataframes() {
    let rows = [Some("2.5"), Some("0"), None].map(|amount| KeyStatistics {
        average_daily_volume_3m: amount.map(|value| {
            QuantityAmount::from_decimal(paft_decimal::parse_decimal(value).unwrap()).unwrap()
        }),
        ..KeyStatistics::default()
    });
    let expected = [
        AnyValue::Decimal(25_000_000_000, 38, 10),
        AnyValue::Decimal(0, 38, 10),
        AnyValue::Null,
    ];
    let batch = rows.to_dataframe().unwrap();
    let column = batch.column("average_daily_volume_3m.amount").unwrap();
    assert_eq!(column.dtype(), &DataType::Decimal(38, 10));
    assert_eq!(
        batch.schema(),
        KeyStatistics::empty_dataframe().unwrap().schema()
    );
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(column.get(index).unwrap(), expected[index]);
        assert_eq!(
            row.to_dataframe()
                .unwrap()
                .column(column.name())
                .unwrap()
                .get(0)
                .unwrap(),
            expected[index]
        );
    }
    let nested = rows.map(|statistics| StatisticsContext { statistics });
    let df = nested.to_dataframe().unwrap();
    let values = df
        .column("statistics.average_daily_volume_3m.amount")
        .unwrap();
    for (index, expected) in expected.iter().enumerate() {
        assert_eq!(values.get(index).unwrap(), *expected);
    }
}

#[test]
fn market_cap_exports_subcent_values_without_settlement_columns() {
    let rows = [Some("1.001"), Some("0"), None].map(|amount| KeyStatistics {
        market_cap: amount.map(|value| {
            MonetaryAmount::from_canonical_str(value, Currency::Iso(IsoCurrency::USD)).unwrap()
        }),
        ..KeyStatistics::default()
    });
    let amounts = [
        AnyValue::Decimal(10_010_000_000, 38, 10),
        AnyValue::Decimal(0, 38, 10),
        AnyValue::Null,
    ];
    let currencies = [Some("USD"), Some("USD"), None];
    let batch = rows.to_dataframe().unwrap();
    assert_eq!(
        batch.schema(),
        KeyStatistics::empty_dataframe().unwrap().schema()
    );
    assert!(batch.column("market_cap.minor_units").is_err());
    assert_eq!(
        batch.column("market_cap.amount").unwrap().dtype(),
        &DataType::Decimal(38, 10)
    );
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(
            batch
                .column("market_cap.amount")
                .unwrap()
                .get(index)
                .unwrap(),
            amounts[index]
        );
        assert_eq!(
            batch
                .column("market_cap.currency")
                .unwrap()
                .str()
                .unwrap()
                .get(index),
            currencies[index]
        );
        let single = row.to_dataframe().unwrap();
        assert_eq!(
            single.column("market_cap.amount").unwrap().get(0).unwrap(),
            amounts[index]
        );
        assert_eq!(
            single
                .column("market_cap.currency")
                .unwrap()
                .str()
                .unwrap()
                .get(0),
            currencies[index]
        );
    }
    let nested = rows.map(|statistics| StatisticsContext { statistics });
    let df = nested.to_dataframe().unwrap();
    assert!(df.column("statistics.market_cap.minor_units").is_err());
    for (index, expected) in amounts.iter().enumerate() {
        assert_eq!(
            df.column("statistics.market_cap.amount")
                .unwrap()
                .get(index)
                .unwrap(),
            *expected
        );
        assert_eq!(
            df.column("statistics.market_cap.currency")
                .unwrap()
                .str()
                .unwrap()
                .get(index),
            currencies[index]
        );
    }
}
