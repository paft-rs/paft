//! Serde tests for the financial statement rows.
//!
//! These pin the exact JSON wire format of the statement rows - field names,
//! the encoding of each value type, and missing-field behaviour - so that a
//! change to any of them is a deliberate, visible break rather than a silent
//! one for stored payloads and provider mappers.

use paft_decimal::Decimal;
use paft_domain::ReportingPeriod;
use paft_fundamentals::{BalanceSheetRow, CashflowRow, IncomeStatementRow};
use paft_money::{Currency, IsoCurrency, Money, Price, QuantityAmount};
use serde_json::{Value, from_str, json, to_string};
use std::str::FromStr;

fn usd(amount: &str) -> Money {
    Money::new(
        Decimal::from_str(amount).unwrap(),
        Currency::Iso(IsoCurrency::USD),
    )
    .unwrap()
}

fn usd_price(amount: &str) -> Price {
    Price::new(
        Decimal::from_str(amount).unwrap(),
        Currency::Iso(IsoCurrency::USD),
    )
}

fn shares(amount: &str) -> QuantityAmount {
    QuantityAmount::from_decimal(Decimal::from_str(amount).unwrap()).unwrap()
}

fn income_row() -> IncomeStatementRow {
    IncomeStatementRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        total_revenue: Some(usd("10000")),
        cost_of_revenue: Some(usd("4000")),
        gross_profit: Some(usd("6000")),
        research_and_development: Some(usd("1000")),
        selling_general_and_administrative: Some(usd("1500")),
        operating_expenses: Some(usd("2500")),
        operating_income: Some(usd("3500")),
        interest_income: Some(usd("120")),
        interest_expense: Some(usd("200")),
        // ebit = pretax_income + interest_expense - interest_income
        //      = 3420 + 200 - 120; ebitda = ebit + d&a = 3500 + 600.
        ebit: Some(usd("3500")),
        ebitda: Some(usd("4100")),
        pretax_income: Some(usd("3420")),
        income_tax_expense: Some(usd("700")),
        depreciation_and_amortization: Some(usd("600")),
        net_income: Some(usd("2720")),
        net_income_common_stockholders: Some(usd("2700")),
        basic_eps: Some(usd_price("2.72")),
        diluted_eps: Some(usd_price("2.69")),
        basic_average_shares: Some(shares("1000000.25")),
        diluted_average_shares: Some(shares("1010000.5")),
    }
}

fn cashflow_row() -> CashflowRow {
    CashflowRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        operating_cashflow: Some(usd("1200")),
        capital_expenditures: Some(usd("-300")),
        free_cash_flow: Some(usd("900")),
        net_income: Some(usd("700")),
        depreciation_and_amortization: Some(usd("250")),
        stock_based_compensation: Some(usd("80")),
        change_in_working_capital: Some(usd("-40")),
        investing_cashflow: Some(usd("-500")),
        financing_cashflow: Some(usd("-10")),
        issuance_of_debt: Some(usd("300")),
        repayment_of_debt: Some(usd("-150")),
        repurchase_of_capital_stock: Some(usd("-100")),
        cash_dividends_paid: Some(usd("-60")),
        end_cash_position: Some(usd("1000")),
    }
}

fn balance_sheet_row() -> BalanceSheetRow {
    BalanceSheetRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        total_assets: Some(usd("5000")),
        total_liabilities: Some(usd("2000")),
        total_equity: Some(usd("3000")),
        cash: Some(usd("600")),
        long_term_debt: Some(usd("1200")),
        shares_outstanding: Some(1_000_000),
        current_assets: Some(usd("2000")),
        current_liabilities: Some(usd("600")),
        accounts_receivable: Some(usd("400")),
        inventory: Some(usd("300")),
        accounts_payable: Some(usd("250")),
        net_property_plant_equipment: Some(usd("1800")),
        goodwill: Some(usd("700")),
        intangible_assets: Some(usd("250")),
        total_debt: Some(usd("1500")),
        current_debt: Some(usd("300")),
        cash_and_short_term_investments: Some(usd("800")),
        other_current_assets: Some(usd("150")),
        other_current_liabilities: Some(usd("120")),
        retained_earnings: Some(usd("900")),
        common_stock: Some(usd("100")),
        treasury_stock: Some(usd("-50")),
        minority_interest: None,
        working_capital: Some(usd("1400")),
        tangible_book_value: Some(usd("2050")),
    }
}

/// Every field of every statement row must be present in the serialized object,
/// under the exact name declared on the struct, and no others. JSON object key
/// order is not significant, so the key sets are compared sorted.
#[test]
fn statement_rows_serialize_every_field_under_its_declared_name() {
    let income: Value = from_str(&to_string(&income_row()).unwrap()).unwrap();
    let mut expected_income = vec![
        "period",
        "total_revenue",
        "cost_of_revenue",
        "gross_profit",
        "research_and_development",
        "selling_general_and_administrative",
        "operating_expenses",
        "operating_income",
        "interest_income",
        "interest_expense",
        "ebit",
        "ebitda",
        "pretax_income",
        "income_tax_expense",
        "depreciation_and_amortization",
        "net_income",
        "net_income_common_stockholders",
        "basic_eps",
        "diluted_eps",
        "basic_average_shares",
        "diluted_average_shares",
    ];
    let mut actual: Vec<&str> = income
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    actual.sort_unstable();
    expected_income.sort_unstable();
    assert_eq!(actual, expected_income);

    let balance: Value = from_str(&to_string(&balance_sheet_row()).unwrap()).unwrap();
    let mut expected_balance = vec![
        "period",
        "total_assets",
        "total_liabilities",
        "total_equity",
        "cash",
        "long_term_debt",
        "shares_outstanding",
        "current_assets",
        "current_liabilities",
        "accounts_receivable",
        "inventory",
        "accounts_payable",
        "net_property_plant_equipment",
        "goodwill",
        "intangible_assets",
        "total_debt",
        "current_debt",
        "cash_and_short_term_investments",
        "other_current_assets",
        "other_current_liabilities",
        "retained_earnings",
        "common_stock",
        "treasury_stock",
        "minority_interest",
        "working_capital",
        "tangible_book_value",
    ];
    let mut actual: Vec<&str> = balance
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    actual.sort_unstable();
    expected_balance.sort_unstable();
    assert_eq!(actual, expected_balance);

    let cashflow: Value = from_str(&to_string(&cashflow_row()).unwrap()).unwrap();
    let mut expected_cashflow = vec![
        "period",
        "operating_cashflow",
        "capital_expenditures",
        "free_cash_flow",
        "net_income",
        "depreciation_and_amortization",
        "stock_based_compensation",
        "change_in_working_capital",
        "investing_cashflow",
        "financing_cashflow",
        "issuance_of_debt",
        "repayment_of_debt",
        "repurchase_of_capital_stock",
        "cash_dividends_paid",
        "end_cash_position",
    ];
    let mut actual: Vec<&str> = cashflow
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    actual.sort_unstable();
    expected_cashflow.sort_unstable();
    assert_eq!(actual, expected_cashflow);
}

/// Each value type on a statement row has its own wire encoding: `Money`
/// carries its captured `minor_units`, `Price` does not, and `QuantityAmount`
/// is a transparent decimal string. `shares_outstanding` stays a JSON integer.
#[test]
fn statement_value_types_have_their_expected_wire_encodings() {
    let income: Value = from_str(&to_string(&income_row()).unwrap()).unwrap();

    assert_eq!(income["period"], json!("2024"));
    assert_eq!(
        income["total_revenue"],
        json!({"amount": "10000", "currency": "USD", "minor_units": 2})
    );
    assert_eq!(
        income["basic_eps"],
        json!({"amount": "2.72", "currency": "USD"})
    );
    // Weighted-average shares are fractional: a transparent decimal *string*,
    // not a JSON number.
    assert_eq!(income["basic_average_shares"], json!("1000000.25"));
    assert_eq!(income["diluted_average_shares"], json!("1010000.5"));

    // The point-in-time share count stays an integral JSON number.
    let balance: Value = from_str(&to_string(&balance_sheet_row()).unwrap()).unwrap();
    assert_eq!(balance["shares_outstanding"], json!(1_000_000));
    assert!(balance["minority_interest"].is_null());
}

/// Cash outflows and negative reconciliation adjustments keep their
/// negative sign through a serde round trip.
#[test]
fn cashflow_negative_signs_survive_serialization() {
    let cashflow: Value = from_str(&to_string(&cashflow_row()).unwrap()).unwrap();
    for field in [
        "capital_expenditures",
        "repayment_of_debt",
        "repurchase_of_capital_stock",
        "cash_dividends_paid",
        "change_in_working_capital",
        "investing_cashflow",
    ] {
        assert!(
            cashflow[field]["amount"].as_str().unwrap().starts_with('-'),
            "{field} should preserve its negative sign"
        );
    }
    assert_eq!(cashflow["issuance_of_debt"]["amount"], json!("300"));
    assert_eq!(cashflow["end_cash_position"]["amount"], json!("1000"));
}

#[test]
fn statement_rows_round_trip_through_json() {
    let income = income_row();
    assert_eq!(
        from_str::<IncomeStatementRow>(&to_string(&income).unwrap()).unwrap(),
        income
    );

    let balance = balance_sheet_row();
    assert_eq!(
        from_str::<BalanceSheetRow>(&to_string(&balance).unwrap()).unwrap(),
        balance
    );

    let cashflow = cashflow_row();
    assert_eq!(
        from_str::<CashflowRow>(&to_string(&cashflow).unwrap()).unwrap(),
        cashflow
    );
}

/// Statement rows are forward-compatible payloads: omitting any optional field
/// deserializes to `None` rather than failing, so payloads written before these
/// fields existed still load.
#[test]
fn omitted_optional_fields_deserialize_to_none() {
    let income: IncomeStatementRow = from_str(r#"{"period":"2024"}"#).unwrap();
    assert_eq!(income.period, ReportingPeriod::annual(2024).unwrap());
    assert!(income.total_revenue.is_none());
    assert!(income.ebitda.is_none());
    assert!(income.basic_average_shares.is_none());

    let balance: BalanceSheetRow = from_str(r#"{"period":"2024"}"#).unwrap();
    assert!(balance.shares_outstanding.is_none());
    assert!(balance.tangible_book_value.is_none());

    let cashflow: CashflowRow = from_str(r#"{"period":"2024"}"#).unwrap();
    assert!(cashflow.stock_based_compensation.is_none());
    assert!(cashflow.end_cash_position.is_none());
}

/// Weighted-average share counts are non-negative by construction: a negative
/// value is rejected at deserialization rather than silently accepted.
#[test]
fn negative_weighted_average_shares_are_rejected() {
    let json = r#"{"period":"2024","basic_average_shares":"-1000"}"#;
    assert!(from_str::<IncomeStatementRow>(json).is_err());
}
