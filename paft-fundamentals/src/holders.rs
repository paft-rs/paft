//! Holder, insider activity, and ownership summary types.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString};

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
#[cfg(feature = "dataframe")]
use paft_core::dataframe::ToDataFrame;
use paft_core::domain::Money;

/// Transaction types for insider activities with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of transaction types while gracefully
/// handling unknown or provider-specific transaction types through the `Other` variant.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, AsRefStr, EnumString,
)]
#[strum(ascii_case_insensitive)]
#[serde(from = "String", into = "String")]
pub enum TransactionType {
    /// Purchase or acquisition of shares
    #[strum(to_string = "BUY", serialize = "PURCHASE", serialize = "ACQUISITION")]
    Buy,
    /// Sale or disposal of shares
    #[strum(to_string = "SELL", serialize = "SALE", serialize = "DISPOSAL")]
    Sell,
    /// Stock award or grant
    #[strum(to_string = "AWARD", serialize = "GRANT", serialize = "STOCK_AWARD")]
    Award,
    /// Exercise of options
    #[strum(to_string = "EXERCISE", serialize = "OPTION_EXERCISE")]
    Exercise,
    /// Gift of shares
    #[strum(to_string = "GIFT")]
    Gift,
    /// Conversion of securities
    #[strum(to_string = "CONVERSION")]
    Conversion,
    /// Unknown or provider-specific transaction type
    Other(String),
}

impl From<String> for TransactionType {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        Self::from_str(&s).unwrap_or_else(|_| Self::Other(s.to_uppercase()))
    }
}

impl From<TransactionType> for String {
    fn from(transaction_type: TransactionType) -> Self {
        match transaction_type {
            TransactionType::Other(s) => s,
            _ => transaction_type.to_string(),
        }
    }
}

/// Insider positions in a company with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of insider positions while gracefully
/// handling unknown or provider-specific positions through the `Other` variant.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, AsRefStr, EnumString,
)]
#[strum(ascii_case_insensitive)]
#[serde(from = "String", into = "String")]
pub enum InsiderPosition {
    /// Officer of the company
    #[strum(to_string = "OFFICER")]
    Officer,
    /// Director or board member
    #[strum(to_string = "DIRECTOR", serialize = "BOARD_MEMBER")]
    Director,
    /// Beneficial owner (typically >10% ownership)
    #[strum(
        to_string = "OWNER",
        serialize = "BENEFICIAL_OWNER",
        serialize = "10%_OWNER"
    )]
    Owner,
    /// Chief Executive Officer
    #[strum(to_string = "CEO", serialize = "CHIEF_EXECUTIVE_OFFICER")]
    Ceo,
    /// Chief Financial Officer
    #[strum(to_string = "CFO", serialize = "CHIEF_FINANCIAL_OFFICER")]
    Cfo,
    /// Chief Operating Officer
    #[strum(to_string = "COO", serialize = "CHIEF_OPERATING_OFFICER")]
    Coo,
    /// Chief Technology Officer
    #[strum(to_string = "CTO", serialize = "CHIEF_TECHNOLOGY_OFFICER")]
    Cto,
    /// President
    #[strum(to_string = "PRESIDENT")]
    President,
    /// Vice President
    #[strum(to_string = "VP", serialize = "VICE_PRESIDENT")]
    VicePresident,
    /// Secretary
    #[strum(to_string = "SECRETARY")]
    Secretary,
    /// Treasurer
    #[strum(to_string = "TREASURER")]
    Treasurer,
    /// Unknown or provider-specific position
    Other(String),
}

impl From<String> for InsiderPosition {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        Self::from_str(&s).unwrap_or_else(|_| Self::Other(s.to_uppercase()))
    }
}

impl From<InsiderPosition> for String {
    fn from(position: InsiderPosition) -> Self {
        match position {
            InsiderPosition::Other(s) => s,
            _ => position.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Summary percentages for major holder categories.
pub struct MajorHolder {
    /// The category of the holder (e.g., "% of Shares Held by All Insider").
    pub category: String,
    /// The value associated with the category as a numeric fraction (e.g., 0.255 for 25.5%).
    pub value: f64,
}

/// Represents a single institutional or mutual fund holder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct InstitutionalHolder {
    /// The name of the holding institution or fund.
    pub holder: String,
    /// The number of shares held.
    pub shares: Option<u64>,
    /// The date of the last reported position as a Unix timestamp.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date_reported: DateTime<Utc>,
    /// The percentage of the company's outstanding shares held by this entity.
    pub pct_held: Option<f64>,
    /// The market value of the shares held.
    pub value: Option<Money>,
}

/// Represents a single insider transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct InsiderTransaction {
    /// The name of the insider who executed the transaction.
    pub insider: String,
    /// The insider's relationship to the company with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub position: InsiderPosition,
    /// The type of transaction with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub transaction_type: TransactionType,
    /// The number of shares involved in the transaction.
    pub shares: Option<u64>,
    /// The total value of the transaction.
    pub value: Option<Money>,
    /// The transaction date as a Unix timestamp.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub transaction_date: DateTime<Utc>,
    /// A URL to the source filing for the transaction, if available.
    pub url: String,
}

/// Represents a single insider on the company's roster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct InsiderRosterHolder {
    /// The name of the insider.
    pub name: String,
    /// The insider's position in the company with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub position: InsiderPosition,
    /// A description of the most recent transaction made by this insider.
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub most_recent_transaction: TransactionType,
    /// The date of the latest transaction as a Unix timestamp.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub latest_transaction_date: DateTime<Utc>,
    /// The number of shares owned directly by the insider.
    pub shares_owned_directly: Option<u64>,
    /// The date of the direct ownership filing as a Unix timestamp.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub position_direct_date: DateTime<Utc>,
}

/// A summary of net share purchase activity by insiders over a specific period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct NetSharePurchaseActivity {
    /// The period the summary covers (e.g., "3m").
    pub period: String,
    /// The total number of shares purchased by insiders.
    pub buy_shares: Option<u64>,
    /// The number of separate buy transactions.
    pub buy_count: Option<u64>,
    /// The total number of shares sold by insiders.
    pub sell_shares: Option<u64>,
    /// The number of separate sell transactions.
    pub sell_count: Option<u64>,
    /// The net number of shares purchased or sold.
    pub net_shares: Option<i64>,
    /// The net number of transactions.
    pub net_count: Option<i64>,
    /// The total number of shares held by all insiders.
    pub total_insider_shares: Option<u64>,
    /// The net shares purchased/sold as a percentage of total insider shares.
    pub net_percent_insider_shares: Option<f64>,
}
