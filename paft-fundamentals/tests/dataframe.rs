#![cfg(feature = "dataframe")]
use iso_currency::Currency as IsoCurrency;
use paft_domain::Period;
use paft_fundamentals::analysis::{EarningsYear, RecommendationRow};
use paft_fundamentals::statements::IncomeStatementRow;
use paft_money::Money;
use paft_utils::dataframe::ToDataFrame;
use rust_decimal::Decimal;

#[test]
fn earnings_year_to_dataframe() {
    let e = EarningsYear {
        year: 2024,
        revenue: None,
        earnings: Some(
            Money::new(
                Decimal::from(10),
                paft_money::Currency::Iso(IsoCurrency::USD),
            )
            .unwrap(),
        ),
    };
    let df = e.to_dataframe().unwrap();
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
fn recommendation_row_to_dataframe() {
    let r = RecommendationRow {
        period: Period::Year { year: 2024 },
        strong_buy: None,
        buy: None,
        hold: None,
        sell: None,
        strong_sell: None,
    };
    let df = r.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}
