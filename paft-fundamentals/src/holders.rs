//! Holder, insider activity, and ownership summary types.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use chrono::{DateTime, Utc};
#[cfg(feature = "dataframe")]
use df_derive::ToDataFrame;
#[cfg(feature = "dataframe")]
use paft_core::dataframe::ToDataFrame;
use paft_core::domain::{Money, Period};
use paft_core::error::PaftError;

use paft_core::domain::string_canonical::Canonical;

/// Transaction types for insider activities with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of transaction types while gracefully
/// handling unknown or provider-specific transaction types through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix) and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TransactionType {
    /// Purchase or acquisition of shares
    Buy,
    /// Sale or disposal of shares
    Sell,
    /// Stock award or grant
    Award,
    /// Exercise of options
    Exercise,
    /// Gift of shares
    Gift,
    /// Conversion of securities
    Conversion,
    /// Unknown or provider-specific transaction type
    Other(Canonical),
}

impl TransactionType {
    /// Attempts to parse a transaction type, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `PaftError::InvalidEnumValue` when `input` is empty/whitespace.
    pub fn try_from_str(input: &str) -> Result<Self, PaftError> {
        Self::from_str(input)
    }
}

// Centralized code() and string impls via macro
paft_core::string_enum_with_code!(
    TransactionType, Other, "TransactionType",
    {
        "BUY" => TransactionType::Buy,
        "SELL" => TransactionType::Sell,
        "AWARD" => TransactionType::Award,
        "EXERCISE" => TransactionType::Exercise,
        "GIFT" => TransactionType::Gift,
        "CONVERSION" => TransactionType::Conversion
    },
    {
        // Aliases
        "PURCHASE" => TransactionType::Buy,
        "ACQUISITION" => TransactionType::Buy,
        "SALE" => TransactionType::Sell,
        "DISPOSAL" => TransactionType::Sell,
        "GRANT" => TransactionType::Award,
        "STOCK_AWARD" => TransactionType::Award,
        "OPTION_EXERCISE" => TransactionType::Exercise
    }
);

// Display equals code for these enums
paft_core::impl_display_via_code!(TransactionType);

/// Insider positions in a company with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of insider positions while gracefully
/// handling unknown or provider-specific positions through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix) and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InsiderPosition {
    /// Officer of the company
    Officer,
    /// Director or board member
    Director,
    /// Beneficial owner (typically >10% ownership)
    Owner,
    /// Chief Executive Officer
    Ceo,
    /// Chief Financial Officer
    Cfo,
    /// Chief Operating Officer
    Coo,
    /// Chief Technology Officer
    Cto,
    /// President
    President,
    /// Vice President
    VicePresident,
    /// Secretary
    Secretary,
    /// Treasurer
    Treasurer,
    /// Unknown or provider-specific position
    Other(Canonical),
}

impl InsiderPosition {
    /// Attempts to parse an insider position, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `PaftError::InvalidEnumValue` when `input` is empty/whitespace.
    pub fn try_from_str(input: &str) -> Result<Self, PaftError> {
        Self::from_str(input)
    }
}

// Centralized code() and string impls via macro
paft_core::string_enum_with_code!(
    InsiderPosition, Other, "InsiderPosition",
    {
        "OFFICER" => InsiderPosition::Officer,
        "DIRECTOR" => InsiderPosition::Director,
        "OWNER" => InsiderPosition::Owner,
        "CEO" => InsiderPosition::Ceo,
        "CFO" => InsiderPosition::Cfo,
        "COO" => InsiderPosition::Coo,
        "CTO" => InsiderPosition::Cto,
        "PRESIDENT" => InsiderPosition::President,
        "VP" => InsiderPosition::VicePresident,
        "SECRETARY" => InsiderPosition::Secretary,
        "TREASURER" => InsiderPosition::Treasurer
    },
    {
        // Aliases
        "BOARD_MEMBER" => InsiderPosition::Director,
        "BENEFICIAL_OWNER" => InsiderPosition::Owner,
        "10_OWNER" => InsiderPosition::Owner,
        "CHIEF_EXECUTIVE_OFFICER" => InsiderPosition::Ceo,
        "CHIEF_FINANCIAL_OFFICER" => InsiderPosition::Cfo,
        "CHIEF_OPERATING_OFFICER" => InsiderPosition::Coo,
        "CHIEF_TECHNOLOGY_OFFICER" => InsiderPosition::Cto,
        "VICE_PRESIDENT" => InsiderPosition::VicePresident
    }
);

paft_core::impl_display_via_code!(InsiderPosition);

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
    /// The period the summary covers (e.g., `Period::Quarter { year: 2023, quarter: 4 }`).
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: Period,
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
