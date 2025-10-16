#![cfg(feature = "dataframe")]
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::UTC as TzUtc;
use iso_currency::Currency as IsoCurrency;
use paft_domain::{Exchange, Symbol};
use paft_market::{
    market::{
        news::NewsArticle,
        options::{OptionChain, OptionContract, OptionGreeks},
        quote::Quote,
    },
    responses::history::{Candle, HistoryMeta},
};
use paft_money::{Currency, Decimal, Money};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};

fn usd(amount: i64) -> Money {
    Money::new(Decimal::from(amount), Currency::Iso(IsoCurrency::USD)).unwrap()
}

fn sample_ts(secs: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(secs, 0).unwrap()
}

#[test]
fn vec_quote_to_dataframe_smoke() {
    let quotes = [Quote {
        symbol: Symbol::new("AAPL").unwrap(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(usd(150)),
        previous_close: Some(usd(147)),
        exchange: Some(Exchange::NASDAQ),
        market_state: None,
    }];

    let df = quotes.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "symbol"));
}

#[test]
fn quote_to_dataframe_smoke() {
    let quote = Quote {
        symbol: Symbol::new("AAPL").unwrap(),
        shortname: Some("Apple Inc.".to_string()),
        price: Some(usd(150)),
        previous_close: Some(usd(147)),
        exchange: Some(Exchange::NASDAQ),
        market_state: None,
    };

    let df = quote.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "symbol"));
    assert_eq!(df.height(), 1);
}

#[test]
fn quote_update_to_dataframe_smoke() {
    use paft_market::market::quote::QuoteUpdate;
    let update = QuoteUpdate {
        symbol: Symbol::new("AAPL").unwrap(),
        price: Some(usd(150)),
        previous_close: Some(usd(147)),
        ts: chrono::DateTime::from_timestamp(0, 0).unwrap(),
    };

    let df = update.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "symbol"));
}

#[test]
fn news_article_to_dataframe() {
    let article = NewsArticle {
        uuid: "article-123".to_string(),
        title: "Example Headline".to_string(),
        publisher: Some("Reuters".to_string()),
        link: Some("https://example.com/news".to_string()),
        published_at: sample_ts(1_700_000_000),
    };

    let df = article.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn option_greeks_to_dataframe() {
    let greeks = OptionGreeks {
        delta: Some(0.5),
        gamma: Some(0.01),
        theta: Some(-0.1),
        vega: Some(0.2),
        rho: Some(0.05),
    };

    let df = greeks.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

fn sample_contract() -> OptionContract {
    OptionContract {
        contract_symbol: Symbol::new("AAPL240621C00150000").unwrap(),
        strike: usd(150),
        price: Some(usd(5)),
        bid: Some(usd(4)),
        ask: Some(usd(6)),
        volume: Some(1_000),
        open_interest: Some(5_000),
        implied_volatility: Some(0.25),
        in_the_money: true,
        expiration_date: NaiveDate::from_ymd_opt(2024, 6, 21).unwrap(),
        expiration_at: Some(sample_ts(1_719_196_800)),
        last_trade_at: Some(sample_ts(1_700_000_000)),
        greeks: Some(OptionGreeks {
            delta: Some(0.5),
            gamma: Some(0.02),
            theta: Some(-0.1),
            vega: Some(0.3),
            rho: Some(0.05),
        }),
    }
}

#[test]
fn option_contract_to_dataframe() {
    let contract = sample_contract();

    let df = contract.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn option_chain_to_dataframe() {
    let contract = sample_contract();
    let chain = OptionChain {
        calls: vec![contract.clone()],
        puts: vec![OptionContract {
            in_the_money: false,
            ..contract
        }],
    };

    let df = chain.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn candle_to_dataframe() {
    let candle = Candle {
        ts: sample_ts(1_700_000_000),
        open: usd(150),
        high: usd(155),
        low: usd(148),
        close: usd(152),
        close_unadj: Some(usd(151)),
        volume: Some(2_500_000),
    };

    let df = candle.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn history_meta_to_dataframe() {
    let meta = HistoryMeta {
        timezone: Some(TzUtc),
        utc_offset_seconds: Some(0),
    };

    let df = meta.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}
