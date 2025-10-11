//! Fundamentals types for financial statements, analysis, holders, and ESG.
//!
//! Provider-agnostic, strongly-typed models for company/fund profiles,
//! ownership, analyst coverage, and financial statements used across the paft
//! ecosystem. Types prefer canonical string forms for serde/display and
//! validated builders where appropriate.
//!
//! # Quickstart
//! ```rust
//! use paft_fundamentals::{Earnings, EarningsYear, Profile, CompanyProfile};
//!
//! let earnings = Earnings {
//!     yearly: vec![EarningsYear { year: 2023, revenue: None, earnings: None }],
//!     quarterly: vec![],
//!     quarterly_eps: vec![],
//! };
//! assert_eq!(earnings.yearly[0].year, 2023);
//!
//! let profile = Profile::Company(CompanyProfile {
//!     name: "Example Corp".to_string(),
//!     sector: Some("Technology".to_string()),
//!     industry: None,
//!     website: None,
//!     address: None,
//!     summary: None,
//!     isin: None,
//! });
//! match profile {
//!     Profile::Company(c) => assert_eq!(c.name, "Example Corp"),
//!     _ => unreachable!(),
//! }
//! ```
//!
//! # Feature flags
//! - `rust-decimal` (default): `paft-money` uses `rust-decimal`
//! - `bigdecimal`: `paft-money` uses `bigdecimal`
//! - `dataframe`: enable `polars`/`df-derive` integration for dataframe export
//!
//! # Serde
//! All models serialize with stable, human-readable representations; dataframe
//! support emits string codes for enums.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod analysis;
pub mod esg;
pub mod holders;
pub mod profile;
pub mod statements;

pub use analysis::{
    AnalysisSummary, Earnings, EarningsQuarter, EarningsQuarterEps, EarningsTrendRow, EarningsYear,
    PriceTarget, RecommendationAction, RecommendationGrade, RecommendationRow,
    RecommendationSummary, UpgradeDowngradeRow,
};
pub use esg::{EsgInvolvement, EsgScores, EsgSummary};
pub use holders::{
    InsiderPosition, InsiderRosterHolder, InsiderTransaction, InstitutionalHolder, MajorHolder,
    NetSharePurchaseActivity, TransactionType,
};
pub use profile::{Address, CompanyProfile, FundKind, FundProfile, Profile, ShareCount};
pub use statements::{BalanceSheetRow, Calendar, CashflowRow, IncomeStatementRow};
