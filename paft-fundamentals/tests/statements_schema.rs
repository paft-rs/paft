#![cfg(feature = "dataframe")]
//! Exact `DataFrame` schema assertions for the financial statement rows.
//!
//! These pin the full ordered column list and dtype of every statement row, so
//! adding, removing, renaming, or retyping a field is a visible, deliberate
//! change rather than a silent break for downstream consumers that read these
//! frames by column name.

use paft_decimal::Decimal;
use paft_domain::ReportingPeriod;
use paft_fundamentals::{BalanceSheetRow, CashflowRow, IncomeStatementRow};
use paft_money::{Currency, IsoCurrency, Money, Price, QuantityAmount};
use paft_utils::dataframe::ToDataFrame;
use polars::prelude::{DataFrame, DataType};
use std::str::FromStr;

fn usd(amount: i64) -> Money {
    Money::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD)).unwrap()
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

/// Asserts the frame's columns match `expected` exactly, in order and dtype.
fn assert_schema(df: &DataFrame, expected: &[(&str, DataType)]) {
    let actual: Vec<(String, DataType)> = df
        .schema()
        .iter()
        .map(|(name, dtype)| (name.to_string(), dtype.clone()))
        .collect();
    let expected: Vec<(String, DataType)> = expected
        .iter()
        .map(|(name, dtype)| ((*name).to_string(), dtype.clone()))
        .collect();
    assert_eq!(actual, expected);
}

const INCOME_STATEMENT_SCHEMA: &[(&str, DataType)] = &[
    ("period", DataType::String),
    ("total_revenue.amount", DataType::Decimal(38, 10)),
    ("total_revenue.currency", DataType::String),
    ("cost_of_revenue.amount", DataType::Decimal(38, 10)),
    ("cost_of_revenue.currency", DataType::String),
    ("gross_profit.amount", DataType::Decimal(38, 10)),
    ("gross_profit.currency", DataType::String),
    ("research_and_development.amount", DataType::Decimal(38, 10)),
    ("research_and_development.currency", DataType::String),
    (
        "selling_general_and_administrative.amount",
        DataType::Decimal(38, 10),
    ),
    (
        "selling_general_and_administrative.currency",
        DataType::String,
    ),
    ("operating_expenses.amount", DataType::Decimal(38, 10)),
    ("operating_expenses.currency", DataType::String),
    ("operating_income.amount", DataType::Decimal(38, 10)),
    ("operating_income.currency", DataType::String),
    ("interest_income.amount", DataType::Decimal(38, 10)),
    ("interest_income.currency", DataType::String),
    ("interest_expense.amount", DataType::Decimal(38, 10)),
    ("interest_expense.currency", DataType::String),
    ("ebit.amount", DataType::Decimal(38, 10)),
    ("ebit.currency", DataType::String),
    ("ebitda.amount", DataType::Decimal(38, 10)),
    ("ebitda.currency", DataType::String),
    ("pretax_income.amount", DataType::Decimal(38, 10)),
    ("pretax_income.currency", DataType::String),
    ("income_tax_expense.amount", DataType::Decimal(38, 10)),
    ("income_tax_expense.currency", DataType::String),
    (
        "depreciation_and_amortization.amount",
        DataType::Decimal(38, 10),
    ),
    ("depreciation_and_amortization.currency", DataType::String),
    ("net_income.amount", DataType::Decimal(38, 10)),
    ("net_income.currency", DataType::String),
    (
        "net_income_common_stockholders.amount",
        DataType::Decimal(38, 10),
    ),
    ("net_income_common_stockholders.currency", DataType::String),
    ("basic_eps.amount", DataType::Decimal(38, 10)),
    ("basic_eps.currency", DataType::String),
    ("diluted_eps.amount", DataType::Decimal(38, 10)),
    ("diluted_eps.currency", DataType::String),
    ("basic_average_shares.amount", DataType::Decimal(38, 10)),
    ("diluted_average_shares.amount", DataType::Decimal(38, 10)),
];

const BALANCE_SHEET_SCHEMA: &[(&str, DataType)] = &[
    ("period", DataType::String),
    ("total_assets.amount", DataType::Decimal(38, 10)),
    ("total_assets.currency", DataType::String),
    ("total_liabilities.amount", DataType::Decimal(38, 10)),
    ("total_liabilities.currency", DataType::String),
    ("total_equity.amount", DataType::Decimal(38, 10)),
    ("total_equity.currency", DataType::String),
    ("cash.amount", DataType::Decimal(38, 10)),
    ("cash.currency", DataType::String),
    ("long_term_debt.amount", DataType::Decimal(38, 10)),
    ("long_term_debt.currency", DataType::String),
    ("shares_outstanding", DataType::UInt64),
    ("current_assets.amount", DataType::Decimal(38, 10)),
    ("current_assets.currency", DataType::String),
    ("current_liabilities.amount", DataType::Decimal(38, 10)),
    ("current_liabilities.currency", DataType::String),
    ("accounts_receivable.amount", DataType::Decimal(38, 10)),
    ("accounts_receivable.currency", DataType::String),
    ("inventory.amount", DataType::Decimal(38, 10)),
    ("inventory.currency", DataType::String),
    ("accounts_payable.amount", DataType::Decimal(38, 10)),
    ("accounts_payable.currency", DataType::String),
    (
        "net_property_plant_equipment.amount",
        DataType::Decimal(38, 10),
    ),
    ("net_property_plant_equipment.currency", DataType::String),
    ("goodwill.amount", DataType::Decimal(38, 10)),
    ("goodwill.currency", DataType::String),
    ("intangible_assets.amount", DataType::Decimal(38, 10)),
    ("intangible_assets.currency", DataType::String),
    ("total_debt.amount", DataType::Decimal(38, 10)),
    ("total_debt.currency", DataType::String),
    ("current_debt.amount", DataType::Decimal(38, 10)),
    ("current_debt.currency", DataType::String),
    (
        "cash_and_short_term_investments.amount",
        DataType::Decimal(38, 10),
    ),
    ("cash_and_short_term_investments.currency", DataType::String),
    ("other_current_assets.amount", DataType::Decimal(38, 10)),
    ("other_current_assets.currency", DataType::String),
    (
        "other_current_liabilities.amount",
        DataType::Decimal(38, 10),
    ),
    ("other_current_liabilities.currency", DataType::String),
    ("retained_earnings.amount", DataType::Decimal(38, 10)),
    ("retained_earnings.currency", DataType::String),
    ("common_stock.amount", DataType::Decimal(38, 10)),
    ("common_stock.currency", DataType::String),
    ("treasury_stock.amount", DataType::Decimal(38, 10)),
    ("treasury_stock.currency", DataType::String),
    ("minority_interest.amount", DataType::Decimal(38, 10)),
    ("minority_interest.currency", DataType::String),
    ("working_capital.amount", DataType::Decimal(38, 10)),
    ("working_capital.currency", DataType::String),
    ("tangible_book_value.amount", DataType::Decimal(38, 10)),
    ("tangible_book_value.currency", DataType::String),
];

const CASHFLOW_SCHEMA: &[(&str, DataType)] = &[
    ("period", DataType::String),
    ("operating_cashflow.amount", DataType::Decimal(38, 10)),
    ("operating_cashflow.currency", DataType::String),
    ("capital_expenditures.amount", DataType::Decimal(38, 10)),
    ("capital_expenditures.currency", DataType::String),
    ("free_cash_flow.amount", DataType::Decimal(38, 10)),
    ("free_cash_flow.currency", DataType::String),
    ("net_income.amount", DataType::Decimal(38, 10)),
    ("net_income.currency", DataType::String),
    (
        "depreciation_and_amortization.amount",
        DataType::Decimal(38, 10),
    ),
    ("depreciation_and_amortization.currency", DataType::String),
    ("stock_based_compensation.amount", DataType::Decimal(38, 10)),
    ("stock_based_compensation.currency", DataType::String),
    (
        "change_in_working_capital.amount",
        DataType::Decimal(38, 10),
    ),
    ("change_in_working_capital.currency", DataType::String),
    ("investing_cashflow.amount", DataType::Decimal(38, 10)),
    ("investing_cashflow.currency", DataType::String),
    ("financing_cashflow.amount", DataType::Decimal(38, 10)),
    ("financing_cashflow.currency", DataType::String),
    ("issuance_of_debt.amount", DataType::Decimal(38, 10)),
    ("issuance_of_debt.currency", DataType::String),
    ("repayment_of_debt.amount", DataType::Decimal(38, 10)),
    ("repayment_of_debt.currency", DataType::String),
    (
        "repurchase_of_capital_stock.amount",
        DataType::Decimal(38, 10),
    ),
    ("repurchase_of_capital_stock.currency", DataType::String),
    ("cash_dividends_paid.amount", DataType::Decimal(38, 10)),
    ("cash_dividends_paid.currency", DataType::String),
    ("end_cash_position.amount", DataType::Decimal(38, 10)),
    ("end_cash_position.currency", DataType::String),
];

#[test]
fn income_statement_row_dataframe_schema_is_exact() {
    let row = IncomeStatementRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        total_revenue: Some(usd(10_000)),
        cost_of_revenue: Some(usd(4_000)),
        gross_profit: Some(usd(6_000)),
        research_and_development: Some(usd(1_000)),
        selling_general_and_administrative: Some(usd(1_500)),
        operating_expenses: Some(usd(2_500)),
        operating_income: Some(usd(3_500)),
        interest_income: Some(usd(120)),
        interest_expense: Some(usd(200)),
        // ebit = pretax_income + interest_expense - interest_income
        //      = 3420 + 200 - 120; ebitda = ebit + d&a = 3500 + 600.
        ebit: Some(usd(3_500)),
        ebitda: Some(usd(4_100)),
        pretax_income: Some(usd(3_420)),
        income_tax_expense: Some(usd(700)),
        depreciation_and_amortization: Some(usd(600)),
        net_income: Some(usd(2_720)),
        net_income_common_stockholders: Some(usd(2_700)),
        basic_eps: Some(usd_price("2.72")),
        diluted_eps: Some(usd_price("2.69")),
        basic_average_shares: Some(shares("1000000.25")),
        diluted_average_shares: Some(shares("1010000.5")),
    };

    let df = row.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    assert_schema(&df, INCOME_STATEMENT_SCHEMA);
}

#[test]
fn balance_sheet_row_dataframe_schema_is_exact() {
    let row = BalanceSheetRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        total_assets: Some(usd(5_000)),
        total_liabilities: Some(usd(2_000)),
        total_equity: Some(usd(3_000)),
        cash: Some(usd(600)),
        long_term_debt: Some(usd(1_200)),
        shares_outstanding: Some(1_000_000),
        current_assets: Some(usd(2_000)),
        current_liabilities: Some(usd(600)),
        accounts_receivable: Some(usd(400)),
        inventory: Some(usd(300)),
        accounts_payable: Some(usd(250)),
        net_property_plant_equipment: Some(usd(1_800)),
        goodwill: Some(usd(700)),
        intangible_assets: Some(usd(250)),
        total_debt: Some(usd(1_500)),
        current_debt: Some(usd(300)),
        cash_and_short_term_investments: Some(usd(800)),
        other_current_assets: Some(usd(150)),
        other_current_liabilities: Some(usd(120)),
        retained_earnings: Some(usd(900)),
        common_stock: Some(usd(100)),
        treasury_stock: Some(usd(-50)),
        minority_interest: None,
        working_capital: Some(usd(1_400)),
        tangible_book_value: Some(usd(2_050)),
    };

    let df = row.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    assert_schema(&df, BALANCE_SHEET_SCHEMA);
}

#[test]
fn cashflow_row_dataframe_schema_is_exact() {
    let row = CashflowRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        operating_cashflow: Some(usd(1_200)),
        capital_expenditures: Some(usd(-300)),
        free_cash_flow: Some(usd(900)),
        net_income: Some(usd(700)),
        depreciation_and_amortization: Some(usd(250)),
        stock_based_compensation: Some(usd(80)),
        change_in_working_capital: Some(usd(-40)),
        investing_cashflow: Some(usd(-500)),
        financing_cashflow: Some(usd(-10)),
        issuance_of_debt: Some(usd(300)),
        repayment_of_debt: Some(usd(-150)),
        repurchase_of_capital_stock: Some(usd(-100)),
        cash_dividends_paid: Some(usd(-60)),
        end_cash_position: Some(usd(1_000)),
    };

    let df = row.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    assert_schema(&df, CASHFLOW_SCHEMA);
}

/// A fully-`None` row must still produce the identical schema, so downstream
/// consumers can concatenate sparse and dense rows.
#[test]
fn all_none_rows_produce_the_same_schema() {
    let income = IncomeStatementRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        total_revenue: None,
        cost_of_revenue: None,
        gross_profit: None,
        research_and_development: None,
        selling_general_and_administrative: None,
        operating_expenses: None,
        operating_income: None,
        interest_income: None,
        interest_expense: None,
        ebit: None,
        ebitda: None,
        pretax_income: None,
        income_tax_expense: None,
        depreciation_and_amortization: None,
        net_income: None,
        net_income_common_stockholders: None,
        basic_eps: None,
        diluted_eps: None,
        basic_average_shares: None,
        diluted_average_shares: None,
    };
    assert_schema(&income.to_dataframe().unwrap(), INCOME_STATEMENT_SCHEMA);

    let balance = BalanceSheetRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        total_assets: None,
        total_liabilities: None,
        total_equity: None,
        cash: None,
        long_term_debt: None,
        shares_outstanding: None,
        current_assets: None,
        current_liabilities: None,
        accounts_receivable: None,
        inventory: None,
        accounts_payable: None,
        net_property_plant_equipment: None,
        goodwill: None,
        intangible_assets: None,
        total_debt: None,
        current_debt: None,
        cash_and_short_term_investments: None,
        other_current_assets: None,
        other_current_liabilities: None,
        retained_earnings: None,
        common_stock: None,
        treasury_stock: None,
        minority_interest: None,
        working_capital: None,
        tangible_book_value: None,
    };
    assert_schema(&balance.to_dataframe().unwrap(), BALANCE_SHEET_SCHEMA);

    let cashflow = CashflowRow {
        period: ReportingPeriod::annual(2024).unwrap(),
        operating_cashflow: None,
        capital_expenditures: None,
        free_cash_flow: None,
        net_income: None,
        depreciation_and_amortization: None,
        stock_based_compensation: None,
        change_in_working_capital: None,
        investing_cashflow: None,
        financing_cashflow: None,
        issuance_of_debt: None,
        repayment_of_debt: None,
        repurchase_of_capital_stock: None,
        cash_dividends_paid: None,
        end_cash_position: None,
    };
    assert_schema(&cashflow.to_dataframe().unwrap(), CASHFLOW_SCHEMA);
}
