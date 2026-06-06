//! Market session state enumeration with helpers and serde support.
use crate::DomainError;
use std::borrow::Cow;

paft_core::other_string_code_type!(
    /// Provider-specific market state that is not modeled by [`MarketState`].
    pub struct OtherMarketState for MarketState;
    type Error = DomainError;
    parse(input) => input.parse::<MarketState>();
    invalid(input) => DomainError::InvalidMarketStateValue {
        value: input.to_string(),
    };
);

/// Market state enumeration.
///
/// Canonical/serde rules:
/// - Emission uses a single canonical form per variant (UPPERCASE ASCII, no spaces)
/// - Parser accepts a superset of tokens (aliases, case-insensitive)
/// - `Other(s)` serializes to its canonical `code()` string (no escape prefix)
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
    Other(OtherMarketState),
}

crate::string_enum_with_code!(
    MarketState, Other(OtherMarketState),
    "MarketState",
    type Error = DomainError;
    invalid(input) => DomainError::InvalidMarketStateValue {
        value: input.to_string(),
    };
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
        "PRE_MARKET" => MarketState::Pre,
        "POST_MARKET" => MarketState::Post,
    }
);

crate::impl_display_via_code!(MarketState);

impl MarketState {
    /// Builds an unknown market state, rejecting tokens modeled by [`MarketState`].
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is empty, cannot be canonicalized, or parses
    /// to a modeled [`MarketState`] variant.
    pub fn other(input: &str) -> Result<Self, DomainError> {
        OtherMarketState::new(input).map(Self::Other)
    }

    /// Human-readable label for displaying this market state.
    #[must_use]
    pub fn full_name(&self) -> Cow<'static, str> {
        match self {
            Self::Pre => Cow::Borrowed("Pre-market"),
            Self::Regular => Cow::Borrowed("Regular session"),
            Self::Post => Cow::Borrowed("Post-market"),
            Self::Closed => Cow::Borrowed("Closed"),
            Self::Halted => Cow::Borrowed("Halted"),
            Self::Suspended => Cow::Borrowed("Suspended"),
            Self::Auction => Cow::Borrowed("Auction"),
            Self::Other(code) => Cow::Owned(code.as_ref().to_string()),
        }
    }

    /// Returns true if the market state indicates active trading
    #[must_use]
    pub const fn is_trading(&self) -> bool {
        matches!(self, Self::Pre | Self::Regular | Self::Post)
    }
}
