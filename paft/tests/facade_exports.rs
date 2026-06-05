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
fn period_components_are_available_from_facade_and_prelude() {
    use chrono::NaiveDate;

    fn assert_export<T>() {}

    assert_export::<paft::domain::CalendarPeriod>();
    assert_export::<paft::domain::Horizon>();
    assert_export::<paft::domain::MarketState>();
    assert_export::<paft::domain::OtherHorizon>();
    assert_export::<paft::domain::OtherMarketState>();
    assert_export::<paft::domain::ReportingPeriod>();
    assert_export::<paft::prelude::CalendarPeriod>();
    assert_export::<paft::prelude::Horizon>();
    assert_export::<paft::prelude::MarketState>();
    assert_export::<paft::prelude::OtherHorizon>();
    assert_export::<paft::prelude::OtherMarketState>();
    assert_export::<paft::prelude::ReportingPeriod>();
    assert_export::<paft::domain::PeriodDate>();
    assert_export::<paft::domain::PeriodYear>();
    assert_export::<paft::domain::QuarterOfYear>();
    assert_export::<paft::prelude::PeriodDate>();
    assert_export::<paft::prelude::PeriodYear>();
    assert_export::<paft::prelude::QuarterOfYear>();

    let date = paft::domain::PeriodDate::new(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()).unwrap();
    assert_eq!(serde_json::to_string(&date).unwrap(), "\"2024-01-02\"");

    let quarter = paft::prelude::QuarterOfYear::Q2;
    assert_eq!(serde_json::to_string(&quarter).unwrap(), "\"2\"");

    assert_eq!(
        paft::domain::MAX_CANONICAL_TOKEN_LEN,
        paft::prelude::MAX_CANONICAL_TOKEN_LEN
    );
    assert_eq!(
        paft::MAX_CANONICAL_TOKEN_LEN,
        paft::money::MAX_CANONICAL_TOKEN_LEN
    );
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
    use paft::prelude::{
        AssetKind, Currency, Instrument, IsoCurrency, NewsRequest as PreludeNewsRequest,
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

    let update: FacadeOptionUpdate = PreludeOptionUpdate::new(
        key,
        Currency::Iso(IsoCurrency::USD),
        DateTime::from_timestamp(0, 0).unwrap(),
    );
    let _: PreludeOptionUpdate = update;

    let amount = PriceAmount::new(Decimal::from(150));
    let quantity = QuantityAmount::from_decimal(Decimal::from(150)).unwrap();
    assert_eq!(quantity.as_decimal(), &Decimal::from(150));

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

#[cfg(feature = "market")]
#[test]
fn history_validation_errors_convert_into_facade_result() {
    use chrono::DateTime;
    use paft::Decimal;
    use paft::market::{Candle, HistoryResponse, HistoryValidationError, Ohlc, OhlcPriceBasis};
    use paft::prelude::{Currency, Error, IsoCurrency, PriceAmount, Result};

    fn amount(value: i64) -> PriceAmount {
        PriceAmount::new(Decimal::from(value))
    }

    fn candle_at(unix_seconds: i64) -> Candle {
        Candle::new(
            DateTime::from_timestamp(unix_seconds, 0).unwrap(),
            Currency::Iso(IsoCurrency::USD),
            Ohlc::new(amount(100), amount(110), amount(95), amount(105)),
        )
    }

    fn validate(history: &HistoryResponse) -> Result<()> {
        history.validate()?;
        Ok(())
    }

    let history = HistoryResponse {
        candles: vec![candle_at(2), candle_at(1)],
        actions: vec![],
        price_basis: OhlcPriceBasis::raw(),
        meta: None,
        provider: (),
    };

    assert!(matches!(
        validate(&history),
        Err(Error::HistoryValidation(
            HistoryValidationError::CandlesNotChronological {
                previous_index: 0,
                previous_ts_millis: 2_000,
                current_index: 1,
                current_ts_millis: 1_000,
            }
        ))
    ));
}

#[cfg(feature = "aggregates")]
#[test]
fn generic_aggregate_exports_are_available_from_facade_and_prelude() {
    fn assert_export<T>() {}

    assert_export::<paft::aggregates::GenericSnapshot>();
    assert_export::<paft::prelude::GenericSnapshot>();
}

#[cfg(feature = "prediction")]
#[test]
fn prediction_exports_are_available_from_facade_and_prelude() {
    fn assert_export<T>() {}

    assert_export::<paft::prediction::PredictionVenue>();
    assert_export::<paft::prediction::PredictionEventId>();
    assert_export::<paft::prediction::PredictionMarketId>();
    assert_export::<paft::prediction::PredictionOutcomeId>();
    assert_export::<paft::prediction::BinaryMarketKey>();
    assert_export::<paft::prediction::BinaryOutcomeInstruments>();
    assert_export::<paft::prediction::OutcomeInstrument>();
    assert_export::<paft::prediction::PredictionEvent>();
    assert_export::<paft::prediction::PredictionMarket>();
    assert_export::<paft::prediction::BinaryMarket>();
    assert_export::<paft::prediction::OutcomePrice>();
    assert_export::<paft::prediction::ContractQuantity>();
    assert_export::<paft::prediction::BinaryOrderBook>();
    assert_export::<paft::prediction::PredictionTrade>();

    assert_export::<paft::prelude::PredictionVenue>();
    assert_export::<paft::prelude::PredictionEventId>();
    assert_export::<paft::prelude::PredictionMarketId>();
    assert_export::<paft::prelude::PredictionOutcomeId>();
    assert_export::<paft::prelude::BinaryMarketKey>();
    assert_export::<paft::prelude::BinaryOutcomeInstruments>();
    assert_export::<paft::prelude::OutcomeInstrument>();
    assert_export::<paft::prelude::PredictionEvent>();
    assert_export::<paft::prelude::PredictionMarket>();
    assert_export::<paft::prelude::BinaryMarket>();
    assert_export::<paft::prelude::OutcomePrice>();
    assert_export::<paft::prelude::ContractQuantity>();
    assert_export::<paft::prelude::BinaryOrderBook>();
    assert_export::<paft::prelude::PredictionTrade>();
}

#[cfg(feature = "fundamentals")]
#[test]
fn analysis_helpers_are_available_from_facade_and_prelude() {
    use paft::fundamentals::{
        EarningsEstimate as FacadeEarningsEstimate, EpsRevisions as FacadeEpsRevisions,
        EpsTrend as FacadeEpsTrend, RevenueEstimate as FacadeRevenueEstimate,
        RevisionPoint as FacadeRevisionPoint, TrendPoint as FacadeTrendPoint,
    };
    use paft::prelude::{
        Currency, EarningsEstimate as PreludeEarningsEstimate, EpsRevisions as PreludeEpsRevisions,
        EpsTrend as PreludeEpsTrend, Horizon, IsoCurrency, Price,
        RevenueEstimate as PreludeRevenueEstimate, RevisionPoint as PreludeRevisionPoint,
        TrendPoint as PreludeTrendPoint,
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
