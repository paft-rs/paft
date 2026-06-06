//! Unified error types for the `paft` facade crate.

/// Unified error type for the `paft` facade.
///
/// This enum aggregates error types from the various `paft` crates, enabling
/// ergonomic error handling when composing functionality across modules.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Error originating from `paft-core`.
    #[error(transparent)]
    Core(#[from] paft_core::error::PaftError),

    /// Error originating from constrained decimal construction.
    #[error(transparent)]
    DecimalConstraint(#[from] paft_decimal::DecimalConstraintError),

    /// Error originating from `paft-domain`.
    #[cfg(feature = "domain")]
    #[error(transparent)]
    Domain(#[from] paft_domain::DomainError),

    /// Error originating from `paft-market`.
    #[cfg(feature = "market")]
    #[error(transparent)]
    Market(#[from] paft_market::MarketError),

    /// Error originating from `paft-market` history response validation.
    #[cfg(feature = "market")]
    #[error(transparent)]
    HistoryValidation(#[from] paft_market::HistoryValidationError),

    /// Error originating from `paft-fundamentals`.
    #[cfg(feature = "fundamentals")]
    #[error(transparent)]
    Fundamentals(#[from] paft_fundamentals::FundamentalsError),

    /// Error originating from `paft-prediction`.
    #[cfg(feature = "prediction")]
    #[error(transparent)]
    Prediction(#[from] paft_prediction::PredictionError),

    /// Error originating from `paft-money` operations.
    #[error(transparent)]
    Money(#[from] paft_money::MoneyError),

    /// Error from configuring currency minor-unit metadata.
    #[error(transparent)]
    MinorUnit(#[from] paft_money::MinorUnitError),

    /// Error originating from parsing money/currency values.
    #[error(transparent)]
    MoneyParse(#[from] paft_money::error::MoneyParseError),

    /// Error from canonicalization utilities.
    #[error(transparent)]
    Canonical(#[from] paft_utils::string_canonical::CanonicalError),
}

/// Convenience alias for `Result<T, paft::Error>`.
pub type Result<T> = std::result::Result<T, Error>;
