#![cfg(feature = "dataframe")]
use paft_domain::{
    Canonical, Exchange, Figi, Isin,
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

#[test]
fn instruments_columnar_round_trips_string_cell_values() {
    // Three rows that exercise every code path the new `from_iter_values` /
    // `from_iter_options` based Columnar impl can hit:
    //
    // * Row 0: canonical Exchange variant + figi populated, isin not.
    // * Row 1: extensible `Exchange::Other(_)` variant + isin populated, figi not.
    // * Row 2: no exchange / figi / isin at all (all-None on every optional).
    let row0 = {
        let mut i =
            Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity)
                .unwrap();
        i.figi = Some(Figi::new("BBG000B9XRY4").unwrap());
        i
    };
    let row1 = {
        let mut i = Instrument::from_symbol_and_exchange(
            "EURUSD=X",
            Exchange::Other(Canonical::try_new("FX").unwrap()),
            AssetKind::Forex,
        )
        .unwrap();
        i.isin = Some(Isin::new("US0378331005").unwrap());
        i
    };
    let row2 = Instrument::from_symbol("BTC", AssetKind::Crypto).unwrap();

    let instruments = [row0, row1, row2];
    let df = instruments.to_dataframe().unwrap();

    assert_eq!(df.height(), 3);

    let kind = df.column("kind").unwrap().str().unwrap();
    assert_eq!(kind.get(0), Some("EQUITY"));
    assert_eq!(kind.get(1), Some("FOREX"));
    assert_eq!(kind.get(2), Some("CRYPTO"));

    let symbol = df.column("symbol").unwrap().str().unwrap();
    assert_eq!(symbol.get(0), Some("AAPL"));
    assert_eq!(symbol.get(1), Some("EURUSD=X"));
    assert_eq!(symbol.get(2), Some("BTC"));

    let exchange = df.column("exchange").unwrap().str().unwrap();
    assert_eq!(exchange.get(0), Some("NASDAQ"));
    assert_eq!(exchange.get(1), Some("FX"));
    assert_eq!(exchange.get(2), None);

    let figi = df.column("figi").unwrap().str().unwrap();
    assert_eq!(figi.get(0), Some("BBG000B9XRY4"));
    assert_eq!(figi.get(1), None);
    assert_eq!(figi.get(2), None);

    let isin = df.column("isin").unwrap().str().unwrap();
    assert_eq!(isin.get(0), None);
    assert_eq!(isin.get(1), Some("US0378331005"));
    assert_eq!(isin.get(2), None);
}
