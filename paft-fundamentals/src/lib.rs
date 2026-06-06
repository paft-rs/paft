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
//!     yearly: vec![EarningsYear::new(2023).unwrap()],
//!     quarterly: vec![],
//!     quarterly_eps: vec![],
//! };
//! assert_eq!(earnings.yearly[0].year.get(), 2023);
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
//! The default money backend is `rust_decimal`.
//!
//! - `bigdecimal`: switch `paft-money` to the `bigdecimal` backend
//! - `dataframe`: enable `polars`/`df-derive-macros` integration for dataframe export
//!
//! # Serde
//! All models serialize with stable, human-readable representations; dataframe
//! support emits string codes for enums.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod analysis;
pub mod error;
pub mod esg;
pub mod holders;
pub mod profile;
pub mod statements;
pub mod statistics;

pub use analysis::{
    AnalysisSummary, Earnings, EarningsEstimate, EarningsQuarter, EarningsQuarterEps,
    EarningsTrendRow, EarningsYear, EpsRevisions, EpsTrend, OtherRecommendationAction,
    OtherRecommendationGrade, PriceTarget, RecommendationAction, RecommendationGrade,
    RecommendationRow, RecommendationSummary, RevenueEstimate, RevisionPoint, TrendPoint,
    UpgradeDowngradeRow,
};
pub use error::FundamentalsError;
pub use esg::{EsgInvolvement, EsgScores, EsgSummary};
pub use holders::{
    InsiderPosition, InsiderRosterHolder, InsiderTransaction, InstitutionalHolder, MajorHolder,
    NetSharePurchaseActivity, OtherInsiderPosition, OtherTransactionType, TransactionType,
};
pub use profile::{
    Address, CompanyProfile, FundKind, FundProfile, OtherFundKind, Profile, ShareCount,
};
pub use statements::{BalanceSheetRow, Calendar, CashflowRow, IncomeStatementRow};
pub use statistics::KeyStatistics;
