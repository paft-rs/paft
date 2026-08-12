//! Financial statements and calendar types under `paft_fundamentals::statements`.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_domain::ReportingPeriod;
use paft_money::{Money, Price, QuantityAmount};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Income statement row.
///
/// # Sign convention
///
/// Expense lines on this struct - [`Self::cost_of_revenue`],
/// [`Self::research_and_development`],
/// [`Self::selling_general_and_administrative`],
/// [`Self::operating_expenses`], [`Self::interest_expense`], and
/// [`Self::income_tax_expense`] - are **positive magnitudes to be subtracted**,
/// as the income statement presents them. They are not signed cash flows: the
/// inflow-positive/outflow-negative convention documented on [`CashflowRow`]
/// does **not** apply here.
///
/// Result lines ([`Self::operating_income`], [`Self::pretax_income`],
/// [`Self::net_income`], and the rest) are signed, and are negative for a
/// loss-making period.
///
/// # Reported vs. derived fields
///
/// The following are *derived* or *aggregate* values, carried as reported by
/// the source and never computed, reconciled, or validated by `paft`:
///
/// - [`Self::operating_expenses`] - aggregate of the operating expense lines.
/// - [`Self::ebit`] and [`Self::ebitda`] - non-GAAP measures on which sources
///   disagree substantially (notably over which items are added back to
///   EBITDA).
/// - [`Self::net_income_common_stockholders`] - net income after preferred
///   dividends and minority interest, with source-specific deductions.
///
/// Recompute from the component line items when you need a value consistent
/// with the rest of the row; use the derived field when you want what the
/// source published.
pub struct IncomeStatementRow {
    /// Financial period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: ReportingPeriod,
    /// Total revenue.
    pub total_revenue: Option<Money>,
    /// Cost of revenue (cost of goods sold).
    pub cost_of_revenue: Option<Money>,
    /// Gross profit.
    pub gross_profit: Option<Money>,
    /// Research and development expense.
    pub research_and_development: Option<Money>,
    /// Selling, general, and administrative expense.
    pub selling_general_and_administrative: Option<Money>,
    /// Operating expenses, **excluding** cost of revenue.
    ///
    /// This is the aggregate of the operating expense lines below gross profit
    /// (research and development, selling/general/administrative, and any other
    /// operating expenses), so the intended identity is:
    ///
    /// `gross_profit - operating_expenses = operating_income`
    ///
    /// Providers that publish a "total expenses" figure inclusive of cost of
    /// revenue must subtract `cost_of_revenue` before mapping into this field.
    ///
    /// A positive magnitude, and an aggregate as reported; see the struct-level
    /// notes on signs and derived fields.
    pub operating_expenses: Option<Money>,
    /// Operating income.
    pub operating_income: Option<Money>,
    /// Interest income.
    pub interest_income: Option<Money>,
    /// Interest expense.
    pub interest_expense: Option<Money>,
    /// Earnings before interest and taxes.
    ///
    /// Non-GAAP measure as reported; see the struct-level note on derived
    /// fields.
    pub ebit: Option<Money>,
    /// Earnings before interest, taxes, depreciation, and amortization.
    ///
    /// Non-GAAP measure as reported; see the struct-level note on derived
    /// fields.
    pub ebitda: Option<Money>,
    /// Pretax income.
    pub pretax_income: Option<Money>,
    /// Income tax expense (provision for income taxes).
    pub income_tax_expense: Option<Money>,
    /// Depreciation and amortization recognized on the income statement.
    pub depreciation_and_amortization: Option<Money>,
    /// Net income.
    pub net_income: Option<Money>,
    /// Net income attributable to common stockholders.
    ///
    /// Derived value as reported; see the struct-level note on derived fields.
    pub net_income_common_stockholders: Option<Money>,
    /// Basic earnings per share.
    pub basic_eps: Option<Price>,
    /// Diluted earnings per share.
    pub diluted_eps: Option<Price>,
    /// Basic weighted-average shares outstanding over the period.
    ///
    /// Weighted averages are fractional by construction, so this is a
    /// full-precision [`QuantityAmount`] rather than an integer count. For the
    /// point-in-time integral share count see
    /// [`BalanceSheetRow::shares_outstanding`].
    pub basic_average_shares: Option<QuantityAmount>,
    /// Diluted weighted-average shares outstanding over the period.
    ///
    /// Fractional for the same reason as [`Self::basic_average_shares`].
    pub diluted_average_shares: Option<QuantityAmount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Balance sheet row.
///
/// # Reported vs. derived fields
///
/// Most fields are reported line items. The following are *derived* or
/// *aggregate* values: they are carried as reported by the source and are never
/// computed, reconciled, or validated by `paft`.
///
/// - [`Self::total_debt`] - aggregate of short- and long-term debt.
/// - [`Self::cash_and_short_term_investments`] - aggregate that overlaps
///   [`Self::cash`].
/// - [`Self::working_capital`] - `current_assets - current_liabilities`.
/// - [`Self::tangible_book_value`] - `total_equity - goodwill -
///   intangible_assets`.
///
/// Sources disagree on what enters each of these (which instruments count as
/// debt, which securities count as short-term investments, whether minority
/// interest is deducted from tangible book value), so a derived field is **not
/// guaranteed** to satisfy the identity above against its sibling fields.
/// Recompute from the component line items when you need a value consistent
/// with the rest of the row; use the derived field when you want what the
/// source published.
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
    /// Total debt (short-term and long-term combined).
    ///
    /// Aggregate as reported; see the struct-level note on derived fields.
    pub total_debt: Option<Money>,
    /// Current portion of debt due within one year.
    pub current_debt: Option<Money>,
    /// Cash, cash equivalents, and short-term investments.
    ///
    /// Aggregate as reported and overlapping [`Self::cash`]; see the
    /// struct-level note on derived fields.
    pub cash_and_short_term_investments: Option<Money>,
    /// Other current assets not separately itemized.
    pub other_current_assets: Option<Money>,
    /// Other current liabilities not separately itemized.
    pub other_current_liabilities: Option<Money>,
    /// Retained earnings.
    pub retained_earnings: Option<Money>,
    /// Common stock par/stated value.
    pub common_stock: Option<Money>,
    /// Treasury stock.
    pub treasury_stock: Option<Money>,
    /// Minority (non-controlling) interest.
    pub minority_interest: Option<Money>,
    /// Working capital, conceptually `current_assets - current_liabilities`.
    ///
    /// Derived value as reported; see the struct-level note on derived fields.
    pub working_capital: Option<Money>,
    /// Tangible book value, conceptually `total_equity - goodwill -
    /// intangible_assets`.
    ///
    /// Derived value as reported; see the struct-level note on derived fields.
    pub tangible_book_value: Option<Money>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Cashflow statement row.
///
/// # Sign convention
///
/// Every *flow* field on this struct is signed from the perspective of the
/// company's cash balance, matching how the statement of cash flows is
/// presented:
///
/// - **Cash inflows are positive.**
/// - **Cash outflows are negative.**
///
/// This applies to outflows that are conventionally quoted as positive
/// magnitudes elsewhere: [`Self::capital_expenditures`],
/// [`Self::repayment_of_debt`], [`Self::repurchase_of_capital_stock`], and
/// [`Self::cash_dividends_paid`] are all negative for a company that spent
/// cash. Providers that publish these as unsigned magnitudes must negate them
/// before mapping into this struct.
///
/// The convention covers flows only. [`Self::end_cash_position`] is a
/// point-in-time *balance*, not a flow, and is positive for a company holding
/// cash.
///
/// # Reported vs. derived fields
///
/// [`Self::free_cash_flow`] is a derived value carried as reported by the
/// source; `paft` never computes or reconciles it against
/// [`Self::operating_cashflow`] and [`Self::capital_expenditures`], and sources
/// disagree on what enters it.
pub struct CashflowRow {
    /// Financial period with structured variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: ReportingPeriod,
    /// Operating cashflow.
    pub operating_cashflow: Option<Money>,
    /// Capital expenditures.
    ///
    /// Negative when cash was spent on capital assets, per the struct-level
    /// sign convention.
    pub capital_expenditures: Option<Money>,
    /// Free cash flow.
    ///
    /// Derived value as reported; see the struct-level note on derived fields.
    pub free_cash_flow: Option<Money>,
    /// Net income.
    pub net_income: Option<Money>,
    /// Depreciation and amortization added back to net income.
    pub depreciation_and_amortization: Option<Money>,
    /// Stock-based compensation added back to net income.
    pub stock_based_compensation: Option<Money>,
    /// Change in working capital as an adjustment within operating cash flow.
    ///
    /// This is the cash-flow-statement adjustment line, **not** the
    /// balance-sheet delta of
    /// [`BalanceSheetRow::working_capital`] between two periods. The two carry
    /// opposite signs: an *increase* in working capital consumes cash and
    /// therefore appears here as a **negative** adjustment.
    pub change_in_working_capital: Option<Money>,
    /// Net cash flow from investing activities.
    ///
    /// Typically negative for a company investing more than it divests.
    pub investing_cashflow: Option<Money>,
    /// Net cash flow from financing activities.
    pub financing_cashflow: Option<Money>,
    /// Proceeds from issuance of debt.
    ///
    /// Positive when the company raised cash, per the struct-level sign
    /// convention.
    pub issuance_of_debt: Option<Money>,
    /// Repayment of debt.
    ///
    /// Negative when cash was spent repaying debt, per the struct-level sign
    /// convention.
    pub repayment_of_debt: Option<Money>,
    /// Repurchase of capital stock (buybacks).
    ///
    /// Negative when cash was spent on repurchases, per the struct-level sign
    /// convention. Positive values represent net issuance.
    pub repurchase_of_capital_stock: Option<Money>,
    /// Cash dividends paid.
    ///
    /// Negative when cash was distributed to shareholders, per the
    /// struct-level sign convention.
    pub cash_dividends_paid: Option<Money>,
    /// Cash and cash equivalents at end of period.
    ///
    /// A point-in-time balance, not a flow: the struct-level sign convention
    /// does not apply, and this is positive for a company holding cash.
    pub end_cash_position: Option<Money>,
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
