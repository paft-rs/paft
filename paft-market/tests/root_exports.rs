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
    assert_export::<paft_market::responses::OhlcPriceBasis>();
    assert_export::<paft_market::responses::PriceBasis>();

    assert_export::<paft_market::requests::HistoryFlags>();
    assert_export::<paft_market::requests::SearchRequestBuilder>();
}
