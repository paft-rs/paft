use iso_currency::Currency as IsoCurrency;
use paft_fundamentals::profile::{Address, CompanyProfile, FundKind, FundProfile, Profile};
use paft_money::{Currency, Decimal, Money};
use serde_json::{from_str, to_string};
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
        isin: Some("US0000000001".into()),
    });
    assert_eq!(company.isin(), Some("US0000000001"));

    let fund = Profile::Fund(FundProfile {
        name: "Index".into(),
        family: None,
        kind: FundKind::Etf,
        isin: None,
    });
    assert_eq!(fund.isin(), None);
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
        url: "https://example.com".into(),
    };

    let json = serde_json::to_string(&tx).unwrap();
    let back: paft_fundamentals::holders::InsiderTransaction = serde_json::from_str(&json).unwrap();
    assert_eq!(tx, back);
}
