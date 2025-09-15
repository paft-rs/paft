use paft_fundamentals::analysis::{RecommendationAction, RecommendationGrade};
use paft_fundamentals::holders::{InsiderPosition, TransactionType};
use paft_fundamentals::profile::FundKind;
use serde_json::{from_str, to_string};

#[test]
fn fund_kind_string_roundtrip_and_other() {
    // Known variant (case-insensitive parsing, canonical to_string)
    let k: FundKind = "etf".to_string().into();
    assert_eq!(k.to_string(), "ETF");

    // Unknown → Other(UPPERCASED), and back to string preserves provided value
    let other: FundKind = "weird_fund".to_string().into();
    match other {
        FundKind::Other(ref s) => assert_eq!(s, "WEIRD_FUND"),
        _ => panic!("expected Other"),
    }
    let s: String = other.into();
    assert_eq!(s, "WEIRD_FUND");
}

#[test]
fn recommendation_grade_case_normalization_and_serde() {
    // Case-insensitive parsing
    let g: RecommendationGrade = "strong sell".to_string().into();
    assert_eq!(g.to_string(), "STRONG_SELL");

    // Serde string roundtrip
    let json = to_string(&g).unwrap();
    let back: RecommendationGrade = from_str(&json).unwrap();
    assert_eq!(g, back);
}

#[test]
fn recommendation_action_aliases_and_other() {
    // Alias mapping
    let a: RecommendationAction = "reiterate".to_string().into();
    assert_eq!(a.to_string(), "MAINTAIN");

    // Unknown goes to Other(UPPERCASED)
    let other: RecommendationAction = "foo".to_string().into();
    match other {
        RecommendationAction::Other(s) => assert_eq!(s, "FOO"),
        _ => panic!("expected Other"),
    }
}

#[test]
fn insider_position_aliases_and_roundtrip() {
    // Alias
    let p: InsiderPosition = "chief_financial_officer".to_string().into();
    assert_eq!(p.to_string(), "CFO");

    // Roundtrip via serde
    let json = to_string(&p).unwrap();
    let back: InsiderPosition = from_str(&json).unwrap();
    assert_eq!(p, back);
}

#[test]
fn transaction_type_aliases_and_other() {
    // Aliases (multiple serialize forms should map into canonical)
    let t1: TransactionType = "purchase".to_string().into();
    assert_eq!(t1.to_string(), "BUY");
    let t2: TransactionType = "stock_award".to_string().into();
    assert_eq!(t2.to_string(), "AWARD");

    // Unknown → Other
    let other: TransactionType = "special".to_string().into();
    match other {
        TransactionType::Other(s) => assert_eq!(s, "SPECIAL"),
        _ => panic!("expected Other"),
    }
}

// --- Migrated tests from paft/tests/enums.rs ---

#[test]
fn transaction_type_serialization() {
    let buy = TransactionType::Buy;
    let json = serde_json::to_string(&buy).unwrap();
    let deserialized: TransactionType = serde_json::from_str(&json).unwrap();
    assert_eq!(buy, deserialized);
    assert_eq!(json, "\"BUY\"");

    // Test Other variant
    let other = TransactionType::Other("VESTING".to_string());
    let json = serde_json::to_string(&other).unwrap();
    let deserialized: TransactionType = serde_json::from_str(&json).unwrap();
    assert_eq!(other, deserialized);
    assert_eq!(json, "\"VESTING\"");
}

#[test]
fn transaction_type_string_conversion() {
    // Test known variants
    let buy: TransactionType = "BUY".to_string().into();
    assert_eq!(buy, TransactionType::Buy);

    let sell: TransactionType = "SALE".to_string().into(); // Alternative serialization
    assert_eq!(sell, TransactionType::Sell);

    // Test unknown variant - should normalize to uppercase
    let unknown: TransactionType = "vesting".to_string().into();
    assert_eq!(unknown, TransactionType::Other("VESTING".to_string()));
}

#[test]
fn insider_position_serialization() {
    let ceo = InsiderPosition::Ceo;
    let json = serde_json::to_string(&ceo).unwrap();
    let deserialized: InsiderPosition = serde_json::from_str(&json).unwrap();
    assert_eq!(ceo, deserialized);
    assert_eq!(json, "\"CEO\"");

    // Test Other variant
    let other = InsiderPosition::Other("CHIEF_STRATEGY_OFFICER".to_string());
    let json = serde_json::to_string(&other).unwrap();
    let deserialized: InsiderPosition = serde_json::from_str(&json).unwrap();
    assert_eq!(other, deserialized);
    assert_eq!(json, "\"CHIEF_STRATEGY_OFFICER\"");
}

#[test]
fn insider_position_string_conversion() {
    // Test known variants
    let officer: InsiderPosition = "OFFICER".to_string().into();
    assert_eq!(officer, InsiderPosition::Officer);

    let cfo: InsiderPosition = "CHIEF_FINANCIAL_OFFICER".to_string().into(); // Alternative serialization
    assert_eq!(cfo, InsiderPosition::Cfo);

    // Test unknown variant - should normalize to uppercase
    let unknown: InsiderPosition = "chief_strategy_officer".to_string().into();
    assert_eq!(
        unknown,
        InsiderPosition::Other("CHIEF_STRATEGY_OFFICER".to_string())
    );
}

#[test]
fn fund_kind_serialization() {
    let etf = FundKind::Etf;
    let json = serde_json::to_string(&etf).unwrap();
    let deserialized: FundKind = serde_json::from_str(&json).unwrap();
    assert_eq!(etf, deserialized);
    assert_eq!(json, "\"ETF\"");

    // Test Other variant
    let other = FundKind::Other("INTERVAL_FUND".to_string());
    let json = serde_json::to_string(&other).unwrap();
    let deserialized: FundKind = serde_json::from_str(&json).unwrap();
    assert_eq!(other, deserialized);
    assert_eq!(json, "\"INTERVAL_FUND\"");
}

#[test]
fn fund_kind_string_conversion() {
    // Test known variants
    let etf: FundKind = "ETF".to_string().into();
    assert_eq!(etf, FundKind::Etf);

    let mutual: FundKind = "MUTUAL".to_string().into(); // Alternative serialization
    assert_eq!(mutual, FundKind::MutualFund);

    // Test unknown variant - should normalize to uppercase
    let unknown: FundKind = "interval_fund".to_string().into();
    assert_eq!(unknown, FundKind::Other("INTERVAL_FUND".to_string()));
}

#[test]
fn recommendation_grade_serialization() {
    let strong_buy = RecommendationGrade::StrongBuy;
    let json = serde_json::to_string(&strong_buy).unwrap();
    let deserialized: RecommendationGrade = serde_json::from_str(&json).unwrap();
    assert_eq!(strong_buy, deserialized);
    assert_eq!(json, "\"STRONG_BUY\"");

    // Test Other variant
    let other = RecommendationGrade::Other("MARKET_PERFORM".to_string());
    let json = serde_json::to_string(&other).unwrap();
    let deserialized: RecommendationGrade = serde_json::from_str(&json).unwrap();
    assert_eq!(other, deserialized);
    assert_eq!(json, "\"MARKET_PERFORM\"");
}

#[test]
fn recommendation_grade_string_conversion() {
    // Test known variants
    let buy: RecommendationGrade = "BUY".to_string().into();
    assert_eq!(buy, RecommendationGrade::Buy);

    let outperform: RecommendationGrade = "OVERWEIGHT".to_string().into(); // Alternative serialization
    assert_eq!(outperform, RecommendationGrade::Outperform);

    // Test unknown variant
    let unknown: RecommendationGrade = "market_perform".to_string().into();
    assert_eq!(
        unknown,
        RecommendationGrade::Other("MARKET_PERFORM".to_string())
    );
}

#[test]
fn recommendation_action_serialization() {
    let upgrade = RecommendationAction::Upgrade;
    let json = serde_json::to_string(&upgrade).unwrap();
    let deserialized: RecommendationAction = serde_json::from_str(&json).unwrap();
    assert_eq!(upgrade, deserialized);
    assert_eq!(json, "\"UPGRADE\"");

    // Test Other variant
    let other = RecommendationAction::Other("REINITIATE".to_string());
    let json = serde_json::to_string(&other).unwrap();
    let deserialized: RecommendationAction = serde_json::from_str(&json).unwrap();
    assert_eq!(other, deserialized);
    assert_eq!(json, "\"REINITIATE\"");
}

#[test]
fn recommendation_action_string_conversion() {
    // Test known variants
    let upgrade: RecommendationAction = "UPGRADE".to_string().into();
    assert_eq!(upgrade, RecommendationAction::Upgrade);

    let initiate: RecommendationAction = "INITIATED".to_string().into(); // Alternative serialization
    assert_eq!(initiate, RecommendationAction::Initiate);

    // Test unknown variant
    let unknown: RecommendationAction = "reinitiate".to_string().into();
    assert_eq!(
        unknown,
        RecommendationAction::Other("REINITIATE".to_string())
    );
}

#[test]
fn insider_transaction_with_enums() {
    use chrono::{DateTime, Utc};
    use paft_core::domain::{Currency, Money};
    use paft_fundamentals::holders::InsiderTransaction;
    use rust_decimal::Decimal;

    let transaction = InsiderTransaction {
        insider: "John Doe".to_string(),
        position: InsiderPosition::Ceo,
        transaction_type: TransactionType::Buy,
        shares: Some(1000),
        value: Some(Money::new(Decimal::new(50000, 0), Currency::USD)),
        transaction_date: DateTime::<Utc>::from_timestamp(1_609_459_200, 0).unwrap(),
        url: "https://example.com/filing".to_string(),
    };

    // Test serialization
    let json = serde_json::to_string(&transaction).unwrap();
    let deserialized: InsiderTransaction = serde_json::from_str(&json).unwrap();
    assert_eq!(transaction, deserialized);

    // Verify the enums are serialized as strings
    assert!(json.contains("\"CEO\""));
    assert!(json.contains("\"BUY\""));
}

#[test]
fn fund_profile_with_enum() {
    use paft_fundamentals::profile::FundProfile;

    let profile = FundProfile {
        name: "Vanguard S&P 500 ETF".to_string(),
        family: Some("Vanguard".to_string()),
        kind: FundKind::Etf,
        isin: Some("US9229087690".to_string()),
    };

    // Test serialization
    let json = serde_json::to_string(&profile).unwrap();
    let deserialized: FundProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(profile, deserialized);

    // Verify the enum is serialized as string
    assert!(json.contains("\"ETF\""));
}

#[test]
fn upgrade_downgrade_row_with_enums() {
    use chrono::{DateTime, Utc};
    use paft_fundamentals::analysis::UpgradeDowngradeRow;

    let row = UpgradeDowngradeRow {
        ts: DateTime::<Utc>::from_timestamp(1_609_459_200, 0).unwrap(),
        firm: Some("Goldman Sachs".to_string()),
        from_grade: Some(RecommendationGrade::Hold),
        to_grade: Some(RecommendationGrade::Buy),
        action: Some(RecommendationAction::Upgrade),
    };

    // Test serialization
    let json = serde_json::to_string(&row).unwrap();
    let deserialized: UpgradeDowngradeRow = serde_json::from_str(&json).unwrap();
    assert_eq!(row, deserialized);

    // Verify the enums are serialized as strings
    assert!(json.contains("\"HOLD\""));
    assert!(json.contains("\"BUY\""));
    assert!(json.contains("\"UPGRADE\""));
}
