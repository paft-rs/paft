//! Fixed-point prediction-market price, quantity, payout, and grid types.

use crate::error::PredictionError;
use paft_decimal::{Decimal, from_minor_units, to_canonical_string};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{
    fmt,
    num::{NonZeroU32, NonZeroU64},
};

const FIXED_SCALE: u64 = 1_000_000;
const FIXED_SCALE_DIGITS: usize = 6;
const FIXED_SCALE_DIGITS_U32: u32 = 6;

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

    /// Construct an outcome price from an exact decimal value.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] when the value is negative, greater than
    /// `1`, not finite in the active decimal backend, or cannot be represented
    /// exactly in millionths.
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_decimal(value: Decimal) -> Result<Self, PredictionError> {
        Self::from_canonical_str(&to_canonical_string(&value))
    }

    /// Parse an exact decimal outcome price.
    ///
    /// The accepted scale is millionths. Extra trailing zero fractional digits
    /// are accepted, but non-zero digits beyond six decimal places are rejected
    /// instead of rounded.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] when `value` is not a plain decimal string,
    /// is negative, is greater than `1`, or cannot be represented exactly in
    /// millionths.
    pub fn from_canonical_str(value: &str) -> Result<Self, PredictionError> {
        let micros = parse_fixed_point_units(value, "outcome price")?;
        if micros > u64::from(Self::SCALE) {
            return Err(PredictionError::invalid_fixed_point_decimal(
                "outcome price",
                value,
                "expected value in 0..=1",
            ));
        }

        let micros = u32::try_from(micros).map_err(|_| {
            PredictionError::invalid_fixed_point_decimal(
                "outcome price",
                value,
                "expected value in 0..=1",
            )
        })?;
        Self::from_micros(micros)
    }

    /// Convert this price into a decimal value.
    #[must_use]
    pub fn to_decimal(self) -> Decimal {
        from_minor_units(i128::from(self.0), FIXED_SCALE_DIGITS_U32)
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

impl fmt::Display for OutcomePrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_fixed_point_units(f, u64::from(self.micros()))
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

impl fmt::Display for PriceTick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_fixed_point_units(f, u64::from(self.micros()))
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
#[serde(deny_unknown_fields)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
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
    /// empty, a band has descending bounds, a band endpoint is not reachable
    /// on its tick grid, or bands overlap.
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
    /// empty, a band has descending bounds, a band endpoint is not reachable
    /// on its tick grid, or bands overlap.
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

            if !(band.end.micros() - band.start.micros()).is_multiple_of(band.tick.micros()) {
                return Err(PredictionError::InvalidPriceGrid {
                    reason: "band end must align to the band's tick grid",
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

impl<'de> Deserialize<'de> for PriceGrid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct PriceGridShadow {
            bands: Vec<PriceBand>,
        }

        let shadow = PriceGridShadow::deserialize(deserializer)?;
        Self::new(shadow.bands).map_err(de::Error::custom)
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

    /// Construct a contract quantity from an exact decimal value.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] when the value is negative, cannot be stored
    /// as a `u64` microcontract count, or cannot be represented exactly in
    /// millionths.
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_decimal(value: Decimal) -> Result<Self, PredictionError> {
        Self::from_canonical_str(&to_canonical_string(&value))
    }

    /// Parse an exact decimal contract quantity.
    ///
    /// The accepted scale is millionths. Extra trailing zero fractional digits
    /// are accepted, but non-zero digits beyond six decimal places are rejected
    /// instead of rounded.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] when `value` is not a plain decimal string,
    /// is negative, cannot be stored as a `u64` microcontract count, or cannot
    /// be represented exactly in millionths.
    pub fn from_canonical_str(value: &str) -> Result<Self, PredictionError> {
        Ok(Self(parse_fixed_point_units(value, "contract quantity")?))
    }

    /// Convert this quantity into a decimal value.
    #[must_use]
    pub fn to_decimal(self) -> Decimal {
        from_minor_units(i128::from(self.0), FIXED_SCALE_DIGITS_U32)
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

impl fmt::Display for ContractQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_fixed_point_units(f, self.microcontracts())
    }
}

impl From<ContractQuantity> for u64 {
    fn from(value: ContractQuantity) -> Self {
        value.microcontracts()
    }
}

/// Non-zero fixed-point contract/share quantity in millionths of one contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct NonZeroContractQuantity(NonZeroU64);

impl NonZeroContractQuantity {
    /// Fixed-point scale: one whole contract is `1_000_000` microcontracts.
    pub const SCALE: u64 = ContractQuantity::SCALE;
    /// One whole contract.
    pub const ONE: Self = Self(NonZeroU64::new(Self::SCALE).expect("scale is non-zero"));

    /// Construct a non-zero quantity from an already non-zero microcontract count.
    #[must_use]
    pub const fn from_nonzero_microcontracts(value: NonZeroU64) -> Self {
        Self(value)
    }

    /// Construct a non-zero quantity from fixed-point microcontracts.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidContractQuantity`] when `value` is
    /// zero.
    pub const fn from_microcontracts(value: u64) -> Result<Self, PredictionError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(PredictionError::InvalidContractQuantity {
                microcontracts: value,
            }),
        }
    }

    /// Construct from a zero-capable contract quantity.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError::InvalidContractQuantity`] when `value` is
    /// zero.
    pub const fn from_quantity(value: ContractQuantity) -> Result<Self, PredictionError> {
        Self::from_microcontracts(value.microcontracts())
    }

    /// Construct a non-zero contract quantity from an exact decimal value.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] when the value is zero, negative, cannot be
    /// stored as a `u64` microcontract count, or cannot be represented exactly
    /// in millionths.
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_decimal(value: Decimal) -> Result<Self, PredictionError> {
        Self::from_canonical_str(&to_canonical_string(&value))
    }

    /// Parse an exact non-zero decimal contract quantity.
    ///
    /// The accepted scale is millionths. Extra trailing zero fractional digits
    /// are accepted, but non-zero digits beyond six decimal places are rejected
    /// instead of rounded.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionError`] when `value` is zero, is not a plain decimal
    /// string, is negative, cannot be stored as a `u64` microcontract count, or
    /// cannot be represented exactly in millionths.
    pub fn from_canonical_str(value: &str) -> Result<Self, PredictionError> {
        Self::from_microcontracts(parse_fixed_point_units(value, "contract quantity")?)
    }

    /// Convert this quantity into a zero-capable contract quantity.
    #[must_use]
    pub const fn to_quantity(self) -> ContractQuantity {
        ContractQuantity::from_microcontracts(self.microcontracts())
    }

    /// Convert this quantity into a decimal value.
    #[must_use]
    pub fn to_decimal(self) -> Decimal {
        from_minor_units(i128::from(self.microcontracts()), FIXED_SCALE_DIGITS_U32)
    }

    /// Returns the fixed-point microcontract count.
    #[must_use]
    pub const fn microcontracts(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for NonZeroContractQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_fixed_point_units(f, self.microcontracts())
    }
}

impl TryFrom<u64> for NonZeroContractQuantity {
    type Error = PredictionError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_microcontracts(value)
    }
}

impl TryFrom<ContractQuantity> for NonZeroContractQuantity {
    type Error = PredictionError;

    fn try_from(value: ContractQuantity) -> Result<Self, Self::Error> {
        Self::from_quantity(value)
    }
}

impl From<NonZeroContractQuantity> for u64 {
    fn from(value: NonZeroContractQuantity) -> Self {
        value.microcontracts()
    }
}

impl From<NonZeroContractQuantity> for ContractQuantity {
    fn from(value: NonZeroContractQuantity) -> Self {
        value.to_quantity()
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

impl fmt::Display for OutcomePayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_fixed_point_units(f, self.micropayouts())
    }
}

impl From<OutcomePayout> for u64 {
    fn from(value: OutcomePayout) -> Self {
        value.micropayouts()
    }
}

fn parse_fixed_point_units(value: &str, kind: &'static str) -> Result<u64, PredictionError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(PredictionError::invalid_fixed_point_decimal(
            kind,
            value,
            "expected a plain decimal string",
        ));
    }

    let (is_negative, unsigned) = match trimmed.as_bytes().first().copied() {
        Some(b'+') => (false, &trimmed[1..]),
        Some(b'-') => (true, &trimmed[1..]),
        Some(_) => (false, trimmed),
        None => {
            return Err(PredictionError::invalid_fixed_point_decimal(
                kind,
                value,
                "expected a plain decimal string",
            ));
        }
    };

    if unsigned.is_empty() {
        return Err(PredictionError::invalid_fixed_point_decimal(
            kind,
            value,
            "expected digits",
        ));
    }

    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty() && fraction.is_empty() {
        return Err(PredictionError::invalid_fixed_point_decimal(
            kind,
            value,
            "expected digits",
        ));
    }

    if !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(PredictionError::invalid_fixed_point_decimal(
            kind,
            value,
            "expected a plain decimal string",
        ));
    }

    if fraction.len() > FIXED_SCALE_DIGITS
        && fraction.as_bytes()[FIXED_SCALE_DIGITS..]
            .iter()
            .any(|byte| *byte != b'0')
    {
        return Err(PredictionError::invalid_fixed_point_decimal(
            kind,
            value,
            "more than 6 fractional digits",
        ));
    }

    let whole_units = parse_digit_units(whole, kind, value)?;
    let accepted_fraction_len = fraction.len().min(FIXED_SCALE_DIGITS);
    let accepted_fraction = &fraction[..accepted_fraction_len];
    let mut fraction_units = parse_digit_units(accepted_fraction, kind, value)?;
    for _ in accepted_fraction_len..FIXED_SCALE_DIGITS {
        fraction_units *= 10;
    }

    let scaled = whole_units
        .checked_mul(u128::from(FIXED_SCALE))
        .and_then(|whole| whole.checked_add(fraction_units))
        .ok_or_else(|| {
            PredictionError::invalid_fixed_point_decimal(kind, value, "exceeds fixed-point storage")
        })?;

    if is_negative && scaled != 0 {
        return Err(PredictionError::invalid_fixed_point_decimal(
            kind,
            value,
            "expected a non-negative decimal",
        ));
    }

    u64::try_from(scaled).map_err(|_| {
        PredictionError::invalid_fixed_point_decimal(kind, value, "exceeds fixed-point storage")
    })
}

fn parse_digit_units(
    digits: &str,
    kind: &'static str,
    value: &str,
) -> Result<u128, PredictionError> {
    let mut units = 0_u128;
    for byte in digits.bytes() {
        let digit = u128::from(byte - b'0');
        units = units
            .checked_mul(10)
            .and_then(|current| current.checked_add(digit))
            .ok_or_else(|| {
                PredictionError::invalid_fixed_point_decimal(
                    kind,
                    value,
                    "exceeds fixed-point storage",
                )
            })?;
    }
    Ok(units)
}

fn fmt_fixed_point_units(f: &mut fmt::Formatter<'_>, units: u64) -> fmt::Result {
    let whole = units / FIXED_SCALE;
    let mut fraction = units % FIXED_SCALE;

    if fraction == 0 {
        return write!(f, "{whole}");
    }

    let mut width = FIXED_SCALE_DIGITS;
    while fraction.is_multiple_of(10) {
        fraction /= 10;
        width -= 1;
    }

    write!(f, "{whole}.{fraction:0width$}")
}
