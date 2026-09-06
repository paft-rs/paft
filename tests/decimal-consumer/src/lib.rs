#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use paft::prelude::{Decimal, decimal};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Payload {
        #[serde(with = "paft::decimal::serde::canonical_str")]
        amount: Decimal,
        #[serde(default, with = "paft::decimal::serde::option_canonical_str")]
        optional: Option<Decimal>,
    }

    #[test]
    fn parser_classifies_errors_through_the_facade() {
        use paft::decimal::DecimalParseError;
        assert_eq!(
            decimal::parse_decimal(" +1.2300 ").unwrap(),
            Decimal::new(123, 2)
        );
        assert_eq!(
            decimal::parse_decimal("1e3"),
            Err(DecimalParseError::InvalidSyntax)
        );
        assert_eq!(
            decimal::parse_decimal("0.00000000000000000000000000001"),
            Err(DecimalParseError::NotRepresentable)
        );
    }

    #[test]
    fn exact_helpers_are_available_without_another_paft_dependency() {
        let a = decimal::parse_decimal("1.25").unwrap();
        let b = decimal::parse_decimal("0.5").unwrap();
        for (actual, expected) in [
            (decimal::checked_add_exact(&a, &b), "1.75"),
            (decimal::checked_sub_exact(&a, &b), "0.75"),
            (decimal::checked_mul_exact(&a, &b), "0.625"),
            (decimal::checked_div_exact(&a, &b), "2.5"),
        ] {
            assert_eq!(actual, Some(decimal::parse_decimal(expected).unwrap()));
        }
        assert!(decimal::checked_add_exact(&Decimal::MAX, &Decimal::ONE).is_none());
        assert!(decimal::checked_div_exact(&Decimal::ONE, &Decimal::from(3)).is_none());
    }

    #[test]
    fn consumer_payloads_use_canonical_serde_via_the_facade() {
        let payload: Payload =
            serde_json::from_str(r#"{"amount":"+1.2300","optional":"0.00010"}"#).unwrap();
        assert_eq!(
            serde_json::to_value(&payload).unwrap(),
            serde_json::json!({"amount": "1.23", "optional": "0.0001"})
        );
        assert_eq!(
            serde_json::from_str::<Payload>(&serde_json::to_string(&payload).unwrap()).unwrap(),
            payload
        );
        let absent: Payload = serde_json::from_str(r#"{"amount":"0"}"#).unwrap();
        assert_eq!(absent.optional, None);
        for invalid in [
            r#"{"amount":1.23}"#,
            r#"{"amount":"1e3"}"#,
            r#"{"amount":"0.00000000000000000000000000001"}"#,
            r#"{"amount":"1","optional":"0.00000000000000000000000000001"}"#,
        ] {
            assert!(serde_json::from_str::<Payload>(invalid).is_err());
        }
    }
}
