#[cfg(feature = "market")]
#[test]
fn market_exports_are_available_from_facade_and_prelude() {
    use chrono::DateTime;
    use paft::Decimal;
    use paft::market::{
        NewsRequest as FacadeNewsRequest, NewsTab as FacadeNewsTab,
        OptionContractKey as FacadeOptionContractKey,
        OptionExpirationsResponse as FacadeOptionExpirationsResponse,
        OptionGreeks as FacadeOptionGreeks, OptionSide as FacadeOptionSide,
        OptionUpdate as FacadeOptionUpdate, SearchRequestBuilder as FacadeSearchRequestBuilder,
        TimeSpec as FacadeTimeSpec,
    };
    use paft::money::IsoCurrency;
    use paft::prelude::{
        AssetKind, Currency, Instrument, NewsRequest as PreludeNewsRequest,
        NewsTab as PreludeNewsTab, Ohlc as PreludeOhlc,
        OptionContractKey as PreludeOptionContractKey,
        OptionExpirationsResponse as PreludeOptionExpirationsResponse,
        OptionGreeks as PreludeOptionGreeks, OptionSide as PreludeOptionSide,
        OptionUpdate as PreludeOptionUpdate, Price, PriceAmount, Range,
        SearchRequestBuilder as PreludeSearchRequestBuilder, TimeSpec as PreludeTimeSpec,
    };
    use std::num::NonZeroU32;

    let greeks: FacadeOptionGreeks = PreludeOptionGreeks::default();
    let _: PreludeOptionGreeks = greeks;

    let expirations: FacadeOptionExpirationsResponse = PreludeOptionExpirationsResponse::default();
    let _: PreludeOptionExpirationsResponse = expirations;

    let tab: FacadeNewsTab = PreludeNewsTab::default();
    let news: FacadeNewsRequest = PreludeNewsRequest {
        count: NonZeroU32::new(10).unwrap(),
        tab,
    };
    let _: PreludeNewsRequest = news;

    let time_spec: FacadeTimeSpec = PreludeTimeSpec::Range(Range::M1);
    let _: PreludeTimeSpec = time_spec;

    let search: FacadeSearchRequestBuilder = PreludeSearchRequestBuilder::new("AAPL");
    let _: PreludeSearchRequestBuilder = search;

    let key: FacadeOptionContractKey = PreludeOptionContractKey::new(
        Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        PreludeOptionSide::Call,
        Price::new(Decimal::from(150), Currency::Iso(IsoCurrency::USD)),
        chrono::NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
    );
    let key: PreludeOptionContractKey = key;
    let side: FacadeOptionSide = PreludeOptionSide::Put;
    let _: PreludeOptionSide = side;

    let update: FacadeOptionUpdate =
        PreludeOptionUpdate::new(key, DateTime::from_timestamp(0, 0).unwrap());
    let _: PreludeOptionUpdate = update;

    let amount = PriceAmount::new(Decimal::from(150));

    #[cfg(not(feature = "bigdecimal"))]
    let _: PreludeOhlc = PreludeOhlc::new(amount, amount, amount, amount);

    #[cfg(feature = "bigdecimal")]
    let _: PreludeOhlc = PreludeOhlc::new(amount.clone(), amount.clone(), amount.clone(), amount);
}

#[cfg(feature = "market")]
#[test]
fn generic_market_exports_are_available_from_facade_and_prelude() {
    fn assert_export<T>() {}

    assert_export::<paft::market::GenericBookLevel>();
    assert_export::<paft::market::GenericCandle>();
    assert_export::<paft::market::GenericCandleUpdate>();
    assert_export::<paft::market::GenericDownloadEntry>();
    assert_export::<paft::market::GenericDownloadResponse>();
    assert_export::<paft::market::GenericHistoryResponse>();
    assert_export::<paft::market::GenericNewsArticle>();
    assert_export::<paft::market::GenericOptionChain>();
    assert_export::<paft::market::GenericOptionContract>();
    assert_export::<paft::market::GenericOptionUpdate>();
    assert_export::<paft::market::GenericOrderBook>();
    assert_export::<paft::market::GenericQuote>();
    assert_export::<paft::market::GenericQuoteUpdate>();
    assert_export::<paft::market::GenericSearchResponse>();
    assert_export::<paft::market::GenericSearchResult>();
    assert_export::<paft::market::AdjustmentAnchor>();
    assert_export::<paft::market::AdjustmentMethod>();
    assert_export::<paft::market::CorporateActionAdjustmentCause>();
    assert_export::<paft::market::CorporateActionAdjustmentCauses>();
    assert_export::<paft::market::HistoryFlags>();
    assert_export::<paft::market::Ohlc>();
    assert_export::<paft::market::OhlcPriceBasis>();
    assert_export::<paft::market::PriceBasis>();
    assert_export::<paft::market::SearchRequestBuilder>();
    assert_export::<paft::market::DownloadEntry>();

    assert_export::<paft::prelude::GenericBookLevel>();
    assert_export::<paft::prelude::GenericCandle>();
    assert_export::<paft::prelude::GenericCandleUpdate>();
    assert_export::<paft::prelude::GenericDownloadEntry>();
    assert_export::<paft::prelude::GenericDownloadResponse>();
    assert_export::<paft::prelude::GenericHistoryResponse>();
    assert_export::<paft::prelude::GenericNewsArticle>();
    assert_export::<paft::prelude::GenericOptionChain>();
    assert_export::<paft::prelude::GenericOptionContract>();
    assert_export::<paft::prelude::GenericOptionUpdate>();
    assert_export::<paft::prelude::GenericOrderBook>();
    assert_export::<paft::prelude::GenericQuote>();
    assert_export::<paft::prelude::GenericQuoteUpdate>();
    assert_export::<paft::prelude::GenericSearchResponse>();
    assert_export::<paft::prelude::GenericSearchResult>();
    assert_export::<paft::prelude::AdjustmentAnchor>();
    assert_export::<paft::prelude::AdjustmentMethod>();
    assert_export::<paft::prelude::CorporateActionAdjustmentCause>();
    assert_export::<paft::prelude::CorporateActionAdjustmentCauses>();
    assert_export::<paft::prelude::HistoryFlags>();
    assert_export::<paft::prelude::Ohlc>();
    assert_export::<paft::prelude::OhlcPriceBasis>();
    assert_export::<paft::prelude::PriceBasis>();
    assert_export::<paft::prelude::SearchRequestBuilder>();
    assert_export::<paft::prelude::DownloadEntry>();
}

#[cfg(feature = "aggregates")]
#[test]
fn generic_aggregate_exports_are_available_from_facade_and_prelude() {
    fn assert_export<T>() {}

    assert_export::<paft::aggregates::GenericSnapshot>();
    assert_export::<paft::prelude::GenericSnapshot>();
}

#[cfg(feature = "fundamentals")]
#[test]
fn analysis_helpers_are_available_from_facade_and_prelude() {
    use paft::fundamentals::{
        EarningsEstimate as FacadeEarningsEstimate, EpsRevisions as FacadeEpsRevisions,
        EpsTrend as FacadeEpsTrend, RevenueEstimate as FacadeRevenueEstimate,
        RevisionPoint as FacadeRevisionPoint, TrendPoint as FacadeTrendPoint,
    };
    use paft::money::IsoCurrency;
    use paft::prelude::{
        Currency, EarningsEstimate as PreludeEarningsEstimate, EpsRevisions as PreludeEpsRevisions,
        EpsTrend as PreludeEpsTrend, Period, Price, RevenueEstimate as PreludeRevenueEstimate,
        RevisionPoint as PreludeRevisionPoint, TrendPoint as PreludeTrendPoint,
    };

    let earnings_estimate: FacadeEarningsEstimate = PreludeEarningsEstimate::default();
    let _: PreludeEarningsEstimate = earnings_estimate;

    let revenue_estimate: FacadeRevenueEstimate = PreludeRevenueEstimate::default();
    let _: PreludeRevenueEstimate = revenue_estimate;

    let eps_trend: FacadeEpsTrend = PreludeEpsTrend::default();
    let _: PreludeEpsTrend = eps_trend;

    let eps_revisions: FacadeEpsRevisions = PreludeEpsRevisions::default();
    let _: PreludeEpsRevisions = eps_revisions;

    let price = Price::from_canonical_str("1.00", Currency::Iso(IsoCurrency::USD)).unwrap();
    let trend_point: FacadeTrendPoint = PreludeTrendPoint::new(Period::Year { year: 2024 }, price);
    let _: PreludeTrendPoint = trend_point;

    let revision_point: FacadeRevisionPoint =
        PreludeRevisionPoint::new(Period::Year { year: 2024 }, 2, 1);
    let _: PreludeRevisionPoint = revision_point;
}
