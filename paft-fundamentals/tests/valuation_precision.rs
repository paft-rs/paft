use chrono::NaiveDate;
use paft_decimal::{Decimal, parse_decimal};
use paft_fundamentals::{InstitutionalHolder, RevenueEstimate};
use paft_money::{Currency, IsoCurrency, MonetaryAmount, Price, QuantityAmount};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn amount(value: &str) -> MonetaryAmount {
    MonetaryAmount::from_canonical_str(value, usd()).unwrap()
}

fn consensus() -> RevenueEstimate {
    let low = amount("1000000.00");
    let high = amount("1000000.01");
    let avg = low.try_add(&high).unwrap().try_div(&Decimal::TWO).unwrap();
    RevenueEstimate {
        avg: Some(avg),
        low: Some(low),
        high: Some(high),
        year_ago_revenue: Some(amount("999999.00001")),
        num_analysts: Some(2),
        growth: None,
    }
}

fn holding() -> InstitutionalHolder {
    let price = Price::from_canonical_str("1.234567", usd()).unwrap();
    let shares = QuantityAmount::from_decimal(Decimal::TEN).unwrap();
    InstitutionalHolder {
        holder: "Example Fund".into(),
        shares: Some(10),
        date_reported: NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        pct_held: None,
        value: Some(price.try_total(&shares).unwrap()),
    }
}

#[test]
fn consensus_and_holding_values_do_not_round_to_settlement_scale() {
    let estimate = consensus();
    assert_eq!(
        estimate.avg.as_ref().unwrap().amount(),
        parse_decimal("1000000.005").unwrap()
    );
    let wire = serde_json::to_value(&estimate).unwrap();
    assert_eq!(
        wire["avg"],
        serde_json::json!({"amount": "1000000.005", "currency": "USD"})
    );
    assert_eq!(
        serde_json::from_value::<RevenueEstimate>(wire.clone()).unwrap(),
        estimate
    );
    // Every analytical monetary field accepts a sub-cent value, including
    // low/high estimates and the comparable prior-year baseline.
    for field in ["avg", "low", "high", "year_ago_revenue"] {
        let mut subcent = wire.clone();
        subcent[field] = serde_json::json!({"amount": "0.00001", "currency": "USD"});
        let decoded: RevenueEstimate = serde_json::from_value(subcent.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), subcent);
    }

    let holder = holding();
    assert_eq!(
        holder.value.as_ref().unwrap().amount(),
        parse_decimal("12.34567").unwrap()
    );
    let wire = serde_json::to_value(&holder).unwrap();
    assert_eq!(
        wire["value"],
        serde_json::json!({"amount": "12.34567", "currency": "USD"})
    );
    assert_eq!(
        serde_json::from_value::<InstitutionalHolder>(wire).unwrap(),
        holder
    );
}

#[cfg(feature = "dataframe")]
#[test]
fn valuation_dataframes_preserve_subcent_zero_and_absent_values() {
    use paft_utils::dataframe::{Columnar, ToDataFrame};
    use polars::prelude::{DataFrame, DataType};

    fn check<T: ToDataFrame + Columnar>(rows: &[T], field: &str, expected: i128) -> DataFrame {
        let frame = T::columnar_to_dataframe(rows).unwrap();
        assert_eq!(frame.schema(), T::empty_dataframe().unwrap().schema());
        assert!(
            frame
                .get_column_names()
                .iter()
                .all(|name| !name.ends_with("minor_units"))
        );
        let column = frame.column(&format!("{field}.amount")).unwrap();
        assert_eq!(column.dtype(), &DataType::Decimal(38, 10));
        let values = column.decimal().unwrap().physical();
        assert_eq!(
            [values.get(0), values.get(1), values.get(2)],
            [Some(expected), Some(0), None]
        );
        let currencies = frame
            .column(&format!("{field}.currency"))
            .unwrap()
            .str()
            .unwrap();
        assert_eq!(
            [currencies.get(0), currencies.get(1), currencies.get(2)],
            [Some("USD"), Some("USD"), None]
        );
        for (i, row) in rows.iter().enumerate() {
            assert!(
                frame
                    .slice(i64::try_from(i).unwrap(), 1)
                    .equals_missing(&row.to_dataframe().unwrap())
            );
        }
        frame
    }

    let zero = RevenueEstimate {
        avg: Some(amount("0")),
        low: Some(amount("0")),
        high: Some(amount("0")),
        year_ago_revenue: Some(amount("0")),
        ..RevenueEstimate::default()
    };
    let estimates = [consensus(), zero, RevenueEstimate::default()];
    for (field, expected) in [
        ("avg", 10_000_000_050_000_000),
        ("low", 10_000_000_000_000_000),
        ("high", 10_000_000_100_000_000),
        ("year_ago_revenue", 9_999_990_000_100_000),
    ] {
        check(&estimates, field, expected);
    }
    let mut zero = holding();
    zero.value = Some(amount("0"));
    let mut absent = holding();
    absent.value = None;
    check(&[holding(), zero, absent], "value", 123_456_700_000);
}
