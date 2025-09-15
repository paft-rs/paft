//! Core domain types used throughout the library.

pub mod currency;
pub mod exchange;
pub mod instrument;
pub mod market_state;
pub mod money;
pub mod period;

pub use currency::Currency;
pub use exchange::Exchange;
pub use instrument::{AssetKind, Instrument};
pub use market_state::MarketState;
pub use money::{ExchangeRate, Money, MoneyError};
pub use period::Period;
