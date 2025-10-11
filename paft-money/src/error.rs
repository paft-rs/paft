use crate::currency::Currency;
use crate::decimal::Decimal;
use thiserror::Error;

#[cfg(feature = "money-formatting")]
use crate::locale::Locale;

/// Errors emitted by the paft-money crate.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MoneyParseError {
    /// Invalid value provided for an enum parser.
    #[error("Invalid {enum_name} value: '{value}'")]
    InvalidEnumValue {
        /// Enum type name for context (e.g., "Currency").
        enum_name: &'static str,
        /// The offending input value.
        value: String,
    },
}

/// Errors that can occur when performing operations on Money values.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// Occurs when attempting to perform arithmetic operations on Money values with different currencies.
    #[error("currency mismatch: expected {expected}, found {found}")]
    CurrencyMismatch {
        /// The expected currency.
        expected: Currency,
        /// The actual currency found.
        found: Currency,
    },
    /// Occurs when converting a Money amount to cents fails due to overflow or precision issues.
    #[error("could not convert amount to minor units")]
    ConversionError,
    /// Occurs when attempting to divide by zero.
    #[error("division by zero")]
    DivisionByZero,
    /// Occurs when an exchange rate is invalid (e.g., negative or zero rate).
    #[error("invalid exchange rate: {rate}")]
    InvalidExchangeRate {
        /// The invalid rate value.
        rate: Decimal,
    },
    /// Occurs when attempting to convert using an incompatible exchange rate.
    #[error(
        "incompatible exchange rate: from {from} to {to}, but money currency is {money_currency}"
    )]
    IncompatibleExchangeRate {
        /// The source currency of the exchange rate.
        from: Currency,
        /// The target currency of the exchange rate.
        to: Currency,
        /// The currency of the money being converted.
        money_currency: Currency,
    },
    /// Occurs when attempting to use a currency without registered metadata.
    #[error("metadata not registered for currency {currency}")]
    MetadataNotFound {
        /// The currency missing metadata.
        currency: Currency,
    },
    /// Occurs when parsing a decimal amount fails.
    #[error("invalid decimal")]
    InvalidDecimal,
    /// Occurs when parsing a currency fails.
    #[error("invalid currency: {source}")]
    InvalidCurrency {
        /// Underlying currency parsing error.
        source: MoneyParseError,
    },
    /// Occurs when a localized amount has invalid separators or characters.
    #[cfg(feature = "money-formatting")]
    #[error("invalid localized amount format")]
    InvalidAmountFormat,
    /// Occurs when digit groups do not match the expected locale pattern.
    #[cfg(feature = "money-formatting")]
    #[error("invalid grouping for locale")]
    InvalidGrouping,
    /// Occurs when the detected currency symbol or code does not match the provided currency.
    #[cfg(feature = "money-formatting")]
    #[error("currency affix does not match provided currency")]
    MismatchedCurrencyAffix,
    /// Occurs when fraction digits exceed the currency exponent during parsing.
    #[cfg(feature = "money-formatting")]
    #[error("fraction scale {digits} exceeds currency exponent {exponent}")]
    ScaleTooLarge {
        /// Observed fractional digits.
        digits: usize,
        /// Expected exponent for the currency.
        exponent: u8,
    },
    /// Occurs when attempting to use an unsupported locale for formatting or parsing.
    #[cfg(feature = "money-formatting")]
    #[error("unsupported locale: {locale:?}")]
    UnsupportedLocale {
        /// Requested locale.
        locale: Locale,
    },
}
