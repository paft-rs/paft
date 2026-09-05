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

    let symbol = currency
        .symbol()
        .unwrap_or_else(|| Cow::Borrowed(currency.code()));
    let locale = locale_override.unwrap_or_else(|| currency.default_locale());
    let spec = locale.spec();
    let amount_slice = strip_currency_affixes(rest, symbol.as_ref(), currency.code(), &spec)?;
    if !amount_slice.bytes().any(|byte| byte.is_ascii_digit()) {
        return Err(MoneyError::InvalidAmountFormat);
    }
    if !amount_slice.starts_with(|c: char| c.is_ascii_digit())
        || !amount_slice.ends_with(|c: char| c.is_ascii_digit())
    {
        return Err(MoneyError::MismatchedCurrencyAffix);
    }

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

    decimal::parse_decimal(&canonical).map_err(localized_decimal_error)
}

const fn localized_decimal_error(error: decimal::DecimalParseError) -> MoneyError {
    match error {
        decimal::DecimalParseError::NotRepresentable => MoneyError::NotRepresentable,
        _ => MoneyError::InvalidAmountFormat,
    }
}

/// Minimum whitespace separating an affix from the amount or another affix.
/// Shared with the formatter so numeric symbols cannot merge into the amount.
pub fn minimum_affix_spacing(affix: &str, spec: &LocalFormat) -> usize {
    if !affix.bytes().any(|byte| byte.is_ascii_digit()) {
        return 0;
    }
    // In a space-grouping locale, one space could be part of a bare number.
    // Two spaces distinguish a numeric-looking affix from a digit group.
    if spec.group_separator.is_whitespace()
        && affix.chars().all(|ch| {
            ch.is_ascii_digit() || ch == spec.group_separator || ch == spec.decimal_separator
        })
    {
        2
    } else {
        1
    }
}

fn strip_currency_affixes<'a>(
    input: &'a str,
    symbol: &str,
    code: &str,
    spec: &LocalFormat,
) -> Result<&'a str, MoneyError> {
    let symbol = symbol.trim();
    let strip_prefix = |input: &'a str| {
        [symbol, code]
            .into_iter()
            .filter_map(|affix| {
                strip_prefix_affix(input, affix, spec).map(|rest| (affix.len(), rest))
            })
            .max_by_key(|(len, _)| *len)
            .map_or(input, |(_, rest)| rest)
    };
    let prefix_first = strip_currency_suffixes(strip_prefix(input), symbol, code, spec);
    let suffix_first = strip_prefix(strip_currency_suffixes(input, symbol, code, spec));

    // A numeric symbol can match the amount itself (for example "1.23 CODE"
    // with symbol "1.23"). Keep the interpretation that leaves a number, and
    // reject conflicting numeric interpretations instead of choosing an amount.
    let looks_numeric = |value: &str| {
        value.starts_with(|ch: char| ch.is_ascii_digit())
            && value.ends_with(|ch: char| ch.is_ascii_digit())
            && value.chars().all(|ch| {
                ch.is_ascii_digit() || ch == spec.group_separator || ch == spec.decimal_separator
            })
    };
    match (looks_numeric(prefix_first), looks_numeric(suffix_first)) {
        (true, true) if prefix_first != suffix_first => Err(MoneyError::InvalidAmountFormat),
        (false, true) => Ok(suffix_first),
        _ => Ok(prefix_first),
    }
}

fn strip_currency_suffixes<'a>(
    input: &'a str,
    symbol: &str,
    code: &str,
    spec: &LocalFormat,
) -> &'a str {
    let Some((affix, rest)) = [symbol, code]
        .into_iter()
        .filter_map(|affix| strip_suffix_affix(input, affix, spec).map(|rest| (affix, rest)))
        .max_by_key(|(affix, _)| affix.len())
    else {
        return input;
    };

    // `LocalizedMoney::with_code` can place both symbol and code after the
    // amount. Only this ordered pair permits a second suffix.
    if affix.eq_ignore_ascii_case(code) && !symbol.eq_ignore_ascii_case(code) {
        strip_suffix_affix(rest, symbol, spec).unwrap_or(rest)
    } else {
        rest
    }
}

fn strip_prefix_affix<'a>(input: &'a str, affix: &str, spec: &LocalFormat) -> Option<&'a str> {
    if affix.is_empty() || !input.get(..affix.len())?.eq_ignore_ascii_case(affix) {
        return None;
    }
    let rest = &input[affix.len()..];
    let spaces = rest.chars().take_while(|ch| ch.is_whitespace()).count();
    (spaces >= minimum_affix_spacing(affix, spec)).then(|| rest.trim_start())
}

fn strip_suffix_affix<'a>(input: &'a str, affix: &str, spec: &LocalFormat) -> Option<&'a str> {
    let start = input.len().checked_sub(affix.len())?;
    if affix.is_empty() || !input.get(start..)?.eq_ignore_ascii_case(affix) {
        return None;
    }
    let rest = &input[..start];
    let spaces = rest
        .chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count();
    (spaces >= minimum_affix_spacing(affix, spec)).then(|| rest.trim_end())
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

    let repeat = spec.grouping.last().map_or(3, |size| size.get());
    let total = segments.len();

    for (idx, segment) in segments.iter().rev().enumerate() {
        let expected = spec.grouping.get(idx).map_or(repeat, |size| size.get());
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
