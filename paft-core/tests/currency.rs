//! Tests covering canonical/alias behavior for currencies.

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
    assert_eq!(String::from(parsed.clone()), "USD_LITE");

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
    assert_eq!(Currency::JPY.decimal_places(), 0);
    assert_eq!(Currency::KRW.decimal_places(), 0);
    assert_eq!(Currency::BTC.decimal_places(), 8);
    assert_eq!(Currency::ETH.decimal_places(), 18);
    assert_eq!(Currency::XMR.decimal_places(), 12);
}

#[test]
fn currency_reserve_currency_helper() {
    assert!(Currency::USD.is_reserve_currency());
    assert!(Currency::EUR.is_reserve_currency());
    assert!(Currency::GBP.is_reserve_currency());
    assert!(Currency::JPY.is_reserve_currency());
    assert!(Currency::CHF.is_reserve_currency());
    assert!(!Currency::CAD.is_reserve_currency());
}

#[allow(clippy::too_many_lines)]
fn cases() -> Vec<Case> {
    use Currency::*;

    vec![
        Case {
            variant: USD,
            canonical: "USD",
            full_name: "US Dollar",
            aliases: &["US_DOLLAR", "US DOLLAR", "USDOLLAR", "DOLLAR"],
        },
        Case {
            variant: EUR,
            canonical: "EUR",
            full_name: "Euro",
            aliases: &["EURO"],
        },
        Case {
            variant: GBP,
            canonical: "GBP",
            full_name: "Pound Sterling",
            aliases: &["POUND", "POUND STERLING"],
        },
        Case {
            variant: JPY,
            canonical: "JPY",
            full_name: "Japanese Yen",
            aliases: &[],
        },
        Case {
            variant: CAD,
            canonical: "CAD",
            full_name: "Canadian Dollar",
            aliases: &[],
        },
        Case {
            variant: AUD,
            canonical: "AUD",
            full_name: "Australian Dollar",
            aliases: &[],
        },
        Case {
            variant: CHF,
            canonical: "CHF",
            full_name: "Swiss Franc",
            aliases: &[],
        },
        Case {
            variant: CNY,
            canonical: "CNY",
            full_name: "Chinese Yuan",
            aliases: &[],
        },
        Case {
            variant: HKD,
            canonical: "HKD",
            full_name: "Hong Kong Dollar",
            aliases: &[],
        },
        Case {
            variant: SGD,
            canonical: "SGD",
            full_name: "Singapore Dollar",
            aliases: &[],
        },
        Case {
            variant: INR,
            canonical: "INR",
            full_name: "Indian Rupee",
            aliases: &[],
        },
        Case {
            variant: BRL,
            canonical: "BRL",
            full_name: "Brazilian Real",
            aliases: &[],
        },
        Case {
            variant: MXN,
            canonical: "MXN",
            full_name: "Mexican Peso",
            aliases: &[],
        },
        Case {
            variant: KRW,
            canonical: "KRW",
            full_name: "South Korean Won",
            aliases: &[],
        },
        Case {
            variant: NZD,
            canonical: "NZD",
            full_name: "New Zealand Dollar",
            aliases: &[],
        },
        Case {
            variant: NOK,
            canonical: "NOK",
            full_name: "Norwegian Krone",
            aliases: &[],
        },
        Case {
            variant: SEK,
            canonical: "SEK",
            full_name: "Swedish Krona",
            aliases: &[],
        },
        Case {
            variant: DKK,
            canonical: "DKK",
            full_name: "Danish Krone",
            aliases: &[],
        },
        Case {
            variant: PLN,
            canonical: "PLN",
            full_name: "Polish Zloty",
            aliases: &[],
        },
        Case {
            variant: CZK,
            canonical: "CZK",
            full_name: "Czech Koruna",
            aliases: &[],
        },
        Case {
            variant: HUF,
            canonical: "HUF",
            full_name: "Hungarian Forint",
            aliases: &[],
        },
        Case {
            variant: RUB,
            canonical: "RUB",
            full_name: "Russian Ruble",
            aliases: &[],
        },
        Case {
            variant: TRY,
            canonical: "TRY",
            full_name: "Turkish Lira",
            aliases: &[],
        },
        Case {
            variant: ZAR,
            canonical: "ZAR",
            full_name: "South African Rand",
            aliases: &[],
        },
        Case {
            variant: ILS,
            canonical: "ILS",
            full_name: "Israeli Shekel",
            aliases: &[],
        },
        Case {
            variant: THB,
            canonical: "THB",
            full_name: "Thai Baht",
            aliases: &[],
        },
        Case {
            variant: MYR,
            canonical: "MYR",
            full_name: "Malaysian Ringgit",
            aliases: &[],
        },
        Case {
            variant: PHP,
            canonical: "PHP",
            full_name: "Philippine Peso",
            aliases: &[],
        },
        Case {
            variant: IDR,
            canonical: "IDR",
            full_name: "Indonesian Rupiah",
            aliases: &[],
        },
        Case {
            variant: VND,
            canonical: "VND",
            full_name: "Vietnamese Dong",
            aliases: &[],
        },
        Case {
            variant: BTC,
            canonical: "BTC",
            full_name: "Bitcoin",
            aliases: &[],
        },
        Case {
            variant: ETH,
            canonical: "ETH",
            full_name: "Ethereum",
            aliases: &[],
        },
        Case {
            variant: XMR,
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
