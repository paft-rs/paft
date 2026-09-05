use paft_decimal as decimal;
use paft_money::{Currency, IsoCurrency, MonetaryAmount, Money, MoneyError, Price};
use serde_json::json;

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

#[test]
fn money_rejects_significant_digits_beyond_backend_precision() {
    for literal in [
        "1.00000000000000000000000000001",
        "-1.00000000000000000000000000001",
        "0.00000000000000000000000000001",
        "-0.00000000000000000000000000001",
    ] {
        let error = Money::from_canonical_str(literal, usd()).unwrap_err();
        if decimal::MAX_DECIMAL_PRECISION == 28 {
            assert_eq!(error, MoneyError::InvalidDecimal);
        } else {
            assert!(matches!(error, MoneyError::PrecisionExceeded { .. }));
        }
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
fn money_preserves_settlement_precision_or_rejects_backend_overflow() {
    // Only two fractional digits, but their coefficient exceeds 96 bits.
    let literal = "7922816251426433759354395033.51";
    let parsed = Money::from_canonical_str(literal, usd());
    let wire = json!({ "amount": literal, "currency": "USD", "minor_units": 2 });
    let deserialized = serde_json::from_value::<Money>(wire.clone());
    if decimal::MAX_DECIMAL_PRECISION == 28 {
        assert_eq!(parsed, Err(MoneyError::InvalidDecimal));
        assert!(deserialized.is_err());
    } else {
        let amount = parsed.unwrap();
        assert_eq!(decimal::to_canonical_string(&amount.amount()), literal);
        assert_eq!(deserialized.unwrap(), amount);
        assert_eq!(serde_json::to_value(amount).unwrap(), wire);
    }
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
        let deserialized_price = serde_json::from_value::<Price>(wire.clone());

        if decimal::MAX_DECIMAL_PRECISION == 28 {
            assert_eq!(amount, Err(MoneyError::InvalidDecimal));
            assert_eq!(price, Err(MoneyError::InvalidDecimal));
            assert!(deserialized_amount.is_err());
            assert!(deserialized_price.is_err());
        } else {
            let amount = amount.unwrap();
            let price = price.unwrap();
            assert_eq!(decimal::to_canonical_string(&amount.amount()), literal);
            assert_eq!(decimal::to_canonical_string(&price.amount()), literal);
            assert_eq!(deserialized_amount.unwrap(), amount);
            assert_eq!(deserialized_price.unwrap(), price);
            assert_eq!(serde_json::to_value(amount).unwrap(), wire);
            assert_eq!(serde_json::to_value(price).unwrap(), wire);
        }
    }
}

#[test]
fn money_accepts_insignificant_zeros_beyond_backend_precision() {
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
