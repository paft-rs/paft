use chrono::{TimeZone, Utc};
use paft_core::domain::{Currency, Money};
use paft_fundamentals::holders::{InsiderPosition, InsiderTransaction, TransactionType};
use paft_fundamentals::profile::{Address, CompanyProfile, FundKind, FundProfile, Profile};
use rust_decimal::Decimal;
use serde_json::{from_str, json, to_string};

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
    let tx = InsiderTransaction {
        insider: "Jane Doe".into(),
        position: InsiderPosition::Cfo,
        transaction_type: TransactionType::Buy,
        shares: Some(1000),
        value: Some(Money::new(Decimal::new(12345, 2), Currency::USD)),
        transaction_date: Utc.with_ymd_and_hms(2024, 9, 1, 0, 0, 0).unwrap(),
        url: "https://example.com".into(),
    };
    let s = to_string(&tx).unwrap();
    // Ensure json contains expected string enums and ts_seconds
    let v: serde_json::Value = from_str(&s).unwrap();
    assert_eq!(v["position"], json!("CFO"));
    assert_eq!(v["transaction_type"], json!("BUY"));
    assert_eq!(v["transaction_date"], json!(1_725_148_800));

    let back: InsiderTransaction = from_str(&s).unwrap();
    assert_eq!(back, tx);
}
