//! ESG scores and involvement types.

use chrono::NaiveDate;
use paft_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::FundamentalsError;

#[cfg(feature = "dataframe")]
use df_derive_macros::ToDataFrame;

/// Shared methodology identity and optional provenance for one ESG report.
///
/// `scheme_id` identifies the scoring/reporting methodology governing the
/// summary's fields and involvement categories. A provider name alone is
/// insufficient when it exposes multiple methodologies. Adapters assign stable,
/// consistently used `namespace:methodology` identifiers; case is preserved.
/// The first colon separates two nonempty portions, with no embedded whitespace.
/// The methodology portion is otherwise opaque; PAFT maintains no registry.
///
/// One scheme may define different scales or meanings for different components.
/// Matching identifiers alone do not prove reports comparable: versions, dates,
/// comparison groups, and metric definitions may matter. Missing provenance means
/// not supplied, not irrelevant or implicitly identical. Validation establishes
/// syntax, not correct attribution by the adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
pub struct EsgContext {
    scheme_id: String,
    methodology_reference: Option<String>,
    methodology_version: Option<String>,
    assessment_date: Option<NaiveDate>,
    comparison_group: Option<String>,
}

impl EsgContext {
    /// Construct a context from an actual adapter-assigned methodology identity.
    /// Optional provenance starts absent; no fallback scheme is invented.
    ///
    /// # Errors
    /// Rejects blank identities, missing namespace/methodology portions, and
    /// embedded whitespace. Surrounding whitespace is trimmed.
    pub fn new(scheme_id: impl Into<String>) -> Result<Self, FundamentalsError> {
        Ok(Self {
            scheme_id: scheme(scheme_id.into())?,
            methodology_reference: None,
            methodology_version: None,
            assessment_date: None,
            comparison_group: None,
        })
    }

    /// Adapter-assigned scoring/reporting methodology identifier.
    #[must_use]
    pub const fn scheme_id(&self) -> &str {
        self.scheme_id.as_str()
    }

    /// Optional URI, provider document identifier, or opaque textual reference.
    /// PAFT neither resolves this reference nor machine-interprets its contents.
    #[must_use]
    pub fn methodology_reference(&self) -> Option<&str> {
        self.methodology_reference.as_deref()
    }

    /// Methodology version when supplied; absence does not imply equality.
    #[must_use]
    pub fn methodology_version(&self) -> Option<&str> {
        self.methodology_version.as_deref()
    }

    /// Assessment date when supplied, distinct from download/publication time.
    #[must_use]
    pub const fn assessment_date(&self) -> Option<NaiveDate> {
        self.assessment_date
    }

    /// Industry, peer group, or other comparison population when supplied.
    #[must_use]
    pub fn comparison_group(&self) -> Option<&str> {
        self.comparison_group.as_deref()
    }

    /// Replace the methodology identity, preserving the old context on failure.
    /// Callers must maintain correct attribution of any attached measurements.
    /// # Errors
    /// Rejects the same invalid identity syntax as [`Self::new`].
    pub fn set_scheme_id(&mut self, value: impl Into<String>) -> Result<(), FundamentalsError> {
        self.scheme_id = scheme(value.into())?;
        Ok(())
    }

    /// Replace or remove the methodology reference; failure leaves it unchanged.
    /// # Errors
    /// Rejects supplied blank strings; trims surrounding whitespace otherwise.
    pub fn set_methodology_reference(
        &mut self,
        value: Option<String>,
    ) -> Result<(), FundamentalsError> {
        self.methodology_reference = optional_text("methodology_reference", value)?;
        Ok(())
    }

    /// Replace or remove the methodology version; failure leaves it unchanged.
    /// # Errors
    /// Rejects supplied blank strings; trims surrounding whitespace otherwise.
    pub fn set_methodology_version(
        &mut self,
        value: Option<String>,
    ) -> Result<(), FundamentalsError> {
        self.methodology_version = optional_text("methodology_version", value)?;
        Ok(())
    }

    /// Set or remove the already validated calendar assessment date.
    pub const fn set_assessment_date(&mut self, value: Option<NaiveDate>) {
        self.assessment_date = value;
    }

    /// Replace or remove the comparison group; failure leaves it unchanged.
    /// # Errors
    /// Rejects supplied blank strings; trims surrounding whitespace otherwise.
    pub fn set_comparison_group(&mut self, value: Option<String>) -> Result<(), FundamentalsError> {
        self.comparison_group = optional_text("comparison_group", value)?;
        Ok(())
    }
}

fn text(field: &'static str, value: String) -> Result<String, FundamentalsError> {
    if value.trim().is_empty() {
        return Err(FundamentalsError::InvalidEsgContext {
            field,
            value,
            reason: "supplied strings must not be blank",
        });
    }
    Ok(value.trim().to_owned())
}

fn optional_text(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, FundamentalsError> {
    value.map(|value| text(field, value)).transpose()
}

fn scheme(value: String) -> Result<String, FundamentalsError> {
    let normalized = text("scheme_id", value.clone())?;
    if normalized.chars().any(char::is_whitespace)
        || !normalized
            .split_once(':')
            .is_some_and(|(namespace, methodology)| {
                !namespace.is_empty() && !methodology.is_empty()
            })
    {
        return Err(FundamentalsError::InvalidEsgContext {
            field: "scheme_id",
            value,
            reason: "expected nonempty namespace:methodology portions without embedded whitespace",
        });
    }
    Ok(normalized)
}

impl<'de> Deserialize<'de> for EsgContext {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            scheme_id: String,
            methodology_reference: Option<String>,
            methodology_version: Option<String>,
            assessment_date: Option<NaiveDate>,
            comparison_group: Option<String>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let mut context = Self::new(wire.scheme_id).map_err(serde::de::Error::custom)?;
        context
            .set_methodology_reference(wire.methodology_reference)
            .map_err(serde::de::Error::custom)?;
        context
            .set_methodology_version(wire.methodology_version)
            .map_err(serde::de::Error::custom)?;
        context.set_assessment_date(wire.assessment_date);
        context
            .set_comparison_group(wire.comparison_group)
            .map_err(serde::de::Error::custom)?;
        Ok(context)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// ESG involvement component for controversial activities or sectors.
/// Interpretation as part of a complete report requires an [`EsgContext`].
pub struct EsgInvolvement {
    /// Involvement category.
    pub category: String,
    /// Provider-specific involvement score or flag.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub score: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// Provider-reported ESG score components, interpreted through an [`EsgContext`].
///
/// Independent serialization/export does not make these a complete report or
/// establish cross-provider comparability. The scheme defines each metric.
pub struct EsgScores {
    /// Environmental score.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub environmental: Option<Decimal>,
    /// Social score.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub social: Option<Decimal>,
    /// Governance score.
    #[serde(default, with = "paft_decimal::serde::option_canonical_str")]
    pub governance: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(ToDataFrame))]
/// One ESG report with shared methodology context for all its components.
///
/// Different scheme identities require separate summaries unless an explicitly
/// identified composite methodology governs the combined report. Use
/// `Option<EsgSummary>` for absence; legacy summaries need an actual scheme,
/// never an invented fallback. Components retain standalone serde/export support.
pub struct EsgSummary {
    /// Methodology governing every score and involvement category in this report.
    pub context: EsgContext,
    /// Optional aggregate scores.
    pub scores: Option<EsgScores>,
    /// List of involvement categories.
    pub involvement: Vec<EsgInvolvement>,
}

impl EsgSummary {
    /// Construct a report with known context and no supplied measurements yet.
    #[must_use]
    pub const fn new(context: EsgContext) -> Self {
        Self {
            context,
            scores: None,
            involvement: Vec::new(),
        }
    }
}
