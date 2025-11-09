//! Identifier newtypes for the paft ecosystem.

mod figi;
mod isin;
mod prediction;
mod symbol;

pub use figi::Figi;
pub use isin::Isin;
pub use prediction::{EventID, OutcomeID};
pub use symbol::Symbol;
