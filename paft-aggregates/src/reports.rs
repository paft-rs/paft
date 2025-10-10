use super::Info;
use paft_market::responses::history::HistoryResponse;
use paft_market::responses::search::SearchResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InfoReport {
    pub symbol: String,
    pub info: Option<Info>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SearchReport {
    pub response: Option<SearchResponse>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DownloadReport {
    pub history: Option<HistoryResponse>,
    pub warnings: Vec<String>,
}
