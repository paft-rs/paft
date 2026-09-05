use super::Instrument;
use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
use polars::prelude::{DataFrame, DataType, PolarsResult};
use std::borrow::Cow;

/// Borrow the identity fields and compute the key at export time so public
/// instrument mutations cannot leave a cached key or label stale.
#[derive(df_derive_macros::ToDataFrame)]
struct InstrumentRow<'a> {
    symbol: &'a str,
    exchange: Option<&'a str>,
    figi: Option<&'a str>,
    isin: Option<&'a str>,
    kind: &'a str,
    key: String,
    #[df_derive(as_str)]
    display: Cow<'a, str>,
}

impl<'a> From<&'a Instrument> for InstrumentRow<'a> {
    fn from(instrument: &'a Instrument) -> Self {
        Self {
            symbol: instrument.symbol.as_str(),
            exchange: instrument.exchange.as_ref().map(super::Exchange::code),
            figi: instrument.figi.as_ref().map(AsRef::as_ref),
            isin: instrument.isin.as_ref().map(AsRef::as_ref),
            kind: instrument.kind.code(),
            key: instrument.unique_key(),
            display: instrument.display_key(),
        }
    }
}

impl ToDataFrame for Instrument {
    fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        InstrumentRow::from(self).to_dataframe()
    }

    fn empty_dataframe() -> PolarsResult<DataFrame> {
        InstrumentRow::empty_dataframe()
    }

    fn schema() -> PolarsResult<Vec<(String, DataType)>> {
        InstrumentRow::schema()
    }
}

impl Columnar for Instrument {
    fn columnar_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
        let rows: Vec<_> = items.iter().map(InstrumentRow::from).collect();
        rows.as_slice().to_dataframe()
    }

    fn columnar_from_refs(items: &[&Self]) -> PolarsResult<DataFrame> {
        let rows: Vec<_> = items.iter().copied().map(InstrumentRow::from).collect();
        rows.as_slice().to_dataframe()
    }
}
