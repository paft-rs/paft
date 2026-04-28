// Pedagogical example — shape and clarity are intentional.
#![allow(
    clippy::unnecessary_wraps,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::missing_panics_doc
)]

//! Provider metadata in Polars `DataFrame` exports.
//!
//! When the `dataframe` feature is enabled, the canonical fields of every
//! payload type round-trip into Polars columns via the `ToDataFrame` derive.
//! This example shows two complementary stories:
//!
//! 1. The default `M = ()` case is **completely transparent**: no extra
//!    columns appear, the schema is identical to what you'd have without the
//!    metadata escape hatch.
//!
//! 2. A custom `M` that itself derives `ToDataFrame` is **flattened** into
//!    extra columns, prefixed with `meta.`. So adding HFT timestamps does
//!    not require changing your downstream Polars pipeline — the metadata
//!    columns just appear alongside the canonical ones.
//!
//! Run with:
//!     cargo run -p paft --example metadata_dataframe --features full
//!
//! Note: this example only compiles with `--features full` (or any feature
//! set that enables `dataframe` together with `market`).

use chrono::{DateTime, Utc};
use df_derive::ToDataFrame;
use iso_currency::Currency as IsoCurrency;
use paft::market::quote::{GenericQuote, Quote};
use paft::market::responses::history::{GenericCandle, GenericHistoryResponse};
use paft::prelude::{
    AssetKind, Currency, Exchange, Instrument, MarketState, Money, ToDataFrame, ToDataFrameVec,
};
use paft::{Decimal, Result};
use serde::{Deserialize, Serialize};

/// Minimal HFT metadata struct that itself derives `ToDataFrame`. Each field
/// becomes its own column in the resulting `DataFrame` (under a `meta.`
/// prefix when nested inside a bigger payload).
///
/// The `#[df_derive(...)]` container attributes are only required because
/// this example lives inside the `paft` crate's `examples/` directory.
/// In a downstream crate that simply depends on `paft`, you can drop those
/// attributes and the macro will resolve `::paft::dataframe::*` automatically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, ToDataFrame)]
#[df_derive(
    trait = "::paft_utils::dataframe::ToDataFrame",
    columnar = "::paft_utils::dataframe::Columnar"
)]
struct HftMeta {
    rx_ns: u64,
    seq: u64,
    venue: String,
}

fn main() -> Result<()> {
    println!("== 1. Standard Quote DataFrame schema (M = ()) ==");
    standard_quote_schema()?;

    println!("\n== 2. Quote enriched with HftMeta ==");
    enriched_quote_dataframe()?;

    println!("\n== 3. HistoryResponse with one HftMeta per candle ==");
    history_dataframe()?;

    Ok(())
}

/// With `M = ()`, the `meta` field contributes zero columns. The schema is
/// identical to what you'd get from paft 0.7.x — no surprises for existing
/// downstream pipelines.
fn standard_quote_schema() -> Result<()> {
    let q = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity)?,
        shortname: Some("Apple Inc.".to_string()),
        price: Some(money(150)),
        previous_close: Some(money(147)),
        day_volume: Some(78_900_000),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        provider: (),
    };
    let df = q.to_dataframe().unwrap();
    println!("columns: {:?}", df.get_column_names());
    println!("rows: {}", df.height());
    println!("\nDataFrame:\n{df}");
    Ok(())
}

/// Same shape, but with a real `HftMeta` payload. Notice the new columns:
/// `meta.rx_ns`, `meta.seq`, `meta.venue` — derived automatically from
/// `HftMeta`'s `ToDataFrame` impl.
fn enriched_quote_dataframe() -> Result<()> {
    let quotes: Vec<GenericQuote<HftMeta>> = vec![
        GenericQuote {
            instrument: Instrument::from_symbol("AAPL", AssetKind::Equity)?,
            shortname: Some("Apple Inc.".to_string()),
            price: Some(money(150)),
            previous_close: Some(money(147)),
            day_volume: Some(78_900_000),
            exchange: Some(Exchange::NASDAQ),
            market_state: Some(MarketState::Regular),
            provider: HftMeta {
                rx_ns: 1_700_000_000_000_000_001,
                seq: 1,
                venue: "NASDAQ".into(),
            },
        },
        GenericQuote {
            instrument: Instrument::from_symbol("MSFT", AssetKind::Equity)?,
            shortname: Some("Microsoft".to_string()),
            price: Some(money(420)),
            previous_close: Some(money(418)),
            day_volume: Some(20_000_000),
            exchange: Some(Exchange::NASDAQ),
            market_state: Some(MarketState::Regular),
            provider: HftMeta {
                rx_ns: 1_700_000_000_000_000_002,
                seq: 2,
                venue: "NASDAQ".into(),
            },
        },
    ];

    let df = quotes.as_slice().to_dataframe().unwrap();
    println!("columns: {:?}", df.get_column_names());
    println!("rows: {}", df.height());
    println!("\nDataFrame:\n{df}");
    Ok(())
}

/// `M` flows down into `Vec<GenericCandle<M>>` too. Every candle row gets
/// its own `meta.*` columns automatically.
fn history_dataframe() -> Result<()> {
    let response: GenericHistoryResponse<HftMeta> = GenericHistoryResponse {
        candles: vec![
            mk_candle(1_700_000_000, 150, 152, 149, 151, 1),
            mk_candle(1_700_000_060, 151, 153, 150, 152, 2),
            mk_candle(1_700_000_120, 152, 154, 151, 153, 3),
        ],
        actions: vec![],
        adjusted: true,
        meta: None,
        provider: HftMeta {
            rx_ns: 1_700_000_000_000_000_000,
            seq: 0,
            venue: "AAPL_BARS".into(),
        },
    };

    let df = response.candles.as_slice().to_dataframe().unwrap();
    println!("columns: {:?}", df.get_column_names());
    println!("rows: {}", df.height());
    println!("\nDataFrame:\n{df}");
    Ok(())
}

fn mk_candle(
    ts_secs: i64,
    open: i64,
    high: i64,
    low: i64,
    close: i64,
    seq: u64,
) -> GenericCandle<HftMeta> {
    GenericCandle {
        ts: ts(ts_secs),
        open: money(open),
        high: money(high),
        low: money(low),
        close: money(close),
        close_unadj: None,
        volume: Some(1_000),
        provider: HftMeta {
            rx_ns: 1_700_000_000_000_000_000 + seq,
            seq,
            venue: "AAPL".into(),
        },
    }
}

fn money(units: i64) -> Money {
    Money::new(Decimal::from(units), Currency::Iso(IsoCurrency::USD)).unwrap()
}

const fn ts(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap()
}
