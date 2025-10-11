use chrono::{TimeZone, Utc};
use paft_aggregates::{DownloadReport, Info, InfoReport, SearchReport};
use paft_domain::{AssetKind, Exchange};
use paft_market::market::action::Action;
use paft_market::responses::history::{Candle, HistoryMeta, HistoryResponse};
use paft_market::responses::search::{SearchResponse, SearchResult};
use paft_money::IsoCurrency;
use paft_money::{Currency, Money};

#[test]
fn info_report_roundtrip() {
    let report = InfoReport {
        symbol: "AAPL".into(),
        info: Some(Info {
            symbol: "AAPL".into(),
            ..Default::default()
        }),
        warnings: vec!["Incomplete dataset".into()],
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: InfoReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back, report);
}

#[test]
fn search_report_roundtrip() {
    let response = SearchResponse {
        results: vec![SearchResult {
            symbol: "AAPL".into(),
            name: Some("Apple".into()),
            exchange: Some(Exchange::NASDAQ),
            kind: AssetKind::Equity,
        }],
    };
    let report = SearchReport {
        response: Some(response),
        warnings: vec![],
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: SearchReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back, report);
}

#[test]
fn download_report_roundtrip() {
    let usd = Currency::Iso(IsoCurrency::USD);
    let candle = Candle {
        ts: Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
        open: Money::from_canonical_str("1.00", usd.clone()).unwrap(),
        high: Money::from_canonical_str("2.00", usd.clone()).unwrap(),
        low: Money::from_canonical_str("0.50", usd.clone()).unwrap(),
        close: Money::from_canonical_str("1.50", usd).unwrap(),
        close_unadj: None,
        volume: Some(1000),
    };
    let history = HistoryResponse {
        candles: vec![candle],
        actions: vec![Action::Split {
            ts: Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
            numerator: 2,
            denominator: 1,
        }],
        adjusted: true,
        meta: Some(HistoryMeta {
            timezone: None,
            utc_offset_seconds: Some(0),
        }),
    };
    let report = DownloadReport {
        history: Some(history),
        warnings: vec!["Note".into()],
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: DownloadReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back, report);
}
