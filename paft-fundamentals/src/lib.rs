//! Fundamentals types for financial statements, analysis, holders, and ESG.
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
