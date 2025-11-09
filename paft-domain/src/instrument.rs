//! Instrument identifier and asset classification domain types.

use super::Exchange;
use crate::identifier::{IdentifierScheme, PredictionId, SecurityId};
use crate::{
    DomainError,
    identifiers::{EventID, Figi, OutcomeID, Symbol},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, str::FromStr};

/// Kinds of financial instruments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum AssetKind {
    /// Common stock or equity-like instruments.
    #[default]
    Equity,
    /// Cryptocurrency assets.
    Crypto,
    /// Funds and ETFs.
    Fund,
    /// Market indexes.
    Index,
    /// Foreign exchange currency pairs.
    Forex,
    /// Bonds and fixed income.
    Bond,
    /// Commodities.
    Commodity,
    /// Option contracts.
    Option,
    /// Commodity futures.
    Future,
    /// Real Estate Investment Trusts.
    REIT,
    /// Warrants.
    Warrant,
    /// Convertible bonds/securities.
    Convertible,
    /// Non-fungible tokens.
    NFT,
    /// Perpetual futures contracts (no expiration date).
    PerpetualFuture,
    /// Leveraged tokens (e.g., 3x leveraged Bitcoin tokens).
    LeveragedToken,
    /// Liquidity provider tokens (`DeFi` protocol tokens).
    LPToken,
    /// Liquid staking tokens (e.g., stETH, rETH).
    LST,
    /// Real-world assets (tokenized physical assets).
    RWA,
    /// Prediction market.
    PredictionMarket,
}

crate::string_enum_closed_with_code!(
    AssetKind,
    "AssetKind",
    {
        "EQUITY" => AssetKind::Equity,
        "CRYPTO" => AssetKind::Crypto,
        "FUND" => AssetKind::Fund,
        "INDEX" => AssetKind::Index,
        "FOREX" => AssetKind::Forex,
        "BOND" => AssetKind::Bond,
        "COMMODITY" => AssetKind::Commodity,
        "OPTION" => AssetKind::Option,
        "FUTURE" => AssetKind::Future,
        "REIT" => AssetKind::REIT,
        "WARRANT" => AssetKind::Warrant,
        "CONVERTIBLE" => AssetKind::Convertible,
        "NFT" => AssetKind::NFT,
        "PERPETUAL_FUTURE" => AssetKind::PerpetualFuture,
        "LEVERAGED_TOKEN" => AssetKind::LeveragedToken,
        "LP_TOKEN" => AssetKind::LPToken,
        "LST" => AssetKind::LST,
        "RWA" => AssetKind::RWA,
        "PREDICTION_MARKET" => AssetKind::PredictionMarket,
    },
    {
        "STOCK" => AssetKind::Equity,
        "FX" => AssetKind::Forex,
    }
);

crate::impl_display_via_code!(AssetKind);

impl AssetKind {
    /// Human-readable label for displaying this asset kind.
    #[must_use]
    pub const fn full_name(&self) -> &'static str {
        match self {
            Self::Equity => "Equity",
            Self::Crypto => "Crypto",
            Self::Fund => "Fund",
            Self::Index => "Index",
            Self::Forex => "Forex",
            Self::Bond => "Bond",
            Self::Commodity => "Commodity",
            Self::Option => "Option",
            Self::Future => "Future",
            Self::REIT => "REIT",
            Self::Warrant => "Warrant",
            Self::Convertible => "Convertible",
            Self::NFT => "NFT",
            Self::PerpetualFuture => "Perpetual Future",
            Self::LeveragedToken => "Leveraged Token",
            Self::LPToken => "LP Token",
            Self::LST => "Liquid Staking Token",
            Self::RWA => "Real-World Asset",
            Self::PredictionMarket => "Prediction Market",
        }
    }
}

/// Logical instrument identity and asset classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    /// Identifier scheme (family of IDs)
    id: IdentifierScheme,
    /// Asset class and behavior
    kind: AssetKind,
}

impl Instrument {
    /// Construct an instrument from an identifier scheme and asset kind.
    #[must_use]
    pub const fn new(id: IdentifierScheme, kind: AssetKind) -> Self {
        Self { id, kind }
    }

    /// Construct a new `Instrument` with just a symbol and kind.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidSymbol` if the provided symbol violates canonical invariants.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(symbol), err)
    )]
    pub fn from_symbol(symbol: impl AsRef<str>, kind: AssetKind) -> Result<Self, DomainError> {
        Ok(Self {
            id: IdentifierScheme::Security(SecurityId {
                symbol: Symbol::new(symbol.as_ref())?,
                exchange: None,
                figi: None,
                isin: None,
            }),
            kind,
        })
    }

    /// Construct a new `Instrument` with symbol, exchange, and kind.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidSymbol` if the provided symbol violates canonical invariants.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(symbol), err)
    )]
    pub fn from_symbol_and_exchange(
        symbol: impl AsRef<str>,
        exchange: Exchange,
        kind: AssetKind,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id: IdentifierScheme::Security(SecurityId {
                symbol: Symbol::new(symbol.as_ref())?,
                exchange: Some(exchange),
                figi: None,
                isin: None,
            }),
            kind,
        })
    }

    /// Construct a new `Instrument` for a security identified by a FIGI and symbol.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidFigi` or `DomainError::InvalidSymbol` if validation fails.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn from_figi(figi: &str, symbol: Symbol, kind: AssetKind) -> Result<Self, DomainError> {
        Ok(Self {
            id: IdentifierScheme::Security(SecurityId {
                symbol,
                exchange: None,
                figi: Some(Figi::new(figi)?),
                isin: None,
            }),
            kind,
        })
    }

    /// Construct a new `Instrument` for a prediction market outcome.
    ///
    /// # Errors
    /// Returns `DomainError` variants if ID validation fails.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn from_prediction_market(event_id: &str, outcome_id: &str) -> Result<Self, DomainError> {
        Ok(Self {
            id: IdentifierScheme::Prediction(PredictionId {
                event_id: EventID::new(event_id)?,
                outcome_id: OutcomeID::new(outcome_id)?,
            }),
            kind: AssetKind::PredictionMarket,
        })
    }

    /// Returns the best available unique identifier for this instrument.
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        self.id.unique_key()
    }

    /// Returns the asset kind.
    #[must_use]
    pub const fn kind(&self) -> &AssetKind {
        &self.kind
    }

    /// Returns the identifier scheme.
    #[must_use]
    pub const fn id(&self) -> &IdentifierScheme {
        &self.id
    }
}

#[cfg(feature = "dataframe")]
mod dataframe_impl {
    use super::Instrument;
    use crate::identifier::IdentifierScheme;
    use paft_utils::dataframe::{Columnar, ToDataFrame};
    use polars::datatypes::DataType;
    use polars::prelude::{DataFrame, NamedFrom, PolarsResult, Series};

    impl ToDataFrame for Instrument {
        fn to_dataframe(&self) -> PolarsResult<DataFrame> {
            <Self as Columnar>::columnar_to_dataframe(std::slice::from_ref(self))
        }

        fn empty_dataframe() -> PolarsResult<DataFrame> {
            DataFrame::new(vec![
                Series::new_empty("kind".into(), &DataType::String).into(),
                Series::new_empty("symbol".into(), &DataType::String).into(),
                Series::new_empty("exchange".into(), &DataType::String).into(),
                Series::new_empty("figi".into(), &DataType::String).into(),
                Series::new_empty("isin".into(), &DataType::String).into(),
                Series::new_empty("event_id".into(), &DataType::String).into(),
                Series::new_empty("outcome_id".into(), &DataType::String).into(),
            ])
        }

        fn schema() -> PolarsResult<Vec<(&'static str, DataType)>> {
            Ok(vec![
                ("kind", DataType::String),
                ("symbol", DataType::String),
                ("exchange", DataType::String),
                ("figi", DataType::String),
                ("isin", DataType::String),
                ("event_id", DataType::String),
                ("outcome_id", DataType::String),
            ])
        }
    }

    impl Columnar for Instrument {
        fn columnar_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
            let kinds: Vec<String> = items.iter().map(|i| i.kind().to_string()).collect();
            let symbols: Vec<Option<String>> = items
                .iter()
                .map(|i| match i.id() {
                    IdentifierScheme::Security(s) => Some(s.symbol.to_string()),
                    IdentifierScheme::Prediction(_) => None,
                })
                .collect();
            let exchanges: Vec<Option<String>> = items
                .iter()
                .map(|i| match i.id() {
                    IdentifierScheme::Security(s) => {
                        s.exchange.as_ref().map(std::string::ToString::to_string)
                    }
                    IdentifierScheme::Prediction(_) => None,
                })
                .collect();
            let figis: Vec<Option<String>> = items
                .iter()
                .map(|i| match i.id() {
                    IdentifierScheme::Security(s) => {
                        s.figi.as_ref().map(|f| f.as_ref().to_string())
                    }
                    IdentifierScheme::Prediction(_) => None,
                })
                .collect();
            let isins: Vec<Option<String>> = items
                .iter()
                .map(|i| match i.id() {
                    IdentifierScheme::Security(s) => {
                        s.isin.as_ref().map(|v| v.as_ref().to_string())
                    }
                    IdentifierScheme::Prediction(_) => None,
                })
                .collect();
            let event_ids: Vec<Option<String>> = items
                .iter()
                .map(|i| match i.id() {
                    IdentifierScheme::Prediction(p) => Some(p.event_id.as_ref().to_string()),
                    IdentifierScheme::Security(_) => None,
                })
                .collect();
            let outcome_ids: Vec<Option<String>> = items
                .iter()
                .map(|i| match i.id() {
                    IdentifierScheme::Prediction(p) => Some(p.outcome_id.as_ref().to_string()),
                    IdentifierScheme::Security(_) => None,
                })
                .collect();

            let df = DataFrame::new(vec![
                Series::new("kind".into(), kinds).into(),
                Series::new("symbol".into(), symbols).into(),
                Series::new("exchange".into(), exchanges).into(),
                Series::new("figi".into(), figis).into(),
                Series::new("isin".into(), isins).into(),
                Series::new("event_id".into(), event_ids).into(),
                Series::new("outcome_id".into(), outcome_ids).into(),
            ])?;
            Ok(df)
        }
    }
}
