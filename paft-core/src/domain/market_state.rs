//! Market session state enumeration with helpers and serde support.
use std::str::FromStr;
/// Market state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum MarketState {
    /// Pre-market trading hours
    Pre,
    /// Regular trading hours
    Regular,
    /// Post-market trading hours
    Post,
    /// Market is closed
    #[default]
    Closed,
    /// Trading is halted (temporary)
    Halted,
    /// Trading is suspended (longer-term)
    Suspended,
    /// Auction period (opening/closing)
    Auction,
}

crate::string_enum_closed_with_code!(
    MarketState,
    "MarketState",
    {
        "PREMARKET" => MarketState::Pre,
        "REGULAR" => MarketState::Regular,
        "POSTMARKET" => MarketState::Post,
        "CLOSED" => MarketState::Closed,
        "HALTED" => MarketState::Halted,
        "SUSPENDED" => MarketState::Suspended,
        "AUCTION" => MarketState::Auction
    },
    {
        "PRE" => MarketState::Pre,
        "POST" => MarketState::Post,
    }
);

crate::impl_display_via_code!(MarketState);

impl MarketState {
    /// Human-readable label for displaying this market state.
    #[must_use]
    pub const fn full_name(&self) -> &'static str {
        match self {
            Self::Pre => "Pre-market",
            Self::Regular => "Regular session",
            Self::Post => "Post-market",
            Self::Closed => "Closed",
            Self::Halted => "Halted",
            Self::Suspended => "Suspended",
            Self::Auction => "Auction",
        }
    }

    /// Returns true if the market state indicates active trading
    #[must_use]
    pub const fn is_trading(&self) -> bool {
        matches!(self, Self::Pre | Self::Regular | Self::Post)
    }
}
