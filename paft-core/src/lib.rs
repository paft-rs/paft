//! Core infrastructure utilities for the paft ecosystem.
//!
//! This crate provides:
//! - Workspace-wide error type [`PaftError`]
//! - Macros for canonical string enums (open/closed) and `Display` via `code()`
//! - Reusable serde helpers for common timestamp encodings
//!
//! # Quickstart
//! ```rust
//! use paft_core::{PaftError, string_enum_closed_with_code, impl_display_via_code};
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
//! # Serde helpers
//! See [`serde_helpers`] for helpers like `ts_milliseconds_vec`.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Error definitions shared across crates.
pub mod error;
/// Private serde helper modules for custom serialization patterns.
pub mod serde_helpers;

/// Internal re-export of `paft_utils` so the `string_enum_*` macros can resolve
/// canonicalization helpers via `$crate::__utils::...` regardless of how the
/// consumer crate names or vendors `paft-utils`. Not part of the public API.
#[doc(hidden)]
pub use paft_utils as __utils;

/// Internal re-export of `serde` so the `string_enum_*` macros can resolve
/// serialization traits through `$crate` without forcing downstream crates to
/// depend on `serde` directly.
#[doc(hidden)]
pub use serde as __serde;

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
        $crate::__string_enum_base! {
            $Type, $enum_name,
            error(
                $crate::error::PaftError,
                invalid(input) => $crate::error::PaftError::InvalidEnumValue {
                    enum_name: $enum_name,
                    value: input.to_string(),
                }
            ),
            { $( $alias => $variant ),+ }
        }
    };

    (
        $Type:ident, $enum_name:literal,
        error($Error:ty, invalid($invalid_input:ident) => $invalid:expr), {
            $( $alias:literal => $variant:path ),+ $(,)?
        }
    ) => {
        impl $crate::__utils::StringCode for $Type {
            fn code(&self) -> &str { $Type::code(self) }
        }

        impl $crate::__serde::Serialize for $Type {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: $crate::__serde::Serializer,
            {
                serializer.serialize_str(self.code())
            }
        }

        impl<'de> $crate::__serde::Deserialize<'de> for $Type {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: $crate::__serde::Deserializer<'de>,
            {
                let raw = <String as $crate::__serde::Deserialize>::deserialize(deserializer)?;
                <Self as ::std::str::FromStr>::from_str(&raw)
                    .map_err($crate::__serde::de::Error::custom)
            }
        }

        impl ::std::str::FromStr for $Type {
            type Err = $Error;

            fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    return Err({
                        let $invalid_input = input;
                        $invalid
                    });
                }
                let token = $crate::__utils::canonicalize(trimmed);
                let parsed = match token.as_ref() {
                    $( $alias => $variant, )*
                    _ => {
                        return Err({
                            let $invalid_input = input;
                            $invalid
                        });
                    }
                };
                Ok(parsed)
            }
        }

        impl ::std::convert::TryFrom<String> for $Type {
            type Error = $Error;

            fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
                <Self as ::std::str::FromStr>::from_str(&value)
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
        $crate::__string_enum_base! {
            $Type, $enum_name,
            other(
                $OtherVariant,
                error(
                    $crate::error::PaftError,
                    invalid(input) => $crate::error::PaftError::InvalidEnumValue {
                        enum_name: $enum_name,
                        value: input.to_string(),
                    }
                )
            ),
            { $( $alias => $variant ),+ }
        }
    };

    (
        $Type:ident, $enum_name:literal,
        other($OtherVariant:path, error($Error:ty, invalid($invalid_input:ident) => $invalid:expr)), {
            $( $alias:literal => $variant:path ),+ $(,)?
        }
    ) => {
        impl $crate::__utils::StringCode for $Type {
            fn code(&self) -> &str { $Type::code(self) }
            fn is_canonical(&self) -> bool { $Type::is_canonical(self) }
        }

        impl $Type {
            /// Returns true when this value represents a canonical variant.
            #[must_use]
            pub const fn is_canonical(&self) -> bool { !matches!(self, $OtherVariant(_)) }
        }

        impl $crate::__serde::Serialize for $Type {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: $crate::__serde::Serializer,
            {
                serializer.serialize_str(self.code())
            }
        }

        impl<'de> $crate::__serde::Deserialize<'de> for $Type {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: $crate::__serde::Deserializer<'de>,
            {
                let raw = <String as $crate::__serde::Deserialize>::deserialize(deserializer)?;
                <Self as ::std::str::FromStr>::from_str(&raw)
                    .map_err($crate::__serde::de::Error::custom)
            }
        }

        impl ::std::str::FromStr for $Type {
            type Err = $Error;

            fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    return Err({
                        let $invalid_input = input;
                        $invalid
                    });
                }
                let token = $crate::__utils::canonicalize(trimmed);
                let parsed = match token.as_ref() {
                    $( $alias => $variant, )*
                    _ => {
                        let canon = $crate::__utils::Canonical::try_new(trimmed)
                            .map_err(|_| {
                                let $invalid_input = input;
                                $invalid
                            })?;
                        return Ok($OtherVariant(canon));
                    }
                };
                Ok(parsed)
            }
        }

        impl ::std::convert::TryFrom<String> for $Type {
            type Error = $Error;

            fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
                <Self as ::std::str::FromStr>::from_str(&value)
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
        $Type:ident, $enum_name:literal, other($OtherVariant:path, $OtherCode:ty), {
            $( $alias:literal => $variant:path ),+ $(,)?
        }
    ) => {
        $crate::__string_enum_base! {
            $Type, $enum_name,
            other(
                $OtherVariant,
                $OtherCode,
                error(
                    $crate::error::PaftError,
                    invalid(input) => $crate::error::PaftError::InvalidEnumValue {
                        enum_name: $enum_name,
                        value: input.to_string(),
                    }
                )
            ),
            { $( $alias => $variant ),+ }
        }
    };

    (
        $Type:ident, $enum_name:literal,
        other(
            $OtherVariant:path,
            $OtherCode:ty,
            error($Error:ty, invalid($invalid_input:ident) => $invalid:expr)
        ), {
            $( $alias:literal => $variant:path ),+ $(,)?
        }
    ) => {
        impl $crate::__utils::StringCode for $Type {
            fn code(&self) -> &str { $Type::code(self) }
            fn is_canonical(&self) -> bool { $Type::is_canonical(self) }
        }

        impl $Type {
            /// Returns true when this value represents a canonical variant.
            #[must_use]
            pub const fn is_canonical(&self) -> bool { !matches!(self, $OtherVariant(_)) }
        }

        impl $crate::__serde::Serialize for $Type {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: $crate::__serde::Serializer,
            {
                serializer.serialize_str(self.code())
            }
        }

        impl<'de> $crate::__serde::Deserialize<'de> for $Type {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: $crate::__serde::Deserializer<'de>,
            {
                let raw = <String as $crate::__serde::Deserialize>::deserialize(deserializer)?;
                <Self as ::std::str::FromStr>::from_str(&raw)
                    .map_err($crate::__serde::de::Error::custom)
            }
        }

        impl ::std::str::FromStr for $Type {
            type Err = $Error;

            fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    return Err({
                        let $invalid_input = input;
                        $invalid
                    });
                }
                let token = $crate::__utils::canonicalize(trimmed);
                let parsed = match token.as_ref() {
                    $( $alias => $variant, )*
                    _ => {
                        let canon = $crate::__utils::Canonical::try_new(trimmed)
                            .map_err(|_| {
                                let $invalid_input = input;
                                $invalid
                            })?;
                        return Ok($OtherVariant(<$OtherCode>::from_canonical_unchecked(canon)));
                    }
                };
                Ok(parsed)
            }
        }

        impl ::std::convert::TryFrom<String> for $Type {
            type Error = $Error;

            fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
                <Self as ::std::str::FromStr>::from_str(&value)
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

/// Helper for enum-specific unknown-code wrapper types.
#[doc(hidden)]
#[macro_export]
macro_rules! other_string_code_type {
    (
        $(#[$meta:meta])*
        $vis:vis struct $OtherCode:ident for $Enum:ident;
        type Error = $Error:ty;
        parse($parse_input:ident) => $parse:expr;
        invalid($invalid_input:ident) => $invalid:expr $(;)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        $vis struct $OtherCode($crate::__utils::Canonical);

        impl $OtherCode {
            /// Builds an unknown code, rejecting tokens modeled by the owning enum.
            ///
            /// # Errors
            ///
            /// Returns an error if the input is empty, cannot be canonicalized,
            /// or parses to a modeled enum variant.
            pub fn new(input: &str) -> ::std::result::Result<Self, $Error> {
                match {
                    let $parse_input = input;
                    $parse
                }? {
                    $Enum::Other(code) => Ok(code),
                    _ => Err({
                        let $invalid_input = input;
                        $invalid
                    }),
                }
            }

            /// Returns this unknown code's canonical string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_ref()
            }

            #[doc(hidden)]
            pub(crate) const fn from_canonical_unchecked(
                code: $crate::__utils::Canonical,
            ) -> Self {
                Self(code)
            }
        }

        impl ::std::convert::AsRef<str> for $OtherCode {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl ::std::str::FromStr for $OtherCode {
            type Err = $Error;

            fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
                Self::new(input)
            }
        }

        impl ::std::fmt::Display for $OtherCode {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

/// Open (extensible) string enum with macro-provided `code()` and open parsing.
#[doc(hidden)]
#[macro_export]
macro_rules! string_enum_with_code {
    (
        $Type:ident, $Other:ident($OtherCode:ty), $enum_name:literal,
        type Error = $Error:ty;
        invalid($invalid_input:ident) => $invalid:expr;
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

        impl ::std::convert::AsRef<str> for $Type {
            fn as_ref(&self) -> &str { self.code() }
        }

        $crate::__string_enum_base! {
            $Type, $enum_name,
            other(
                $Type::$Other,
                $OtherCode,
                error($Error, invalid($invalid_input) => $invalid)
            ),
            { $( $canon_token => $canon_variant ),+ $(, $alias => $variant )* }
        }
    };

    (
        $Type:ident, $Other:ident($OtherCode:ty), $enum_name:literal,
        type Error = $Error:ty;
        invalid($invalid_input:ident) => $invalid:expr;
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? }
    ) => {
        $crate::string_enum_with_code!(
            $Type, $Other($OtherCode), $enum_name,
            type Error = $Error;
            invalid($invalid_input) => $invalid;
            { $( $canon_token => $canon_variant ),+ },
            {}
        );
    };

    (
        $Type:ident, $Other:ident($OtherCode:ty), $enum_name:literal,
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? },
        { $( $alias:literal => $variant:path ),* $(,)? }
    ) => {
        $crate::string_enum_with_code! {
            $Type, $Other($OtherCode), $enum_name,
            type Error = $crate::error::PaftError;
            invalid(input) => $crate::error::PaftError::InvalidEnumValue {
                enum_name: $enum_name,
                value: input.to_string(),
            };
            { $( $canon_token => $canon_variant ),+ },
            { $( $alias => $variant ),* }
        }
    };

    (
        $Type:ident, $Other:ident($OtherCode:ty), $enum_name:literal,
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? }
    ) => {
        $crate::string_enum_with_code!(
            $Type, $Other($OtherCode), $enum_name,
            { $( $canon_token => $canon_variant ),+ },
            {}
        );
    };

    (
        $Type:ident, $Other:ident, $enum_name:literal,
        type Error = $Error:ty;
        invalid($invalid_input:ident) => $invalid:expr;
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

        impl ::std::convert::AsRef<str> for $Type {
            fn as_ref(&self) -> &str { self.code() }
        }

        $crate::__string_enum_base! {
            $Type, $enum_name,
            other($Type::$Other, error($Error, invalid($invalid_input) => $invalid)),
            { $( $canon_token => $canon_variant ),+ $(, $alias => $variant )* }
        }
    };

    (
        $Type:ident, $Other:ident, $enum_name:literal,
        type Error = $Error:ty;
        invalid($invalid_input:ident) => $invalid:expr;
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? }
    ) => {
        $crate::string_enum_with_code!(
            $Type, $Other, $enum_name,
            type Error = $Error;
            invalid($invalid_input) => $invalid;
            { $( $canon_token => $canon_variant ),+ },
            {}
        );
    };

    (
        $Type:ident, $Other:ident, $enum_name:literal,
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? },
        { $( $alias:literal => $variant:path ),* $(,)? }
    ) => {
        $crate::string_enum_with_code! {
            $Type, $Other, $enum_name,
            type Error = $crate::error::PaftError;
            invalid(input) => $crate::error::PaftError::InvalidEnumValue {
                enum_name: $enum_name,
                value: input.to_string(),
            };
            { $( $canon_token => $canon_variant ),+ },
            { $( $alias => $variant ),* }
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
        type Error = $Error:ty;
        invalid($invalid_input:ident) => $invalid:expr;
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

        impl ::std::convert::AsRef<str> for $Type {
            fn as_ref(&self) -> &str { self.code() }
        }

        $crate::__string_enum_base! {
            $Type, $enum_name,
            error($Error, invalid($invalid_input) => $invalid),
            { $( $canon_token => $canon_variant ),+ $(, $alias => $variant )* }
        }
    };

    (
        $Type:ident, $enum_name:literal,
        type Error = $Error:ty;
        invalid($invalid_input:ident) => $invalid:expr;
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? }
    ) => {
        $crate::string_enum_closed_with_code!(
            $Type, $enum_name,
            type Error = $Error;
            invalid($invalid_input) => $invalid;
            { $( $canon_token => $canon_variant ),+ },
            {}
        );
    };

    (
        $Type:ident, $enum_name:literal,
        { $( $canon_token:literal => $canon_variant:path ),+ $(,)? },
        { $( $alias:literal => $variant:path ),* $(,)? }
    ) => {
        $crate::string_enum_closed_with_code! {
            $Type, $enum_name,
            type Error = $crate::error::PaftError;
            invalid(input) => $crate::error::PaftError::InvalidEnumValue {
                enum_name: $enum_name,
                value: input.to_string(),
            };
            { $( $canon_token => $canon_variant ),+ },
            { $( $alias => $variant ),* }
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
