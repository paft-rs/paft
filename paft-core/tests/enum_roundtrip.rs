use paft_core::PaftError;
use paft_core::domain::{
    AssetKind, Currency, Exchange, MarketState, Period, string_canonical::Canonical,
};
use std::str::FromStr;

fn assert_display_parse_display_idempotent<T>(token: &str)
where
    T: ToString + FromStr<Err = paft_core::error::PaftError> + PartialEq + Clone,
{
    let parsed = T::from_str(token).unwrap();
    let display1 = parsed.to_string();
    let reparsed = T::from_str(&display1).unwrap();
    let display2 = reparsed.to_string();
    assert_eq!(display1, display2);
}

#[test]
fn other_roundtrip_is_stable_for_core_enums() {
    // Currency
    assert_display_parse_display_idempotent::<Currency>("USD");
    assert_display_parse_display_idempotent::<Currency>("us dollar");
    let other_currency = Currency::from_str("foo-bar").unwrap();
    assert_eq!(other_currency.to_string(), "FOO_BAR");

    // Exchange
    assert_display_parse_display_idempotent::<Exchange>("NASDAQ");
    let other_exchange = Exchange::from_str("some-ex").unwrap();
    assert_eq!(other_exchange.to_string(), "SOME_EX");

    // Period
    assert_display_parse_display_idempotent::<Period>("2023Q4");
    assert_display_parse_display_idempotent::<Period>("2023-12-31");
    assert_display_parse_display_idempotent::<Period>("FY2023"); // normalizes to 2023
    let other_period = Period::from_str("custom range").unwrap();
    assert_eq!(other_period.to_string(), "CUSTOM_RANGE");
}

#[test]
fn rejects_inputs_that_canonicalize_to_empty_core_enums() {
    let empties = ["***", "__", "   "];

    for input in &empties {
        // Currency
        let err = Currency::from_str(input).unwrap_err();
        match err {
            paft_core::error::PaftError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "Currency");
                assert_eq!(value, (*input).to_string());
            }
            other => panic!("unexpected error: {other}"),
        }

        // Exchange
        let err = Exchange::from_str(input).unwrap_err();
        match err {
            paft_core::error::PaftError::InvalidEnumValue { enum_name, value } => {
                assert_eq!(enum_name, "Exchange");
                assert_eq!(value, (*input).to_string());
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}

#[test]
fn display_matches_wire_codes_for_core_enums() {
    let usd = Currency::Iso(iso_currency::Currency::USD);
    assert_eq!(usd.to_string(), usd.code());

    let nasdaq = Exchange::NASDAQ;
    assert_eq!(nasdaq.to_string(), nasdaq.code());

    let asset = AssetKind::Crypto;
    assert_eq!(asset.to_string(), asset.code());

    let state = MarketState::Regular;
    assert_eq!(state.to_string(), state.code());
}

#[test]
fn closed_enums_reject_unknown_tokens() {
    assert!(AssetKind::from_str("UNKNOWN_KIND").is_err());
    assert!(MarketState::from_str("UNKNOWN_STATE").is_err());
}

#[test]
fn extensible_enums_preserve_other_canonical_tokens() {
    let token = Currency::from_str("My Token").unwrap();
    match token {
        Currency::Other(ref canon) => assert_eq!(canon.as_ref(), "MY_TOKEN"),
        other => panic!("expected Other variant, got {other:?}"),
    }

    let venue = Exchange::from_str("my-exchange").unwrap();
    match venue {
        Exchange::Other(ref canon) => assert_eq!(canon.as_ref(), "MY_EXCHANGE"),
        other => panic!("expected Other variant, got {other:?}"),
    }
}

#[test]
fn canonical_from_str_uses_precise_error() {
    let err = Canonical::from_str("***").unwrap_err();
    match err {
        PaftError::InvalidCanonicalToken { value } => assert_eq!(value, "***"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}
