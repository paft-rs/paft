#![cfg(all(
    feature = "dataframe",
    feature = "market",
    feature = "fundamentals",
    feature = "aggregates"
))]

use chrono::{DateTime, NaiveDate, Utc};
use paft::aggregates::Snapshot;
use paft::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
use paft::domain::{AssetKind, Instrument};
use paft::fundamentals::{Calendar, KeyStatistics, UpgradeDowngradeRow};
use paft::market::{
    news::NewsArticle,
    options::{OptionContract, OptionContractKey, OptionSide, OptionUpdate},
    orderbook::OrderBook,
    quote::{GenericQuote, Quote, QuoteUpdate},
    responses::history::{Candle, Ohlc},
};
use paft::money::{Currency, IsoCurrency, Price, PriceAmount};
use polars::prelude::{AnyValue, DataType, PolarsError, TimeUnit};
use serde::{Serialize, de::DeserializeOwned};

#[derive(df_derive_macros::ToDataFrame)]
struct OptionalRow<T> {
    row: Option<T>,
}

#[derive(df_derive_macros::ToDataFrame)]
struct ListRows<T> {
    rows: Vec<T>,
}

fn instrument() -> Instrument {
    Instrument::from_symbol("TEST", AssetKind::Equity).unwrap()
}

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn key() -> OptionContractKey {
    OptionContractKey::new(
        instrument(),
        OptionSide::Call,
        Price::new(1.into(), usd()),
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
    )
}

fn assert_instant(value: AnyValue<'_>, ts: DateTime<Utc>) {
    match value {
        AnyValue::Datetime(millis, TimeUnit::Milliseconds, None)
        | AnyValue::DatetimeOwned(millis, TimeUnit::Milliseconds, None) => {
            assert_eq!(DateTime::from_timestamp_millis(millis), Some(ts));
        }
        AnyValue::List(values) => {
            assert_eq!(values.len(), 1);
            assert_instant(values.get(0).unwrap(), ts);
        }
        other => panic!("unexpected timestamp column value: {other:?}"),
    }
}

fn assert_precision_error(error: &PolarsError) {
    assert!(matches!(error, PolarsError::ComputeError(_)));
    assert!(
        error
            .to_string()
            .contains("cannot be preserved as Unix milliseconds")
    );
}

fn exercise<T>(build: impl Fn(DateTime<Utc>) -> T, column: &str)
where
    T: ToDataFrame + Columnar + Clone + Serialize + DeserializeOwned + Eq + std::fmt::Debug,
{
    // The final instant is outside the i64 nanosecond range, but exact in milliseconds.
    for millis in [-1_001, -1, 0, 1, 1_001, 16_725_225_600_123] {
        let ts = DateTime::from_timestamp_millis(millis).unwrap();
        let row = build(ts);
        let json = serde_json::to_string(&row).unwrap();
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), row);
        for df in [
            row.to_dataframe().unwrap(),
            T::columnar_to_dataframe(std::slice::from_ref(&row)).unwrap(),
            T::columnar_from_refs(&[&row]).unwrap(),
        ] {
            assert_instant(df.column(column).unwrap().get(0).unwrap(), ts);
            assert_eq!(df.schema(), T::empty_dataframe().unwrap().schema());
        }
        let nested = OptionalRow {
            row: Some(row.clone()),
        }
        .to_dataframe()
        .unwrap();
        assert_instant(
            nested
                .column(&format!("row.{column}"))
                .unwrap()
                .get(0)
                .unwrap(),
            ts,
        );
        let nested = ListRows { rows: vec![row] }.to_dataframe().unwrap();
        assert_instant(
            nested
                .column(&format!("rows.{column}"))
                .unwrap()
                .get(0)
                .unwrap(),
            ts,
        );
    }

    let valid = build(DateTime::UNIX_EPOCH);
    for (seconds, nanos) in [
        (0, 1),
        (0, 1_000_001),
        (0, 123_456_789),
        (-1, 999_999_999),
        (-1, 998_999_999),
        (59, 1_000_000_000),
        (59, 1_001_000_000),
        (-1, 1_000_000_000),
    ] {
        let invalid = build(DateTime::from_timestamp(seconds, nanos).unwrap());
        assert!(serde_json::to_string(&invalid).is_err());
        assert_precision_error(&invalid.to_dataframe().unwrap_err());
        assert_precision_error(&T::columnar_from_refs(&[&valid, &invalid]).unwrap_err());
        assert_precision_error(&[valid.clone(), invalid.clone()].to_dataframe().unwrap_err());
        let nested = OptionalRow {
            row: Some(invalid.clone()),
        };
        assert_precision_error(&nested.to_dataframe().unwrap_err());
        let nested = ListRows {
            rows: vec![valid.clone(), invalid],
        };
        assert_precision_error(&[nested].to_dataframe().unwrap_err());
    }
    assert_eq!(
        OptionalRow::<T> { row: None }
            .to_dataframe()
            .unwrap()
            .height(),
        1
    );
    assert_eq!(
        ListRows::<T> { rows: vec![] }
            .to_dataframe()
            .unwrap()
            .height(),
        1
    );
}

#[test]
fn market_timestamp_exports_match_the_json_precision_policy() {
    exercise(|ts| QuoteUpdate::new(instrument(), usd(), ts), "ts");
    exercise(
        |ts| {
            let mut row = Quote::new(instrument(), usd());
            row.as_of = Some(ts);
            row
        },
        "as_of",
    );
    exercise(
        |ts| {
            let mut row = OrderBook::new(instrument(), usd());
            row.as_of = Some(ts);
            row
        },
        "as_of",
    );
    exercise(
        |ts| NewsArticle {
            uuid: "news".into(),
            title: "Title".into(),
            publisher: None,
            link: None,
            published_at: ts,
            provider: (),
        },
        "published_at",
    );
    exercise(
        |ts| {
            Candle::new(
                ts,
                usd(),
                Ohlc::new(
                    PriceAmount::new(1.into()),
                    PriceAmount::new(1.into()),
                    PriceAmount::new(1.into()),
                    PriceAmount::new(1.into()),
                ),
            )
        },
        "ts",
    );
    exercise(|ts| OptionUpdate::new(key(), usd(), ts), "ts");
    // Check each optional contract timestamp independently.
    exercise(
        |ts| {
            let mut row = OptionContract::new(key(), usd());
            row.expiration_at = Some(ts);
            row
        },
        "expiration_at",
    );
    exercise(
        |ts| {
            let mut row = OptionContract::new(key(), usd());
            row.last_trade_at = Some(ts);
            row
        },
        "last_trade_at",
    );
}

#[test]
fn fundamentals_and_aggregate_timestamp_exports_match_json() {
    exercise(
        |ts| KeyStatistics {
            as_of: Some(ts),
            ..KeyStatistics::default()
        },
        "as_of",
    );
    exercise(
        |ts| Calendar {
            earnings_dates: vec![ts],
            ex_dividend_date: None,
            dividend_payment_date: None,
        },
        "earnings_dates",
    );
    exercise(
        |ts| UpgradeDowngradeRow {
            ts,
            firm: None,
            from_grade: None,
            to_grade: None,
            action: None,
        },
        "ts",
    );
    exercise(
        |ts| {
            let mut row = Snapshot::new(instrument(), usd());
            row.as_of = Some(ts);
            row
        },
        "as_of",
    );
}

#[test]
fn checked_projection_preserves_borrowed_provider_metadata_and_null_timestamps() {
    #[derive(Default, df_derive_macros::ToDataFrame)]
    struct Metadata<'a> {
        label: &'a str,
    }
    let label = String::from("borrowed");
    let mut row: GenericQuote<Metadata<'_>> = GenericQuote::new(instrument(), usd());
    row.provider = Metadata { label: &label };
    let df = row.to_dataframe().unwrap();
    assert_eq!(
        df.column("provider.label").unwrap().str().unwrap().get(0),
        Some("borrowed")
    );
    let ts = df.column("as_of").unwrap();
    assert_eq!(
        ts.dtype(),
        &DataType::Datetime(TimeUnit::Milliseconds, None)
    );
    assert_eq!(ts.get(0).unwrap(), AnyValue::Null);
}
