// Pedagogical example — shape and clarity are intentional.
#![allow(
    clippy::unnecessary_wraps,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::missing_panics_doc,
    clippy::inconsistent_digit_grouping,
    clippy::similar_names
)]

//! Nested metadata propagation.
//!
//! When a parent type holds nested refactored types (e.g.
//! `GenericOrderBook<M>::asks: Vec<GenericBookLevel<M>>`), `M` is
//! propagated to the inner element. This means a single metadata type flows
//! through the whole tree — every entry in the order book carries the same
//! metadata shape, every candle in the history response, every contract in
//! the option chain, and so on.
//!
//! Run with:
//!     cargo run -p paft --example nested_metadata_propagation --features full
//!
//! What this example demonstrates:
//! 1. `GenericOrderBook<M>` propagates `M` down into each `GenericBookLevel<M>`.
//! 2. `GenericHistoryResponse<M>` propagates `M` down into each `GenericCandle<M>`.
//! 3. `GenericOptionChain<M>` propagates `M` into every contract.
//! 4. `GenericDownloadResponse<M>` propagates `M` two levels deep
//!    (`DownloadEntry` → `HistoryResponse` → `Candle`), and the
//!    `iter_by_symbol` helper still works without modification.
//! 5. `GenericCandleUpdate<M>` propagates `M` into the embedded `GenericCandle<M>`.

use chrono::{DateTime, Utc};
use paft::market::options::{
    GenericOptionChain, GenericOptionContract, OptionContractKey, OptionGreeks, OptionSide,
};
use paft::market::orderbook::{GenericBookLevel, GenericOrderBook};
use paft::market::quote::GenericQuote;
use paft::market::responses::download::{GenericDownloadEntry, GenericDownloadResponse};
use paft::market::responses::history::{
    GenericCandle, GenericCandleUpdate, GenericHistoryResponse, Ohlc, OhlcPriceBasis, PriceBasis,
};
use paft::money::IsoCurrency;
use paft::prelude::{
    Action, AssetKind, Currency, Exchange, HistoryMeta, Instrument, Interval, MarketState, Price,
    PriceAmount, QuantityAmount,
};
use paft::{Decimal, NonNegativeDecimal, Result};
use serde::{Deserialize, Serialize};

/// One metadata struct flows through every nested type in this example.
/// Pretend this came from a feed handler that stamps every event with a
/// monotonic sequence number, an arrival timestamp, and the channel name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct FeedMeta {
    /// Hardware NIC timestamp at arrival, ns since epoch.
    rx_ns: u64,
    /// Per-channel monotonic sequence number.
    seq: u64,
    /// Channel name (e.g. "L2_AAPL", "L1_OPT_AAPL").
    channel: String,
}

fn feed_meta(seq: u64, channel: &str) -> FeedMeta {
    FeedMeta {
        rx_ns: 1_700_000_000_000_000_000 + seq,
        seq,
        channel: channel.to_string(),
    }
}

fn main() -> Result<()> {
    println!("== 1. OrderBook (Vec<Entry<M>>) ==");
    order_book_propagation()?;

    println!("\n== 2. HistoryResponse (Vec<Candle<M>>) ==");
    history_propagation()?;

    println!("\n== 3. OptionChain (contracts carry M) ==");
    option_chain_propagation()?;

    println!("\n== 4. DownloadResponse (two levels deep) ==");
    download_propagation()?;

    println!("\n== 5. CandleUpdate (single nested field) ==");
    candle_update_propagation()?;

    Ok(())
}

/// `GenericOrderBook<FeedMeta>::asks` is `Vec<GenericBookLevel<FeedMeta>>`.
/// Each entry carries its own per-tick `FeedMeta`, AND the book itself has
/// one (e.g. for the snapshot timestamp).
fn order_book_propagation() -> Result<()> {
    let book: GenericOrderBook<FeedMeta> = GenericOrderBook {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )?,
        as_of: Some(ts(1_700_000_000)),
        currency: usd(),
        asks: vec![
            entry(150_50, 100, feed_meta(1, "L2_AAPL")),
            entry(150_55, 250, feed_meta(2, "L2_AAPL")),
            entry(150_60, 800, feed_meta(3, "L2_AAPL")),
        ],
        bids: vec![
            entry(150_45, 120, feed_meta(4, "L2_AAPL")),
            entry(150_40, 300, feed_meta(5, "L2_AAPL")),
        ],
        provider: feed_meta(6, "L2_AAPL_SNAPSHOT"),
    };

    println!(
        "Book snapshot provider: seq={} channel={}",
        book.provider.seq, book.provider.channel
    );
    println!(
        "Top of book ask: price={} size={} (entry seq={})",
        book.asks[0].price,
        book.asks[0].size.clone().unwrap_or_else(|| quantity(0)),
        book.asks[0].provider.seq,
    );

    // Round-trip preserves the per-entry metadata, not just the snapshot one.
    let json = serde_json::to_string(&book).unwrap();
    let parsed: GenericOrderBook<FeedMeta> = serde_json::from_str(&json).unwrap();
    assert_eq!(book, parsed);
    assert_eq!(parsed.bids[1].provider.seq, 5);
    println!("Per-entry metadata preserved through JSON ✓");
    Ok(())
}

/// `GenericHistoryResponse<FeedMeta>` propagates `M` to each candle.
/// Note that `meta: Option<HistoryMeta>` is still there for the
/// canonical timezone payload — it's a sibling of `provider: M`, not a clash.
fn history_propagation() -> Result<()> {
    let response: GenericHistoryResponse<FeedMeta> = GenericHistoryResponse {
        candles: vec![
            candle(
                1_700_000_000,
                150,
                152,
                149,
                151,
                feed_meta(101, "BARS_AAPL"),
            ),
            candle(
                1_700_000_060,
                151,
                153,
                150,
                152,
                feed_meta(102, "BARS_AAPL"),
            ),
            candle(
                1_700_000_120,
                152,
                154,
                151,
                153,
                feed_meta(103, "BARS_AAPL"),
            ),
        ],
        actions: vec![Action::Dividend {
            ts: ts(1_699_900_000),
            amount: price(0),
        }],
        price_basis: OhlcPriceBasis::uniform(PriceBasis::provider_latest_adjusted()),
        meta: Some(HistoryMeta {
            timezone: Some("America/New_York".parse().unwrap()),
            utc_offset_seconds: Some(-18_000),
        }),
        provider: feed_meta(100, "BARS_AAPL_BATCH"),
    };

    println!("Bar batch provider: seq={}", response.provider.seq);
    for c in &response.candles {
        println!(
            "  bar @ ts={} close={} per-bar seq={}",
            c.ts.timestamp(),
            c.ohlc.close,
            c.provider.seq
        );
    }

    // The provider payload `M` is serde-flattened — its keys appear at the
    // top level of the JSON, side-by-side with `meta` (the HistoryMeta object).
    let json = serde_json::to_string(&response).unwrap();
    assert!(
        json.contains(r#""meta":{"timezone""#),
        "HistoryMeta should serialize under the JSON key \"meta\"",
    );
    assert!(
        json.contains(r#""rx_ns""#),
        "provider keys should be flattened to top level"
    );
    Ok(())
}

/// Each contract in the chain propagates the same `M`.
fn option_chain_propagation() -> Result<()> {
    let chain: GenericOptionChain<FeedMeta> = GenericOptionChain {
        contracts: vec![
            option_contract(
                "AAPL241220C00150000",
                OptionSide::Call,
                150,
                true,
                feed_meta(201, "OPT_AAPL"),
            ),
            option_contract(
                "AAPL241220P00150000",
                OptionSide::Put,
                150,
                false,
                feed_meta(202, "OPT_AAPL"),
            ),
        ],
        provider: feed_meta(200, "OPT_AAPL_CHAIN"),
    };
    println!("Chain meta seq: {}", chain.provider.seq);
    let first_call = chain.calls().next().expect("example chain has a call");
    let first_put = chain.puts().next().expect("example chain has a put");
    println!(
        "First call ITM={:?} per-leg seq={}",
        first_call.in_the_money, first_call.provider.seq,
    );
    println!(
        "First put  ITM={:?} per-leg seq={}",
        first_put.in_the_money, first_put.provider.seq,
    );
    Ok(())
}

/// Two levels of nesting: download → entry → history → candle, all carrying
/// the same `M`. The existing `iter_by_symbol` zero-copy helper still works
/// — it now just returns `&GenericHistoryResponse<M>` instead of `&HistoryResponse`.
fn download_propagation() -> Result<()> {
    let download: GenericDownloadResponse<FeedMeta> = GenericDownloadResponse {
        entries: vec![
            download_entry("AAPL", AssetKind::Equity, feed_meta(301, "BATCH_AAPL")),
            download_entry("MSFT", AssetKind::Equity, feed_meta(302, "BATCH_MSFT")),
        ],
        provider: feed_meta(300, "BATCH"),
    };

    for (symbol, history) in download.iter_by_symbol() {
        println!(
            "{}: {} candle(s), batch seq={}",
            symbol,
            history.candles.len(),
            history.provider.seq,
        );
    }

    // Make sure deserialise round-trips the deepest leaf.
    let json = serde_json::to_string(&download).unwrap();
    let parsed: GenericDownloadResponse<FeedMeta> = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed.entries[0].history.candles[0].provider.channel,
        "BATCH_AAPL"
    );
    println!("Per-candle metadata preserved across two nesting levels ✓");
    Ok(())
}

/// `GenericCandleUpdate<M>` is a streaming event that wraps a single
/// `GenericCandle<M>`. Both the event and the contained candle carry `M`.
fn candle_update_propagation() -> Result<()> {
    let update: GenericCandleUpdate<FeedMeta> = GenericCandleUpdate {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity)?,
        interval: Interval::I1m,
        candle: candle(
            1_700_000_000,
            150,
            152,
            149,
            151,
            feed_meta(401, "STREAM_AAPL_BAR"),
        ),
        is_final: false,
        provider: feed_meta(400, "STREAM_AAPL"),
    };

    println!(
        "Update meta seq={}, contained-candle meta seq={}, final={}",
        update.provider.seq, update.candle.provider.seq, update.is_final,
    );

    // The standard alias is also still streamable — same shape, just with `()`.
    let plain_quote = GenericQuote::<()> {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity)?,
        name: None,
        currency: usd(),
        price: Some(amount(150)),
        bid: None,
        ask: None,
        previous_close: None,
        day_volume: None,
        market_state: Some(MarketState::Regular),
        as_of: None,
        provider: (),
    };
    println!(
        "Standard quote (no metadata) still works: {}",
        plain_quote.instrument.symbol.as_str()
    );
    Ok(())
}

// ---- helpers ----

fn entry(price_cents: i64, size_units: i64, provider: FeedMeta) -> GenericBookLevel<FeedMeta> {
    GenericBookLevel {
        price: PriceAmount::new(Decimal::from(price_cents) / Decimal::from(100)),
        size: Some(quantity(size_units)),
        provider,
    }
}

fn non_negative(value: Decimal) -> NonNegativeDecimal {
    NonNegativeDecimal::new(value).unwrap()
}

fn candle(
    ts_secs: i64,
    open: i64,
    high: i64,
    low: i64,
    close: i64,
    provider: FeedMeta,
) -> GenericCandle<FeedMeta> {
    GenericCandle {
        ts: ts(ts_secs),
        currency: usd(),
        ohlc: Ohlc::new(amount(open), amount(high), amount(low), amount(close)),
        close_unadj: None,
        volume: Some(quantity(1_000)),
        provider,
    }
}

fn option_contract(
    symbol: &str,
    side: OptionSide,
    strike: i64,
    in_the_money: bool,
    provider: FeedMeta,
) -> GenericOptionContract<FeedMeta> {
    GenericOptionContract {
        key: OptionContractKey::new(
            Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
            side,
            price(strike),
            chrono::NaiveDate::from_ymd_opt(2024, 12, 20).unwrap(),
        )
        .with_contract_instrument(Instrument::from_symbol(symbol, AssetKind::Option).unwrap()),
        currency: usd(),
        price: Some(amount(5)),
        bid: Some(amount(4)),
        ask: Some(amount(6)),
        volume: Some(100),
        open_interest: Some(500),
        implied_volatility: Some(non_negative(Decimal::from(25) / Decimal::from(100))),
        in_the_money: Some(in_the_money),
        expiration_at: None,
        last_trade_at: None,
        greeks: Some(OptionGreeks::default()),
        provider,
    }
}

fn download_entry(
    symbol: &str,
    kind: AssetKind,
    provider: FeedMeta,
) -> GenericDownloadEntry<FeedMeta> {
    GenericDownloadEntry {
        instrument: Instrument::from_symbol(symbol, kind).unwrap(),
        history: GenericHistoryResponse {
            candles: vec![candle(1_700_000_000, 100, 102, 99, 101, provider.clone())],
            actions: vec![],
            price_basis: OhlcPriceBasis::uniform(PriceBasis::provider_latest_adjusted()),
            meta: None,
            provider: provider.clone(),
        },
        provider,
    }
}

fn price(units: i64) -> Price {
    Price::new(Decimal::from(units), usd())
}

fn amount(units: i64) -> PriceAmount {
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
