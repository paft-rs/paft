//! Executable adapter examples for PAFT's normative time contract.
//! Provider field semantics are cited below; values are synthetic unless stated.

use chrono::{DateTime, FixedOffset, NaiveDate, Offset, TimeZone, Timelike, Utc};
use chrono_tz::America::New_York;
use paft_decimal::Decimal;
use paft_market::{Action, Candle, CandleUpdate, HistoryMeta, Interval, Ohlc};
use paft_money::{Currency, IsoCurrency, Price, PriceAmount};
use std::num::NonZeroU32;

fn candle(ts: DateTime<Utc>) -> Candle {
    let price = PriceAmount::new(Decimal::ONE);
    Candle::new(
        ts,
        Currency::Iso(IsoCurrency::USD),
        Ohlc::new(price.clone(), price.clone(), price.clone(), price),
    )
}

#[test]
fn provider_bar_boundaries_map_to_the_actual_window_start() {
    // Databento: ts_event is the inclusive start, in nanoseconds.
    // https://databento.com/docs/schemas-and-data-formats/ohlcv
    let ts_event = 1_499_040_000_000_000_000;
    let databento = candle(DateTime::from_timestamp_nanos(ts_event));

    // Binance klines have separate open/close times. This synthetic one-minute
    // window uses the millisecond close convention shown in their examples.
    // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints
    let open_time = 1_499_040_000_000_i64;
    let close_time = 1_499_040_059_999_i64;
    let binance = candle(DateTime::from_timestamp_millis(open_time).unwrap());
    assert_eq!(databento.ts, binance.ts);
    assert_ne!(binance.ts.timestamp_millis(), close_time);
    assert_eq!(close_time + 1 - open_time, 60_000);

    // If an adapter has only the inclusive close, recovering the start requires
    // a known actual duration; a session label or unknown window is insufficient.
    let from_close = |known_window_ms: Option<i64>| {
        let start = close_time.checked_add(1)?.checked_sub(known_window_ms?)?;
        Some(candle(DateTime::from_timestamp_millis(start)?))
    };
    assert_eq!(from_close(Some(60_000)).unwrap().ts, databento.ts);
    assert!(from_close(None).is_none());

    let instrument =
        paft_domain::Instrument::from_symbol("EXAMPLE", paft_domain::AssetKind::Equity).unwrap();
    let mut forming = CandleUpdate::new(instrument, Interval::I1m, binance, false);
    forming.candle.ohlc.close = PriceAmount::new(Decimal::TWO);
    forming.is_final = true;
    assert_eq!(forming.candle.ts, databento.ts);
}

#[test]
fn session_calendar_and_offset_reference_survive_daylight_saving() {
    // Explicit adapter context: the aggregate covers a 09:30 New York session
    // open on each supplied trading date. The date label alone cannot imply it.
    let open = |day| {
        New_York
            .with_ymd_and_hms(2026, 3, day, 9, 30, 0)
            .single()
            .unwrap()
            .with_timezone(&Utc)
    };
    let rows = [candle(open(9)), candle(open(6))]; // Deliberately unordered.
    let earliest = rows.iter().map(|row| row.ts).min().unwrap();
    let meta = HistoryMeta {
        timezone: Some(New_York),
        utc_offset_seconds: Some(i64::from(
            earliest
                .with_timezone(&New_York)
                .offset()
                .fix()
                .local_minus_utc(),
        )),
    };
    assert_eq!(meta.utc_offset_seconds, Some(-18_000));
    assert_eq!(rows[0].ts.hour(), 13);
    assert_eq!(rows[1].ts.hour(), 14);
    for row in &rows {
        let local = row.ts.with_timezone(&meta.timezone.unwrap());
        assert_eq!((local.hour(), local.minute()), (9, 30));
        // A UTC-day bar covers a different window; do not just relabel it.
        let utc_day_start = local.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        assert_ne!(row.ts, utc_day_start);
    }
    let old_offset =
        FixedOffset::east_opt(i32::try_from(meta.utc_offset_seconds.unwrap()).unwrap()).unwrap();
    assert_eq!(rows[0].ts.with_timezone(&old_offset).hour(), 8); // Wrong for March 9.
}

#[test]
fn action_mapping_uses_entitlement_or_effective_trading_date() {
    // These distinct date roles are documented in Alpaca's announcement model:
    // https://docs.alpaca.markets/us/reference/getcorporateannouncements
    let announcement = serde_json::json!({
        "declaration_date": "2026-03-02",
        "ex_date": "2026-03-06",
        "record_date": "2026-03-09",
        "payable_date": "2026-03-20"
    });
    // In this example the source confirms its split ex_date is the first date
    // trading on the new share basis. Never infer that from a legal date alone.
    let map = |source: &serde_json::Value, kind| -> Option<Action> {
        let date = source.get("ex_date")?.as_str()?.parse::<NaiveDate>().ok()?;
        let price = Price::new(Decimal::ONE, Currency::Iso(IsoCurrency::USD));
        match kind {
            "dividend" => Some(Action::Dividend {
                date,
                amount: price,
            }),
            "capital_gain" => Some(Action::CapitalGain { date, gain: price }),
            "split" => Some(Action::Split {
                date,
                numerator: NonZeroU32::new(2).unwrap(),
                denominator: NonZeroU32::new(1).unwrap(),
            }),
            _ => None,
        }
    };
    for kind in ["dividend", "capital_gain", "split"] {
        let action = map(&announcement, kind).unwrap();
        let wire = serde_json::to_value(&action).unwrap();
        assert_eq!(wire["date"], "2026-03-06");
        for other in ["declaration_date", "record_date", "payable_date"] {
            assert_ne!(wire["date"], announcement[other]);
        }
        let mut unavailable = announcement.clone();
        unavailable.as_object_mut().unwrap().remove("ex_date");
        assert!(map(&unavailable, kind).is_none());
        unavailable["ex_date"] = serde_json::Value::Null;
        assert!(map(&unavailable, kind).is_none());
    }
}
