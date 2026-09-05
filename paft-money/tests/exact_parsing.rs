use paft_decimal as decimal;
use paft_money::{Currency, IsoCurrency, MonetaryAmount, Money, MoneyError, Price};
use serde_json::json;

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

#[test]
fn money_rejects_significant_digits_beyond_decimal_precision() {
    for literal in [
        "1.00000000000000000000000000001",
        "-1.00000000000000000000000000001",
        "0.00000000000000000000000000001",
        "-0.00000000000000000000000000001",
    ] {
        let error = Money::from_canonical_str(literal, usd()).unwrap_err();
        assert_eq!(error, MoneyError::NotRepresentable);
        assert!(
            serde_json::from_value::<Money>(json!({
                "amount": literal,
                "currency": "USD",
                "minor_units": 2,
            }))
            .is_err()
        );
    }
}

#[test]
fn money_preserves_settlement_precision_or_rejects_decimal_overflow() {
    // Only two fractional digits, but their coefficient exceeds 96 bits.
    let literal = "7922816251426433759354395033.51";
    let parsed = Money::from_canonical_str(literal, usd());
    let wire = json!({ "amount": literal, "currency": "USD", "minor_units": 2 });
    let deserialized = serde_json::from_value::<Money>(wire);
    assert_eq!(parsed, Err(MoneyError::NotRepresentable));
    assert!(deserialized.is_err());
}

#[test]
fn full_precision_amounts_parse_losslessly_or_reject_the_value() {
    for literal in [
        "1.00000000000000000000000000001",
        "-0.00000000000000000000000000001",
        "7922816251426433759354395033.51",
    ] {
        let amount = MonetaryAmount::from_canonical_str(literal, usd());
        let price = Price::from_canonical_str(literal, usd());
        let wire = json!({ "amount": literal, "currency": "USD" });
        let deserialized_amount = serde_json::from_value::<MonetaryAmount>(wire.clone());
        let deserialized_price = serde_json::from_value::<Price>(wire);

        assert_eq!(amount, Err(MoneyError::NotRepresentable));
        assert_eq!(price, Err(MoneyError::NotRepresentable));
        assert!(deserialized_amount.is_err());
        assert!(deserialized_price.is_err());
    }
}

#[test]
fn money_accepts_insignificant_zeros_beyond_decimal_precision() {
    for (literal, canonical) in [
        ("  +001.2300000000000000000000000000000  ", "1.23"),
        ("-1.2300000000000000000000000000000", "-1.23"),
        ("-0.0000000000000000000000000000000", "0"),
    ] {
        let amount = Money::from_canonical_str(literal, usd()).unwrap();
        assert_eq!(decimal::to_canonical_string(&amount.amount()), canonical);
        let deserialized = serde_json::from_value::<Money>(json!({
            "amount": literal,
            "currency": "USD",
            "minor_units": 2,
        }))
        .unwrap();
        assert_eq!(deserialized, amount);
        assert_eq!(serde_json::to_value(amount).unwrap()["amount"], canonical);
    }
}

#[test]
fn quoted_prices_preserve_precision_and_settlement_rejects_it() {
    let literal = "1.234567";
    let expected = decimal::parse_decimal(literal).unwrap();
    let price = Price::from_canonical_str(literal, usd()).unwrap();
    assert_eq!(price.amount(), expected);
    assert_eq!(serde_json::to_value(&price).unwrap()["amount"], literal);
    assert_eq!(
        serde_json::from_value::<Price>(json!({"amount": literal, "currency": "USD"})).unwrap(),
        price
    );
    assert!(matches!(
        Money::from_canonical_str(literal, usd()),
        Err(MoneyError::PrecisionExceeded { .. })
    ));
    assert!(
        serde_json::from_value::<Money>(
            json!({"amount": literal, "currency": "USD", "minor_units": 2})
        )
        .is_err()
    );
    assert_eq!(
        price.to_money().unwrap().amount(),
        decimal::parse_decimal("1.23").unwrap()
    );
}

#[test]
fn contextual_amounts_and_exchange_rate_shadows_use_exact_ingestion() {
    use paft_money::{ExchangeRate, PriceAmount, QuantityAmount};
    for literal in [
        "0.00000000000000000000000000001",
        "79228162514264337593543950336",
    ] {
        assert!(
            serde_json::from_value::<PriceAmount>(json!(literal))
                .unwrap_err()
                .to_string()
                .contains("not exactly representable by PAFT")
        );
        assert!(serde_json::from_value::<QuantityAmount>(json!(literal)).is_err());
        assert!(
            serde_json::from_value::<ExchangeRate>(
                json!({"from":"USD", "to":"EUR", "rate":literal})
            )
            .is_err()
        );
    }
    assert!(
        serde_json::from_value::<ExchangeRate>(
            json!({"from":"USD", "to":"USD", "rate":"1.00000000000000000000000000001"})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<QuantityAmount>(json!("-0.00000000000000000000000000001"))
            .is_err()
    );
}

#[cfg(feature = "money-formatting")]
#[test]
fn localized_parsing_distinguishes_representability_and_currency_scale() {
    use paft_money::Locale;
    assert_eq!(
        Money::from_str_locale(
            "79,228,162,514,264,337,593,543,950,336.00",
            usd(),
            Locale::EnUs
        ),
        Err(MoneyError::NotRepresentable)
    );
    assert!(matches!(
        Money::from_str_locale("1.234", usd(), Locale::EnUs),
        Err(MoneyError::ScaleTooLarge { .. })
    ));
    assert_eq!(
        Money::from_str_locale("1,234.50", usd(), Locale::EnUs).unwrap(),
        Money::from_canonical_str("1234.5", usd()).unwrap()
    );
}
