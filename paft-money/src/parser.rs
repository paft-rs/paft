use std::borrow::Cow;

use crate::currency::Currency;
use crate::decimal::{self, Decimal};
use crate::error::MoneyError;
use crate::locale::{LocalFormat, Locale};

/// Parses a human-formatted money string using locale information.
#[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", err))]
pub fn parse_localized_str(
    input: &str,
    currency: &Currency,
    locale_override: Option<Locale>,
    strict: bool,
) -> Result<Decimal, MoneyError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(MoneyError::InvalidAmountFormat);
    }

    let mut rest = trimmed;
    let mut negative = false;
    if let Some(stripped) = rest.strip_prefix('-') {
        negative = true;
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix('+') {
        rest = stripped;
    }

    rest = rest.trim_start();

    let first_digit = rest
        .find(|c: char| c.is_ascii_digit())
        .ok_or(MoneyError::InvalidAmountFormat)?;
    let last_digit = rest
        .rfind(|c: char| c.is_ascii_digit())
        .ok_or(MoneyError::InvalidAmountFormat)?;

    let prefix = rest[..first_digit].trim();
    let suffix = rest[last_digit + 1..].trim();
    let amount_slice = &rest[first_digit..=last_digit];

    let symbol = currency
        .symbol()
        .unwrap_or_else(|| Cow::Borrowed(currency.code()));
    let symbol_str = symbol.as_ref();
    let currency_code = currency.code();

    if !prefix.is_empty() {
        match_affix(prefix, Some(symbol_str), currency_code)?;
    }

    if !suffix.is_empty() {
        match_affix(suffix, Some(symbol_str), currency_code)?;
    }

    let locale = locale_override.unwrap_or_else(|| currency.default_locale());
    let spec = locale.spec();

    let mut decimal_count = 0_usize;
    for ch in amount_slice.chars() {
        if ch.is_ascii_digit() {
            continue;
        }
        if ch == spec.group_separator {
            continue;
        }
        if ch == spec.decimal_separator {
            decimal_count += 1;
            if decimal_count > 1 {
                return Err(MoneyError::InvalidAmountFormat);
            }
            continue;
        }
        return Err(MoneyError::InvalidAmountFormat);
    }

    let (integer_part, fraction_part) = split_parts(amount_slice, spec.decimal_separator);
    if decimal_count > 0 && fraction_part.is_empty() {
        return Err(MoneyError::InvalidAmountFormat);
    }

    if strict {
        validate_grouping(integer_part, &spec)?;
    } else if integer_part.contains(spec.group_separator)
        && validate_grouping(integer_part, &spec).is_err()
    {
        return Err(MoneyError::InvalidGrouping);
    }

    if fraction_part.contains(spec.group_separator) {
        return Err(MoneyError::InvalidAmountFormat);
    }
    if fraction_part.chars().any(|c| !c.is_ascii_digit()) {
        return Err(MoneyError::InvalidAmountFormat);
    }

    let integer_digits: String = integer_part
        .chars()
        .filter(|c| *c != spec.group_separator)
        .collect();
    let fraction_digits: String = fraction_part.to_string();

    let is_zero_value =
        integer_digits.chars().all(|c| c == '0') && fraction_digits.chars().all(|c| c == '0');

    let exponent = currency.decimal_places()?;
    if fraction_digits.len() > usize::from(exponent) {
        return Err(MoneyError::ScaleTooLarge {
            digits: fraction_digits.len(),
            exponent,
        });
    }

    let mut canonical = if integer_digits.is_empty() {
        "0".to_string()
    } else {
        integer_digits
    };
    if !fraction_digits.is_empty() {
        canonical.push('.');
        canonical.push_str(&fraction_digits);
    }

    if is_zero_value {
        negative = false;
    }

    if negative {
        canonical.insert(0, '-');
    }

    decimal::parse_decimal(&canonical).ok_or(MoneyError::InvalidAmountFormat)
}

fn match_affix(token: &str, symbol: Option<&str>, code: &str) -> Result<(), MoneyError> {
    if symbol.is_some_and(|sym| token.eq_ignore_ascii_case(sym)) {
        return Ok(());
    }
    if token.eq_ignore_ascii_case(code) {
        return Ok(());
    }
    Err(MoneyError::MismatchedCurrencyAffix)
}

fn split_parts(core: &str, decimal_separator: char) -> (&str, &str) {
    core.rfind(decimal_separator).map_or((core, ""), |idx| {
        (&core[..idx], &core[idx + decimal_separator.len_utf8()..])
    })
}

fn validate_grouping(int_part: &str, spec: &LocalFormat) -> Result<(), MoneyError> {
    if !int_part.contains(spec.group_separator) {
        if int_part.chars().all(|c| c.is_ascii_digit()) {
            return Ok(());
        }
        return Err(MoneyError::InvalidAmountFormat);
    }

    let mut segments = Vec::new();
    let mut current = String::new();
    for ch in int_part.chars() {
        if ch == spec.group_separator {
            if current.is_empty() {
                return Err(MoneyError::InvalidGrouping);
            }
            segments.push(current.clone());
            current.clear();
        } else if ch.is_ascii_digit() {
            current.push(ch);
        } else {
            return Err(MoneyError::InvalidAmountFormat);
        }
    }
    if current.is_empty() {
        return Err(MoneyError::InvalidGrouping);
    }
    segments.push(current);

    let repeat = spec.grouping.last().copied().unwrap_or(3);
    let total = segments.len();

    for (idx, segment) in segments.iter().rev().enumerate() {
        let expected = spec.grouping.get(idx).copied().unwrap_or(repeat);
        if idx == total - 1 {
            if segment.is_empty() || segment.len() > expected {
                return Err(MoneyError::InvalidGrouping);
            }
        } else if segment.len() != expected {
            return Err(MoneyError::InvalidGrouping);
        }
    }

    Ok(())
}
