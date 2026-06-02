use paft_domain::{AssetKind, DomainError, Exchange, MarketState, Period};
use paft_money::Currency;
use std::str::FromStr;

fn assert_display_parse_display_idempotent<T, E>(token: &str)
where
    T: ToString + FromStr<Err = E> + PartialEq + Clone,
    E: std::fmt::Debug,
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
    assert_display_parse_display_idempotent::<Currency, _>("USD");
    assert_display_parse_display_idempotent::<Currency, _>("us dollar");
    let other_currency = Currency::from_str("foo-bar").unwrap();
    assert_eq!(other_currency.to_string(), "FOO_BAR");

    // Exchange
    assert_display_parse_display_idempotent::<Exchange, _>("NASDAQ");
    let other_exchange = Exchange::from_str("some-ex").unwrap();
    assert_eq!(other_exchange.to_string(), "SOME_EX");

    // AssetKind
    assert_display_parse_display_idempotent::<AssetKind, _>("EQUITY");
    let other_asset = AssetKind::from_str("structured note").unwrap();
    assert_eq!(other_asset.to_string(), "STRUCTURED_NOTE");

    // Period
    assert_display_parse_display_idempotent::<Period, _>("2023Q4");
    assert_display_parse_display_idempotent::<Period, _>("2023-12-31");
    assert_display_parse_display_idempotent::<Period, _>("FY2023"); // normalizes to 2023
    let other_period = Period::from_str("custom range").unwrap();
    assert_eq!(other_period.to_string(), "CUSTOM_RANGE");
}

#[test]
fn rejects_inputs_that_canonicalize_to_empty_core_enums() {
    let empties = ["***", "__", "   "];

    for input in &empties {
        // Currency
        let err = Currency::from_str(input).unwrap_err();
        assert!(matches!(
            err,
            paft_money::MoneyParseError::InvalidEnumValue { enum_name, value }
                if enum_name == "Currency" && value.as_str() == *input
        ));

        // AssetKind
        let err = AssetKind::from_str(input).unwrap_err();
        assert!(matches!(
            err,
            paft_core::PaftError::InvalidEnumValue { enum_name, value }
                if enum_name == "AssetKind" && value.as_str() == *input
        ));

        // Exchange
        let err = Exchange::try_from_str(input).unwrap_err();
        match err {
            DomainError::InvalidExchangeValue { value } => {
                assert_eq!(value, (*input).to_string());
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}

#[test]
fn display_matches_wire_codes_for_core_enums() {
    let usd = Currency::from_str("USD").unwrap();
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
    assert!(MarketState::from_str("UNKNOWN_STATE").is_err());
}

#[test]
fn market_state_aliases_parse_to_expected_sessions() {
    assert_eq!(MarketState::from_str("PRE").unwrap(), MarketState::Pre);
    assert_eq!(
        MarketState::from_str("PRE_MARKET").unwrap(),
        MarketState::Pre
    );

    assert_eq!(MarketState::from_str("POST").unwrap(), MarketState::Post);
    assert_eq!(
        MarketState::from_str("POSTMARKET").unwrap(),
        MarketState::Post
    );
    assert_eq!(
        MarketState::from_str("POST_MARKET").unwrap(),
        MarketState::Post
    );
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

    let asset = AssetKind::from_str("structured note").unwrap();
    match asset {
        AssetKind::Other(ref canon) => assert_eq!(canon.as_ref(), "STRUCTURED_NOTE"),
        other => panic!("expected Other variant, got {other:?}"),
    }

    let asset: AssetKind = serde_json::from_str("\"structured note\"").unwrap();
    match asset {
        AssetKind::Other(ref canon) => assert_eq!(canon.as_ref(), "STRUCTURED_NOTE"),
        other => panic!("expected Other variant, got {other:?}"),
    }
}
