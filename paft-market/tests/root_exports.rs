#[test]
fn generic_metadata_types_are_root_exports() {
    fn assert_export<T>() {}

    assert_export::<paft_market::GenericBookLevel>();
    assert_export::<paft_market::GenericCandle>();
    assert_export::<paft_market::GenericCandleUpdate>();
    assert_export::<paft_market::GenericDownloadEntry>();
    assert_export::<paft_market::GenericDownloadResponse>();
    assert_export::<paft_market::GenericHistoryResponse>();
    assert_export::<paft_market::GenericNewsArticle>();
    assert_export::<paft_market::GenericOptionChain>();
    assert_export::<paft_market::GenericOptionContract>();
    assert_export::<paft_market::GenericOptionUpdate>();
    assert_export::<paft_market::GenericOrderBook>();
    assert_export::<paft_market::GenericQuote>();
    assert_export::<paft_market::GenericQuoteUpdate>();
    assert_export::<paft_market::GenericSearchResponse>();
    assert_export::<paft_market::GenericSearchResult>();
    assert_export::<paft_market::AdjustmentAnchor>();
    assert_export::<paft_market::AdjustmentMethod>();
    assert_export::<paft_market::CorporateActionAdjustmentCause>();
    assert_export::<paft_market::CorporateActionAdjustmentCauses>();
    assert_export::<paft_market::HistoryFlags>();
    assert_export::<paft_market::Ohlc>();
    assert_export::<paft_market::OhlcPriceBasis>();
    assert_export::<paft_market::PriceBasis>();
    assert_export::<paft_market::SearchRequestBuilder>();
    assert_export::<paft_market::DownloadEntry>();
}

#[test]
fn generic_metadata_types_are_responses_and_market_exports() {
    fn assert_export<T>() {}

    assert_export::<paft_market::market::GenericBookLevel>();
    assert_export::<paft_market::market::GenericNewsArticle>();
    assert_export::<paft_market::market::GenericOptionChain>();
    assert_export::<paft_market::market::GenericOptionContract>();
    assert_export::<paft_market::market::GenericOptionUpdate>();
    assert_export::<paft_market::market::GenericOrderBook>();
    assert_export::<paft_market::market::GenericQuote>();
    assert_export::<paft_market::market::GenericQuoteUpdate>();

    assert_export::<paft_market::responses::GenericCandle>();
    assert_export::<paft_market::responses::GenericCandleUpdate>();
    assert_export::<paft_market::responses::GenericDownloadEntry>();
    assert_export::<paft_market::responses::GenericDownloadResponse>();
    assert_export::<paft_market::responses::GenericHistoryResponse>();
    assert_export::<paft_market::responses::GenericSearchResponse>();
    assert_export::<paft_market::responses::GenericSearchResult>();
    assert_export::<paft_market::responses::DownloadEntry>();
    assert_export::<paft_market::responses::AdjustmentAnchor>();
    assert_export::<paft_market::responses::AdjustmentMethod>();
    assert_export::<paft_market::responses::CorporateActionAdjustmentCause>();
    assert_export::<paft_market::responses::CorporateActionAdjustmentCauses>();
    assert_export::<paft_market::responses::Ohlc>();
    assert_export::<paft_market::responses::OhlcPriceBasis>();
    assert_export::<paft_market::responses::PriceBasis>();

    assert_export::<paft_market::requests::HistoryFlags>();
    assert_export::<paft_market::requests::SearchRequestBuilder>();
}

#[test]
fn standard_payload_aliases_implement_eq() {
    fn assert_eq_impl<T: Eq>() {}

    assert_eq_impl::<paft_market::BookLevel>();
    assert_eq_impl::<paft_market::Candle>();
    assert_eq_impl::<paft_market::CandleUpdate>();
    assert_eq_impl::<paft_market::DownloadEntry>();
    assert_eq_impl::<paft_market::DownloadResponse>();
    assert_eq_impl::<paft_market::HistoryResponse>();
    assert_eq_impl::<paft_market::NewsArticle>();
    assert_eq_impl::<paft_market::Ohlc>();
    assert_eq_impl::<paft_market::OptionChain>();
    assert_eq_impl::<paft_market::OptionContract>();
    assert_eq_impl::<paft_market::OptionUpdate>();
    assert_eq_impl::<paft_market::OrderBook>();
    assert_eq_impl::<paft_market::Quote>();
    assert_eq_impl::<paft_market::QuoteUpdate>();
    assert_eq_impl::<paft_market::SearchResponse>();
    assert_eq_impl::<paft_market::SearchResult>();
}

#[test]
fn generic_payloads_accept_partial_eq_only_metadata() {
    fn assert_partial_eq_impl<T: PartialEq>() {}

    assert_partial_eq_impl::<paft_market::GenericQuote<f64>>();
    assert_partial_eq_impl::<paft_market::GenericHistoryResponse<f64>>();
}
