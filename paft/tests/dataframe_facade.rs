#![cfg(feature = "dataframe")]

use paft::Decimal;
use paft::prelude::{Currency, ExchangeRate, IsoCurrency, ToDataFrame, ToDataFrameVec};
use paft_decimal::from_minor_units;
use polars::prelude::{AnyValue, DataType};

#[test]
fn paft_reexports_dataframe_traits_for_paft_types() {
    fn assert_paft_dataframe<T: paft::dataframe::ToDataFrame + paft::dataframe::Columnar>() {}
    assert_paft_dataframe::<ExchangeRate>();

    let rate = ExchangeRate::new(
        Currency::Iso(IsoCurrency::USD),
        Currency::Iso(IsoCurrency::EUR),
        Decimal::from(1),
    )
    .unwrap();

    let df = rate.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    let columns = df.get_column_names();
    assert!(columns.iter().any(|c| c.as_str() == "from"));
    assert!(columns.iter().any(|c| c.as_str() == "to"));
    assert!(columns.iter().any(|c| c.as_str() == "rate"));

    let rows = vec![rate];
    let df = rows.as_slice().to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[derive(Clone, df_derive_macros::ToDataFrame)]
struct TupleDecimalRow {
    maybe: Option<(Decimal,)>,
}

#[test]
fn paft_decimal_runtime_handles_parent_option_tuple_projection() {
    let rows = vec![
        TupleDecimalRow {
            maybe: Some((from_minor_units(123, 2),)),
        },
        TupleDecimalRow { maybe: None },
    ];

    let df = rows.as_slice().to_dataframe().unwrap();
    assert_eq!(df.shape(), (2, 1));
    assert_eq!(
        df.column("maybe.field_0").unwrap().dtype(),
        &DataType::Decimal(38, 10),
    );
    assert_eq!(
        df.column("maybe.field_0").unwrap().get(0).unwrap(),
        AnyValue::Decimal(12_300_000_000, 38, 10),
    );
    assert_eq!(
        df.column("maybe.field_0").unwrap().get(1).unwrap(),
        AnyValue::Null,
    );
}
