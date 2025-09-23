//! Tests covering canonical/alias behavior for currencies.

use iso_currency::Currency as IsoCurrency;
use paft_core::domain::Currency;
use std::str::FromStr;

struct Case {
    variant: Currency,
    canonical: &'static str,
    full_name: &'static str,
    aliases: &'static [&'static str],
}

#[test]
#[allow(clippy::too_many_lines)]
fn currency_round_trips_display_fromstr_and_serde() {
    for case in cases() {
        assert_round_trip(&case);
    }
}

#[test]
fn currency_other_values_uppercase_and_round_trip() {
    let parsed = Currency::try_from_str("usd-lite").unwrap();
    assert_eq!(parsed.to_string(), "USD_LITE");

    let json = serde_json::to_string(&parsed).unwrap();
    assert_eq!(json, "\"USD_LITE\"");
    let back: Currency = serde_json::from_str(&json).unwrap();
    assert_eq!(back, parsed);
}

#[test]
fn currency_try_from_and_serde_reject_empty() {
    let err = Currency::try_from_str("").unwrap_err();
    match err {
        paft_core::error::PaftError::InvalidEnumValue { enum_name, value } => {
            assert_eq!(enum_name, "Currency");
            assert_eq!(value, "");
        }
        other => panic!("unexpected error: {other}"),
    }

    let result: Result<Currency, _> = serde_json::from_str("\"\"");
    assert!(result.is_err());
}

#[test]
fn currency_decimal_place_expectations() {
    assert_eq!(Currency::Iso(IsoCurrency::JPY).decimal_places(), 0);
    assert_eq!(Currency::Iso(IsoCurrency::KRW).decimal_places(), 0);
    assert_eq!(Currency::BTC.decimal_places(), 8);
    assert_eq!(Currency::ETH.decimal_places(), 18);
    assert_eq!(Currency::XMR.decimal_places(), 12);
}

#[test]
fn currency_reserve_currency_helper() {
    assert!(Currency::Iso(IsoCurrency::USD).is_reserve_currency());
    assert!(Currency::Iso(IsoCurrency::EUR).is_reserve_currency());
    assert!(Currency::Iso(IsoCurrency::GBP).is_reserve_currency());
    assert!(Currency::Iso(IsoCurrency::JPY).is_reserve_currency());
    assert!(Currency::Iso(IsoCurrency::CHF).is_reserve_currency());
    assert!(!Currency::Iso(IsoCurrency::CAD).is_reserve_currency());
}

#[allow(clippy::too_many_lines)]
fn cases() -> Vec<Case> {
    vec![
        Case {
            variant: Currency::Iso(IsoCurrency::USD),
            canonical: "USD",
            full_name: IsoCurrency::USD.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::EUR),
            canonical: "EUR",
            full_name: IsoCurrency::EUR.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::GBP),
            canonical: "GBP",
            full_name: IsoCurrency::GBP.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::JPY),
            canonical: "JPY",
            full_name: IsoCurrency::JPY.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::CAD),
            canonical: "CAD",
            full_name: IsoCurrency::CAD.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::AUD),
            canonical: "AUD",
            full_name: IsoCurrency::AUD.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::CHF),
            canonical: "CHF",
            full_name: IsoCurrency::CHF.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::CNY),
            canonical: "CNY",
            full_name: IsoCurrency::CNY.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::HKD),
            canonical: "HKD",
            full_name: IsoCurrency::HKD.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::SGD),
            canonical: "SGD",
            full_name: IsoCurrency::SGD.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::INR),
            canonical: "INR",
            full_name: IsoCurrency::INR.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::BRL),
            canonical: "BRL",
            full_name: IsoCurrency::BRL.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::MXN),
            canonical: "MXN",
            full_name: IsoCurrency::MXN.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::KRW),
            canonical: "KRW",
            full_name: IsoCurrency::KRW.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::NZD),
            canonical: "NZD",
            full_name: IsoCurrency::NZD.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::NOK),
            canonical: "NOK",
            full_name: IsoCurrency::NOK.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::SEK),
            canonical: "SEK",
            full_name: IsoCurrency::SEK.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::DKK),
            canonical: "DKK",
            full_name: IsoCurrency::DKK.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::PLN),
            canonical: "PLN",
            full_name: IsoCurrency::PLN.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::CZK),
            canonical: "CZK",
            full_name: IsoCurrency::CZK.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::HUF),
            canonical: "HUF",
            full_name: IsoCurrency::HUF.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::RUB),
            canonical: "RUB",
            full_name: IsoCurrency::RUB.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::TRY),
            canonical: "TRY",
            full_name: IsoCurrency::TRY.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::ZAR),
            canonical: "ZAR",
            full_name: IsoCurrency::ZAR.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::ILS),
            canonical: "ILS",
            full_name: IsoCurrency::ILS.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::THB),
            canonical: "THB",
            full_name: IsoCurrency::THB.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::MYR),
            canonical: "MYR",
            full_name: IsoCurrency::MYR.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::PHP),
            canonical: "PHP",
            full_name: IsoCurrency::PHP.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::IDR),
            canonical: "IDR",
            full_name: IsoCurrency::IDR.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::Iso(IsoCurrency::VND),
            canonical: "VND",
            full_name: IsoCurrency::VND.name(),
            aliases: &[],
        },
        Case {
            variant: Currency::BTC,
            canonical: "BTC",
            full_name: "Bitcoin",
            aliases: &[],
        },
        Case {
            variant: Currency::ETH,
            canonical: "ETH",
            full_name: "Ethereum",
            aliases: &[],
        },
        Case {
            variant: Currency::XMR,
            canonical: "XMR",
            full_name: "Monero",
            aliases: &[],
        },
    ]
}

fn assert_round_trip(case: &Case) {
    let display = case.variant.to_string();
    assert_eq!(display, case.canonical);
    assert_eq!(case.variant.code(), case.canonical);
    assert_eq!(case.variant.full_name(), case.full_name);

    let parsed = Currency::from_str(case.canonical).unwrap();
    assert_eq!(parsed, case.variant);

    let json = serde_json::to_string(&case.variant).unwrap();
    assert_eq!(json, format!("\"{}\"", case.canonical));
    let back: Currency = serde_json::from_str(&json).unwrap();
    assert_eq!(back, case.variant);

    for alias in case.aliases {
        let parsed_alias = Currency::from_str(alias).unwrap();
        assert_eq!(parsed_alias, case.variant);

        let redisplay = parsed_alias.to_string();
        assert_eq!(redisplay, case.canonical);

        let reparsed = Currency::from_str(parsed_alias.code()).unwrap();
        assert_eq!(reparsed, case.variant);

        let alias_json = format!("\"{alias}\"");
        let alias_back: Currency = serde_json::from_str(&alias_json).unwrap();
        assert_eq!(alias_back, case.variant);
    }
}
