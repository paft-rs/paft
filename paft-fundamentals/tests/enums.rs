use paft_fundamentals::analysis::{RecommendationAction, RecommendationGrade};
use paft_fundamentals::holders::{InsiderPosition, TransactionType};
use paft_fundamentals::profile::FundKind;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::str::FromStr;

struct Case<T> {
    variant: T,
    canonical: &'static str,
    aliases: &'static [&'static str],
}

#[test]
fn recommendation_grade_round_trip_aliases() {
    for case in recommendation_grade_cases() {
        assert_enum_round_trip(&case);
    }
}

#[test]
fn recommendation_action_round_trip_aliases() {
    for case in recommendation_action_cases() {
        assert_enum_round_trip(&case);
    }
}

#[test]
fn transaction_type_round_trip_aliases() {
    for case in transaction_type_cases() {
        assert_enum_round_trip(&case);
    }
}

#[test]
fn insider_position_round_trip_aliases() {
    for case in insider_position_cases() {
        assert_enum_round_trip(&case);
    }
}

#[test]
fn fund_kind_round_trip_aliases() {
    for case in fund_kind_cases() {
        assert_enum_round_trip(&case);
    }
}

#[test]
fn enums_uppercase_other_variants() {
    let g = RecommendationGrade::try_from_str("market perform").unwrap();
    assert_eq!(g, RecommendationGrade::Hold);

    let a = RecommendationAction::try_from_str("affirm").unwrap();
    assert_eq!(a.to_string(), "AFFIRM");

    let t = TransactionType::try_from_str("vesting").unwrap();
    assert_eq!(t.to_string(), "VESTING");

    let p = InsiderPosition::try_from_str("chief_strategy_officer").unwrap();
    assert_eq!(p.to_string(), "CHIEF_STRATEGY_OFFICER");

    let fk = FundKind::try_from_str("interval_fund").unwrap();
    assert_eq!(fk.to_string(), "INTERVAL_FUND");
}

fn recommendation_grade_cases() -> Vec<Case<RecommendationGrade>> {
    use RecommendationGrade::*;

    vec![
        Case {
            variant: StrongBuy,
            canonical: "STRONG_BUY",
            aliases: &[],
        },
        Case {
            variant: Buy,
            canonical: "BUY",
            aliases: &[],
        },
        Case {
            variant: Hold,
            canonical: "HOLD",
            aliases: &["NEUTRAL", "MARKET_PERFORM"],
        },
        Case {
            variant: Sell,
            canonical: "SELL",
            aliases: &[],
        },
        Case {
            variant: StrongSell,
            canonical: "STRONG_SELL",
            aliases: &[],
        },
        Case {
            variant: Outperform,
            canonical: "OUTPERFORM",
            aliases: &["OVERWEIGHT"],
        },
        Case {
            variant: Underperform,
            canonical: "UNDERPERFORM",
            aliases: &["UNDERWEIGHT"],
        },
    ]
}

fn recommendation_action_cases() -> Vec<Case<RecommendationAction>> {
    use RecommendationAction::*;

    vec![
        Case {
            variant: Upgrade,
            canonical: "UPGRADE",
            aliases: &["UP"],
        },
        Case {
            variant: Downgrade,
            canonical: "DOWNGRADE",
            aliases: &["DOWN"],
        },
        Case {
            variant: Initiate,
            canonical: "INIT",
            aliases: &["INITIATED", "INITIATE"],
        },
        Case {
            variant: Maintain,
            canonical: "MAINTAIN",
            aliases: &["REITERATE"],
        },
        Case {
            variant: Resume,
            canonical: "RESUME",
            aliases: &[],
        },
        Case {
            variant: Suspend,
            canonical: "SUSPEND",
            aliases: &[],
        },
    ]
}

fn transaction_type_cases() -> Vec<Case<TransactionType>> {
    use TransactionType::*;

    vec![
        Case {
            variant: Buy,
            canonical: "BUY",
            aliases: &["PURCHASE", "ACQUISITION"],
        },
        Case {
            variant: Sell,
            canonical: "SELL",
            aliases: &["SALE", "DISPOSAL"],
        },
        Case {
            variant: Award,
            canonical: "AWARD",
            aliases: &["GRANT", "STOCK_AWARD"],
        },
        Case {
            variant: Exercise,
            canonical: "EXERCISE",
            aliases: &["OPTION_EXERCISE"],
        },
        Case {
            variant: Gift,
            canonical: "GIFT",
            aliases: &[],
        },
        Case {
            variant: Conversion,
            canonical: "CONVERSION",
            aliases: &[],
        },
    ]
}

fn insider_position_cases() -> Vec<Case<InsiderPosition>> {
    use InsiderPosition::*;

    vec![
        Case {
            variant: Officer,
            canonical: "OFFICER",
            aliases: &[],
        },
        Case {
            variant: Director,
            canonical: "DIRECTOR",
            aliases: &["BOARD_MEMBER"],
        },
        Case {
            variant: Owner,
            canonical: "OWNER",
            aliases: &["BENEFICIAL_OWNER", "10%_OWNER"],
        },
        Case {
            variant: Ceo,
            canonical: "CEO",
            aliases: &["CHIEF_EXECUTIVE_OFFICER"],
        },
        Case {
            variant: Cfo,
            canonical: "CFO",
            aliases: &["CHIEF_FINANCIAL_OFFICER"],
        },
        Case {
            variant: Coo,
            canonical: "COO",
            aliases: &["CHIEF_OPERATING_OFFICER"],
        },
        Case {
            variant: Cto,
            canonical: "CTO",
            aliases: &["CHIEF_TECHNOLOGY_OFFICER"],
        },
        Case {
            variant: President,
            canonical: "PRESIDENT",
            aliases: &[],
        },
        Case {
            variant: VicePresident,
            canonical: "VP",
            aliases: &["VICE_PRESIDENT"],
        },
        Case {
            variant: Secretary,
            canonical: "SECRETARY",
            aliases: &[],
        },
        Case {
            variant: Treasurer,
            canonical: "TREASURER",
            aliases: &[],
        },
    ]
}

fn fund_kind_cases() -> Vec<Case<FundKind>> {
    use FundKind::*;

    vec![
        Case {
            variant: Etf,
            canonical: "ETF",
            aliases: &["EXCHANGE_TRADED_FUND"],
        },
        Case {
            variant: MutualFund,
            canonical: "MUTUAL_FUND",
            aliases: &["MUTUAL"],
        },
        Case {
            variant: IndexFund,
            canonical: "INDEX_FUND",
            aliases: &["INDEX"],
        },
        Case {
            variant: ClosedEndFund,
            canonical: "CLOSED_END_FUND",
            aliases: &["CEF"],
        },
        Case {
            variant: MoneyMarketFund,
            canonical: "MONEY_MARKET_FUND",
            aliases: &["MMF"],
        },
        Case {
            variant: HedgeFund,
            canonical: "HEDGE_FUND",
            aliases: &[],
        },
        Case {
            variant: Reit,
            canonical: "REIT",
            aliases: &["REAL_ESTATE_INVESTMENT_TRUST"],
        },
        Case {
            variant: UnitInvestmentTrust,
            canonical: "UIT",
            aliases: &["UNIT_INVESTMENT_TRUST"],
        },
    ]
}

fn assert_enum_round_trip<T>(case: &Case<T>)
where
    T: Clone
        + PartialEq
        + Debug
        + ToString
        + FromStr<Err = paft_core::error::PaftError>
        + Serialize
        + DeserializeOwned,
{
    let display = case.variant.to_string();
    assert_eq!(display, case.canonical);

    let parsed = T::from_str(case.canonical).unwrap();
    assert_eq!(parsed, case.variant);

    let json = serde_json::to_string(&case.variant).unwrap();
    assert_eq!(json, format!("\"{display}\""));
    let back: T = serde_json::from_str(&json).unwrap();
    assert_eq!(back, case.variant);

    for alias in case.aliases {
        let parsed_alias = T::from_str(alias).unwrap();
        assert_eq!(parsed_alias, case.variant);
        assert_eq!(parsed_alias.to_string(), display);

        let alias_json = format!("\"{alias}\"");
        let alias_back: T = serde_json::from_str(&alias_json).unwrap();
        assert_eq!(alias_back, case.variant);
    }
}
