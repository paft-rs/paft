use paft_decimal::{Decimal, parse_decimal};
use paft_money::{
    Currency, Locale, Money, MoneyError, clear_currency_metadata, currency_metadata,
    set_currency_metadata,
};
use serde_json::json;

#[test]
fn corrected_tokens_use_native_precision_for_ingestion_and_settlement() {
    for code in ["LINK", "UNI", "MATIC"] {
        let currency = Currency::try_from_str(code).unwrap();
        let unit = "0.000000000000000001";
        let money = Money::from_canonical_str(unit, currency.clone()).unwrap();
        assert_eq!(money, Money::from_minor_units(1, currency.clone()).unwrap());
        assert_eq!(money.minor_units(), 18);
        assert_eq!(money.amount(), parse_decimal(unit).unwrap());

        let wire = json!({"amount": unit, "currency": code, "minor_units": 18});
        assert_eq!(serde_json::to_value(&money).unwrap(), wire);
        assert_eq!(serde_json::from_value::<Money>(wire).unwrap(), money);

        assert!(matches!(
            Money::from_canonical_str("0.0000000000000000001", currency.clone()),
            Err(MoneyError::PrecisionExceeded {
                max_scale: 18,
                actual_scale: 19,
                ..
            })
        ));

        let rounded = Money::new(
            parse_decimal("0.0000000000000000015").unwrap(),
            currency.clone(),
        )
        .unwrap();
        let two_units = Money::from_minor_units(2, currency).unwrap();
        assert_eq!(rounded, two_units);
        assert_eq!(money.try_add(&money).unwrap(), two_units);
        assert_eq!(
            money.try_mul(&parse_decimal("1.5").unwrap()).unwrap(),
            two_units
        );
    }
}

#[test]
fn corrected_defaults_require_explicit_migration_of_legacy_scale() {
    for code in ["LINK", "UNI", "MATIC"] {
        let mut wire = json!({
            "amount": "0.00000001", "currency": code, "minor_units": 8,
        });
        let error = serde_json::from_value::<Money>(wire.clone()).unwrap_err();
        assert!(error.to_string().contains("minor-unit scale mismatch"));

        // Migrating a correct major-unit amount preserves that amount, not
        // the old integer unit count. Recover original native counts separately.
        wire["minor_units"] = json!(18);
        let migrated = serde_json::from_value::<Money>(wire).unwrap();
        assert_eq!(migrated.amount(), parse_decimal("0.00000001").unwrap());
        assert_eq!(migrated.as_minor_units().unwrap(), 10_000_000_000);
    }
}

#[test]
fn network_dependent_codes_require_explicit_denomination() {
    // Native variants and sources are recorded in CURRENCY_DENOMINATIONS.md.
    for (code, variants) in [
        ("USDC", [(6, "0.000001"), (7, "0.0000001")]),
        ("USDT", [(6, "0.000001"), (8, "0.00000001")]),
        ("BNB", [(8, "0.00000001"), (18, "0.000000000000000001")]),
        ("AVAX", [(9, "0.000000001"), (18, "0.000000000000000001")]),
    ] {
        let currency = Currency::try_from_str(code).unwrap();
        assert!(currency_metadata(code).is_none(), "{code}");
        let expected_error = MoneyError::MetadataNotFound {
            currency: currency.clone(),
        };
        assert_eq!(currency.decimal_places(), Err(expected_error.clone()));
        assert_eq!(currency.minor_unit_scale(), Err(expected_error.clone()));
        assert_eq!(
            Money::from_minor_units(1, currency.clone()),
            Err(expected_error.clone())
        );
        assert_eq!(
            Money::new(Decimal::ONE, currency.clone()),
            Err(expected_error.clone())
        );
        assert_eq!(
            Money::from_canonical_str("1", currency.clone()),
            Err(expected_error)
        );

        // Captured scale remains sufficient for restoration without metadata.
        let first_scale = variants[0].0;
        let wire = json!({"amount": "1", "currency": code, "minor_units": first_scale});
        let captured = serde_json::from_value::<Money>(wire.clone()).unwrap();

        for (exponent, unit) in variants {
            set_currency_metadata(code, code, exponent, code, true, Locale::EnUs).unwrap();
            assert_eq!(currency.decimal_places().unwrap(), exponent);
            let minor = Money::from_minor_units(1, currency.clone()).unwrap();
            assert_eq!(minor.minor_units(), exponent);
            assert_eq!(minor.amount(), parse_decimal(unit).unwrap());
            assert_eq!(minor.as_minor_units().unwrap(), 1);

            assert_eq!(captured.minor_units(), first_scale);
            assert_eq!(
                captured.as_minor_units().unwrap(),
                10_i128.pow(u32::from(first_scale))
            );
            assert_eq!(serde_json::to_value(&captured).unwrap(), wire);
            if exponent == first_scale {
                assert_eq!(
                    serde_json::from_value::<Money>(wire.clone()).unwrap(),
                    captured
                );
            } else {
                assert!(serde_json::from_value::<Money>(wire.clone()).is_err());
                assert!(matches!(
                    captured.try_add(&minor),
                    Err(MoneyError::MinorUnitMismatch { .. })
                ));
            }
            clear_currency_metadata(code);
            assert!(currency_metadata(code).is_none());
        }
    }
}
