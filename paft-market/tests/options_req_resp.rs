use chrono::{DateTime, NaiveDate};
use paft_decimal::{Decimal, NonNegativeDecimal};
use paft_domain::{AssetKind, Instrument};
use paft_market::market::OptionUpdate as MarketOptionUpdate;
use paft_market::{
    FieldUpdate, MarketError, OptionChainRequest, OptionContract, OptionContractKey,
    OptionExpirationsRequest, OptionExpirationsResponse, OptionGreeks, OptionSide, OptionUpdate,
};
use paft_money::{Currency, IsoCurrency, Price, PriceAmount};
use std::str::FromStr;

fn usd(amount: i64) -> Price {
    Price::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD))
}

const fn usd_currency() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

const fn eur_currency() -> Currency {
    Currency::Iso(IsoCurrency::EUR)
}

fn dec(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap()
}

fn option_key() -> OptionContractKey {
    OptionContractKey::new(
        Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        OptionSide::Call,
        usd(150),
        NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
    )
}

#[test]
fn option_side_from_str_uses_stable_codes() {
    for (side, code) in [(OptionSide::Call, "CALL"), (OptionSide::Put, "PUT")] {
        assert_eq!(side.to_string(), code);
        assert_eq!(code.parse::<OptionSide>().unwrap(), side);
        assert_eq!(serde_json::to_value(side).unwrap(), serde_json::json!(code));
    }
}

#[test]
fn option_side_from_str_rejects_unknown_code() {
    let err = "BUY_TO_OPEN".parse::<OptionSide>().unwrap_err();

    assert!(matches!(
        err,
        MarketError::InvalidEnumValue {
            enum_name: "OptionSide",
            value,
        } if value == "BUY_TO_OPEN"
    ));
}

#[test]
fn option_expirations_request_roundtrip() {
    let req = OptionExpirationsRequest {
        underlying: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let de: OptionExpirationsRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, de);
}

#[test]
fn option_expirations_request_deserialization_unknown_field_rejected() {
    let req = OptionExpirationsRequest {
        underlying: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
    };
    let mut value = serde_json::to_value(&req).unwrap();
    value
        .as_object_mut()
        .expect("option expirations request serializes as an object")
        .insert("underling".to_owned(), serde_json::json!("AAPL"));

    assert!(serde_json::from_value::<OptionExpirationsRequest>(value).is_err());
}

#[test]
fn option_chain_request_roundtrip() {
    let req = OptionChainRequest {
        underlying: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        expiration: NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let de: OptionChainRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, de);
}

#[test]
fn option_chain_request_deserialization_unknown_field_rejected() {
    let req = OptionChainRequest {
        underlying: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        expiration: NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
    };
    let mut value = serde_json::to_value(&req).unwrap();
    value
        .as_object_mut()
        .expect("option chain request serializes as an object")
        .insert("expriation".to_owned(), serde_json::json!("2025-01-17"));

    assert!(serde_json::from_value::<OptionChainRequest>(value).is_err());
}

#[test]
fn option_expirations_response_roundtrip() {
    let resp = OptionExpirationsResponse {
        dates: vec![
            NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
        ],
    };
    let json = serde_json::to_string(&resp).unwrap();
    let de: OptionExpirationsResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(resp, de);
}

#[test]
fn option_expirations_new_sorted_canonicalizes_dates() {
    let resp = OptionExpirationsResponse::new_sorted(vec![
        NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
        NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
        NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
    ]);

    assert_eq!(
        resp.dates,
        vec![
            NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
        ],
    );
    assert!(resp.is_sorted_unique());
}

#[test]
fn option_expirations_sorted_unique_validation_is_strict() {
    assert!(OptionExpirationsResponse { dates: vec![] }.is_sorted_unique());
    assert!(
        OptionExpirationsResponse {
            dates: vec![
                NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
                NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
            ],
        }
        .is_sorted_unique()
    );
    assert!(
        !OptionExpirationsResponse {
            dates: vec![
                NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
                NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
            ],
        }
        .is_sorted_unique()
    );
    assert!(
        !OptionExpirationsResponse {
            dates: vec![
                NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
                NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
            ],
        }
        .is_sorted_unique()
    );
}

#[test]
fn option_contract_in_the_money_distinguishes_unknown_from_false() {
    let unknown: OptionContract = OptionContract::new(option_key(), usd_currency());
    assert_eq!(unknown.in_the_money, None);

    let mut value = serde_json::to_value(&unknown).unwrap();
    value
        .as_object_mut()
        .expect("option contract serializes as an object")
        .remove("in_the_money");

    let decoded_unknown: OptionContract = serde_json::from_value(value).unwrap();
    assert_eq!(decoded_unknown.in_the_money, None);

    let mut explicit_false = OptionContract::new(option_key(), usd_currency());
    explicit_false.in_the_money = Some(false);

    let value = serde_json::to_value(&explicit_false).unwrap();
    assert_eq!(value.get("in_the_money"), Some(&serde_json::json!(false)));

    let decoded_false: OptionContract = serde_json::from_value(value).unwrap();
    assert_eq!(decoded_false.in_the_money, Some(false));
}

#[test]
fn option_quote_currency_is_independent_from_strike_currency() {
    let mut contract = OptionContract::new(option_key(), eur_currency());
    contract.price = Some(PriceAmount::new(dec("5.25")));

    assert_eq!(contract.key.strike.currency(), &usd_currency());
    assert_eq!(contract.currency, eur_currency());

    let value = serde_json::to_value(&contract).unwrap();
    assert_eq!(value["strike"]["currency"], serde_json::json!("USD"));
    assert_eq!(value["currency"], serde_json::json!("EUR"));
    assert_eq!(value["price"], serde_json::json!("5.25"));
}

#[test]
fn option_contract_key_distinguishes_listed_contract_instrument() {
    let base = option_key();
    let standard = base.clone().with_contract_instrument(
        Instrument::from_symbol("AAPL250117C00150000", AssetKind::Option).unwrap(),
    );
    let adjusted = base.with_contract_instrument(
        Instrument::from_symbol("AAPL1250117C00150000", AssetKind::Option).unwrap(),
    );

    assert_ne!(standard, adjusted);

    let contract = OptionContract::new(standard.clone(), usd_currency());
    let value = serde_json::to_value(&contract).unwrap();
    assert_eq!(
        value["contract_instrument"]["symbol"],
        serde_json::json!("AAPL250117C00150000")
    );

    let decoded: OptionContract = serde_json::from_value(value).unwrap();
    assert_eq!(
        decoded.key.contract_instrument,
        standard.contract_instrument
    );
}

#[test]
fn option_greeks_decimal_serde_uses_canonical_strings() {
    let greeks = OptionGreeks {
        delta: Some(dec("0.5000")),
        gamma: Some(dec("0.0100")),
        ..OptionGreeks::default()
    };

    let value = serde_json::to_value(&greeks).unwrap();
    assert_eq!(value.get("delta"), Some(&serde_json::json!("0.5")));
    assert_eq!(value.get("gamma"), Some(&serde_json::json!("0.01")));

    let decoded: OptionGreeks = serde_json::from_value(serde_json::json!({
        "delta": "+0.5000"
    }))
    .unwrap();
    assert_eq!(decoded.delta, Some(dec("0.5000")));

    assert!(serde_json::from_value::<OptionGreeks>(serde_json::json!({ "delta": 0.5 })).is_err());
}

#[test]
fn option_update_ts_serde_uses_unix_milliseconds() {
    let update: MarketOptionUpdate = OptionUpdate::new(
        option_key(),
        usd_currency(),
        DateTime::from_timestamp(1_640_995_200, 789_000_000).unwrap(),
    );

    let value = serde_json::to_value(&update).unwrap();
    assert_eq!(
        value.get("ts"),
        Some(&serde_json::json!(1_640_995_200_789_i64))
    );
    assert_eq!(value.get("side"), Some(&serde_json::json!("CALL")));
    assert_eq!(value.get("currency"), Some(&serde_json::json!("USD")));
    assert!(value.get("underlying").is_some());

    let deserialized: OptionUpdate = serde_json::from_value(value).unwrap();
    assert_eq!(update, deserialized);
}

fn incremental_sequence() -> Vec<OptionUpdate> {
    let base = serde_json::to_value(OptionUpdate::new(
        option_key(),
        usd_currency(),
        DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
    ))
    .unwrap();
    // Adversarial sequence modeled on Deribit's incremental ticker contract:
    // https://docs.deribit.com/subscriptions/market-data/incremental_tickerinstrument_name
    [
        serde_json::json!({"bid": "12.50"}),
        serde_json::json!({"ask": "13", "last_price": "12.75", "implied_volatility": "0.25"}),
        serde_json::json!({"bid": null}),
    ]
    .into_iter()
    .map(|fields| {
        let mut wire = base.clone();
        wire.as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        serde_json::from_value(wire).unwrap()
    })
    .collect()
}

#[test]
fn incremental_option_updates_preserve_state_through_json() {
    let sequence = incremental_sequence();
    let mut bid = None;
    for (update, expected) in sequence.iter().zip([
        Some(PriceAmount::new(dec("12.5"))),
        Some(PriceAmount::new(dec("12.5"))),
        None,
    ]) {
        let wire = serde_json::to_value(update).unwrap();
        let decoded: OptionUpdate = serde_json::from_value(wire).unwrap();
        assert_eq!(&decoded, update);
        decoded.bid.apply_to(&mut bid);
        assert_eq!(bid, expected);
    }
    assert_eq!(serde_json::to_value(&sequence[0]).unwrap()["bid"], "12.5");
    assert!(
        serde_json::to_value(&sequence[1])
            .unwrap()
            .get("bid")
            .is_none()
    );
    assert_eq!(
        serde_json::to_value(&sequence[2]).unwrap().get("bid"),
        Some(&serde_json::Value::Null)
    );

    let mut wire = serde_json::to_value(&sequence[1]).unwrap();
    for field in ["bid", "ask", "last_price", "implied_volatility"] {
        wire[field] = serde_json::Value::Null;
    }
    let cleared: OptionUpdate = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(cleared.bid, FieldUpdate::Clear);
    assert_eq!(cleared.ask, FieldUpdate::Clear);
    assert_eq!(cleared.last_price, FieldUpdate::Clear);
    assert_eq!(cleared.implied_volatility, FieldUpdate::Clear);
    wire["implied_volatility"] = serde_json::json!("-0.1");
    assert!(serde_json::from_value::<OptionUpdate>(wire).is_err());
    assert_eq!(
        sequence[1].implied_volatility,
        FieldUpdate::Set(NonNegativeDecimal::new(dec("0.25")).unwrap())
    );
    assert!(serde_json::to_string(&FieldUpdate::<PriceAmount>::Unchanged).is_err());
}

#[cfg(feature = "dataframe")]
#[test]
fn incremental_option_dataframe_retains_operations_and_values() {
    use paft_utils::dataframe::{Columnar, ToDataFrame};

    let sequence = incremental_sequence();
    let frame = OptionUpdate::columnar_to_dataframe(&sequence).unwrap();
    assert_eq!(
        frame.schema(),
        OptionUpdate::empty_dataframe().unwrap().schema()
    );
    for field in ["bid", "ask", "last_price", "implied_volatility"] {
        let operations = frame
            .column(&format!("{field}.operation"))
            .unwrap()
            .str()
            .unwrap();
        let values = frame
            .column(&format!("{field}.value"))
            .unwrap()
            .decimal()
            .unwrap();
        let expected = if field == "bid" {
            ["SET", "UNCHANGED", "CLEAR"]
        } else {
            ["UNCHANGED", "SET", "UNCHANGED"]
        };
        for (row, operation) in expected.into_iter().enumerate() {
            assert_eq!(operations.get(row), Some(operation));
            assert_eq!(values.physical().get(row).is_some(), operation == "SET");
            let single = sequence[row].to_dataframe().unwrap();
            assert!(
                frame
                    .slice(i64::try_from(row).unwrap(), 1)
                    .equals_missing(&single)
            );
        }
    }
    let operations = frame.column("bid.operation").unwrap().str().unwrap();
    let values = frame.column("bid.value").unwrap().decimal().unwrap();
    let mut bid = None;
    for (row, expected) in [Some(125_000_000_000_i128), Some(125_000_000_000), None]
        .into_iter()
        .enumerate()
    {
        match operations.get(row).unwrap() {
            "SET" => bid = values.physical().get(row),
            "CLEAR" => bid = None,
            "UNCHANGED" => {}
            other => panic!("unknown operation: {other}"),
        }
        assert_eq!(bid, expected);
    }
}
