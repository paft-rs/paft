// Pedagogical example — shape and clarity are intentional.
#![allow(
    clippy::unnecessary_wraps,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::missing_panics_doc
)]

//! Provider-metadata escape hatch.
//!
//! Every market data payload type in paft is now generic over a metadata
//! payload `M`. The standard public API (`Quote`, `Candle`, `OrderBook`, …)
//! is preserved as a type alias resolving `M = ()`, so existing call sites
//! continue to work unchanged. Power users — typically HFT / market-data
//! integrators — can plug in a custom `M` and get strongly-typed access to
//! provider-specific JSON fields without forking the crate.
//!
//! Run with:
//!     cargo run -p paft --example provider_metadata --features full
//!
//! What this example demonstrates:
//! 1. The standard `Quote` (i.e. `GenericQuote<()>`) keeps its old behaviour.
//! 2. A custom `HftMeta` carries provider-specific timestamps and sequence
//!    numbers.
//! 3. `#[serde(flatten)]` means the extra fields land at the **top level** of
//!    the JSON, side-by-side with the canonical fields.
//! 4. Inbound provider JSON with extra keys deserializes losslessly into
//!    `GenericQuote<HftMeta>` — no manual extraction step.
//! 5. Multiple metadata shapes can coexist in the same program (e.g. one for
//!    quotes, another for candles).

use chrono::{DateTime, Utc};
use iso_currency::Currency as IsoCurrency;
use paft::market::quote::{GenericQuote, GenericQuoteUpdate, Quote, QuoteUpdate};
use paft::prelude::{AssetKind, Currency, Exchange, Instrument, MarketState, Money};
use paft::{Decimal, Result};
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    println!("== 1. Standard Quote (M = ()) ==");
    standard_quote_unchanged()?;

    println!("\n== 2. Custom HFT metadata round-trip ==");
    hft_quote_round_trip()?;

    println!("\n== 3. Deserialize provider JSON with extra keys ==");
    parse_provider_json()?;

    println!("\n== 4. Different metadata per stream ==");
    different_meta_per_stream()?;

    Ok(())
}

/// `Quote` is `GenericQuote<()>`. Existing code keeps compiling: every field
/// is the same; the only new thing is `provider: ()`, which serialises to
/// nothing (because `()` flattens to no keys) and deserialises from any JSON
/// object regardless of unknown extra keys.
///
/// Two equivalent ways to construct the standard case:
///
/// 1. `Quote::new(instrument)` — concise, all optional fields default to
///    `None`, `provider` defaults to `()`.
/// 2. The full struct literal — useful when you want to set every field
///    explicitly and is the only option for non-`Default` `M` types.
fn standard_quote_unchanged() -> Result<()> {
    // (1) the ergonomic constructor:
    let mut quote = Quote::new(Instrument::from_symbol("AAPL", AssetKind::Equity)?);
    quote.name = Some("Apple Inc.".to_string());
    quote.price = Some(money(150));
    quote.previous_close = Some(money(147));
    quote.day_volume = Some(78_900_000);
    quote.exchange = Some(Exchange::NASDAQ);
    quote.market_state = Some(MarketState::Regular);

    // (2) equivalent full literal — note the `provider: ()` is required because
    //     struct literals must list every field. Real production code rarely
    //     needs this form for the no-metadata case.
    let _equivalent = Quote {
        instrument: Instrument::from_symbol("AAPL", AssetKind::Equity)?,
        name: Some("Apple Inc.".to_string()),
        price: Some(money(150)),
        previous_close: Some(money(147)),
        day_volume: Some(78_900_000),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: (),
    };

    let json = serde_json::to_string_pretty(&quote).unwrap();
    println!("Standard quote JSON (no extra keys, just the canonical shape):");
    println!("{json}");

    let round_trip: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, round_trip);
    println!("Round-trip OK ✓");
    Ok(())
}

/// Provider-specific HFT metadata. Anything you want — a timestamp the
/// upstream feed handler stamped on arrival, a per-exchange sequence number,
/// a wire-format flag — as long as it can `Serialize`/`Deserialize`/`Default`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct HftMeta {
    /// Hardware timestamp from the NIC, in nanoseconds since epoch.
    received_ns: u64,
    /// Per-exchange monotonic sequence number, used to detect gaps.
    exchange_seq: u64,
    /// Internal correlation id. Useful for tracing across systems.
    correlation_id: String,
}

fn hft_quote_round_trip() -> Result<()> {
    // Construct a quote enriched with HFT metadata. Everything else looks
    // identical to the standard case, except the type is the generic
    // `GenericQuote<HftMeta>` and the `provider` field carries a real value.
    let quote: GenericQuote<HftMeta> = GenericQuote {
        instrument: Instrument::from_symbol_and_exchange(
            "AAPL",
            Exchange::NASDAQ,
            AssetKind::Equity,
        )?,
        name: Some("Apple Inc.".to_string()),
        price: Some(money(150)),
        previous_close: Some(money(147)),
        day_volume: Some(78_900_000),
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: HftMeta {
            received_ns: 1_700_000_000_123_456_789,
            exchange_seq: 42_424_242,
            correlation_id: "trace-abc-1".into(),
        },
    };

    // Because of `#[serde(flatten)]` on the `provider` field, the
    // provider-specific keys appear at the top level of the JSON document —
    // alongside the canonical fields — rather than nested under a "provider"
    // object.
    let json = serde_json::to_string_pretty(&quote).unwrap();
    println!("HFT-enriched quote JSON (provider keys are flattened):");
    println!("{json}");

    let round_trip: GenericQuote<HftMeta> = serde_json::from_str(&json).unwrap();
    assert_eq!(quote, round_trip);
    println!("HFT round-trip OK ✓");
    Ok(())
}

/// Imagine your feed handler receives this JSON from a venue. Without the
/// escape hatch you'd have to either drop the extra keys, log them as
/// `serde_json::Value`, or fork the crate. With the escape hatch they
/// deserialise straight into a typed struct.
fn parse_provider_json() -> Result<()> {
    let provider_json = serde_json::json!({
        // Canonical paft fields:
        "instrument": {
            "symbol": "AAPL",
            "exchange": "NASDAQ",
            "figi": null,
            "isin": null,
            "kind": "EQUITY"
        },
        "name": "Apple Inc.",
        "price": { "amount": "150", "currency": "USD" },
        "previous_close": { "amount": "147", "currency": "USD" },
        "day_volume": 78900000,
        "exchange": "NASDAQ",
        "market_state": "REGULAR",
        // Provider-specific fields — flattened next to the canonical ones:
        "received_ns": 1_700_000_000_123_456_789u64,
        "exchange_seq": 42_424_242,
        "correlation_id": "trace-abc-1"
    });

    let typed: GenericQuote<HftMeta> = serde_json::from_value(provider_json).unwrap();
    println!(
        "Parsed: symbol={}, received_ns={}, seq={}, correlation_id={}",
        typed.instrument.symbol.as_str(),
        typed.provider.received_ns,
        typed.provider.exchange_seq,
        typed.provider.correlation_id,
    );
    Ok(())
}

/// Different streams can carry different metadata in the same program.
/// Here a quote stream uses `HftMeta`, while an order-update stream uses
/// `BrokerMeta`. The two types coexist without interfering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct BrokerMeta {
    /// Internal account this update is scoped to.
    account_id: String,
    /// True when the update was generated by a synthetic order (e.g. a
    /// smart-order-router child slice) rather than a real venue acknowledgement.
    synthetic: bool,
}

fn different_meta_per_stream() -> Result<()> {
    // Quote stream: HFT-flavoured metadata.
    let market_data: GenericQuote<HftMeta> = GenericQuote {
        instrument: Instrument::from_symbol("MSFT", AssetKind::Equity)?,
        name: Some("Microsoft".to_string()),
        price: Some(money(420)),
        previous_close: Some(money(418)),
        day_volume: None,
        exchange: Some(Exchange::NASDAQ),
        market_state: Some(MarketState::Regular),
        bid: None,
        ask: None,
        provider: HftMeta {
            received_ns: 1_700_000_000_222_333_444,
            exchange_seq: 99,
            correlation_id: "msft-quote-1".into(),
        },
    };
    println!(
        "Market-data update: {} @ {} (seq={})",
        market_data.instrument.symbol.as_str(),
        market_data.price.as_ref().unwrap(),
        market_data.provider.exchange_seq,
    );

    // Same code base, different stream: broker-flavoured metadata on quote
    // updates rather than on quotes themselves.
    let broker_update: GenericQuoteUpdate<BrokerMeta> = GenericQuoteUpdate {
        instrument: Instrument::from_symbol("MSFT", AssetKind::Equity)?,
        price: Some(money(421)),
        previous_close: Some(money(418)),
        volume: Some(100),
        ts: ts(1_700_000_000),
        provider: BrokerMeta {
            account_id: "ACC-7".into(),
            synthetic: true,
        },
    };
    println!(
        "Broker update: {} for account {} (synthetic={})",
        broker_update.instrument.symbol.as_str(),
        broker_update.provider.account_id,
        broker_update.provider.synthetic,
    );

    // The standard alias still works for code paths that don't care about
    // metadata at all.
    let plain: QuoteUpdate = QuoteUpdate {
        instrument: Instrument::from_symbol("MSFT", AssetKind::Equity)?,
        price: Some(money(421)),
        previous_close: Some(money(418)),
        volume: Some(100),
        ts: ts(1_700_000_000),
        provider: (),
    };
    println!(
        "Plain update (no metadata): {} @ {}",
        plain.instrument.symbol.as_str(),
        plain.price.as_ref().unwrap(),
    );
    Ok(())
}

fn money(units: i64) -> Money {
    Money::new(Decimal::from(units), Currency::Iso(IsoCurrency::USD)).unwrap()
}

const fn ts(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap()
}
