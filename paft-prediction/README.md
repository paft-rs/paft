paft-prediction
===============

Provider-neutral prediction-market identity, metadata, quotes, books, and
trades for the paft ecosystem.

[![Crates.io](https://img.shields.io/crates/v/paft-prediction)](https://crates.io/crates/paft-prediction)
[![Docs.rs](https://docs.rs/paft-prediction/badge.svg)](https://docs.rs/paft-prediction)
[![Downloads](https://img.shields.io/crates/d/paft-prediction)](https://crates.io/crates/paft-prediction)

Install
-------

Prefer the facade crate for most applications:

```toml
[dependencies]
paft = { version = "0.9.0", features = ["prediction"] }
```

Advanced (direct dependency, minimal features):

```toml
[dependencies]
paft-prediction = { version = "0.9.0", default-features = false }
```

With DataFrame integration:

```toml
[dependencies]
paft-prediction = { version = "0.9.0", default-features = false, features = ["dataframe"] }
paft-utils = { version = "0.9.0", default-features = false, features = ["dataframe"] } # trait imports for direct users
```

What's inside
-------------

- `PredictionVenue` plus role-specific opaque ids:
  `PredictionSeriesId`, `PredictionEventId`, `PredictionMarketId`, and
  `PredictionOutcomeId`. Provider ids preserve case and punctuation, trim only
  surrounding whitespace, reject whitespace/control characters, and cap at 256
  bytes.
- `PredictionEventKey`, `PredictionMarketKey`, `BinaryMarketKey`, and
  `OutcomeInstrument`: venue-namespaced identities for event containers,
  atomic markets, and tradable outcome shares/tokens/contracts.
- `PredictionEvent`, `PredictionMarket`, `BinaryMarket`, `MultiOutcomeMarket`,
  `ScalarMarket`, `EventStructure`, and `ClaimDescriptor`: metadata models that
  separate event/group context from atomic yes/no claims. `BinaryMarket`
  requires `BinaryOutcomeInstruments` so venue-native YES/NO tradable ids stay
  in the core model.
- `OutcomePrice`, `PriceTick`, `PriceGrid`, `PriceBand`,
  `ContractQuantity`, and `OutcomePayout`: compact fixed-point integer
  primitives for prices, ticks, quantities, and contextual unit payouts.
  Decimal parse/display helpers use canonical decimal text while serde remains
  compact integer micros.
- `BinaryQuote`, `PredictionQuoteLevel`, `BinaryOrderBook`, `OutcomeOrderBook`,
  `PredictionBookLevel`, and `PredictionTrade`: market-data payloads. Quotes
  allow best bid/ask quantities to be absent, while depth book levels require
  displayed quantity. `BinaryOrderBook` stores a canonical YES-view book and
  derives NO-side top-of-book values by complement.
- `PredictionError`: non-exhaustive validation error for constructors, serde,
  price-grid validation, and book-order validation.

Features
--------

- `bigdecimal`: switch the shared decimal backend from `rust_decimal` to `bigdecimal`
- `dataframe`: Polars integration for flat prediction identity types; direct users import `ToDataFrame`/`ToDataFrameVec` from `paft_utils::dataframe`

Quickstart
----------

```rust
use paft_prediction::{
    BinaryMarketKey, BinaryOrderBook, BinaryOutcomeInstruments, ContractQuantity,
    OutcomeInstrument, OutcomePrice, PredictionBookLevel, PredictionError,
};

fn run() -> Result<(), PredictionError> {
    let kalshi_yes =
        OutcomeInstrument::new("KALSHI", "KXHIGHNY-24JAN01-T60", "YES")?;
    assert_eq!(kalshi_yes.to_string(), "KALSHI:KXHIGHNY-24JAN01-T60/YES");
    let kalshi_key = BinaryMarketKey::new("KALSHI", "KXHIGHNY-24JAN01-T60")?;
    assert_eq!(
        kalshi_key.synthetic_no_instrument().outcome_id.as_str(),
        "NO"
    );

    let polymarket_market =
        "0x5eed579ff6763914d78a966c83473ba2485ac8910d0a0914eef6d9fcb33085de";
    let polymarket_outcomes = BinaryOutcomeInstruments::new(
        OutcomeInstrument::new(
            "POLYMARKET",
            polymarket_market,
            "73470541315377973562501025254719659796416871135081220986683321361000395461644",
        )?,
        OutcomeInstrument::new(
            "POLYMARKET",
            polymarket_market,
            "56393761733830483601097051857899348522495376869600726893014309766300892311293",
        )?,
    )?;
    assert_ne!(
        polymarket_outcomes.yes().unique_key(),
        polymarket_outcomes.no().unique_key()
    );

    let mut book = BinaryOrderBook::new(kalshi_key);
    book.yes_bids.push(PredictionBookLevel::new(
        OutcomePrice::from_micros(410_000)?,
        ContractQuantity::from_microcontracts(2_000_000),
    ));
    book.yes_asks.push(PredictionBookLevel::new(
        OutcomePrice::from_micros(430_000)?,
        ContractQuantity::from_microcontracts(3_000_000),
    ));

    assert_eq!(book.best_no_bid().unwrap().price.micros(), 570_000);
    assert_eq!(book.yes_spread().unwrap().micros(), 20_000);

    Ok(())
}

run().unwrap();
```

Prediction notes
----------------

- `PredictionEvent` is a container/grouping. `BinaryMarket` is the atomic
  yes/no claim. `OutcomeInstrument` is the tradable outcome share/token/contract.
- Provider-native identifiers are opaque and venue-namespaced; do not infer
  relationships by parsing tickers, slugs, or condition ids.
- Polymarket-specific mechanics such as negative risk belong in
  `EventStructure`/provider metadata, not as universal fields on every market.

Links
-----

- API docs: [docs.rs/paft-prediction](https://docs.rs/paft-prediction)
- Workspace overview: [GitHub: workspace README](https://github.com/paft-rs/paft/blob/main/README.md)
- License: [LICENSE](../LICENSE)
