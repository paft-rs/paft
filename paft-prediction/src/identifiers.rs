//! Venue and opaque identifier newtypes for prediction markets.

use crate::error::PredictionError;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;
use std::{convert::TryFrom, fmt, str::FromStr};

/// Maximum accepted byte length for provider-native prediction identifiers.
pub const MAX_PREDICTION_ID_LEN: usize = 256;

fn contains_disallowed_identifier_char(value: &str) -> bool {
    value.chars().any(|c| c.is_control() || c.is_whitespace())
}

pub(crate) fn validate_opaque_identifier(
    kind: &'static str,
    input: &str,
) -> Result<SmolStr, PredictionError> {
    let trimmed = input.trim();
    if trimmed.is_empty()
        || trimmed.len() > MAX_PREDICTION_ID_LEN
        || contains_disallowed_identifier_char(trimmed)
    {
        return Err(PredictionError::invalid_identifier(kind, input.to_string()));
    }

    Ok(SmolStr::new(trimmed))
}

fn validate_venue(input: &str) -> Result<SmolStr, PredictionError> {
    let trimmed = input.trim();
    if trimmed.is_empty()
        || trimmed.len() > MAX_PREDICTION_ID_LEN
        || contains_disallowed_identifier_char(trimmed)
    {
        return Err(PredictionError::InvalidVenue {
            value: input.to_string(),
        });
    }

    Ok(SmolStr::new(trimmed))
}

macro_rules! opaque_prediction_id {
    (
        $(#[$meta:meta])*
        pub struct $name:ident;
        kind = $kind:literal;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(SmolStr);

        impl $name {
            /// Construct a new validated opaque identifier.
            ///
            /// # Errors
            ///
            /// Returns [`PredictionError::InvalidIdentifier`] when the trimmed
            /// value is empty, exceeds [`MAX_PREDICTION_ID_LEN`] bytes, or
            /// contains whitespace/control characters.
            pub fn new(value: &str) -> Result<Self, PredictionError> {
                validate_opaque_identifier($kind, value).map(Self)
            }

            /// Returns the provider-native identifier string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl FromStr for $name {
            type Err = PredictionError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                Self::new(input)
            }
        }

        impl TryFrom<String> for $name {
            type Error = PredictionError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(&value)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0.to_string()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = String::deserialize(deserializer)?;
                Self::new(&raw).map_err(de::Error::custom)
            }
        }
    };
}

opaque_prediction_id!(
    /// Opaque provider-native recurring series/group identifier.
    pub struct PredictionSeriesId;
    kind = "prediction series ID";
);

opaque_prediction_id!(
    /// Opaque provider-native event/group identifier.
    pub struct PredictionEventId;
    kind = "prediction event ID";
);

opaque_prediction_id!(
    /// Opaque provider-native atomic market/claim identifier.
    pub struct PredictionMarketId;
    kind = "prediction market ID";
);

opaque_prediction_id!(
    /// Opaque provider-native outcome/instrument identifier.
    pub struct PredictionOutcomeId;
    kind = "prediction outcome ID";
);

/// Provider venue for prediction-market data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PredictionVenue {
    /// Kalshi.
    Kalshi,
    /// Polymarket.
    Polymarket,
    /// Manifold Markets.
    Manifold,
    /// Provider-specific or future venue code.
    Other(OtherPredictionVenue),
}

impl PredictionVenue {
    /// Builds an unknown venue value, rejecting modeled venue codes.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidVenue`] if the input is empty, unsafe,
    /// too long, or names a modeled venue.
    pub fn other(input: &str) -> Result<Self, PredictionError> {
        OtherPredictionVenue::new(input).map(Self::Other)
    }

    /// Parses a venue from a string.
    ///
    /// Modeled venue names are case-insensitive. Unknown venue codes are
    /// preserved exactly after trimming surrounding whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidVenue`] if the input is empty, unsafe,
    /// or too long.
    pub fn try_from_str(input: &str) -> Result<Self, PredictionError> {
        let value = validate_venue(input)?;
        let normalized = value.as_str();

        if normalized.eq_ignore_ascii_case("KALSHI") {
            Ok(Self::Kalshi)
        } else if normalized.eq_ignore_ascii_case("POLYMARKET") {
            Ok(Self::Polymarket)
        } else if normalized.eq_ignore_ascii_case("MANIFOLD") {
            Ok(Self::Manifold)
        } else {
            Ok(Self::Other(OtherPredictionVenue(value)))
        }
    }

    /// Returns the canonical venue code for modeled venues or the provider
    /// code for `Other`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Kalshi => "KALSHI",
            Self::Polymarket => "POLYMARKET",
            Self::Manifold => "MANIFOLD",
            Self::Other(code) => code.as_str(),
        }
    }
}

impl AsRef<str> for PredictionVenue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for PredictionVenue {
    type Err = PredictionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(input)
    }
}

impl fmt::Display for PredictionVenue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for PredictionVenue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PredictionVenue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::try_from_str(&raw).map_err(de::Error::custom)
    }
}

/// Provider-specific prediction venue code not modeled by [`PredictionVenue`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OtherPredictionVenue(SmolStr);

impl OtherPredictionVenue {
    /// Construct a new unknown venue code.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidVenue`] when the input is empty,
    /// unsafe, too long, or names a modeled venue.
    pub fn new(input: &str) -> Result<Self, PredictionError> {
        let value = validate_venue(input)?;
        if value.as_str().eq_ignore_ascii_case("KALSHI")
            || value.as_str().eq_ignore_ascii_case("POLYMARKET")
            || value.as_str().eq_ignore_ascii_case("MANIFOLD")
        {
            return Err(PredictionError::InvalidVenue {
                value: input.to_string(),
            });
        }

        Ok(Self(value))
    }

    /// Returns the preserved provider venue code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for OtherPredictionVenue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for OtherPredictionVenue {
    type Err = PredictionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(input)
    }
}

impl fmt::Display for OtherPredictionVenue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for OtherPredictionVenue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for OtherPredictionVenue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::new(&raw).map_err(de::Error::custom)
    }
}
