#![cfg(feature = "dataframe")]
use chrono::{TimeZone, Utc};
use iso_currency::Currency as IsoCurrency;
use paft_domain::{Isin, Period};
use paft_fundamentals::{
    analysis::{
        AnalysisSummary, Earnings, EarningsEstimate, EarningsQuarter, EarningsQuarterEps,
        EarningsTrendRow, EarningsYear, EpsRevisions, EpsTrend, PriceTarget, RecommendationAction,
        RecommendationGrade, RecommendationRow, RecommendationSummary, RevenueEstimate,
        RevisionPoint, TrendPoint, UpgradeDowngradeRow,
    },
    esg::{EsgInvolvement, EsgScores},
    holders::{
        InsiderPosition, InsiderRosterHolder, InsiderTransaction, InstitutionalHolder, MajorHolder,
        NetSharePurchaseActivity, TransactionType,
    },
    profile::{Address, CompanyProfile, FundKind, FundProfile, Profile, ShareCount},
    statements::{BalanceSheetRow, Calendar, CashflowRow, IncomeStatementRow},
};
use paft_money::{Decimal, Money};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

fn usd(amount: i64) -> Money {
    Money::new(
        Decimal::from(amount),
        paft_money::Currency::Iso(IsoCurrency::USD),
    )
    .unwrap()
}

fn sample_ts(secs: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(secs, 0).unwrap()
}

#[test]
fn earnings_to_dataframe() {
    let earnings = Earnings {
        yearly: vec![EarningsYear {
            year: 2024,
            revenue: Some(usd(1200)),
            earnings: Some(usd(450)),
        }],
        quarterly: vec![EarningsQuarter {
            period: Period::Quarter {
                year: 2024,
                quarter: 1,
            },
            revenue: Some(usd(300)),
            earnings: Some(usd(110)),
        }],
        quarterly_eps: vec![EarningsQuarterEps {
            period: Period::Quarter {
                year: 2024,
                quarter: 1,
            },
            actual: Some(usd(2)),
            estimate: Some(usd(1)),
        }],
    };

    let df = earnings.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn earnings_year_to_dataframe() {
    let e = EarningsYear {
        year: 2024,
        revenue: None,
        earnings: Some(usd(10)),
    };
    let df = e.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn earnings_quarter_to_dataframe() {
    let quarter = EarningsQuarter {
        period: Period::Quarter {
            year: 2024,
            quarter: 2,
        },
        revenue: Some(usd(250)),
        earnings: Some(usd(75)),
    };

    let df = quarter.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn earnings_quarter_eps_to_dataframe() {
    let eps = EarningsQuarterEps {
        period: Period::Quarter {
            year: 2024,
            quarter: 2,
        },
        actual: Some(usd(3)),
        estimate: Some(usd(2)),
    };

    let df = eps.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn price_target_to_dataframe() {
    let target = PriceTarget {
        mean: Some(usd(180)),
        high: Some(usd(220)),
        low: Some(usd(150)),
        number_of_analysts: Some(25),
    };

    let df = target.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn recommendation_row_to_dataframe() {
    let r = RecommendationRow {
        period: Period::Year { year: 2024 },
        strong_buy: Some(5),
        buy: Some(7),
        hold: Some(3),
        sell: Some(1),
        strong_sell: Some(0),
    };
    let df = r.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn recommendation_summary_to_dataframe() {
    let summary = RecommendationSummary {
        latest_period: Some(Period::Quarter {
            year: 2024,
            quarter: 2,
        }),
        strong_buy: Some(5),
        buy: Some(7),
        hold: Some(3),
        sell: Some(1),
        strong_sell: Some(0),
        mean: Some(1.5),
        mean_rating_text: Some("Outperform".to_string()),
    };

    let df = summary.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn upgrade_downgrade_row_to_dataframe() {
    let row = UpgradeDowngradeRow {
        ts: sample_ts(1_700_000_000),
        firm: Some("Analyst Firm".to_string()),
        from_grade: Some(RecommendationGrade::Hold),
        to_grade: Some(RecommendationGrade::Buy),
        action: Some(RecommendationAction::Upgrade),
    };

    let df = row.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn analysis_summary_to_dataframe() {
    let summary = AnalysisSummary {
        target_mean_price: Some(usd(200)),
        target_high_price: Some(usd(250)),
        target_low_price: Some(usd(150)),
        number_of_analyst_opinions: Some(15),
        recommendation_mean: Some(1.2),
        recommendation_text: Some("Buy".to_string()),
    };

    let df = summary.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn earnings_estimate_to_dataframe() {
    let estimate = EarningsEstimate {
        avg: Some(usd(3)),
        low: Some(usd(2)),
        high: Some(usd(4)),
        year_ago_eps: Some(usd(1)),
        num_analysts: Some(10),
        growth: Some(0.15),
    };

    let df = estimate.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn revenue_estimate_to_dataframe() {
    let estimate = RevenueEstimate {
        avg: Some(usd(1_000)),
        low: Some(usd(900)),
        high: Some(usd(1_100)),
        year_ago_revenue: Some(usd(800)),
        num_analysts: Some(12),
        growth: Some(0.2),
    };

    let df = estimate.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn trend_point_to_dataframe() {
    let point = TrendPoint::new(
        Period::Quarter {
            year: 2023,
            quarter: 4,
        },
        usd(2),
    );

    let df = point.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn eps_trend_to_dataframe() {
    let trend = EpsTrend {
        current: Some(usd(3)),
        historical: vec![TrendPoint::new(Period::Year { year: 2022 }, usd(2))],
    };

    let df = trend.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn revision_point_to_dataframe() {
    let point = RevisionPoint::new(
        Period::Quarter {
            year: 2023,
            quarter: 4,
        },
        4,
        1,
    );

    let df = point.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn eps_revisions_to_dataframe() {
    let revisions = EpsRevisions {
        historical: vec![RevisionPoint::new(Period::Year { year: 2023 }, 5, 2)],
    };

    let df = revisions.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn earnings_trend_row_to_dataframe() {
    let earnings_estimate = EarningsEstimate {
        avg: Some(usd(3)),
        low: Some(usd(2)),
        high: Some(usd(4)),
        year_ago_eps: Some(usd(1)),
        num_analysts: Some(10),
        growth: Some(0.15),
    };
    let revenue_estimate = RevenueEstimate {
        avg: Some(usd(1_000)),
        low: Some(usd(900)),
        high: Some(usd(1_100)),
        year_ago_revenue: Some(usd(800)),
        num_analysts: Some(12),
        growth: Some(0.2),
    };
    let eps_trend = EpsTrend {
        current: Some(usd(3)),
        historical: vec![TrendPoint::new(
            Period::Quarter {
                year: 2023,
                quarter: 4,
            },
            usd(2),
        )],
    };
    let eps_revisions = EpsRevisions {
        historical: vec![RevisionPoint::new(
            Period::Quarter {
                year: 2023,
                quarter: 4,
            },
            4,
            1,
        )],
    };

    let row = EarningsTrendRow {
        period: Period::Quarter {
            year: 2024,
            quarter: 1,
        },
        growth: Some(0.12),
        earnings_estimate,
        revenue_estimate,
        eps_trend,
        eps_revisions,
    };

    let df = row.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn statements_row_to_dataframe() {
    let row = IncomeStatementRow {
        period: Period::Year { year: 2024 },
        total_revenue: None,
        gross_profit: None,
        operating_income: None,
        net_income: None,
    };
    let df = row.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn balance_sheet_row_to_dataframe() {
    let row = BalanceSheetRow {
        period: Period::Year { year: 2024 },
        total_assets: Some(usd(5_000)),
        total_liabilities: Some(usd(3_000)),
        total_equity: Some(usd(2_000)),
        cash: Some(usd(500)),
        long_term_debt: Some(usd(1_200)),
        shares_outstanding: Some(1_000_000),
    };

    let df = row.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn cashflow_row_to_dataframe() {
    let row = CashflowRow {
        period: Period::Year { year: 2024 },
        operating_cashflow: Some(usd(1_200)),
        capital_expenditures: Some(usd(300)),
        free_cash_flow: Some(usd(900)),
        net_income: Some(usd(700)),
    };

    let df = row.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn calendar_to_dataframe() {
    let calendar = Calendar {
        earnings_dates: vec![sample_ts(1_700_000_000)],
        ex_dividend_date: Some(sample_ts(1_700_086_400)),
        dividend_payment_date: Some(sample_ts(1_700_172_800)),
    };

    let df = calendar.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn major_holder_to_dataframe() {
    let holder = MajorHolder {
        category: "% Held by Insiders".to_string(),
        value: 0.255,
    };

    let df = holder.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn institutional_holder_to_dataframe() {
    let holder = InstitutionalHolder {
        holder: "Example Fund".to_string(),
        shares: Some(10_000),
        date_reported: sample_ts(1_600_000_000),
        pct_held: Some(0.12),
        value: Some(usd(1_200)),
    };

    let df = holder.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn insider_transaction_to_dataframe() {
    let transaction = InsiderTransaction {
        insider: "Jane Doe".to_string(),
        position: InsiderPosition::Director,
        transaction_type: TransactionType::Buy,
        shares: Some(1_500),
        value: Some(usd(200)),
        transaction_date: sample_ts(1_650_000_000),
        url: "https://example.com/filing".to_string(),
    };

    let df = transaction.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn insider_roster_holder_to_dataframe() {
    let holder = InsiderRosterHolder {
        name: "John Smith".to_string(),
        position: InsiderPosition::Officer,
        most_recent_transaction: TransactionType::Sell,
        latest_transaction_date: sample_ts(1_660_000_000),
        shares_owned_directly: Some(5_000),
        position_direct_date: sample_ts(1_659_000_000),
    };

    let df = holder.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn net_share_purchase_activity_to_dataframe() {
    let activity = NetSharePurchaseActivity {
        period: Period::Quarter {
            year: 2023,
            quarter: 4,
        },
        buy_shares: Some(2_000),
        buy_count: Some(10),
        sell_shares: Some(1_500),
        sell_count: Some(8),
        net_shares: Some(500),
        net_count: Some(2),
        total_insider_shares: Some(20_000),
        net_percent_insider_shares: Some(0.025),
    };

    let df = activity.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn address_to_dataframe() {
    let address = Address {
        street1: Some("1 Infinite Loop".to_string()),
        street2: None,
        city: Some("Cupertino".to_string()),
        state: Some("CA".to_string()),
        country: Some("US".to_string()),
        zip: Some("95014".to_string()),
    };

    let df = address.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn company_profile_to_dataframe() {
    let profile = CompanyProfile {
        name: "Apple Inc.".to_string(),
        sector: Some("Technology".to_string()),
        industry: Some("Consumer Electronics".to_string()),
        website: Some("https://apple.com".to_string()),
        address: None,
        summary: Some("Designs and markets consumer electronics.".to_string()),
        isin: Some(Isin::new("US0378331005").unwrap()),
    };

    let df = profile.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn fund_profile_to_dataframe() {
    let profile = FundProfile {
        name: "Index Fund".to_string(),
        family: Some("Example Funds".to_string()),
        kind: FundKind::IndexFund,
        isin: Some(Isin::new("US4642872000").unwrap()),
    };

    let df = profile.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn share_count_to_dataframe() {
    let shares = ShareCount {
        date: sample_ts(1_600_000_000),
        shares: 1_000_000,
    };

    let df = shares.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn esg_scores_to_dataframe() {
    let scores = EsgScores {
        environmental: Some(55.0),
        social: Some(60.5),
        governance: Some(70.2),
    };

    let df = scores.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn esg_involvement_vec_to_dataframe() {
    let involvement = [
        EsgInvolvement {
            category: "Thermal Coal".to_string(),
            score: Some(0.1),
        },
        EsgInvolvement {
            category: "Renewables".to_string(),
            score: Some(0.8),
        },
    ];

    let df = involvement.as_slice().to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
}

#[test]
fn profile_vec_to_dataframe() {
    let company = Profile::Company(CompanyProfile {
        name: "Apple Inc.".to_string(),
        sector: Some("Technology".to_string()),
        industry: Some("Consumer Electronics".to_string()),
        website: Some("https://apple.com".to_string()),
        address: Some(Address {
            street1: Some("1 Infinite Loop".to_string()),
            street2: None,
            city: Some("Cupertino".to_string()),
            state: Some("CA".to_string()),
            country: Some("US".to_string()),
            zip: Some("95014".to_string()),
        }),
        summary: Some("Designs and markets consumer electronics.".to_string()),
        isin: Some(Isin::new("US0378331005").unwrap()),
    });

    let fund = Profile::Fund(FundProfile {
        name: "Index Fund".to_string(),
        family: Some("Example Funds".to_string()),
        kind: FundKind::IndexFund,
        isin: Some(Isin::new("US4642872000").unwrap()),
    });

    let profiles = [company, fund];
    let df = profiles.as_slice().to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    let columns = df.get_column_names();
    assert!(columns.iter().any(|c| c.as_str() == "profile_type"));
}
