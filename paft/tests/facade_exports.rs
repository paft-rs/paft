#[test]
fn constrained_decimal_errors_convert_into_facade_result() {
    use paft::prelude::{Decimal, Error, NonNegativeDecimal, PositiveDecimal, Ratio, Result};

    fn non_negative(value: Decimal) -> Result<NonNegativeDecimal> {
        Ok(NonNegativeDecimal::new(value)?)
    }

    fn positive(value: Decimal) -> Result<PositiveDecimal> {
        Ok(PositiveDecimal::new(value)?)
    }

    fn ratio(value: Decimal) -> Result<Ratio> {
        Ok(Ratio::new(value)?)
    }

    assert!(non_negative(Decimal::from(0)).is_ok());
    assert!(positive(Decimal::from(1)).is_ok());
    assert!(ratio(Decimal::from(1)).is_ok());

    assert!(matches!(
        non_negative(Decimal::from(-1)),
        Err(Error::DecimalConstraint(_))
    ));
    assert!(matches!(
        positive(Decimal::from(0)),
        Err(Error::DecimalConstraint(_))
    ));
    assert!(matches!(
        ratio(Decimal::from(2)),
        Err(Error::DecimalConstraint(_))
    ));
}

#[cfg(feature = "fundamentals")]
#[test]
fn fundamentals_errors_convert_into_facade_result() {
    use paft::prelude::{Error, FundamentalsError, RecommendationGrade, Result};

    fn assert_export<T>() {}

    fn recommendation_grade(input: &str) -> Result<RecommendationGrade> {
        Ok(RecommendationGrade::try_from_str(input)?)
    }

    assert!(matches!(
        recommendation_grade("   "),
        Err(Error::Fundamentals(FundamentalsError::InvalidEnumValue {
            enum_name: "RecommendationGrade",
            ..
        }))
    ));

    assert_export::<paft::fundamentals::FundamentalsError>();
    assert_export::<paft::prelude::FundamentalsError>();
}

#[cfg(feature = "domain")]
#[test]
fn period_date_is_available_from_facade_and_prelude() {
    fn assert_export<T>() {}

    assert_export::<paft::domain::Horizon>();
    assert_export::<paft::domain::OtherHorizon>();
    assert_export::<paft::prelude::Horizon>();
    assert_export::<paft::prelude::OtherHorizon>();
    assert_export::<paft::domain::PeriodDate>();
    assert_export::<paft::prelude::PeriodDate>();
}

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
        OptionUpdate as PreludeOptionUpdate, Price, PriceAmount, QuantityAmount, Range,
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
    let quantity = QuantityAmount::from_decimal(Decimal::from(150)).unwrap();
    assert_eq!(quantity.as_decimal(), &Decimal::from(150));

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
        EpsTrend as PreludeEpsTrend, Horizon, Price, RevenueEstimate as PreludeRevenueEstimate,
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
    let trend_point: FacadeTrendPoint = PreludeTrendPoint::new(Horizon::days(30).unwrap(), price);
    let _: PreludeTrendPoint = trend_point;

    let revision_point: FacadeRevisionPoint =
        PreludeRevisionPoint::new(Horizon::months(3).unwrap(), 2, 1);
    let _: PreludeRevisionPoint = revision_point;
}
