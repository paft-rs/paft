//! Instrument identifier and asset classification domain types.

use super::Exchange;
use crate::{
    DomainError,
    identifiers::{Figi, Isin, Symbol},
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
        }
    }
}

/// Logical instrument identity for a security.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    /// Canonical ticker symbol.
    pub symbol: Symbol,
    /// Optional trading venue context for disambiguation.
    pub exchange: Option<Exchange>,
    /// Optional global identifier (preferred).
    pub figi: Option<Figi>,
    /// Optional global identifier (fallback).
    pub isin: Option<Isin>,
    /// Asset class and behavior.
    pub kind: AssetKind,
}

impl Instrument {
    /// Construct an instrument from a validated symbol and asset kind.
    #[must_use]
    pub const fn new(symbol: Symbol, kind: AssetKind) -> Self {
        Self {
            symbol,
            exchange: None,
            figi: None,
            isin: None,
            kind,
        }
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
            symbol: Symbol::new(symbol.as_ref())?,
            exchange: None,
            figi: None,
            isin: None,
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
            symbol: Symbol::new(symbol.as_ref())?,
            exchange: Some(exchange),
            figi: None,
            isin: None,
            kind,
        })
    }

    /// Construct a new `Instrument` for a security identified by a FIGI and symbol.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidFigi` if FIGI validation fails.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
    pub fn from_figi(figi: &str, symbol: Symbol, kind: AssetKind) -> Result<Self, DomainError> {
        Ok(Self {
            symbol,
            exchange: None,
            figi: Some(Figi::new(figi)?),
            isin: None,
            kind,
        })
    }

    /// Returns the best available unique identifier for this instrument
    /// (FIGI > ISIN > SYMBOL@EXCHANGE > SYMBOL).
    #[must_use]
    pub fn unique_key(&self) -> Cow<'_, str> {
        if let Some(figi) = &self.figi {
            return Cow::Borrowed(figi.as_ref());
        }
        if let Some(isin) = &self.isin {
            return Cow::Borrowed(isin.as_ref());
        }
        if let Some(exchange) = &self.exchange {
            return Cow::Owned(format!("{}@{}", self.symbol, exchange.code()));
        }
        Cow::Borrowed(self.symbol.as_str())
    }
}

impl std::fmt::Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unique_key())
    }
}

#[cfg(feature = "dataframe")]
mod dataframe_impl {
    use super::Instrument;
    use paft_utils::dataframe::{Columnar, ToDataFrame};
    use polars::datatypes::DataType;
    use polars::prelude::{DataFrame, NamedFrom, PolarsResult, Series};

    impl ToDataFrame for Instrument {
        fn to_dataframe(&self) -> PolarsResult<DataFrame> {
            <Self as Columnar>::columnar_to_dataframe(std::slice::from_ref(self))
        }

        fn empty_dataframe() -> PolarsResult<DataFrame> {
            DataFrame::new(
                0,
                vec![
                    Series::new_empty("kind".into(), &DataType::String).into(),
                    Series::new_empty("symbol".into(), &DataType::String).into(),
                    Series::new_empty("exchange".into(), &DataType::String).into(),
                    Series::new_empty("figi".into(), &DataType::String).into(),
                    Series::new_empty("isin".into(), &DataType::String).into(),
                ],
            )
        }

        fn schema() -> PolarsResult<Vec<(&'static str, DataType)>> {
            Ok(vec![
                ("kind", DataType::String),
                ("symbol", DataType::String),
                ("exchange", DataType::String),
                ("figi", DataType::String),
                ("isin", DataType::String),
            ])
        }
    }

    impl Columnar for Instrument {
        fn columnar_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
            let kinds: Vec<String> = items.iter().map(|i| i.kind.to_string()).collect();
            let symbols: Vec<String> = items.iter().map(|i| i.symbol.to_string()).collect();
            let exchanges: Vec<Option<String>> = items
                .iter()
                .map(|i| i.exchange.as_ref().map(std::string::ToString::to_string))
                .collect();
            let figis: Vec<Option<String>> = items
                .iter()
                .map(|i| i.figi.as_ref().map(|f| f.as_ref().to_string()))
                .collect();
            let isins: Vec<Option<String>> = items
                .iter()
                .map(|i| i.isin.as_ref().map(|v| v.as_ref().to_string()))
                .collect();

            let df = DataFrame::new(
                items.len(),
                vec![
                    Series::new("kind".into(), kinds).into(),
                    Series::new("symbol".into(), symbols).into(),
                    Series::new("exchange".into(), exchanges).into(),
                    Series::new("figi".into(), figis).into(),
                    Series::new("isin".into(), isins).into(),
                ],
            )?;
            Ok(df)
        }
    }
}
