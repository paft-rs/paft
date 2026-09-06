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
        AnyValue::Datetime(nanos, TimeUnit::Nanoseconds, None)
        | AnyValue::DatetimeOwned(nanos, TimeUnit::Nanoseconds, None) => {
            assert_eq!(nanos, ts.timestamp_nanos_opt().unwrap());
            assert_eq!(DateTime::from_timestamp_nanos(nanos), ts);
        }
        AnyValue::List(values) => {
            assert_eq!(values.len(), 1);
            assert_instant(values.get(0).unwrap(), ts);
        }
        other => panic!("unexpected timestamp column value: {other:?}"),
    }
}

fn assert_instant_dtype(dtype: &DataType) {
    if let DataType::List(inner) = dtype {
        assert_instant_dtype(inner);
    } else {
        assert_eq!(dtype, &DataType::Datetime(TimeUnit::Nanoseconds, None));
    }
}

fn assert_export_error(error: &PolarsError, reason: &str, column: &str, ts: DateTime<Utc>) {
    assert!(matches!(error, PolarsError::ComputeError(_)));
    let message = error.to_string();
    assert!(message.contains(reason), "{message}");
    assert!(message.contains(column), "{message}");
    assert!(
        message.contains(&ts.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)),
        "{message}"
    );
}

fn exercise<T>(build: impl Fn(DateTime<Utc>) -> T, column: &str)
where
    T: ToDataFrame + Columnar + Clone + Serialize + DeserializeOwned + Eq + std::fmt::Debug,
{
    for nanos in [
        i64::MIN,
        -1,
        0,
        1,
        1_666_222_102_061_769_000,
        1_666_222_102_061_769_123,
        i64::MAX,
    ] {
        let ts = DateTime::from_timestamp_nanos(nanos);
        let row = build(ts);
        let json = serde_json::to_string(&row).unwrap();
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), row);
        for df in [
            row.to_dataframe().unwrap(),
            T::columnar_to_dataframe(std::slice::from_ref(&row)).unwrap(),
            T::columnar_from_refs(&[&row]).unwrap(),
        ] {
            assert_instant_dtype(df.column(column).unwrap().dtype());
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

    exercise_rejections(&build, column);
    for df in [
        OptionalRow::<T> { row: None }.to_dataframe().unwrap(),
        [
            OptionalRow::<T> { row: None },
            OptionalRow::<T> { row: None },
        ]
        .to_dataframe()
        .unwrap(),
        OptionalRow::<T>::empty_dataframe().unwrap(),
    ] {
        let col = df.column(&format!("row.{column}")).unwrap();
        assert_instant_dtype(col.dtype());
        assert_eq!(col.null_count(), df.height());
    }
    for df in [
        ListRows::<T> { rows: vec![] }.to_dataframe().unwrap(),
        ListRows::<T>::empty_dataframe().unwrap(),
    ] {
        assert_instant_dtype(df.column(&format!("rows.{column}")).unwrap().dtype());
    }
}

fn exercise_rejections<T>(build: &impl Fn(DateTime<Utc>) -> T, column: &str)
where
    T: ToDataFrame + Columnar + Clone + Serialize + DeserializeOwned + Eq + std::fmt::Debug,
{
    let valid = build(DateTime::UNIX_EPOCH);
    for (ts, reason, json_valid) in [
        (
            DateTime::<Utc>::MIN_UTC,
            "outside the DataFrame Unix nanosecond range",
            true,
        ),
        (
            DateTime::<Utc>::MAX_UTC,
            "outside the DataFrame Unix nanosecond range",
            true,
        ),
        (
            DateTime::from_timestamp_nanos(i64::MIN)
                .checked_sub_signed(chrono::TimeDelta::nanoseconds(1))
                .unwrap(),
            "outside the DataFrame Unix nanosecond range",
            true,
        ),
        (
            DateTime::from_timestamp_nanos(i64::MAX)
                .checked_add_signed(chrono::TimeDelta::nanoseconds(1))
                .unwrap(),
            "outside the DataFrame Unix nanosecond range",
            true,
        ),
        (
            DateTime::from_timestamp(59, 1_000_000_000).unwrap(),
            "leap seconds",
            false,
        ),
        (
            DateTime::from_timestamp(-1, 1_001_000_000).unwrap(),
            "leap seconds",
            false,
        ),
    ] {
        let invalid = build(ts);
        let json = serde_json::to_string(&invalid);
        assert_eq!(json.is_ok(), json_valid);
        if let Ok(json) = json {
            assert_eq!(serde_json::from_str::<T>(&json).unwrap(), invalid);
        }
        assert_export_error(&invalid.to_dataframe().unwrap_err(), reason, column, ts);
        let error = T::columnar_from_refs(&[&valid, &invalid]).unwrap_err();
        assert_export_error(&error, reason, column, ts);
        assert!(error.to_string().contains("[1]."));
        assert_export_error(
            &[valid.clone(), invalid.clone()].to_dataframe().unwrap_err(),
            reason,
            column,
            ts,
        );
        let nested = OptionalRow {
            row: Some(invalid.clone()),
        };
        assert_export_error(&nested.to_dataframe().unwrap_err(), reason, column, ts);
        let nested = ListRows {
            rows: vec![valid.clone(), invalid],
        };
        assert_export_error(&[nested].to_dataframe().unwrap_err(), reason, column, ts);
    }
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
    assert_eq!(ts.dtype(), &DataType::Datetime(TimeUnit::Nanoseconds, None));
    assert_eq!(ts.get(0).unwrap(), AnyValue::Null);
}
