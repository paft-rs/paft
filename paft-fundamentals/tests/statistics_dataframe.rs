#![cfg(feature = "dataframe")]

use paft_fundamentals::KeyStatistics;
use paft_money::QuantityAmount;
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
