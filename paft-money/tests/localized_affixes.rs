#![cfg(feature = "money-formatting")]

use paft_money::{
    Currency, Locale, Money, MoneyError, clear_currency_metadata, set_currency_metadata,
};

fn register(code: &str, symbol: &str, locale: Locale, symbol_first: bool) -> Currency {
    set_currency_metadata(code, code, 2, symbol, symbol_first, locale).unwrap();
    Currency::other(code).unwrap()
}

#[test]
fn digit_bearing_symbols_round_trip_before_and_after_the_amount() {
    for (symbol_index, symbol) in ["TOK2", "2TOK", "TO2K", "🪙2", "2", "1.23"]
        .into_iter()
        .enumerate()
    {
        for (locale_index, locale) in [Locale::EnUs, Locale::EnEu, Locale::EnIn, Locale::EnBy]
            .into_iter()
            .enumerate()
        {
            for symbol_first in [true, false] {
                let code = format!("AFFIX_ROUNDTRIP_{symbol_index}_{locale_index}_{symbol_first}");
                let currency = register(&code, symbol, locale, symbol_first);
                for amount in ["0", "1.23", "123", "1234.56", "-1.23", "-1234.56"] {
                    let money = Money::from_canonical_str(amount, currency.clone()).unwrap();
                    for rendered in [
                        money.to_localized_string().unwrap(),
                        money.localized(locale).with_code().into_string().unwrap(),
                        money
                            .localized(locale)
                            .without_symbol()
                            .with_code()
                            .into_string()
                            .unwrap(),
                        money
                            .localized(locale)
                            .symbol_first(!symbol_first)
                            .into_string()
                            .unwrap(),
                    ] {
                        assert_eq!(
                            Money::from_default_locale_str(&rendered, currency.clone()).unwrap(),
                            money,
                            "{rendered}"
                        );
                        assert_eq!(
                            Money::from_str_locale(&rendered, currency.clone(), locale).unwrap(),
                            money,
                            "{rendered}"
                        );
                    }
                }
                clear_currency_metadata(&code);
            }
        }
    }
}

#[test]
fn localized_affixes_preserve_strict_numeric_validation() {
    let code = "AFFIX_VALIDATION2";
    let currency = register(code, "TOK2", Locale::EnUs, true);
    for (input, expected) in [
        ("TOK2 1.23", "1.23"),
        ("1.23 TOK2", "1.23"),
        ("-TOK2 1.23", "-1.23"),
        ("-1.23 TOK2", "-1.23"),
        (" + tok2\t1.23 ", "1.23"),
        ("1.23\u{a0}ToK2", "1.23"),
    ] {
        assert_eq!(
            Money::from_default_locale_str(input, currency.clone()).unwrap(),
            Money::from_canonical_str(expected, currency.clone()).unwrap()
        );
    }
    let money = Money::from_canonical_str("-1.23", currency.clone()).unwrap();
    assert_eq!(money.to_localized_string().unwrap(), "-TOK2 1.23");
    assert_eq!(
        money
            .localized(Locale::EnUs)
            .symbol_first(false)
            .into_string()
            .unwrap(),
        "-1.23 TOK2"
    );

    for input in [
        "TOK21.23",
        "1.23TOK2",
        "TOK2 -1.23",
        "--TOK2 1.23",
        "TOK2",
        "TOK2 1e2",
        "TOK2 1.23 TOK2 TOK2",
    ] {
        assert!(
            Money::from_default_locale_str(input, currency.clone()).is_err(),
            "{input}"
        );
    }
    for input in ["TOK2 1,23.45", "1,23.45 TOK2"] {
        assert_eq!(
            Money::from_default_locale_str(input, currency.clone()),
            Err(MoneyError::InvalidGrouping)
        );
    }
    for input in ["TOK2 1.2.3", "1.2,3 TOK2"] {
        assert_eq!(
            Money::from_default_locale_str(input, currency.clone()),
            Err(MoneyError::InvalidAmountFormat)
        );
    }
    assert_eq!(
        Money::from_default_locale_str("WRONG2 1.23", currency.clone()),
        Err(MoneyError::MismatchedCurrencyAffix)
    );
    assert_eq!(
        Money::from_default_locale_str("TOK2 1.234", currency.clone()),
        Err(MoneyError::ScaleTooLarge {
            digits: 3,
            exponent: 2
        })
    );
    assert_eq!(
        Money::from_default_locale_str("TOK2 79,228,162,514,264,337,593,543,950,336.00", currency),
        Err(MoneyError::NotRepresentable)
    );
    clear_currency_metadata(code);
}

#[test]
fn longest_matching_affix_wins_over_overlapping_symbol_or_code() {
    for (code, symbol, affix) in [
        ("AFFIX_OVERLAP2", "AFFIX_OVERLAP", "affix_overlap2"),
        ("AFFIX_OVERLAP", "AFFIX_OVERLAP2", "affix_overlap2"),
    ] {
        let currency = register(code, symbol, Locale::EnUs, true);
        let expected = Money::from_canonical_str("1.23", currency.clone()).unwrap();
        for input in [format!("{affix} 1.23"), format!("1.23 {affix}")] {
            assert_eq!(
                Money::from_default_locale_str(&input, currency.clone()).unwrap(),
                expected
            );
        }
        clear_currency_metadata(code);
    }
}

#[test]
fn numeric_symbols_do_not_consume_digits_or_single_space_groups() {
    let code = "AFFIX_NUMERIC_SYMBOL";
    let currency = register(code, "2", Locale::EnBy, true);
    let money = Money::from_canonical_str("123", currency.clone()).unwrap();
    assert_eq!(money.to_localized_string().unwrap(), "2  123,00");
    assert_eq!(
        money
            .localized(Locale::EnBy)
            .symbol_first(false)
            .into_string()
            .unwrap(),
        "123,00  2"
    );
    assert_eq!(
        Money::from_default_locale_str("2  123,00", currency.clone()).unwrap(),
        money
    );
    assert_eq!(
        Money::from_default_locale_str("123,00  2", currency.clone()).unwrap(),
        money
    );
    for (input, expected) in [
        ("2123,00", "2123"),
        ("2 123,00", "2123"),
        ("123,22", "123.22"),
    ] {
        assert_eq!(
            Money::from_default_locale_str(input, currency.clone()).unwrap(),
            Money::from_canonical_str(expected, currency.clone()).unwrap()
        );
    }
    assert_eq!(
        money.localized(Locale::EnUs).into_string().unwrap(),
        "2 123.00"
    );
    clear_currency_metadata(code);
}

#[test]
fn conflicting_numeric_affix_placements_are_rejected() {
    let currency = register("3141", "2718", Locale::EnUs, true);
    for input in ["3141 2718", "2718 3141"] {
        assert_eq!(
            Money::from_default_locale_str(input, currency.clone()),
            Err(MoneyError::InvalidAmountFormat)
        );
    }
    clear_currency_metadata("3141");
}
