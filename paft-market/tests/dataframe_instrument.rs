#![cfg(feature = "dataframe")]

use chrono::{NaiveDate, Utc};
use paft_decimal::Decimal;
use paft_domain::{AssetKind, Exchange, Instrument, Isin, Symbol};
use paft_market::{
    Candle, CandleUpdate, Interval, Ohlc, OptionChain, OptionContract, OptionContractKey,
    OptionSide, OptionUpdate, OrderBook, Quote, QuoteUpdate, SearchResult,
};
use paft_money::{Currency, IsoCurrency, Price, PriceAmount};
use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
use polars::prelude::{DataFrame, DataType};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn ambiguous_instruments() -> [[Instrument; 2]; 4] {
    let mut isin = Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap();
    isin.isin = Some(Isin::new("US0378331005").unwrap());
    [
        [
            Instrument::from_symbol("BTC", AssetKind::Crypto).unwrap(),
            Instrument::from_symbol("BTC", AssetKind::Equity).unwrap(),
        ],
        [
            Instrument::from_figi(
                "BBG000B9Y5X2",
                Symbol::new("AAPL").unwrap(),
                AssetKind::Equity,
            )
            .unwrap(),
            Instrument::from_symbol("BBG000B9Y5X2", AssetKind::Equity).unwrap(),
        ],
        [
            isin,
            Instrument::from_symbol("US0378331005", AssetKind::Equity).unwrap(),
        ],
        [
            Instrument::from_symbol("BTC@NASDAQ", AssetKind::Crypto).unwrap(),
            Instrument::from_symbol_and_exchange("BTC", Exchange::NASDAQ, AssetKind::Crypto)
                .unwrap(),
        ],
    ]
}

fn assert_instrument_columns(df: &DataFrame, prefix: &str, instruments: &[Option<&Instrument>]) {
    assert_eq!(df.height(), instruments.len());
    assert!(df.column(prefix).is_err());
    for (suffix, expected) in [
        (
            "security_key",
            instruments
                .iter()
                .map(|i| i.and_then(Instrument::security_key))
                .collect::<Vec<_>>(),
        ),
        (
            "listing_key",
            instruments
                .iter()
                .map(|i| i.and_then(Instrument::listing_key))
                .collect::<Vec<_>>(),
        ),
        (
            "key",
            instruments
                .iter()
                .map(|i| i.map(Instrument::unique_key))
                .collect::<Vec<_>>(),
        ),
        (
            "display",
            instruments
                .iter()
                .map(|i| i.map(ToString::to_string))
                .collect::<Vec<_>>(),
        ),
    ] {
        let column = df.column(&format!("{prefix}.{suffix}")).unwrap();
        assert_eq!(column.dtype(), &DataType::String);
        let values = column.str().unwrap();
        for (row, expected) in expected.iter().enumerate() {
            assert_eq!(values.get(row), expected.as_deref());
        }
    }
}

fn assert_market_rows<T: ToDataFrame + Columnar>(rows: &[T], instruments: &[Instrument]) {
    let df = rows.to_dataframe().unwrap();
    let refs: Vec<_> = instruments.iter().map(Some).collect();
    assert_instrument_columns(&df, "instrument", &refs);
    assert_eq!(T::empty_dataframe().unwrap().schema(), df.schema());
    for (row, instrument) in rows.iter().zip(instruments) {
        let single = row.to_dataframe().unwrap();
        assert_instrument_columns(&single, "instrument", &[Some(instrument)]);
        assert_eq!(single.schema(), df.schema());
    }
}

#[test]
fn market_instrument_exports_distinguish_identical_display_labels() {
    let ts = chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap();
    let price = PriceAmount::new(Decimal::ONE);
    let candle = Candle::new(
        ts,
        usd(),
        Ohlc::new(price.clone(), price.clone(), price.clone(), price),
    );
    for instruments in ambiguous_instruments() {
        assert_eq!(instruments[0].to_string(), instruments[1].to_string());
        assert_ne!(instruments[0].unique_key(), instruments[1].unique_key());
        let refs = instruments.each_ref();
        assert_market_rows(
            &refs.map(|i| OrderBook::new(i.clone(), usd())),
            &instruments,
        );
        assert_market_rows(&refs.map(|i| Quote::new(i.clone(), usd())), &instruments);
        assert_market_rows(
            &refs.map(|i| QuoteUpdate::new(i.clone(), usd(), ts)),
            &instruments,
        );
        assert_market_rows(
            &refs.map(|i| SearchResult {
                instrument: i.clone(),
                name: None,
                provider: (),
            }),
            &instruments,
        );
        assert_market_rows(
            &refs.map(|i| CandleUpdate::new(i.clone(), Interval::D1, candle.clone(), true)),
            &instruments,
        );
    }
}

fn option_keys() -> [OptionContractKey; 3] {
    let mut keys = [AssetKind::Crypto, AssetKind::Equity, AssetKind::Equity].map(|kind| {
        OptionContractKey::new(
            Instrument::from_symbol("BTC", kind).unwrap(),
            OptionSide::Call,
            Price::new(Decimal::ONE, usd()),
            NaiveDate::from_ymd_opt(2026, 12, 18).unwrap(),
        )
    });
    keys[0].contract_instrument = Some(
        Instrument::from_figi(
            "BBG000B9Y5X2",
            Symbol::new("OPTION").unwrap(),
            AssetKind::Option,
        )
        .unwrap(),
    );
    keys[1].contract_instrument =
        Some(Instrument::from_symbol("BBG000B9Y5X2", AssetKind::Option).unwrap());
    keys
}

#[test]
fn option_instrument_exports_preserve_underlying_contract_and_missing_identities() {
    let keys = option_keys();
    let underlyings = keys.each_ref().map(|key| Some(&key.underlying));
    let contracts = keys.each_ref().map(|key| key.contract_instrument.as_ref());
    for instruments in [underlyings, contracts] {
        assert_eq!(
            instruments[0].unwrap().to_string(),
            instruments[1].unwrap().to_string()
        );
        assert_ne!(
            instruments[0].unwrap().unique_key(),
            instruments[1].unwrap().unique_key()
        );
    }

    let ts = chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap();
    let rows = keys
        .each_ref()
        .map(|key| OptionContract::new(key.clone(), usd()));
    let updates = keys
        .each_ref()
        .map(|key| OptionUpdate::new(key.clone(), usd(), ts));
    for (df, empty) in [
        (
            keys.to_dataframe().unwrap(),
            OptionContractKey::empty_dataframe().unwrap(),
        ),
        (
            rows.to_dataframe().unwrap(),
            OptionContract::empty_dataframe().unwrap(),
        ),
        (
            updates.to_dataframe().unwrap(),
            OptionUpdate::empty_dataframe().unwrap(),
        ),
    ] {
        assert_instrument_columns(&df, "underlying", &underlyings);
        assert_instrument_columns(&df, "contract_instrument", &contracts);
        assert_eq!(df.schema(), empty.schema());
        for suffix in [
            "symbol",
            "exchange",
            "figi",
            "isin",
            "kind",
            "key",
            "security_key",
            "listing_key",
            "display",
        ] {
            assert!(
                df.column(&format!("contract_instrument.{suffix}"))
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .is_null()
            );
        }
    }
    for row in 0..keys.len() {
        for df in [
            keys[row].to_dataframe().unwrap(),
            rows[row].to_dataframe().unwrap(),
            updates[row].to_dataframe().unwrap(),
        ] {
            assert_instrument_columns(&df, "underlying", &[underlyings[row]]);
            assert_instrument_columns(&df, "contract_instrument", &[contracts[row]]);
        }
    }
}

#[test]
fn option_chain_list_columns_preserve_instrument_identity_and_nulls() {
    let keys = option_keys();
    let underlyings = keys.each_ref().map(|key| Some(&key.underlying));
    let contracts = keys.each_ref().map(|key| key.contract_instrument.as_ref());
    let chain = OptionChain {
        contracts: keys
            .iter()
            .map(|key| OptionContract::new(key.clone(), usd()))
            .collect(),
        provider: (),
    };
    for df in [
        chain.to_dataframe().unwrap(),
        std::slice::from_ref(&chain).to_dataframe().unwrap(),
    ] {
        assert_eq!(df.height(), 1);
        assert_eq!(
            df.schema(),
            OptionChain::empty_dataframe().unwrap().schema()
        );
        for (prefix, instruments) in [
            ("underlying", underlyings),
            ("contract_instrument", contracts),
        ] {
            for suffix in ["key", "security_key", "listing_key", "display"] {
                let column = df.column(&format!("contracts.{prefix}.{suffix}")).unwrap();
                assert_eq!(column.dtype(), &DataType::List(Box::new(DataType::String)));
                let values = column.list().unwrap().get_as_series(0).unwrap();
                assert_eq!(values.len(), instruments.len());
                for (row, instrument) in instruments.iter().enumerate() {
                    let expected = instrument.and_then(|i| match suffix {
                        "key" => Some(i.unique_key()),
                        "security_key" => i.security_key(),
                        "listing_key" => i.listing_key(),
                        _ => Some(i.to_string()),
                    });
                    assert_eq!(values.str().unwrap().get(row), expected.as_deref());
                }
            }
        }
    }
}

#[test]
fn quotes_with_the_same_isin_retain_distinct_listing_keys() {
    let mut first =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity).unwrap();
    first.isin = Some(Isin::new("US0378331005").unwrap());
    let mut second = first.clone();
    second.exchange = Some(Exchange::other("VENUE2").unwrap());
    let instruments = [first, second];
    let quotes = instruments.each_ref().map(|i| Quote::new(i.clone(), usd()));
    assert_market_rows(&quotes, &instruments);
    let df = quotes.to_dataframe().unwrap();
    let security = df.column("instrument.security_key").unwrap().str().unwrap();
    let listing = df.column("instrument.listing_key").unwrap().str().unwrap();
    assert!(security.get(0).is_some());
    assert_eq!(security.get(0), security.get(1));
    assert!(listing.get(0).is_some());
    assert_ne!(listing.get(0), listing.get(1));
}
