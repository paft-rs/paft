//! Corporate action types under the `paft_market::market::action` namespace.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
use paft_money::Money;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Corporate action attached to a history series.
pub enum Action {
    /// Cash dividend.
    Dividend {
        /// Timestamp.
        #[serde(with = "chrono::serde::ts_seconds")]
        ts: DateTime<Utc>,
        /// Amount paid per share.
        amount: Money,
    },
    /// Stock split.
    Split {
        /// Timestamp.
        #[serde(with = "chrono::serde::ts_seconds")]
        ts: DateTime<Utc>,
        /// Split numerator.
        numerator: u32,
        /// Split denominator.
        denominator: u32,
    },
    /// Capital gain distribution.
    CapitalGain {
        /// Timestamp.
        #[serde(with = "chrono::serde::ts_seconds")]
        ts: DateTime<Utc>,
        /// Distribution amount.
        gain: Money,
    },
}
