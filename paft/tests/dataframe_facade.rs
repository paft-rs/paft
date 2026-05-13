#![cfg(feature = "dataframe")]

use paft::Decimal;
use paft::money::IsoCurrency;
use paft::prelude::{Currency, ExchangeRate, ToDataFrame, ToDataFrameVec};

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
