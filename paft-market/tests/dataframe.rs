#![cfg(feature = "dataframe")]
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::UTC as TzUtc;
use paft_decimal::{Decimal, NonNegativeDecimal};
use paft_domain::{AssetKind, Exchange, Instrument};
use paft_market::{
    market::{
        action::Action,
        news::NewsArticle,
        options::{OptionChain, OptionContract, OptionContractKey, OptionGreeks, OptionSide},
        orderbook::{BookLevel, OrderBook},
        quote::Quote,
    },
    responses::{
        history::{Candle, HistoryMeta, Ohlc},
        search::SearchResult,
    },
};
use paft_money::{Currency, IsoCurrency, Price, PriceAmount, QuantityAmount};
use paft_utils::dataframe::{ToDataFrame, ToDataFrameVec};
use std::num::NonZeroU32;
use std::str::FromStr;

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

fn usd_price(amount: i64) -> Price {
    Price::new(Decimal::from(amount), usd())
}

fn usd_amount(amount: i64) -> PriceAmount {
    PriceAmount::new(Decimal::from(amount))
}

fn quantity(amount: i64) -> QuantityAmount {
    QuantityAmount::from_decimal(Decimal::from(amount)).unwrap()
}

fn candle(ts: chrono::DateTime<Utc>, open: i64, high: i64, low: i64, close: i64) -> Candle {
    Candle {
        ts,
        currency: usd(),
        ohlc: Ohlc::new(
            usd_amount(open),
            usd_amount(high),
            usd_amount(low),
            usd_amount(close),
        ),
        close_unadj: None,
        volume: Some(quantity(2_500_000)),
        provider: (),
    }
}

fn dec(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap()
}

fn non_negative(value: &str) -> NonNegativeDecimal {
    NonNegativeDecimal::new(dec(value)).unwrap()
}

fn sample_ts(secs: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(secs, 0).unwrap()
}

#[test]
fn book_level_to_dataframe_with_size() {
    let level = BookLevel {
        price: usd_amount(100),
        size: Some(quantity(500)),
        provider: (),
    };
    let df = level.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn book_level_to_dataframe_without_size() {
    let level = BookLevel {
        price: usd_amount(100),
        size: None,
        provider: (),
    };
    let df = level.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn order_book_to_dataframe_smoke() {
    let book = OrderBook {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        as_of: Some(sample_ts(1_700_000_000)),
        currency: usd(),
        asks: vec![BookLevel::new(usd_amount(101), Some(quantity(200)))],
        bids: vec![BookLevel::new(usd_amount(99), None)],
        provider: (),
    };
    let df = book.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn search_result_to_dataframe() {
    let result = SearchResult {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )
        .unwrap(),
        name: Some("Apple Inc.".to_string()),

        provider: (),
    };

    let df = result.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn vec_quote_to_dataframe_smoke() {
    let quotes = [Quote {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )
        .unwrap(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(usd_amount(150)),
        previous_close: Some(usd_amount(147)),
        day_volume: None,
        market_state: None,
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    }];

    let df = quotes.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "instrument"));
}

#[test]
fn quote_to_dataframe_smoke() {
    let quote = Quote {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )
        .unwrap(),
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(usd_amount(150)),
        previous_close: Some(usd_amount(147)),
        day_volume: None,
        market_state: None,
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };

    let df = quote.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "instrument"));
    assert_eq!(df.height(), 1);
}

#[test]
fn quote_update_to_dataframe_smoke() {
    use paft_market::market::quote::QuoteUpdate;
    let update = QuoteUpdate {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        currency: usd(),
        price: Some(usd_amount(150)),
        previous_close: Some(usd_amount(147)),
        volume_delta: None,
        ts: chrono::DateTime::from_timestamp(0, 0).unwrap(),

        provider: (),
    };

    let df = update.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "instrument"));
}

#[test]
fn news_article_to_dataframe() {
    let article = NewsArticle {
        uuid: "article-123".to_string(),
        title: "Example Headline".to_string(),
        publisher: Some("Reuters".to_string()),
        link: Some("https://example.com/news".to_string()),
        published_at: sample_ts(1_700_000_000),

        provider: (),
    };

    let df = article.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn option_greeks_to_dataframe() {
    let greeks = OptionGreeks {
        delta: Some(dec("0.5")),
        gamma: Some(dec("0.01")),
        theta: Some(dec("-0.1")),
        vega: Some(dec("0.2")),
        rho: Some(dec("0.05")),
    };

    let df = greeks.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

fn sample_contract() -> OptionContract {
    OptionContract {
        key: OptionContractKey::new(
            Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
            OptionSide::Call,
            usd_price(150),
            NaiveDate::from_ymd_opt(2024, 6, 21).unwrap(),
        ),
        contract_instrument: Some(
            Instrument::from_symbol("AAPL240621C00150000", AssetKind::Option).unwrap(),
        ),
        price: Some(usd_amount(5)),
        bid: Some(usd_amount(4)),
        ask: Some(usd_amount(6)),
        volume: Some(1_000),
        open_interest: Some(5_000),
        implied_volatility: Some(non_negative("0.25")),
        in_the_money: Some(true),
        expiration_at: Some(sample_ts(1_719_196_800)),
        last_trade_at: Some(sample_ts(1_700_000_000)),
        greeks: Some(OptionGreeks {
            delta: Some(dec("0.5")),
            gamma: Some(dec("0.02")),
            theta: Some(dec("-0.1")),
            vega: Some(dec("0.3")),
            rho: Some(dec("0.05")),
        }),
        provider: (),
    }
}

#[test]
fn option_contract_to_dataframe() {
    let contract = sample_contract();

    let df = contract.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "underlying"));
    assert!(cols.iter().any(|c| c.as_str() == "side"));
    assert!(cols.iter().any(|c| c.as_str() == "strike.amount"));
    assert!(cols.iter().any(|c| c.as_str() == "expiration_date"));
    assert!(!cols.iter().any(|c| c.as_str().starts_with("key.")));
    assert_eq!(df.height(), 1);
}

#[test]
fn option_chain_to_dataframe() {
    let contract = sample_contract();
    let chain = OptionChain {
        contracts: vec![
            contract.clone(),
            OptionContract {
                key: OptionContractKey {
                    side: OptionSide::Put,
                    ..contract.key.clone()
                },
                contract_instrument: Some(
                    Instrument::from_symbol("AAPL240621P00150000", AssetKind::Option).unwrap(),
                ),
                in_the_money: Some(false),
                ..contract
            },
        ],
        provider: (),
    };

    assert_eq!(chain.calls().count(), 1);
    assert_eq!(chain.puts().count(), 1);

    let df = chain.to_dataframe().unwrap();
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "contracts.side"));
    assert!(
        !cols
            .iter()
            .any(|c| c.as_str().starts_with("contracts.key."))
    );
    assert_eq!(df.height(), 1);
}

#[test]
fn candle_to_dataframe() {
    let candle = Candle {
        ts: sample_ts(1_700_000_000),
        currency: usd(),
        ohlc: Ohlc::new(
            usd_amount(150),
            usd_amount(155),
            usd_amount(148),
            usd_amount(152),
        ),
        close_unadj: Some(usd_amount(151)),
        volume: Some(quantity(2_500_000)),

        provider: (),
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

#[test]
fn actions_to_dataframe() {
    let actions = [
        Action::Dividend {
            ts: sample_ts(1_700_000_000),
            amount: usd_price(1),
        },
        Action::Split {
            ts: sample_ts(1_600_000_000),
            numerator: NonZeroU32::new(2).unwrap(),
            denominator: NonZeroU32::new(1).unwrap(),
        },
        Action::CapitalGain {
            ts: sample_ts(1_650_000_000),
            gain: usd_price(3),
        },
    ];

    let df = actions.to_dataframe().unwrap();
    assert_eq!(df.height(), 3);
    let columns = df.get_column_names();
    assert!(columns.iter().any(|c| c.as_str() == "action_type"));
}

#[test]
fn candle_update_to_dataframe_smoke() {
    use paft_market::Interval;
    use paft_market::responses::history::CandleUpdate;
    let update = CandleUpdate {
        instrument: paft_domain::Instrument::from_symbol("AAPL", paft_domain::AssetKind::Equity)
            .unwrap(),
        interval: Interval::I1m,
        candle: candle(sample_ts(1_700_000_000), 150, 155, 148, 152),
        is_final: false,

        provider: (),
    };

    let df = update.to_dataframe().unwrap();
    assert_eq!(df.height(), 1);
}

#[test]
fn vec_candle_update_to_dataframe_smoke() {
    use paft_market::Interval;
    use paft_market::responses::history::CandleUpdate;
    let updates = [
        CandleUpdate {
            instrument: paft_domain::Instrument::from_symbol(
                "AAPL",
                paft_domain::AssetKind::Equity,
            )
            .unwrap(),
            interval: Interval::I1m,
            candle: candle(sample_ts(1_700_000_000), 150, 155, 148, 152),
            is_final: false,

            provider: (),
        },
        CandleUpdate {
            instrument: paft_domain::Instrument::from_symbol(
                "AAPL",
                paft_domain::AssetKind::Equity,
            )
            .unwrap(),
            interval: Interval::I1m,
            candle: Candle {
                volume: Some(quantity(1_000_000)),
                ..candle(sample_ts(1_700_000_060), 152, 156, 149, 154)
            },
            is_final: true,

            provider: (),
        },
    ];

    let df = updates.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    let cols = df.get_column_names();
    assert!(cols.iter().any(|c| c.as_str() == "instrument"));
}
