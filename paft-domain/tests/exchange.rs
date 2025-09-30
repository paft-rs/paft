//! Tests covering canonical/alias behavior for exchanges (migrated from paft-core).

use paft_domain::Exchange;
use std::str::FromStr;

struct Case {
    variant: Exchange,
    canonical: &'static str,
    full_name: &'static str,
    aliases: &'static [&'static str],
}

#[test]
fn exchange_round_trips_display_fromstr_and_serde() {
    for case in cases() {
        assert_round_trip(&case);
    }
}

#[test]
fn exchange_other_values_uppercase_and_round_trip() {
    let parsed = Exchange::try_from_str("nasdaq-gs").unwrap();
    assert_eq!(parsed.to_string(), "NASDAQ_GS");
    assert_eq!(parsed.to_string(), "NASDAQ_GS");

    let via_into: String = parsed.clone().into();
    assert_eq!(via_into, "NASDAQ_GS");

    let json = serde_json::to_string(&parsed).unwrap();
    assert_eq!(json, "\"NASDAQ_GS\"");
    let back: Exchange = serde_json::from_str(&json).unwrap();
    assert_eq!(back, parsed);
}

#[test]
fn exchange_is_european_checks_geography() {
    assert!(Exchange::Euronext.is_european_exchange());
    assert!(Exchange::BSE_HU.is_european_exchange()); // Budapest
    assert!(Exchange::PSE_CZ.is_european_exchange()); // Prague
    assert!(!Exchange::BSE.is_european_exchange());
    assert!(!Exchange::PSE.is_european_exchange());
}

#[allow(clippy::too_many_lines)]
fn cases() -> Vec<Case> {
    use Exchange::*;

    vec![
        Case {
            variant: NASDAQ,
            canonical: "NASDAQ",
            full_name: "Nasdaq",
            aliases: &[],
        },
        Case {
            variant: NYSE,
            canonical: "NYSE",
            full_name: "NYSE",
            aliases: &[],
        },
        Case {
            variant: AMEX,
            canonical: "AMEX",
            full_name: "AMEX",
            aliases: &[],
        },
        Case {
            variant: BATS,
            canonical: "BATS",
            full_name: "BATS",
            aliases: &[],
        },
        Case {
            variant: OTC,
            canonical: "OTC",
            full_name: "OTC",
            aliases: &[],
        },
        Case {
            variant: LSE,
            canonical: "LSE",
            full_name: "London Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: TSE,
            canonical: "TSE",
            full_name: "Tokyo Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: HKEX,
            canonical: "HKEX",
            full_name: "Hong Kong Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: SSE,
            canonical: "SSE",
            full_name: "Shanghai Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: SZSE,
            canonical: "SZSE",
            full_name: "Shenzhen Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: TSX,
            canonical: "TSX",
            full_name: "Toronto Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: ASX,
            canonical: "ASX",
            full_name: "Australian Securities Exchange",
            aliases: &[],
        },
        Case {
            variant: Euronext,
            canonical: "EURONEXT",
            full_name: "Euronext",
            aliases: &["Euronext"],
        },
        Case {
            variant: XETRA,
            canonical: "XETRA",
            full_name: "Xetra",
            aliases: &[],
        },
        Case {
            variant: SIX,
            canonical: "SIX",
            full_name: "Swiss Exchange",
            aliases: &[],
        },
        Case {
            variant: BIT,
            canonical: "BIT",
            full_name: "Borsa Italiana",
            aliases: &[],
        },
        Case {
            variant: BME,
            canonical: "BME",
            full_name: "Bolsa de Madrid",
            aliases: &[],
        },
        Case {
            variant: AEX,
            canonical: "AEX",
            full_name: "Euronext Amsterdam",
            aliases: &[],
        },
        Case {
            variant: BRU,
            canonical: "BRU",
            full_name: "Euronext Brussels",
            aliases: &[],
        },
        Case {
            variant: LIS,
            canonical: "LIS",
            full_name: "Euronext Lisbon",
            aliases: &[],
        },
        Case {
            variant: EPA,
            canonical: "EPA",
            full_name: "Euronext Paris",
            aliases: &["EURONEXT PARIS"],
        },
        Case {
            variant: OSL,
            canonical: "OSL",
            full_name: "Oslo Børs",
            aliases: &[],
        },
        Case {
            variant: STO,
            canonical: "STO",
            full_name: "Stockholm Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: CPH,
            canonical: "CPH",
            full_name: "Copenhagen Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: WSE,
            canonical: "WSE",
            full_name: "Warsaw Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: PSE_CZ,
            canonical: "PSE_CZ",
            full_name: "Prague Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: BSE_HU,
            canonical: "BSE_HU",
            full_name: "Budapest Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: MOEX,
            canonical: "MOEX",
            full_name: "Moscow Exchange",
            aliases: &[],
        },
        Case {
            variant: BIST,
            canonical: "BIST",
            full_name: "Istanbul Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: JSE,
            canonical: "JSE",
            full_name: "Johannesburg Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: TASE,
            canonical: "TASE",
            full_name: "Tel Aviv Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: BSE,
            canonical: "BSE",
            full_name: "Bombay Stock Exchange",
            aliases: &["BOMBAY", "BSE INDIA"],
        },
        Case {
            variant: NSE,
            canonical: "NSE",
            full_name: "National Stock Exchange of India",
            aliases: &[],
        },
        Case {
            variant: KRX,
            canonical: "KRX",
            full_name: "Korea Exchange",
            aliases: &[],
        },
        Case {
            variant: SGX,
            canonical: "SGX",
            full_name: "Singapore Exchange",
            aliases: &[],
        },
        Case {
            variant: SET,
            canonical: "SET",
            full_name: "Stock Exchange of Thailand",
            aliases: &[],
        },
        Case {
            variant: KLSE,
            canonical: "KLSE",
            full_name: "Bursa Malaysia",
            aliases: &[],
        },
        Case {
            variant: PSE,
            canonical: "PSE",
            full_name: "Philippine Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: IDX,
            canonical: "IDX",
            full_name: "Indonesia Stock Exchange",
            aliases: &[],
        },
        Case {
            variant: HOSE,
            canonical: "HOSE",
            full_name: "Ho Chi Minh Stock Exchange",
            aliases: &[],
        },
    ]
}

fn assert_round_trip(case: &Case) {
    let display = case.variant.to_string();
    assert_eq!(display, case.canonical);
    assert_eq!(case.variant.full_name(), case.full_name);

    assert_eq!(case.variant.code(), case.canonical);

    let parsed = Exchange::from_str(case.canonical).unwrap();
    assert_eq!(parsed, case.variant);

    let json = serde_json::to_string(&case.variant).unwrap();
    assert_eq!(json, format!("\"{}\"", case.canonical));
    let back: Exchange = serde_json::from_str(&json).unwrap();
    assert_eq!(back, case.variant);

    for alias in case.aliases {
        let parsed_alias = Exchange::from_str(alias).unwrap();
        assert_eq!(parsed_alias, case.variant);

        let re_display = parsed_alias.to_string();
        assert_eq!(re_display, case.canonical);

        let reparsed = Exchange::from_str(parsed_alias.code()).unwrap();
        assert_eq!(reparsed, case.variant);

        let alias_json = format!("\"{alias}\"");
        let alias_back: Exchange = serde_json::from_str(&alias_json).unwrap();
        assert_eq!(alias_back, case.variant);
    }
}
