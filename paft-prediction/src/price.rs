//! Fixed-point prediction-market price, quantity, payout, and grid types.

use crate::error::PredictionError;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::num::NonZeroU32;

/// Fixed-point probability/price in millionths of unit payout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct OutcomePrice(u32);

impl OutcomePrice {
    /// Fixed-point scale: one whole unit is `1_000_000` micros.
    pub const SCALE: u32 = 1_000_000;
    /// Zero price.
    pub const ZERO: Self = Self(0);
    /// One whole unit of payout.
    pub const ONE: Self = Self(Self::SCALE);

    /// Construct an outcome price from fixed-point micros.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidOutcomePrice`] when `value` exceeds
    /// [`Self::SCALE`].
    pub const fn from_micros(value: u32) -> Result<Self, PredictionError> {
        if value <= Self::SCALE {
            Ok(Self(value))
        } else {
            Err(PredictionError::InvalidOutcomePrice { micros: value })
        }
    }

    /// Returns the fixed-point micro value.
    #[must_use]
    pub const fn micros(self) -> u32 {
        self.0
    }

    /// Returns the binary complement, `1.0 - price`.
    #[must_use]
    pub const fn complement(self) -> Self {
        Self(Self::SCALE - self.0)
    }

    /// Returns the integer midpoint between two prices, rounded toward zero.
    #[must_use]
    pub const fn midpoint(self, other: Self) -> Self {
        Self(self.0.midpoint(other.0))
    }

    pub(crate) const fn from_valid_micros(value: u32) -> Self {
        Self(value)
    }
}

impl TryFrom<u32> for OutcomePrice {
    type Error = PredictionError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_micros(value)
    }
}

impl From<OutcomePrice> for u32 {
    fn from(value: OutcomePrice) -> Self {
        value.micros()
    }
}

impl Serialize for OutcomePrice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.micros())
    }
}

impl<'de> Deserialize<'de> for OutcomePrice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u32::deserialize(deserializer)?;
        Self::from_micros(raw).map_err(de::Error::custom)
    }
}

/// Fixed-point tick size in outcome-price micros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PriceTick(NonZeroU32);

impl PriceTick {
    /// Construct a price tick from fixed-point micros.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidPriceTick`] when `value` is zero or
    /// exceeds [`OutcomePrice::SCALE`].
    pub const fn from_micros(value: u32) -> Result<Self, PredictionError> {
        if value == 0 || value > OutcomePrice::SCALE {
            return Err(PredictionError::InvalidPriceTick { micros: value });
        }
        match NonZeroU32::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(PredictionError::InvalidPriceTick { micros: value }),
        }
    }

    /// Returns the fixed-point micro tick.
    #[must_use]
    pub const fn micros(self) -> u32 {
        self.0.get()
    }
}

impl TryFrom<u32> for PriceTick {
    type Error = PredictionError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_micros(value)
    }
}

impl From<PriceTick> for u32 {
    fn from(value: PriceTick) -> Self {
        value.micros()
    }
}

impl Serialize for PriceTick {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.micros())
    }
}

impl<'de> Deserialize<'de> for PriceTick {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u32::deserialize(deserializer)?;
        Self::from_micros(raw).map_err(de::Error::custom)
    }
}

/// Contiguous price band with a fixed tick size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PriceBand {
    /// Inclusive band start.
    pub start: OutcomePrice,
    /// Inclusive band end.
    pub end: OutcomePrice,
    /// Tick size for prices in this band.
    pub tick: PriceTick,
}

impl PriceBand {
    /// Returns `true` when `price` lies in this band.
    #[must_use]
    pub const fn contains(&self, price: OutcomePrice) -> bool {
        self.start.micros() <= price.micros() && price.micros() <= self.end.micros()
    }

    /// Returns `true` when the band has non-descending bounds.
    #[must_use]
    pub const fn has_valid_bounds(&self) -> bool {
        self.start.micros() <= self.end.micros()
    }

    /// Returns `true` when `price` is contained by this band and aligned to
    /// the band's tick.
    #[must_use]
    pub const fn is_on_grid(&self, price: OutcomePrice) -> bool {
        self.contains(price)
            && (price.micros() - self.start.micros()).is_multiple_of(self.tick.micros())
    }
}

/// Piecewise price grid for a prediction market.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PriceGrid {
    /// Ordered price bands that define valid ticks.
    pub bands: Vec<PriceBand>,
}

impl PriceGrid {
    /// Construct a price grid and validate its bands.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidPriceGrid`] when the band list is
    /// empty, a band has descending bounds, or bands overlap.
    pub fn new(bands: Vec<PriceBand>) -> Result<Self, PredictionError> {
        let grid = Self { bands };
        grid.validate()?;
        Ok(grid)
    }

    /// Validate the grid's band structure.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidPriceGrid`] when the band list is
    /// empty, a band has descending bounds, or bands overlap.
    pub fn validate(&self) -> Result<(), PredictionError> {
        if self.bands.is_empty() {
            return Err(PredictionError::InvalidPriceGrid {
                reason: "at least one band is required",
            });
        }

        for band in &self.bands {
            if !band.has_valid_bounds() {
                return Err(PredictionError::InvalidPriceGrid {
                    reason: "band start must be less than or equal to band end",
                });
            }
        }

        for pair in self.bands.windows(2) {
            if pair[0].end.micros() >= pair[1].start.micros() {
                return Err(PredictionError::InvalidPriceGrid {
                    reason: "bands must be non-overlapping and ordered by price",
                });
            }
        }

        Ok(())
    }

    /// Returns the tick that applies to `price`, if any.
    #[must_use]
    pub fn tick_for(&self, price: OutcomePrice) -> Option<PriceTick> {
        self.bands
            .iter()
            .find(|band| band.contains(price))
            .map(|band| band.tick)
    }

    /// Returns `true` when `price` is valid for this grid.
    #[must_use]
    pub fn contains_price(&self, price: OutcomePrice) -> bool {
        self.bands.iter().any(|band| band.is_on_grid(price))
    }

    /// Validate that a price is accepted by this grid.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::PriceOffGrid`] when no band accepts `price`.
    pub fn validate_price(&self, price: OutcomePrice) -> Result<(), PredictionError> {
        if self.contains_price(price) {
            Ok(())
        } else {
            Err(PredictionError::PriceOffGrid {
                micros: price.micros(),
            })
        }
    }
}

/// Fixed-point contract/share quantity in millionths of one contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct ContractQuantity(u64);

impl ContractQuantity {
    /// Fixed-point scale: one whole contract is `1_000_000` microcontracts.
    pub const SCALE: u64 = 1_000_000;
    /// Zero contracts.
    pub const ZERO: Self = Self(0);
    /// One whole contract.
    pub const ONE: Self = Self(Self::SCALE);

    /// Construct a quantity from fixed-point microcontracts.
    #[must_use]
    pub const fn from_microcontracts(value: u64) -> Self {
        Self(value)
    }

    /// Returns the fixed-point microcontract count.
    #[must_use]
    pub const fn microcontracts(self) -> u64 {
        self.0
    }

    /// Returns `true` when the quantity is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<ContractQuantity> for u64 {
    fn from(value: ContractQuantity) -> Self {
        value.microcontracts()
    }
}

/// Currencyless unit-payout amount in millionths of the collateral unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct OutcomePayout(u64);

impl OutcomePayout {
    /// Fixed-point scale: one whole collateral unit is `1_000_000` micropayouts.
    pub const SCALE: u64 = 1_000_000;
    /// Zero payout.
    pub const ZERO: Self = Self(0);
    /// One whole collateral unit of payout.
    pub const ONE: Self = Self(Self::SCALE);

    /// Construct a payout from fixed-point micropayouts.
    #[must_use]
    pub const fn from_micropayouts(value: u64) -> Self {
        Self(value)
    }

    /// Returns the fixed-point micropayout count.
    #[must_use]
    pub const fn micropayouts(self) -> u64 {
        self.0
    }
}

impl From<OutcomePayout> for u64 {
    fn from(value: OutcomePayout) -> Self {
        value.micropayouts()
    }
}
