#![cfg(feature = "dataframe")]

use paft::prelude::{ToDataFrame, ToDataFrameVec};

#[derive(Clone, ToDataFrame)]
#[df_derive(trait = "::paft::dataframe::ToDataFrame")]
struct FacadeRow {
    id: u64,
    symbol: String,
}

#[test]
fn paft_reexports_macro_and_dataframe_traits() {
    fn assert_paft_dataframe<T: paft::dataframe::ToDataFrame + paft::dataframe::Columnar>() {}
    assert_paft_dataframe::<FacadeRow>();

    let rows = vec![
        FacadeRow {
            id: 1,
            symbol: "AAPL".into(),
        },
        FacadeRow {
            id: 2,
            symbol: "MSFT".into(),
        },
    ];

    let df = rows.as_slice().to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    let columns = df.get_column_names();
    assert!(columns.iter().any(|c| c.as_str() == "id"));
    assert!(columns.iter().any(|c| c.as_str() == "symbol"));
}
