//! Market session state enumeration with helpers and serde support.
use crate::Canonical;

/// Market state enumeration.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix) and must be non-empty
/// - `Display` output matches the canonical code for known variants and the raw `s` for `Other(s)`
/// - Serde round-trips preserve identity for canonical variants; unknown tokens normalize to `Other(UPPERCASE)`
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
    /// Provider-specific market state not modeled as a canonical variant.
    Other(Canonical),
}

crate::string_enum_with_code!(
    MarketState, Other,
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
        "OPEN" => MarketState::Regular,
        "PREOPEN" => MarketState::Pre,
        "PRE_OPEN" => MarketState::Pre,
        "PRE_MARKET" => MarketState::Pre,
        "REGULAR_MARKET" => MarketState::Regular,
        "POST_MARKET" => MarketState::Post,
    }
);

crate::impl_display_via_code!(MarketState);

impl MarketState {
    /// Human-readable label for displaying this market state.
    #[must_use]
    pub fn full_name(&self) -> &str {
        match self {
            Self::Pre => "Pre-market",
            Self::Regular => "Regular session",
            Self::Post => "Post-market",
            Self::Closed => "Closed",
            Self::Halted => "Halted",
            Self::Suspended => "Suspended",
            Self::Auction => "Auction",
            Self::Other(code) => code.as_ref(),
        }
    }

    /// Returns true if the market state indicates active trading
    #[must_use]
    pub const fn is_trading(&self) -> bool {
        matches!(self, Self::Pre | Self::Regular | Self::Post)
    }
}
