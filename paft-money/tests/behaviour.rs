use iso_currency::Currency as IsoCurrency;
use paft_money::{Currency, Decimal, ExchangeRate, Money, RoundingStrategy, decimal};
use serde_json::{from_value, json, to_value};

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

const fn jpy() -> Currency {
    Currency::Iso(IsoCurrency::JPY)
}

const fn eth() -> Currency {
    Currency::ETH
}

const fn xmr() -> Currency {
    Currency::XMR
}

fn dec(lit: &str) -> Decimal {
    decimal::parse_decimal(lit).expect("valid decimal literal")
}

#[test]
fn money_new_quantizes_to_currency_scale() {
    paft_money::set_currency_metadata("TOK", "Token", 4).unwrap();
    let currency = Currency::try_from_str("TOK").unwrap();

    let money = Money::new(dec("1.23456"), currency.clone()).unwrap();
    assert_eq!(money.amount(), dec("1.2346"));

    let money = Money::new(dec("1.23454"), currency).unwrap();
    assert_eq!(money.amount(), dec("1.2345"));

    paft_money::clear_currency_metadata("TOK");
}

#[test]
fn money_format_is_stable() {
    let money = Money::from_str("123456.789", usd()).unwrap();
    assert_eq!(money.format(), "123456.79 USD");

    let negative = Money::from_str("-0.5", usd()).unwrap();
    assert_eq!(negative.format(), "-0.5 USD");
}

#[test]
fn money_arithmetic_same_currency() {
    let a = Money::from_str("10.00", usd()).unwrap();
    let b = Money::from_str("2.50", usd()).unwrap();

    assert_eq!(a.try_add(&b).unwrap().amount(), dec("12.5"));
    assert_eq!(a.try_sub(&b).unwrap().amount(), dec("7.5"));
    assert_eq!(a.try_mul(dec("1.1")).unwrap().amount(), dec("11.0"));
    assert_eq!(a.try_div(dec("2")).unwrap().amount(), dec("5"));
}

#[test]
fn money_arithmetic_currency_mismatch_errors() {
    let usd = Money::from_str("10.00", usd()).unwrap();
    let eur = Money::from_str("10.00", Currency::Iso(IsoCurrency::EUR)).unwrap();

    assert!(usd.try_add(&eur).is_err());
    assert!(usd.try_sub(&eur).is_err());
}

#[test]
fn panicking_ops_mirror_try_semantics() {
    #[cfg(feature = "panicking-money-ops")]
    {
        let a = Money::from_str("10.00", usd()).unwrap();
        let b = Money::from_str("5.00", usd()).unwrap();
        assert_eq!((&a + &b).amount(), dec("15.0"));

        let other = Money::from_str("1.00", Currency::Iso(IsoCurrency::EUR)).unwrap();
        let result = std::panic::catch_unwind(|| &a + &other);
        assert!(result.is_err());
    }
}

#[test]
fn exchange_rate_validation_and_inverse() {
    assert!(ExchangeRate::new(usd(), usd(), dec("1.0")).is_err());
    assert!(ExchangeRate::new(usd(), jpy(), decimal::zero()).is_err());

    let rate = ExchangeRate::new(usd(), jpy(), dec("110.0")).unwrap();
    let inverse = rate.inverse();
    let product = rate.rate() * inverse.rate();
    let one = decimal::one();
    // Allow for rounding differences in the backend by rounding to 6 decimals.
    let rounded =
        decimal::round_dp_with_strategy(&product, 6, RoundingStrategy::MidpointAwayFromZero);
    assert_eq!(
        rounded,
        decimal::round_dp_with_strategy(&one, 6, RoundingStrategy::MidpointAwayFromZero)
    );
}

#[test]
fn exchange_rate_conversion_respects_target_scale() {
    let usd_to_jpy = ExchangeRate::new(usd(), jpy(), dec("110.001")).unwrap();
    let usd_money = Money::from_str("1.00", usd()).unwrap();
    let jpy_money = usd_money.try_convert(&usd_to_jpy).unwrap();
    assert_eq!(jpy_money.currency(), &jpy());
    assert_eq!(jpy_money.amount(), dec("110"));

    let custom_rate = ExchangeRate::new(jpy(), usd(), dec("0.0089")).unwrap();
    let jpy_value = Money::from_str("1", jpy()).unwrap();
    let rounding = jpy_value
        .try_convert_with(&custom_rate, RoundingStrategy::ToZero)
        .unwrap();
    assert_eq!(rounding.amount(), dec("0.00"));
}

#[test]
fn exchange_rate_conversion_handles_boundary_scales() {
    let eth_money = Money::from_str("1.000000000000000001", eth()).unwrap();
    let rate = ExchangeRate::new(eth(), usd(), dec("2000.123456")).unwrap();
    let usd_money = eth_money.try_convert(&rate).unwrap();
    assert_eq!(usd_money.currency(), &usd());

    let xmr_money = Money::from_str("0.123456789012", xmr()).unwrap();
    let rate = ExchangeRate::new(xmr(), usd(), dec("150.123456")).unwrap();
    let converted = xmr_money.try_convert(&rate).unwrap();
    assert_eq!(converted.currency(), &usd());
}

#[test]
fn serde_roundtrips_money_and_exchange_rate() {
    let money = Money::from_str("123.45", usd()).unwrap();
    let value = to_value(&money).unwrap();
    assert_eq!(value, json!({"amount": "123.45", "currency": "USD"}));
    let parsed: Money = from_value(value).unwrap();
    assert_eq!(parsed, money);

    let rate = ExchangeRate::new(usd(), jpy(), dec("110.0")).unwrap();
    let value = to_value(&rate).unwrap();
    assert_eq!(value["rate"], json!("110.0"));
    let parsed: ExchangeRate = from_value(value).unwrap();
    assert_eq!(parsed, rate);
}

#[test]
fn currency_decimal_places_fallback_and_metadata() {
    let xau = Currency::Iso(IsoCurrency::XAU);
    assert!(xau.decimal_places().is_err());
    paft_money::set_currency_metadata("XAU", "Gold", 3).unwrap();
    assert_eq!(xau.decimal_places().unwrap(), 3);
    paft_money::clear_currency_metadata("XAU");
}

#[test]
fn currency_minor_unit_scale_limits() {
    let usd = usd();
    assert_eq!(usd.minor_unit_scale().unwrap(), 100);

    let eth_scale = Currency::ETH.minor_unit_scale().unwrap();
    assert_eq!(eth_scale, 1_000_000_000_000_000_000);

    assert!(
        Currency::try_from_str("OVER18")
            .map(|currency| currency.minor_unit_scale())
            .unwrap()
            .is_err()
    );
}

#[test]
fn money_equality_after_quantization() {
    let a = Money::from_str("1.230", usd()).unwrap();
    let b = Money::from_str("1.23", usd()).unwrap();
    assert_eq!(a, b);
}
