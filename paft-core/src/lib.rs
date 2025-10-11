//! Core infrastructure utilities for the paft ecosystem.
//!
//! This crate provides:
//! - Workspace-wide error type [`PaftError`]
//! - Macros for canonical string enums (open/closed) and `Display` via `code()`
//! - Reusable serde helpers for common timestamp encodings
//! - Optional re-exports for lightweight dataframe traits
//!
//! # Quickstart
//! ```rust
//! use paft_core::{PaftError, string_enum_closed_with_code, impl_display_via_code};
//! # use std::str::FromStr;
//!
//! // Define a closed string enum with canonical codes and parsing
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! enum Side { Buy, Sell }
//! paft_core::string_enum_closed_with_code!(
//!     Side, "Side",
//!     { "BUY" => Side::Buy, "SELL" => Side::Sell }
//! );
//! paft_core::impl_display_via_code!(Side);
//!
//! assert_eq!(Side::Buy.code(), "BUY");
//! assert_eq!("sell".parse::<Side>().unwrap(), Side::Sell);
//! assert!(matches!("".parse::<Side>(), Err(PaftError::InvalidEnumValue { .. })));
//! ```
//!
//! # Feature flags
//! - `dataframe`: re-export dataframe traits from `paft-utils`
//!
//! # Serde helpers
//! See [`serde_helpers`] for helpers like `ts_seconds_vec` and `ts_seconds_option`.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Error definitions shared across crates.
pub mod error;
/// Private serde helper modules for custom serialization patterns.
pub mod serde_helpers;

#[cfg(feature = "dataframe")]
/// Re-export `DataFrame` conversion traits from `paft-utils`
pub mod dataframe {
    pub use paft_utils::dataframe::*;
}

/// Internal macro exports for string-backed enums used across the paft workspace.
/// These remain public for crate interoperability but are not covered by semver guarantees.
#[doc(hidden)]
#[macro_export]
macro_rules! __string_enum_base {
    (
        $Type:ident, $enum_name:literal, error, {
            $( $alias:literal => $variant:path ),+ $(,)?
        }
    ) => {
        impl paft_utils::StringCode for $Type {
            fn code(&self) -> &str { $Type::code(self) }
        }

        impl ::serde::Serialize for $Type {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(self.code())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $Type {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let raw = <String as ::serde::Deserialize>::deserialize(deserializer)?;
                Self::from_str(&raw).map_err(::serde::de::Error::custom)
            }
        }

        impl ::std::str::FromStr for $Type {
            type Err = $crate::error::PaftError;

            fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    return Err($crate::error::PaftError::InvalidEnumValue {
                        enum_name: $enum_name,
                        value: input.to_string(),
                    });
                }
                let token = paft_utils::canonicalize(trimmed);
                let parsed = match token.as_ref() {
                    $( $alias => $variant, )*
                    _ => {
                        return Err($crate::error::PaftError::InvalidEnumValue {
                            enum_name: $enum_name,
                            value: input.to_string(),
                        });
                    }
                };
                Ok(parsed)
            }
        }

        impl ::std::convert::TryFrom<String> for $Type {
            type Error = $crate::error::PaftError;

            fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
                Self::from_str(&value)
            }
        }

        impl ::std::convert::From<$Type> for String {
            fn from(v: $Type) -> Self { v.code().to_string() }
        }

        impl $Type {
            #[doc(hidden)]
            pub const __ALIASES: &'static [(&'static str, $Type)] = &[
                $( ($alias, $variant) ),*
            ];
        }
    };

    (
        $Type:ident, $enum_name:literal, other($OtherVariant:path), {
            $( $alias:literal => $variant:path ),+ $(,)?
        }
    ) => {
        impl paft_utils::StringCode for $Type {
            fn code(&self) -> &str { $Type::code(self) }
            fn is_canonical(&self) -> bool { $Type::is_canonical(self) }
        }

        impl $Type {
            /// Returns true when this value represents a canonical variant.
            #[must_use]
            pub const fn is_canonical(&self) -> bool { !matches!(self, $OtherVariant(_)) }
        }

        impl ::serde::Serialize for $Type {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(self.code())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $Type {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let raw = <String as ::serde::Deserialize>::deserialize(deserializer)?;
                Self::from_str(&raw).map_err(::serde::de::Error::custom)
            }
        }

        impl ::std::str::FromStr for $Type {
            type Err = $crate::error::PaftError;

            fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    return Err($crate::error::PaftError::InvalidEnumValue {
                        enum_name: $enum_name,
                        value: input.to_string(),
                    });
                }
                let token = paft_utils::canonicalize(trimmed);
                let parsed = match token.as_ref() {
                    $( $alias => $variant, )*
                    _ => {
                        let canon = paft_utils::Canonical::try_new(trimmed)
                            .map_err(|_| $crate::error::PaftError::InvalidEnumValue {
                                enum_name: $enum_name,
                                value: input.to_string(),
                            })?;
                        return Ok($OtherVariant(canon));
                    }
                };
                Ok(parsed)
            }
        }

        impl ::std::convert::TryFrom<String> for $Type {
            type Error = $crate::error::PaftError;

            fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
                Self::from_str(&value)
            }
        }

        impl ::std::convert::From<$Type> for String {
            fn from(v: $Type) -> Self { v.code().to_string() }
        }

        impl $Type {
            #[doc(hidden)]
            pub const __ALIASES: &'static [(&'static str, $Type)] = &[
                $( ($alias, $variant) ),*
            ];
        }
    };
}

/// Helper to implement Display using the type's `code()` method.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_display_via_code {
    ( $Type:ident ) => {
        impl ::std::fmt::Display for $Type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.code())
            }
        }
    };
}

/// Open (extensible) string enum with macro-provided `code()` and open parsing.
#[doc(hidden)]
#[macro_export]
macro_rules! string_enum_with_code {
    (
        $Type:ident, $Other:ident, $enum_name:literal,
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? },
        { $( $alias:literal => $variant:path ),* $(,)? }
    ) => {
        impl $Type {
            /// Returns the canonical string code for this value.
            #[must_use]
            pub fn code(&self) -> &str {
                match self {
                    $( $canon_variant => $canon_token, )+
                    $Type::$Other(s) => s.as_ref(),
                }
            }
        }

        $crate::__string_enum_base! {
            $Type, $enum_name, other($Type::$Other),
            { $( $canon_token => $canon_variant ),+ $(, $alias => $variant )* }
        }
    };

    (
        $Type:ident, $Other:ident, $enum_name:literal,
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? }
    ) => {
        $crate::string_enum_with_code!(
            $Type, $Other, $enum_name,
            { $( $canon_token => $canon_variant ),+ },
            {}
        );
    };
}

/// Closed string enum with macro-provided `code()` and closed parsing.
#[doc(hidden)]
#[macro_export]
macro_rules! string_enum_closed_with_code {
    (
        $Type:ident, $enum_name:literal,
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? },
        { $( $alias:literal => $variant:path ),* $(,)? }
    ) => {
        impl $Type {
            /// Returns the canonical string code for this value.
            #[must_use]
            pub const fn code(&self) -> &str {
                match self {
                    $( $canon_variant => $canon_token, )+
                }
            }
        }

        $crate::__string_enum_base! {
            $Type, $enum_name, error,
            { $( $canon_token => $canon_variant ),+ $(, $alias => $variant )* }
        }
    };

    (
        $Type:ident, $enum_name:literal,
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? }
    ) => {
        $crate::string_enum_closed_with_code!(
            $Type, $enum_name,
            { $( $canon_token => $canon_variant ),+ },
            {}
        );
    };
}

pub use error::PaftError;

#[cfg(feature = "dataframe")]
pub use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
