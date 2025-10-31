#![cfg(feature = "dataframe")]
use paft_domain::{
    Canonical, Exchange,
    instrument::{AssetKind, Instrument},
};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

#[test]
fn instrument_to_dataframe() {
    let instrument = Instrument::try_new(
        "AAPL",
        AssetKind::Equity,
        Some("BBG000B9XRY4"),
        Some("US0378331005"),
        Some(Exchange::NASDAQ),
    )
    .unwrap();

    let df = instrument.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn instruments_vec_to_dataframe() {
    let instruments = [
        Instrument::try_new(
            "AAPL",
            AssetKind::Equity,
            None,
            None,
            Some(Exchange::NASDAQ),
        )
        .unwrap(),
        Instrument::try_new(
            "EURUSD=X",
            AssetKind::Forex,
            None,
            None,
            Some(Exchange::Other(Canonical::try_new("FX").unwrap())),
        )
        .unwrap(),
    ];

    let df = instruments.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    let columns = df.get_column_names();
    assert!(columns.iter().any(|c| c.as_str() == "symbol"));
    assert!(columns.iter().any(|c| c.as_str() == "kind"));
}
