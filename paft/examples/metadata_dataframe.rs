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
//! payload type round-trip into Polars columns via their `ToDataFrame` impls.
//! This example shows two complementary stories:
//!
//! 1. The default `M = ()` case is **completely transparent**: no extra
//!    columns appear, the schema is identical to what you'd have without the
//!    metadata escape hatch.
//!
//! 2. A custom `M` that itself derives `ToDataFrame` via `df-derive` is
//!    **flattened** into
//!    extra columns, prefixed with `provider.` (the parent field's name).
//!    So adding HFT timestamps does not require changing your downstream
//!    Polars pipeline — the metadata columns just appear alongside the
//!    canonical ones.
//!
//! Run with:
//!     cargo run -p paft --example metadata_dataframe --features full
//!
//! Note: this example only compiles with `--features full` (or any feature
//! set that enables `dataframe` together with `market`).

use chrono::{DateTime, Utc};
use paft::market::quote::{GenericQuote, Quote};
use paft::market::responses::history::{
    GenericCandle, GenericHistoryResponse, Ohlc, OhlcPriceBasis, PriceBasis,
};
use paft::money::IsoCurrency;
use paft::prelude::{
    AssetKind, Currency, Exchange, Instrument, MarketState, PriceAmount, QuantityAmount,
    ToDataFrame, ToDataFrameVec,
};
use paft::{Decimal, Result};
use serde::{Deserialize, Serialize};

/// Minimal HFT metadata struct that itself derives `ToDataFrame`. Each field
/// becomes its own column in the resulting `DataFrame` (under a `provider.`
/// prefix when nested inside a bigger payload — `provider` is the field name
/// used by the parent `GenericQuote<M>` / `GenericCandle<M>` etc.).
///
/// Custom payload structs should derive dataframe support through
/// `df-derive` directly and target paft's runtime traits.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, df_derive_macros::ToDataFrame,
)]
#[df_derive(trait = "::paft::dataframe::ToDataFrame")]
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

/// With `M = ()`, the `provider` field contributes zero columns. The schema
/// is the canonical no-metadata quote schema — no provider columns are added
/// for downstream pipelines.
fn standard_quote_schema() -> Result<()> {
    let q = Quote {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )?,
        name: Some("Apple Inc.".to_string()),
        currency: usd(),
        price: Some(price(150)),
        previous_close: Some(price(147)),
        day_volume: Some(quantity(78_900_000)),
        market_state: Some(MarketState::Regular),
        as_of: None,
        bid: None,
        ask: None,
        provider: (),
    };
    let df = q.to_dataframe().unwrap();
    println!("columns: {:?}", df.get_column_names());
    println!("rows: {}", df.height());
    println!("\nDataFrame:\n{df}");
    Ok(())
}

/// Same shape, but with a real `HftMeta` payload. Notice the new columns:
/// `provider.rx_ns`, `provider.seq`, `provider.venue` — derived automatically
/// from `HftMeta`'s `ToDataFrame` impl.
fn enriched_quote_dataframe() -> Result<()> {
    let quotes: Vec<GenericQuote<HftMeta>> = vec![
        GenericQuote {
            instrument: Instrument::from_symbol_and_exchange(
                "AAPL",
                Exchange::NASDAQ,
                AssetKind::Equity,
            )?,
            name: Some("Apple Inc.".to_string()),
            currency: usd(),
            price: Some(price(150)),
            previous_close: Some(price(147)),
            day_volume: Some(quantity(78_900_000)),
            market_state: Some(MarketState::Regular),
            as_of: None,
            bid: None,
            ask: None,
            provider: HftMeta {
                rx_ns: 1_700_000_000_000_000_001,
                seq: 1,
                venue: "NASDAQ".into(),
            },
        },
        GenericQuote {
            instrument: Instrument::from_symbol_and_exchange(
                "MSFT",
                Exchange::NASDAQ,
                AssetKind::Equity,
            )?,
            name: Some("Microsoft".to_string()),
            currency: usd(),
            price: Some(price(420)),
            previous_close: Some(price(418)),
            day_volume: Some(quantity(20_000_000)),
            market_state: Some(MarketState::Regular),
            as_of: None,
            bid: None,
            ask: None,
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
/// its own `provider.*` columns automatically.
fn history_dataframe() -> Result<()> {
    let response: GenericHistoryResponse<HftMeta> = GenericHistoryResponse {
        candles: vec![
            mk_candle(1_700_000_000, 150, 152, 149, 151, 1),
            mk_candle(1_700_000_060, 151, 153, 150, 152, 2),
            mk_candle(1_700_000_120, 152, 154, 151, 153, 3),
        ],
        actions: vec![],
        price_basis: OhlcPriceBasis::uniform(PriceBasis::provider_latest_adjusted()),
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
        currency: usd(),
        ohlc: Ohlc::new(price(open), price(high), price(low), price(close)),
        close_unadj: None,
        volume: Some(quantity(1_000)),
        provider: HftMeta {
            rx_ns: 1_700_000_000_000_000_000 + seq,
            seq,
            venue: "AAPL".into(),
        },
    }
}

fn price(units: i64) -> PriceAmount {
    PriceAmount::new(Decimal::from(units))
}

fn quantity(units: i64) -> QuantityAmount {
    QuantityAmount::from_decimal(Decimal::from(units)).unwrap()
}

const fn usd() -> Currency {
    Currency::Iso(IsoCurrency::USD)
}

const fn ts(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap()
}
