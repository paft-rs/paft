use chrono::DateTime;
use paft_domain::{AssetKind, Instrument};
use paft_market::market::quote::{Quote, QuoteUpdate};
use paft_money::{Currency, IsoCurrency};

#[test]
fn documented_coinbase_ticker_preserves_its_event_instant() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/coinbase_ticker.json")).unwrap();
    let raw = &fixture["ticker"];
    let ts = paft_core::serde_helpers::parse_timestamp(raw["time"].as_str().unwrap()).unwrap();
    let mut update = QuoteUpdate::new(
        Instrument::from_symbol(raw["product_id"].as_str().unwrap(), AssetKind::Crypto).unwrap(),
        Currency::Iso(IsoCurrency::USD),
        ts,
    );
    update.price = Some(paft_money::PriceAmount::new(
        paft_decimal::parse_decimal(raw["price"].as_str().unwrap()).unwrap(),
    ));
    update.volume = Some(
        paft_money::QuantityAmount::from_decimal(
            paft_decimal::parse_decimal(raw["volume_24h"].as_str().unwrap()).unwrap(),
        )
        .unwrap(),
    );
    let wire = serde_json::to_value(&update).unwrap();
    assert_eq!(wire["ts"], raw["time"]);
    assert_eq!(wire["price"], raw["price"]);
    assert_eq!(wire["volume"], raw["volume_24h"]);
    assert_eq!(serde_json::from_value::<QuoteUpdate>(wire).unwrap(), update);
    #[cfg(feature = "dataframe")]
    {
        use paft_utils::dataframe::ToDataFrame;
        use polars::prelude::{AnyValue, DataType, TimeUnit};
        let df = update.to_dataframe().unwrap();
        let column = df.column("ts").unwrap();
        assert_eq!(
            column.dtype(),
            &DataType::Datetime(TimeUnit::Nanoseconds, None)
        );
        assert_eq!(
            column.get(0).unwrap(),
            AnyValue::Datetime(1_666_222_102_061_769_000, TimeUnit::Nanoseconds, None)
        );
    }
}

#[test]
fn public_quote_timestamps_are_checked_at_serialization() {
    let instrument = Instrument::from_symbol("TEST", AssetKind::Equity).unwrap();
    let currency = Currency::Iso(IsoCurrency::USD);
    let mut quote = Quote::new(instrument.clone(), currency.clone());
    let mut update = QuoteUpdate::new(instrument, currency, DateTime::UNIX_EPOCH);
    for (secs, nanos) in [(59, 1_000_000_000), (-1, 1_001_000_000)] {
        let ts = DateTime::from_timestamp(secs, nanos).unwrap();
        quote.as_of = Some(ts);
        update.ts = ts;
        assert!(serde_json::to_string(&quote).is_err());
        assert!(serde_json::to_string(&update).is_err());
        #[cfg(feature = "dataframe")]
        {
            use paft_utils::dataframe::ToDataFrame;
            assert!(quote.to_dataframe().is_err());
            assert!(update.to_dataframe().is_err());
        }
    }
    for nanos in [
        -1,
        0,
        1,
        1_666_222_102_061_769_000,
        1_666_222_102_061_769_123,
    ] {
        let ts = DateTime::from_timestamp_nanos(nanos);
        quote.as_of = Some(ts);
        update.ts = ts;
        let json = serde_json::to_string(&quote).unwrap();
        assert_eq!(serde_json::from_str::<Quote>(&json).unwrap(), quote);
        let json = serde_json::to_string(&update).unwrap();
        assert_eq!(serde_json::from_str::<QuoteUpdate>(&json).unwrap(), update);
        #[cfg(feature = "dataframe")]
        {
            use paft_utils::dataframe::ToDataFrame;
            use polars::prelude::{AnyValue, TimeUnit};
            for (df, column) in [
                (quote.to_dataframe().unwrap(), "as_of"),
                (update.to_dataframe().unwrap(), "ts"),
            ] {
                assert_eq!(
                    df.column(column).unwrap().get(0).unwrap(),
                    AnyValue::Datetime(nanos, TimeUnit::Nanoseconds, None)
                );
            }
        }
    }
}
