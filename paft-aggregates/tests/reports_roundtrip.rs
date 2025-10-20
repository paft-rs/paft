use chrono::{TimeZone, Utc};
use paft_aggregates::{DownloadReport, Info, InfoReport, SearchReport};
use paft_domain::{AssetKind, Exchange, Instrument, Symbol};
use paft_market::market::action::Action;
use paft_market::responses::download::{DownloadEntry, DownloadResponse};
use paft_market::responses::history::{Candle, HistoryMeta, HistoryResponse};
use paft_market::responses::search::{SearchResponse, SearchResult};
use paft_money::IsoCurrency;
use paft_money::{Currency, Money};

#[test]
fn info_report_roundtrip() {
    let report = InfoReport {
        symbol: Symbol::new("AAPL").unwrap(),
        info: Some(Info {
            symbol: Symbol::new("AAPL").unwrap(),
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
            symbol: Symbol::new("AAPL").unwrap(),
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
    let base_ts = Utc.timestamp_opt(1_700_000_000, 0).unwrap();

    let candle1 = Candle {
        ts: base_ts,
        open: Money::from_canonical_str("1.00", usd.clone()).unwrap(),
        high: Money::from_canonical_str("2.00", usd.clone()).unwrap(),
        low: Money::from_canonical_str("0.50", usd.clone()).unwrap(),
        close: Money::from_canonical_str("1.50", usd.clone()).unwrap(),
        close_unadj: None,
        volume: Some(1000),
    };

    let candle2 = Candle {
        ts: base_ts,
        open: Money::from_canonical_str("10.00", usd.clone()).unwrap(),
        high: Money::from_canonical_str("12.00", usd.clone()).unwrap(),
        low: Money::from_canonical_str("9.50", usd.clone()).unwrap(),
        close: Money::from_canonical_str("11.50", usd).unwrap(),
        close_unadj: None,
        volume: Some(2000),
    };

    let aapl_history = HistoryResponse {
        candles: vec![candle1],
        actions: vec![Action::Split {
            ts: base_ts,
            numerator: 2,
            denominator: 1,
        }],
        adjusted: true,
        meta: Some(HistoryMeta {
            timezone: None,
            utc_offset_seconds: Some(0),
        }),
    };

    let msft_history = HistoryResponse {
        candles: vec![candle2],
        actions: vec![],
        adjusted: false,
        meta: None,
    };

    let entries = vec![
        DownloadEntry {
            instrument: Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
            history: aapl_history,
        },
        DownloadEntry {
            instrument: Instrument::from_symbol("MSFT", AssetKind::Equity).unwrap(),
            history: msft_history,
        },
    ];

    let response = DownloadResponse { entries };

    let report = DownloadReport {
        response: Some(response),
        warnings: vec!["Note".into()],
    };

    let json = serde_json::to_string(&report).unwrap();
    let back: DownloadReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back, report);
}

#[test]
fn download_report_roundtrip_dual_listed() {
    let usd = Currency::Iso(IsoCurrency::USD);
    let base_ts = Utc.timestamp_opt(1_700_000_000, 0).unwrap();

    let candle = Candle {
        ts: base_ts,
        open: Money::from_canonical_str("1.00", usd.clone()).unwrap(),
        high: Money::from_canonical_str("2.00", usd.clone()).unwrap(),
        low: Money::from_canonical_str("0.50", usd.clone()).unwrap(),
        close: Money::from_canonical_str("1.50", usd).unwrap(),
        close_unadj: None,
        volume: Some(1000),
    };

    let payload = HistoryResponse {
        candles: vec![candle],
        actions: vec![],
        adjusted: true,
        meta: None,
    };

    let entries = vec![
        DownloadEntry {
            instrument: Instrument::from_symbol_and_exchange(
                "AAPL",
                Exchange::NASDAQ,
                AssetKind::Equity,
            )
            .unwrap(),
            history: payload.clone(),
        },
        DownloadEntry {
            instrument: Instrument::from_symbol_and_exchange(
                "AAPL",
                Exchange::LSE,
                AssetKind::Equity,
            )
            .unwrap(),
            history: payload,
        },
    ];

    let report = DownloadReport {
        response: Some(DownloadResponse { entries }),
        warnings: vec![],
    };

    let json = serde_json::to_string(&report).unwrap();
    let back: DownloadReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back, report);
}
