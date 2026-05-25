#[cfg(feature = "market")]
#[test]
fn market_exports_are_available_from_facade_and_prelude() {
    use chrono::DateTime;
    use paft::market::{
        NewsRequest as FacadeNewsRequest, NewsTab as FacadeNewsTab,
        OptionExpirationsResponse as FacadeOptionExpirationsResponse,
        OptionGreeks as FacadeOptionGreeks, OptionUpdate as FacadeOptionUpdate,
    };
    use paft::prelude::{
        AssetKind, Instrument, NewsRequest as PreludeNewsRequest, NewsTab as PreludeNewsTab,
        OptionExpirationsResponse as PreludeOptionExpirationsResponse,
        OptionGreeks as PreludeOptionGreeks, OptionUpdate as PreludeOptionUpdate,
    };

    let greeks: FacadeOptionGreeks = PreludeOptionGreeks::default();
    let _: PreludeOptionGreeks = greeks;

    let expirations: FacadeOptionExpirationsResponse = PreludeOptionExpirationsResponse::default();
    let _: PreludeOptionExpirationsResponse = expirations;

    let tab: FacadeNewsTab = PreludeNewsTab::default();
    let news: FacadeNewsRequest = PreludeNewsRequest { count: 10, tab };
    let _: PreludeNewsRequest = news;

    let update: FacadeOptionUpdate = PreludeOptionUpdate::new(
        Instrument::from_symbol("AAPL", AssetKind::Equity).unwrap(),
        DateTime::from_timestamp(0, 0).unwrap(),
    );
    let _: PreludeOptionUpdate = update;
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
