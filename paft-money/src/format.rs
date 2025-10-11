use std::borrow::Cow;

use crate::decimal::{self, Decimal, RoundingStrategy};
use crate::error::MoneyError;
use crate::locale::{LocalFormat, Locale};

/// Elements that can be positioned when rendering a formatted string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatItem {
    /// Negative sign for negative amounts (omitted when positive).
    Sign,
    /// Currency symbol (if available).
    Symbol,
    /// Formatted numeric amount.
    Amount,
    /// Currency code (ISO or custom).
    Code,
    /// Literal space for flexible separation.
    Space,
}

/// Parameters controlling how a money value is formatted.
#[derive(Debug, Clone)]
pub struct Params<'a> {
    /// Ordered list describing how pieces should be rendered.
    pub positions: Vec<FormatItem>,
    /// Desired number of fractional digits (pad with zeros when necessary).
    pub rounding_digits: Option<u32>,
    /// Optional currency symbol to display.
    pub symbol: Option<Cow<'a, str>>,
    /// Optional currency code to display.
    pub code: Option<Cow<'a, str>>,
}

#[allow(clippy::elidable_lifetime_names)]
impl<'a> Params<'a> {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(positions: Vec<FormatItem>) -> Self {
        Self {
            positions,
            rounding_digits: None,
            symbol: None,
            code: None,
        }
    }
}

/// Formatting engine for locale-aware rendering.
pub struct Formatter<'a> {
    value: Decimal,
    format: LocalFormat,
    params: Params<'a>,
}

impl<'a> Formatter<'a> {
    /// Creates a new formatter for the provided Decimal value.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(value: Decimal, locale: Locale, params: Params<'a>) -> Self {
        Self {
            value,
            format: locale.spec(),
            params,
        }
    }

    /// Emits the formatted string according to the locale and params.
    pub fn format(mut self) -> Result<String, MoneyError> {
        // 1) Round once for display if requested
        if let Some(scale) = self.params.rounding_digits {
            self.value = decimal::round_dp_with_strategy(
                &self.value,
                scale,
                RoundingStrategy::MidpointNearestEven,
            );
        }

        // 2) Build canonical string, then pad fraction to requested digits
        let mut canonical = decimal::to_canonical_string(&self.value);

        // detect sign on the rounded value
        let mut negative = canonical.starts_with('-');
        if negative {
            canonical.remove(0);
        }

        // treat -0 as +0
        if self.value == decimal::zero() {
            negative = false;
        }

        if canonical.is_empty() {
            canonical.push('0');
        }
        let (mut integer, mut fraction) = match canonical.split_once('.') {
            Some((i, f)) => (i.to_string(), f.to_string()),
            None => (canonical, String::new()),
        };

        if integer.is_empty() {
            integer.push('0');
        }

        if let Some(scale) = self.params.rounding_digits {
            let target = scale as usize;
            if fraction.len() > target {
                return Err(MoneyError::InvalidAmountFormat);
            }
            while fraction.len() < target {
                fraction.push('0');
            }
            if target == 0 {
                fraction.clear();
            }
        }

        let grouped = apply_grouping(&integer, &self.format.grouping, self.format.group_separator);
        let mut amount = grouped;
        if !fraction.is_empty() {
            amount.push(self.format.decimal_separator);
            amount.push_str(&fraction);
        }

        let symbol = self.params.symbol.as_ref().map_or("", Cow::as_ref);
        let code = self.params.code.as_ref().map_or("", Cow::as_ref);

        let mut out = String::new();
        for item in &self.params.positions {
            match item {
                FormatItem::Sign => {
                    if negative {
                        out.push('-');
                    }
                }
                FormatItem::Symbol => {
                    if !symbol.is_empty() {
                        out.push_str(symbol);
                    }
                }
                FormatItem::Amount => out.push_str(&amount),
                FormatItem::Code => {
                    if !code.is_empty() {
                        out.push_str(code);
                    }
                }
                FormatItem::Space => out.push(' '),
            }
        }
        Ok(out)
    }
}

fn apply_grouping(value: &str, grouping: &[usize], separator: char) -> String {
    if grouping.is_empty() {
        return value.to_string();
    }

    let mut chunks = Vec::new();
    let mut remaining = value.len();
    let mut index = 0_usize;
    let repeat = *grouping.last().unwrap_or(&3);

    while remaining > 0 {
        let size = if index < grouping.len() {
            grouping[index]
        } else {
            repeat
        };

        let start = remaining.saturating_sub(size);
        chunks.push(&value[start..remaining]);
        if start == 0 {
            break;
        }
        remaining = start;
        index += 1;
    }

    chunks.reverse();

    let mut output = String::new();
    for (i, chunk) in chunks.iter().enumerate() {
        if i > 0 {
            output.push(separator);
        }
        output.push_str(chunk);
    }

    output
}
