use paft_decimal::Decimal;
use paft_domain::Isin;
use paft_fundamentals::profile::{Address, CompanyProfile, FundKind, FundProfile, Profile};
use paft_money::{Currency, IsoCurrency, Money};
use serde_json::{from_str, json, to_string, to_value};
use std::str::FromStr;

#[test]
fn profile_isin_accessor() {
    let company = Profile::Company(CompanyProfile {
        name: "ACME".into(),
        sector: None,
        industry: None,
        website: None,
        address: None,
        summary: None,
        isin: Some(Isin::new("US0378331005").unwrap()),
    });
    assert_eq!(company.isin().map(AsRef::as_ref), Some("US0378331005"));

    let fund = Profile::Fund(FundProfile {
        name: "Index".into(),
        family: None,
        kind: FundKind::Etf,
        isin: None,
    });
    assert_eq!(fund.isin(), None);
}

#[test]
fn profile_company_uses_tagged_serde_shape() {
    let profile = Profile::Company(CompanyProfile {
        name: "ACME".into(),
        sector: Some("Industrials".into()),
        industry: None,
        website: Some("https://example.com".into()),
        address: None,
        summary: None,
        isin: Some(Isin::new("US0378331005").unwrap()),
    });

    let value = to_value(&profile).unwrap();
    assert_eq!(
        value,
        json!({
            "kind": "company",
            "name": "ACME",
            "sector": "Industrials",
            "industry": null,
            "website": "https://example.com",
            "address": null,
            "summary": null,
            "isin": "US0378331005",
        })
    );

    let deserialized: Profile = serde_json::from_value(value).unwrap();
    assert_eq!(profile, deserialized);
}

#[test]
fn profile_fund_uses_tagged_serde_shape() {
    let profile = Profile::Fund(FundProfile {
        name: "Index".into(),
        family: None,
        kind: FundKind::Etf,
        isin: None,
    });

    let value = to_value(&profile).unwrap();
    assert_eq!(
        value,
        json!({
            "kind": "fund",
            "name": "Index",
            "family": null,
            "fund_kind": "ETF",
            "isin": null,
        })
    );

    let deserialized: Profile = serde_json::from_value(value).unwrap();
    assert_eq!(profile, deserialized);
}

#[test]
fn profile_rejects_unknown_fields() {
    let value = json!({
        "kind": "fund",
        "name": "Index",
        "family": null,
        "fund_kind": "ETF",
        "isin": null,
        "provider_field": true,
    });

    let err = serde_json::from_value::<Profile>(value).unwrap_err();
    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn address_serde_roundtrip() {
    let addr = Address {
        street1: Some("1 Main".into()),
        street2: None,
        city: Some("Metropolis".into()),
        state: Some("CA".into()),
        country: Some("US".into()),
        zip: Some("94000".into()),
    };
    let s = to_string(&addr).unwrap();
    let back: Address = from_str(&s).unwrap();
    assert_eq!(addr, back);
}

#[test]
fn insider_transaction_serde_with_enums_and_timestamps() {
    let tx = paft_fundamentals::holders::InsiderTransaction {
        insider: "John Doe".to_string(),
        position: paft_fundamentals::holders::InsiderPosition::Officer,
        transaction_type: paft_fundamentals::holders::TransactionType::Buy,
        shares: Some(1000),
        value: Some(
            Money::new(
                Decimal::from_str("123.45").unwrap(),
                Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
        ),
        transaction_date: chrono::DateTime::from_timestamp(1_640_995_200, 0).unwrap(),
        url: Some("https://example.com".into()),
    };

    let json = serde_json::to_string(&tx).unwrap();
    let back: paft_fundamentals::holders::InsiderTransaction = serde_json::from_str(&json).unwrap();
    assert_eq!(tx, back);
}

#[test]
fn insider_transaction_url_can_be_missing() {
    let json = r#"{
        "insider": "John Doe",
        "position": "Officer",
        "transaction_type": "Buy",
        "shares": 1000,
        "value": null,
        "transaction_date": 1640995200000
    }"#;

    let transaction: paft_fundamentals::holders::InsiderTransaction =
        serde_json::from_str(json).unwrap();

    assert_eq!(transaction.url, None);
}
