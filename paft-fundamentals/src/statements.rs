//! Financial statements and calendar types under `paft_fundamentals::fundamentals::statements`.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
use paft_domain::Period;
use paft_money::Money;
#[cfg(feature = "dataframe")]
use paft_utils::dataframe::ToDataFrame;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Income statement row.
pub struct IncomeStatementRow {
    /// Financial period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// Total revenue.
    pub total_revenue: Option<Money>,
    /// Gross profit.
    pub gross_profit: Option<Money>,
    /// Operating income.
    pub operating_income: Option<Money>,
    /// Net income.
    pub net_income: Option<Money>,
    /// Interest expense.
    pub interest_expense: Option<Money>,
    /// Tax expense.
    pub tax_expense: Option<Money>,
    /// Depreciation and amortization.
    pub depreciation_and_amortization: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Balance sheet row.
pub struct BalanceSheetRow {
    /// Financial period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// Total assets.
    pub total_assets: Option<Money>,
    /// Total liabilities.
    pub total_liabilities: Option<Money>,
    /// Total equity.
    pub total_equity: Option<Money>,
    /// Cash and cash equivalents.
    pub cash: Option<Money>,
    /// Long-term debt.
    pub long_term_debt: Option<Money>,
    /// Shares outstanding.
    pub shares_outstanding: Option<u64>,
    /// Accounts receivable.
    pub accounts_receivable: Option<Money>,
    /// Inventory.
    pub inventory: Option<Money>,
    /// Accounts payable.
    pub accounts_payable: Option<Money>,
    /// Current assets.
    pub current_assets: Option<Money>,
    /// Current liabilities.
    pub current_liabilities: Option<Money>,
    /// Net property, plant & equipment.
    pub net_ppe: Option<Money>,
    /// Intangible assets.
    pub intangible_assets: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Cashflow statement row.
pub struct CashflowRow {
    /// Financial period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
    /// Operating cashflow.
    pub operating_cashflow: Option<Money>,
    /// Capital expenditures.
    pub capital_expenditures: Option<Money>,
    /// Free cash flow.
    pub free_cash_flow: Option<Money>,
    /// Net income.
    pub net_income: Option<Money>,
    /// Depreciation and amortization.
    pub depreciation_and_amortization: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Corporate calendar entries (earnings/dividends).
pub struct Calendar {
    /// Upcoming or historical earnings dates.
    #[serde(with = "paft_core::serde_helpers::ts_seconds_vec")]
    pub earnings_dates: Vec<DateTime<Utc>>,
    /// Ex-dividend date.
    #[serde(with = "paft_core::serde_helpers::ts_seconds_option")]
    pub ex_dividend_date: Option<DateTime<Utc>>,
    /// Dividend payment date.
    #[serde(with = "paft_core::serde_helpers::ts_seconds_option")]
    pub dividend_payment_date: Option<DateTime<Utc>>,
}
