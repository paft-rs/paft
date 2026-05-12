//! ESG scores and involvement types.

use paft_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// ESG involvement details for controversial activities or sectors.
pub struct EsgInvolvement {
    /// Involvement category.
    pub category: String,
    /// Provider-specific involvement score or flag.
    pub score: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// ESG scores summary.
pub struct EsgScores {
    /// Environmental score.
    pub environmental: Option<Decimal>,
    /// Social score.
    pub social: Option<Decimal>,
    /// Governance score.
    pub governance: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// ESG summary including scores and involvement details.
pub struct EsgSummary {
    /// Optional aggregate scores.
    pub scores: Option<EsgScores>,
    /// List of involvement categories.
    pub involvement: Vec<EsgInvolvement>,
}
