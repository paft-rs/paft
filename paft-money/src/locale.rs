//! Locale definitions for money formatting and parsing.
//!
//! Grouping patterns are applied from the rightmost digit moving left. For
//! example, the Indian pattern `[3, 2, 2]` renders `12345678` as
//! `1,23,45,678`.
/// Supported locales for money formatting/parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    /// English (United States): grouping 3-3-3, `,` thousands, `.` decimal.
    EnUs,
    /// English (India): grouping 3-2-2, `,` thousands, `.` decimal.
    EnIn,
    /// English (Europe): grouping 3-3-3, `.` thousands, `,` decimal.
    EnEu,
    /// English (Belarus): grouping 3-3-3, space thousands, `,` decimal.
    EnBy,
}

/// Concrete formatting specification for a locale.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "money-formatting")]
pub struct LocalFormat {
    /// Character inserted between digit groups in the integer part.
    pub group_separator: char,
    /// Character separating integer and fractional parts.
    pub decimal_separator: char,
    /// Grouping pattern expressed as chunk sizes from right to left.
    pub grouping: Vec<usize>,
}

#[cfg(feature = "money-formatting")]
impl Locale {
    /// Maps a locale to its formatting implementation details.
    pub(crate) fn spec(self) -> LocalFormat {
        match self {
            Self::EnUs => LocalFormat {
                group_separator: ',',
                decimal_separator: '.',
                grouping: vec![3, 3, 3],
            },
            Self::EnIn => LocalFormat {
                group_separator: ',',
                decimal_separator: '.',
                grouping: vec![3, 2, 2],
            },
            Self::EnEu => LocalFormat {
                group_separator: '.',
                decimal_separator: ',',
                grouping: vec![3, 3, 3],
            },
            Self::EnBy => LocalFormat {
                group_separator: ' ',
                decimal_separator: ',',
                grouping: vec![3, 3, 3],
            },
        }
    }
}
