#![cfg(feature = "money-formatting")]

use iso_currency::Currency as IsoCurrency;
use paft_money::{Currency, Locale, Money, MoneyError, clear_currency_metadata, currency_metadata};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

const fn eur() -> Currency {
    Currency::Iso(IsoCurrency::EUR)
}

const fn inr() -> Currency {
    Currency::Iso(IsoCurrency::INR)
}

const fn byn() -> Currency {
    Currency::Iso(IsoCurrency::BYN)
}

fn register_metadata(
    code: &str,
    name: &str,
    units: u8,
    symbol: &str,
    symbol_first: bool,
    locale: Locale,
) {
    paft_money::set_currency_metadata(
        code,
        name.to_string(),
        units,
        symbol.to_string(),
        symbol_first,
        locale,
    )
    .unwrap();
}

#[test]
fn us_locale_formatting() {
    let zero = Money::from_canonical_str("0", usd()).unwrap();
    assert_eq!(zero.to_localized_string().unwrap(), "$0.00");
    assert_eq!(zero.format(), "0 USD");

    let large = Money::from_canonical_str("100000", usd()).unwrap();
    assert_eq!(large.to_localized_string().unwrap(), "$100,000.00");
    assert_eq!(large.format(), "100000 USD");

    let negative = Money::from_canonical_str("-100000", usd()).unwrap();
    assert_eq!(negative.to_localized_string().unwrap(), "-$100,000.00");

    let huge = Money::from_canonical_str("1000000000", usd()).unwrap();
    assert_eq!(huge.to_localized_string().unwrap(), "$1,000,000,000.00");
}

#[test]
fn eu_locale_formatting() {
    let eur_money = Money::from_canonical_str("1000", eur()).unwrap();
    assert_eq!(eur_money.to_localized_string().unwrap(), "€1.000,00");
}

#[test]
fn india_locale_formatting() {
    let positive = Money::from_canonical_str("100000", inr()).unwrap();
    assert_eq!(positive.to_localized_string().unwrap(), "₹1,00,000.00");

    let negative = Money::from_canonical_str("-10000000", inr()).unwrap();
    assert_eq!(negative.to_localized_string().unwrap(), "-₹1,00,00,000.00");
}

#[test]
fn belarus_locale_formatting() {
    let value = Money::from_canonical_str("1234.56", byn()).unwrap();
    assert_eq!(value.to_localized_string().unwrap(), "1 234,56 Br");
}

#[test]
fn symbol_positioning_respects_metadata() {
    let usd_value = Money::from_canonical_str("1234.56", usd()).unwrap();
    assert_eq!(usd_value.to_localized_string().unwrap(), "$1,234.56");
    assert_eq!(
        usd_value
            .localized(Locale::EnUs)
            .with_code()
            .into_string()
            .unwrap(),
        "$1,234.56 USD"
    );

    let aed_value = Money::from_canonical_str("1234.56", Currency::Iso(IsoCurrency::AED)).unwrap();
    assert_eq!(aed_value.to_localized_string().unwrap(), "1,234.56 د.إ");
    assert_eq!(
        aed_value
            .localized(Locale::EnUs)
            .with_code()
            .into_string()
            .unwrap(),
        "1,234.56 د.إ AED"
    );
    assert_eq!(
        aed_value
            .localized(Locale::EnUs)
            .symbol_first(true)
            .into_string()
            .unwrap(),
        "د.إ 1,234.56"
    );
}

#[test]
fn exponent_specific_formatting() {
    let jpy_value = Money::from_canonical_str("1234.5", Currency::Iso(IsoCurrency::JPY)).unwrap();
    assert_eq!(jpy_value.to_localized_string().unwrap(), "¥1,235");

    let bhd_value =
        Money::from_canonical_str("1234.5674", Currency::Iso(IsoCurrency::BHD)).unwrap();
    assert_eq!(bhd_value.to_localized_string().unwrap(), "BD 1,234.567");
}

#[test]
fn crypto_precision_formatting() {
    let btc_value = Money::from_canonical_str("0.12345678", Currency::BTC).unwrap();
    assert_eq!(btc_value.to_localized_string().unwrap(), "₿0.12345678");

    let eth_value = Money::from_canonical_str("0.123456789012345678", Currency::ETH).unwrap();
    assert_eq!(
        eth_value.to_localized_string().unwrap(),
        "Ξ0.123456789012345678"
    );
}

#[test]
fn amount_string_supports_custom_digits() {
    let usd_value = Money::from_canonical_str("1234.56789", usd()).unwrap();
    assert_eq!(
        usd_value
            .amount_string_with_locale(Locale::EnEu, 4)
            .unwrap(),
        "1.234,5700"
    );
}

#[test]
fn localized_parsing_accepts_common_patterns() {
    let cases = [
        ("$100,000.00", "100000"),
        ("-$100,000.00", "-100000"),
        ("$ 100,000.00", "100000"),
        ("USD 100,000", "100000"),
        ("100,000 USD", "100000"),
        ("100000", "100000"),
    ];

    for (input, canonical) in cases {
        let parsed = Money::from_default_locale_str(input, usd()).unwrap();
        let expected = Money::from_canonical_str(canonical, usd()).unwrap();
        assert_eq!(parsed, expected);
    }

    let eu = Money::from_default_locale_str("€1.000,00", eur()).unwrap();
    assert_eq!(eu.amount().to_string(), "1000.00");

    let india = Money::from_default_locale_str("₹1,00,000.00", inr()).unwrap();
    assert_eq!(india.amount().to_string(), "100000.00");

    let byn = Money::from_default_locale_str("1 234,56 Br", byn()).unwrap();
    assert_eq!(byn.amount().to_string(), "1234.56");
}

#[test]
fn localized_parsing_rejects_invalid_inputs() {
    let usd_currency = usd();
    assert!(matches!(
        Money::from_default_locale_str("1,00.00", usd_currency.clone()).unwrap_err(),
        MoneyError::InvalidGrouping
    ));
    assert!(matches!(
        Money::from_default_locale_str("£1,000.00", usd_currency).unwrap_err(),
        MoneyError::MismatchedCurrencyAffix
    ));

    assert!(matches!(
        Money::from_default_locale_str("1.00,00", eur()).unwrap_err(),
        MoneyError::InvalidGrouping
    ));

    assert!(matches!(
        Money::from_default_locale_str("1.000.000.00", inr()).unwrap_err(),
        MoneyError::InvalidAmountFormat
    ));

    assert!(matches!(
        Money::from_default_locale_str("1.234", usd()).unwrap_err(),
        MoneyError::ScaleTooLarge {
            digits: 3,
            exponent: 2
        }
    ));
}

#[test]
fn localized_parsing_handles_signs() {
    let positive = Money::from_default_locale_str("+100,000.00", usd()).unwrap();
    let expected_positive = Money::from_canonical_str("100000", usd()).unwrap();
    assert_eq!(positive, expected_positive);

    let negative = Money::from_default_locale_str("- USD 100,000", usd()).unwrap();
    let expected_negative = Money::from_canonical_str("-100000", usd()).unwrap();
    assert_eq!(negative, expected_negative);
}

#[test]
fn localized_roundtrip_matches_original() {
    let currencies = [usd(), eur(), inr(), byn()];
    let amounts = ["1234.56", "-7890.12"];

    for currency in currencies {
        for amount in amounts {
            let money = Money::from_canonical_str(amount, currency.clone()).unwrap();
            let rendered = money.to_localized_string().unwrap();
            let parsed = Money::from_default_locale_str(&rendered, currency.clone()).unwrap();
            assert_eq!(parsed, money);
        }
    }
}

#[test]
fn custom_metadata_drives_formatting() {
    let code = "CSTM";
    clear_currency_metadata(code);
    register_metadata(code, "Custom", 2, "¤", false, Locale::EnEu);

    let money = Money::from_canonical_str("1234.5", Currency::try_from_str(code).unwrap()).unwrap();
    assert_eq!(money.to_localized_string().unwrap(), "1.234,50¤");
    assert_eq!(
        money
            .localized(Locale::EnUs)
            .with_code()
            .into_string()
            .unwrap(),
        "1,234.50¤ CSTM"
    );

    clear_currency_metadata(code);
}

#[test]
fn metadata_registration_is_thread_safe() {
    use std::thread;

    let handles: Vec<_> = (0..16)
        .map(|i| format!("THR{i}"))
        .map(|code| {
            thread::spawn(move || {
                register_metadata(&code, "Threaded", 2, &code, true, Locale::EnUs);
                let metadata = currency_metadata(&code).expect("metadata present");
                assert_eq!(metadata.minor_units, 2);
                clear_currency_metadata(&code);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
