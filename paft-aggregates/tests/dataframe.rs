#![cfg(feature = "dataframe")]
use chrono::{NaiveDate, TimeZone, Utc};
use iso_currency::Currency as IsoCurrency;
use paft_aggregates::info::{FastInfo, Info};
use paft_domain::{Exchange, Isin, MarketState, Symbol};
use paft_money::{Currency, Decimal, Money};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

fn usd(amount: i64) -> Money {
    Money::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD)).unwrap()
}

#[test]
fn fast_info_to_dataframe() {
    let info = FastInfo {
        symbol: Symbol::new("AAPL").unwrap(),
        name: Some("Apple Inc.".to_string()),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        currency: Some(Currency::Iso(IsoCurrency::USD)),
        last: Some(usd(150)),
        previous_close: Some(usd(145)),
        volume: Some(1_234_567),
    };

    let df = info.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn info_vec_to_dataframe() {
    let base = Info {
        symbol: Symbol::new("AAPL").unwrap(),
        name: Some("Apple Inc.".to_string()),
        isin: Some(Isin::new("US0378331005").unwrap()),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        currency: Some(Currency::Iso(IsoCurrency::USD)),
        last: Some(usd(150)),
        open: Some(usd(148)),
        high: Some(usd(151)),
        low: Some(usd(147)),
        previous_close: Some(usd(145)),
        day_range_low: Some(usd(147)),
        day_range_high: Some(usd(151)),
        fifty_two_week_low: Some(usd(120)),
        fifty_two_week_high: Some(usd(180)),
        volume: Some(1_000_000),
        average_volume: Some(900_000),
        market_cap: Some(usd(2_500_000)),
        shares_outstanding: Some(16_000_000),
        eps_ttm: Some(usd(6)),
        pe_ttm: Some(25.0),
        dividend_yield: Some(0.015),
        ex_dividend_date: Some(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
        price_target: None,
        recommendation_summary: None,
        esg_scores: None,
        as_of: Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap()),
    };

    let infos = vec![
        base.clone(),
        Info {
            name: Some("Alt".to_string()),
            ..base
        },
    ];
    let df = infos.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    let columns = df.get_column_names();
    assert!(columns.iter().any(|c| c.as_str() == "symbol"));
    assert!(columns.iter().any(|c| c.as_str() == "market_state"));
}
