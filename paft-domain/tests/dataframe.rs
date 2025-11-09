#![cfg(feature = "dataframe")]
use paft_domain::{
    Canonical, Exchange,
    instrument::{AssetKind, Instrument},
};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

#[test]
fn instrument_to_dataframe() {
    let instrument =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity).unwrap();

    let df = instrument.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn instruments_vec_to_dataframe() {
    let instruments = [
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity).unwrap(),
        Instrument::from_symbol_and_exchange(
            "EURUSD=X",
            Exchange::Other(Canonical::try_new("FX").unwrap()),
            AssetKind::Forex,
        )
        .unwrap(),
    ];

    let df = instruments.to_dataframe().unwrap();
    println!("{df:?}");

    assert_eq!(df.height(), 2);
    let columns = df.get_column_names();
    assert!(columns.iter().any(|c| c.as_str() == "symbol"));
    assert!(columns.iter().any(|c| c.as_str() == "kind"));
}
