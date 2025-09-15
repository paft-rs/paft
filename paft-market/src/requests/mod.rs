//! Request namespaces consolidating request builders and parameters.

pub mod history;
pub mod news;
pub mod search;

pub use history::{HistoryRequest, HistoryRequestBuilder, Interval, Range};
pub use search::SearchRequest;
