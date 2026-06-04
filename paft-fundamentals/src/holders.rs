//! Holder, insider activity, and ownership summary types.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use chrono::NaiveDate;
#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;
use paft_decimal::{Decimal, Ratio};
use paft_domain::ReportingPeriod;
use paft_money::Money;

use crate::FundamentalsError;

paft_core::other_string_code_type!(
    /// Provider-specific transaction type not modeled by [`TransactionType`].
    pub struct OtherTransactionType for TransactionType;
    type Error = FundamentalsError;
    parse(input) => TransactionType::from_str(input);
    invalid(input) => FundamentalsError::InvalidEnumValue {
        enum_name: "TransactionType",
        value: input.to_string(),
    };
);

/// Transaction types for insider activities with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of transaction types while gracefully
/// handling unknown or provider-specific transaction types through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix)
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
    Other(OtherTransactionType),
}

impl TransactionType {
    /// Attempts to parse a transaction type, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `FundamentalsError::InvalidEnumValue` when `input` is empty/whitespace.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn try_from_str(input: &str) -> Result<Self, FundamentalsError> {
        Self::from_str(input)
    }

    /// Builds an unknown transaction type, rejecting modeled types and aliases.
    ///
    /// # Errors
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`TransactionType`] variant.
    pub fn other(input: &str) -> Result<Self, FundamentalsError> {
        OtherTransactionType::new(input).map(Self::Other)
    }
}

// Centralized code() and string impls via macro
paft_core::string_enum_with_code!(
    TransactionType, Other(OtherTransactionType), "TransactionType",
    type Error = FundamentalsError;
    invalid(input) => FundamentalsError::InvalidEnumValue {
        enum_name: "TransactionType",
        value: input.to_string(),
    };
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

paft_core::other_string_code_type!(
    /// Provider-specific insider position not modeled by [`InsiderPosition`].
    pub struct OtherInsiderPosition for InsiderPosition;
    type Error = FundamentalsError;
    parse(input) => InsiderPosition::from_str(input);
    invalid(input) => FundamentalsError::InvalidEnumValue {
        enum_name: "InsiderPosition",
        value: input.to_string(),
    };
);

/// Insider positions in a company with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of insider positions while gracefully
/// handling unknown or provider-specific positions through the `Other` variant.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix)
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
    Other(OtherInsiderPosition),
}

impl InsiderPosition {
    /// Attempts to parse an insider position, uppercasing unknown inputs into `Other`.
    ///
    /// # Errors
    /// Returns `FundamentalsError::InvalidEnumValue` when `input` is empty/whitespace.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn try_from_str(input: &str) -> Result<Self, FundamentalsError> {
        Self::from_str(input)
    }

    /// Builds an unknown insider position, rejecting modeled positions and aliases.
    ///
    /// # Errors
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`InsiderPosition`] variant.
    pub fn other(input: &str) -> Result<Self, FundamentalsError> {
        OtherInsiderPosition::new(input).map(Self::Other)
    }
}

// Centralized code() and string impls via macro
paft_core::string_enum_with_code!(
    InsiderPosition, Other(OtherInsiderPosition), "InsiderPosition",
    type Error = FundamentalsError;
    invalid(input) => FundamentalsError::InvalidEnumValue {
        enum_name: "InsiderPosition",
        value: input.to_string(),
    };
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Summary percentages for major holder categories.
pub struct MajorHolder {
    /// The category of the holder (e.g., "% of Shares Held by All Insider").
    pub category: String,
    /// The value associated with the category as a numeric fraction (e.g., 0.255 for 25.5%).
    #[cfg_attr(feature = "dataframe", df_derive(decimal(precision = 38, scale = 10)))]
    pub value: Ratio,
}

/// Represents a single institutional or mutual fund holder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct InstitutionalHolder {
    /// The name of the holding institution or fund.
    pub holder: String,
    /// The number of shares held.
    pub shares: Option<u64>,
    /// The calendar date of the last reported position.
    pub date_reported: NaiveDate,
    /// The percentage of the company's outstanding shares held by this entity.
    #[cfg_attr(feature = "dataframe", df_derive(decimal(precision = 38, scale = 10)))]
    pub pct_held: Option<Ratio>,
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
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub position: InsiderPosition,
    /// The type of transaction with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub transaction_type: TransactionType,
    /// The number of shares involved in the transaction.
    pub shares: Option<u64>,
    /// The total value of the transaction.
    pub value: Option<Money>,
    /// The transaction calendar date.
    pub transaction_date: NaiveDate,
    /// A URL to the source filing for the transaction, if available.
    pub url: Option<String>,
}

/// Represents a single insider on the company's roster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct InsiderRosterHolder {
    /// The name of the insider.
    pub name: String,
    /// The insider's position in the company with canonical variants and extensible fallback.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub position: InsiderPosition,
    /// A description of the most recent transaction made by this insider.
    #[cfg_attr(feature = "dataframe", df_derive(as_str))]
    pub most_recent_transaction: TransactionType,
    /// The calendar date of the latest transaction.
    pub latest_transaction_date: NaiveDate,
    /// The number of shares owned directly by the insider.
    pub shares_owned_directly: Option<u64>,
    /// The calendar date of the direct ownership filing.
    pub position_direct_date: NaiveDate,
}

/// A summary of net share purchase activity by insiders over a specific period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct NetSharePurchaseActivity {
    /// The period the summary covers (e.g., `ReportingPeriod::quarterly(2023, 4)?`).
    #[cfg_attr(feature = "dataframe", df_derive(as_string))]
    pub period: ReportingPeriod,
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
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub net_percent_insider_shares: Option<Decimal>,
}
