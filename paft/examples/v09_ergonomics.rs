//! Small v0.9 ergonomics tour.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p paft --example v09_ergonomics
//! ```
//!
//! This example intentionally avoids provider metadata so the core shapes are
//! easy to inspect: contextual price/quantity amounts, explicit history price
//! basis, validated periods, and horizon-based fundamentals helpers.

use chrono::{DateTime, Utc};
use paft::money::IsoCurrency;
use paft::prelude::{
    AssetKind, Candle, Currency, EpsRevisions, EpsTrend, Exchange, HistoryRequest, HistoryResponse,
    Instrument, Interval, MajorHolder, MarketState, Ohlc, OhlcPriceBasis, Period, Price,
    PriceAmount, PriceBasis, QuantityAmount, Quote, Range, Ratio, RevisionPoint, TrendPoint,
};
use paft::{Decimal, Result};

fn main() -> Result<()> {
    let instrument =
        Instrument::from_symbol_and_exchange("AAPL", Exchange::NASDAQ, AssetKind::Equity)?;
    let currency = Currency::Iso(IsoCurrency::USD);

    let quote = quote(instrument, currency.clone())?;
    let last = quote.price.as_ref().expect("example has a price");
    println!(
        "{} last={} {}",
        quote.instrument.display_key(),
        last,
        quote.currency,
    );

    let request = HistoryRequest::builder()
        .range(Range::M6)
        .interval(Interval::D1)
        .prefer_adjusted_prices(true)
        .build()?;
    assert!(request.prefer_adjusted_prices());

    let history = history_response(currency.clone());
    let close = history.candles[0].ohlc.close.with_currency(currency);
    println!("first close: {close}");
    assert!(matches!(
        history.price_basis,
        OhlcPriceBasis::Uniform {
            basis: PriceBasis::ProviderAdjusted { .. }
        }
    ));

    let annual = Period::annual(2024)?;
    let q4 = Period::quarterly(2024, 4)?;
    println!(
        "periods: {annual} covers {:?}..{:?}; q4 starts {:?}",
        annual.start_date(),
        annual.end_date(),
        q4.start_date(),
    );

    let eps_trend = EpsTrend::new(
        Some(price("1.20")?),
        vec![TrendPoint::try_new_str("90d", price("1.05")?)?],
    );
    assert!(eps_trend.find_by_horizon_str("90d")?.is_some());

    let revisions = EpsRevisions::new(vec![RevisionPoint::try_new_str("30d", 4, 1)?]);
    assert_eq!(
        revisions
            .find_by_horizon_str("30d")?
            .expect("30d revision point")
            .net_revisions(),
        3
    );

    let insiders = MajorHolder {
        category: "% held by insiders".to_string(),
        value: Ratio::new(Decimal::from(135) / Decimal::from(1000))?,
    };
    println!("{}: {}", insiders.category, insiders.value);

    Ok(())
}

fn quote(instrument: Instrument, currency: Currency) -> Result<Quote> {
    let mut quote = Quote::new(instrument, currency);
    quote.name = Some("Apple Inc.".to_string());
    quote.price = Some(amount_cents(19012));
    quote.previous_close = Some(amount_cents(18996));
    quote.day_volume = Some(quantity(78_900_000)?);
    quote.market_state = Some(MarketState::Regular);
    Ok(quote)
}

fn history_response(currency: Currency) -> HistoryResponse {
    HistoryResponse {
        candles: vec![Candle::new(
            ts(1_700_000_000),
            currency,
            Ohlc::new(
                PriceAmount::new(Decimal::from(189)),
                PriceAmount::new(Decimal::from(191)),
                PriceAmount::new(Decimal::from(188)),
                PriceAmount::new(Decimal::from(190)),
            ),
        )],
        actions: vec![],
        price_basis: OhlcPriceBasis::uniform(PriceBasis::provider_latest_adjusted()),
        meta: None,
        provider: (),
    }
}

fn price(value: &str) -> Result<Price> {
    Ok(Price::from_canonical_str(
        value,
        Currency::Iso(IsoCurrency::USD),
    )?)
}

fn amount_cents(cents: i64) -> PriceAmount {
    PriceAmount::new(Decimal::from(cents) / Decimal::from(100))
}

fn quantity(units: i64) -> Result<QuantityAmount> {
    Ok(QuantityAmount::from_decimal(Decimal::from(units))?)
}

const fn ts(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).expect("example timestamp is valid")
}
