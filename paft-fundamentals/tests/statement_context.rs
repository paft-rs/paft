use chrono::NaiveDate;
use paft_fundamentals::{
    BalanceSheetRow, CashflowRow, IncomeStatementRow, StatementDuration, StatementInstant,
};
use serde_json::{from_value, json, to_value};

fn date(value: &str) -> NaiveDate {
    value.parse().unwrap()
}

#[test]
fn duration_validation_matches_serde_and_includes_both_boundary_dates() {
    let start = date("2024-04-01");
    let end = date("2024-06-30");
    let window = StatementDuration::new(start, end).unwrap();
    assert_eq!((window.start(), window.end()), (start, end));
    assert_eq!(
        to_value(window).unwrap(),
        json!({"start":"2024-04-01","end":"2024-06-30"})
    );
    assert_eq!(
        from_value::<StatementDuration>(to_value(window).unwrap()).unwrap(),
        window
    );
    assert!(StatementDuration::new(end, start).is_err());
    assert!(
        from_value::<StatementDuration>(json!({"start":"2024-06-30","end":"2024-04-01"})).is_err()
    );
    assert!(StatementDuration::new(start, start).is_ok());
    for invalid in [
        json!({"start":"2024-04-01"}),
        json!({"start":"2024-04-01","end":"2024-06-30","basis":"ytd"}),
        json!({"start":"2024-02-30","end":"2024-06-30"}),
        json!({"date":"2024-06-30"}),
    ] {
        assert!(from_value::<StatementDuration>(invalid).is_err());
    }
    assert!(
        from_value::<StatementInstant>(json!({"date":"2024-06-30","start":"2024-04-01"})).is_err()
    );
}

fn cashflow(start: &str) -> CashflowRow {
    from_value(json!({
        "period":"2024-Q2",
        "window":{"start":start,"end":"2024-06-30"},
        "operating_cashflow":{"amount":"100","currency":"USD","minor_units":2},
        "end_cash_position":{"amount":"500","currency":"USD","minor_units":2}
    }))
    .unwrap()
}

#[test]
fn equal_fiscal_labels_and_amounts_do_not_erase_different_flow_windows() {
    let quarter = cashflow("2024-04-01");
    let ytd = cashflow("2024-01-01");
    assert_eq!(quarter.period, ytd.period);
    assert_eq!(quarter.operating_cashflow, ytd.operating_cashflow);
    assert_ne!(quarter, ytd);
    for row in [quarter, ytd] {
        let json = to_value(&row).unwrap();
        assert_eq!(json["window"]["start"], row.window.start().to_string());
        assert_eq!(from_value::<CashflowRow>(json).unwrap(), row);
    }
    let income: IncomeStatementRow = from_value(json!({
        "period":"2024-Q2", "window":{"start":"2024-01-01","end":"2024-06-30"},
        "future_provider_field":true
    }))
    .unwrap();
    assert_eq!(
        from_value::<IncomeStatementRow>(to_value(&income).unwrap()).unwrap(),
        income
    );
    let balance: BalanceSheetRow =
        from_value(json!({"period":"2024-Q2","as_of":{"date":"2024-06-30"}})).unwrap();
    assert_eq!(balance.as_of, StatementInstant::new(date("2024-06-30")));
    assert_eq!(
        from_value::<BalanceSheetRow>(to_value(&balance).unwrap()).unwrap(),
        balance
    );
}

#[test]
fn rows_require_their_measurement_context() {
    for payload in [
        json!({"period":"2024-Q2"}),
        json!({"period":"2024-Q2","window":null,"as_of":null}),
    ] {
        assert!(from_value::<IncomeStatementRow>(payload.clone()).is_err());
        assert!(from_value::<CashflowRow>(payload.clone()).is_err());
        assert!(from_value::<BalanceSheetRow>(payload).is_err());
    }
    assert!(
        from_value::<IncomeStatementRow>(
            json!({"period":"2024-Q2","window":{"date":"2024-06-30"}})
        )
        .is_err()
    );
    assert!(
        from_value::<BalanceSheetRow>(
            json!({"period":"2024-Q2","as_of":{"start":"2024-04-01","end":"2024-06-30"}})
        )
        .is_err()
    );
}

#[cfg(feature = "dataframe")]
#[test]
fn dataframe_preserves_flow_windows_and_balance_instants() {
    use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};
    use polars::prelude::{AnyValue, DataType};
    let rows = [cashflow("2024-04-01"), cashflow("2024-01-01")];
    let days = |value: NaiveDate| {
        i32::try_from(value.signed_duration_since(date("1970-01-01")).num_days()).unwrap()
    };
    let df = rows.to_dataframe().unwrap();
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(
            df.column("window.start").unwrap().get(index).unwrap(),
            AnyValue::Date(days(row.window.start()))
        );
        assert_eq!(
            df.column("window.end").unwrap().get(index).unwrap(),
            AnyValue::Date(days(row.window.end()))
        );
        assert!(
            row.to_dataframe()
                .unwrap()
                .equals_missing(&df.slice(i64::try_from(index).unwrap(), 1))
        );
    }
    assert_ne!(
        df.column("window.start").unwrap().get(0).unwrap(),
        df.column("window.start").unwrap().get(1).unwrap()
    );
    assert_eq!(
        CashflowRow::empty_dataframe().unwrap().schema(),
        df.schema()
    );
    let balance: BalanceSheetRow =
        from_value(json!({"period":"2024-Q2","as_of":{"date":"2024-06-30"}})).unwrap();
    let df = balance.to_dataframe().unwrap();
    assert_eq!(df.column("as_of.date").unwrap().dtype(), &DataType::Date);
    assert_eq!(
        df.column("as_of.date").unwrap().get(0).unwrap(),
        AnyValue::Date(days(balance.as_of.date))
    );
}
