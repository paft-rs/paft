use std::borrow::Cow;

use crate::decimal::{self, RoundingStrategy};
use crate::locale::{LocalFormat, Locale};
use crate::money::MoneyError;

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
    value: Cow<'a, str>,
    format: LocalFormat,
    params: Params<'a>,
}

impl<'a> Formatter<'a> {
    /// Creates a new formatter for the provided canonical string.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(value: &'a str, locale: Locale, params: Params<'a>) -> Self {
        Self {
            value: Cow::Borrowed(value),
            format: locale.spec(),
            params,
        }
    }

    /// Emits the formatted string according to the locale and params.
    pub fn format(mut self) -> Result<String, MoneyError> {
        if let Some(scale) = self.params.rounding_digits {
            let parsed = decimal::parse_decimal(self.value.as_ref())
                .ok_or(MoneyError::InvalidAmountFormat)?;
            let rounded = decimal::round_dp_with_strategy(
                &parsed,
                scale,
                RoundingStrategy::MidpointNearestEven,
            );
            self.value = Cow::Owned(decimal::to_canonical_string(&rounded));
        }

        let mut raw = self.value.as_ref();
        let mut negative = false;
        if let Some(stripped) = raw.strip_prefix('-') {
            negative = true;
            raw = stripped;
        }

        if negative && is_zero(raw) {
            negative = false;
        }

        if raw.is_empty() {
            raw = "0";
        }

        let (mut integer, mut fraction) = match raw.split_once('.') {
            Some((int, frac)) => (int.to_string(), frac.to_string()),
            None => (raw.to_string(), String::new()),
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

        let mut output = String::new();
        for item in &self.params.positions {
            match item {
                FormatItem::Sign => {
                    if negative {
                        output.push('-');
                    }
                }
                FormatItem::Symbol => {
                    if !symbol.is_empty() {
                        output.push_str(symbol);
                    }
                }
                FormatItem::Amount => output.push_str(&amount),
                FormatItem::Code => {
                    if !code.is_empty() {
                        output.push_str(code);
                    }
                }
                FormatItem::Space => output.push(' '),
            }
        }

        Ok(output)
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

fn is_zero(raw: &str) -> bool {
    raw.chars().all(|c| c == '0' || c == '.')
}
