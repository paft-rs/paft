//! Market session state enumeration with helpers and serde support.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString};

/// Market state enumeration with canonical variants and extensible fallback.
///
/// This enum provides type-safe handling of market states while gracefully
/// handling unknown or provider-specific states through the `Other` variant.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    AsRefStr,
    EnumString,
    Default,
)]
#[strum(ascii_case_insensitive)]
#[serde(from = "String", into = "String")]
pub enum MarketState {
    /// Pre-market trading hours
    #[strum(to_string = "PRE", serialize = "PREMARKET", serialize = "PRE-MARKET")]
    Pre,
    /// Regular trading hours
    #[strum(to_string = "REGULAR", serialize = "REG")]
    Regular,
    /// Post-market trading hours
    #[strum(
        to_string = "POST",
        serialize = "POSTMARKET",
        serialize = "POST-MARKET"
    )]
    Post,
    /// Market is closed
    #[strum(to_string = "CLOSED", serialize = "CLOSE")]
    #[default]
    Closed,
    /// Trading is halted
    #[strum(to_string = "HALTED", serialize = "HALT")]
    Halted,
    /// Unknown or provider-specific market state
    Other(String),
}

impl From<String> for MarketState {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        Self::from_str(&s).unwrap_or_else(|_| Self::Other(s.to_uppercase()))
    }
}

impl From<MarketState> for String {
    fn from(state: MarketState) -> Self {
        match state {
            MarketState::Other(s) => s,
            _ => state.to_string(),
        }
    }
}

impl MarketState {
    /// Returns true if the market state indicates active trading
    #[must_use]
    pub const fn is_trading(&self) -> bool {
        matches!(self, Self::Pre | Self::Regular | Self::Post)
    }

    /// Returns the string code for this enum variant
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Other(s) => s,
            _ => self.as_ref(),
        }
    }
}
