use chrono::DateTime;
use paft_domain::{AssetKind, Instrument};
use paft_market::market::quote::{Quote, QuoteUpdate};
use paft_money::{Currency, IsoCurrency};

#[test]
fn public_quote_timestamps_are_checked_at_serialization() {
    let instrument = Instrument::from_symbol("TEST", AssetKind::Equity).unwrap();
    let currency = Currency::Iso(IsoCurrency::USD);
    let mut quote = Quote::new(instrument.clone(), currency.clone());
    let mut update = QuoteUpdate::new(instrument, currency, DateTime::UNIX_EPOCH);
    for (secs, nanos) in [(0, 1_000_001), (-1, 999_999_999), (59, 1_000_000_000)] {
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
    for millis in [-1, 0, 1] {
        let ts = DateTime::from_timestamp_millis(millis).unwrap();
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
                    AnyValue::Datetime(millis, TimeUnit::Milliseconds, None)
                );
            }
        }
    }
}
