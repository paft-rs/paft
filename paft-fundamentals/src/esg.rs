//! ESG scores and involvement types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// ESG involvement details for controversial activities or sectors.
pub struct EsgInvolvement {
    /// Involvement category.
    pub category: String,
    /// Provider-specific involvement score or flag.
    pub score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// ESG scores summary.
pub struct EsgScores {
    /// Environmental score.
    pub environmental: Option<f64>,
    /// Social score.
    pub social: Option<f64>,
    /// Governance score.
    pub governance: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// ESG summary including scores and involvement details.
pub struct EsgSummary {
    /// Optional aggregate scores.
    pub scores: Option<EsgScores>,
    /// List of involvement categories.
    pub involvement: Vec<EsgInvolvement>,
}
