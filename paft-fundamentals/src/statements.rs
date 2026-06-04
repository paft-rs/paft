//! Financial statements and calendar types under `paft_fundamentals::statements`.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_domain::ReportingPeriod;
use paft_money::Money;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Income statement row.
pub struct IncomeStatementRow {
    /// Financial period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: ReportingPeriod,
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
    /// Income tax expense (provision for income taxes).
    pub income_tax_expense: Option<Money>,
    /// Depreciation and amortization recognized on the income statement.
    pub depreciation_and_amortization: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Balance sheet row.
pub struct BalanceSheetRow {
    /// Financial period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: ReportingPeriod,
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
    /// Total current assets.
    pub current_assets: Option<Money>,
    /// Total current liabilities.
    pub current_liabilities: Option<Money>,
    /// Accounts receivable, net.
    pub accounts_receivable: Option<Money>,
    /// Inventory.
    pub inventory: Option<Money>,
    /// Accounts payable.
    pub accounts_payable: Option<Money>,
    /// Property, plant, and equipment, net of accumulated depreciation.
    pub net_property_plant_equipment: Option<Money>,
    /// Goodwill.
    pub goodwill: Option<Money>,
    /// Intangible assets excluding goodwill.
    pub intangible_assets: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Cashflow statement row.
pub struct CashflowRow {
    /// Financial period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: ReportingPeriod,
    /// Operating cashflow.
    pub operating_cashflow: Option<Money>,
    /// Capital expenditures.
    pub capital_expenditures: Option<Money>,
    /// Free cash flow.
    pub free_cash_flow: Option<Money>,
    /// Net income.
    pub net_income: Option<Money>,
    /// Depreciation and amortization added back to net income.
    pub depreciation_and_amortization: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Corporate calendar entries (earnings/dividends).
pub struct Calendar {
    /// Upcoming or historical earnings dates.
    #[serde(with = "paft_core::serde_helpers::ts_milliseconds_vec")]
    pub earnings_dates: Vec<DateTime<Utc>>,
    /// Ex-dividend calendar date.
    #[serde(default)]
    pub ex_dividend_date: Option<NaiveDate>,
    /// Dividend payment calendar date.
    #[serde(default)]
    pub dividend_payment_date: Option<NaiveDate>,
}
