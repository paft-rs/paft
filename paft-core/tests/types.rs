//! Tests for canonical types module

use chrono::NaiveDate;
use paft_core::domain::{MarketState, Period};

#[test]
fn test_market_state_from_string() {
    assert_eq!(MarketState::from("PRE".to_string()), MarketState::Pre);
    assert_eq!(MarketState::from("premarket".to_string()), MarketState::Pre);
    assert_eq!(
        MarketState::from("REGULAR".to_string()),
        MarketState::Regular
    );
    assert_eq!(MarketState::from("POST".to_string()), MarketState::Post);
    assert_eq!(MarketState::from("CLOSED".to_string()), MarketState::Closed);
    assert_eq!(MarketState::from("HALTED".to_string()), MarketState::Halted);
    assert_eq!(
        MarketState::from("AUCTION".to_string()),
        MarketState::Other("AUCTION".to_string())
    );
}

#[test]
fn test_market_state_methods() {
    assert!(MarketState::Pre.is_trading());
    assert!(MarketState::Regular.is_trading());
    assert!(MarketState::Post.is_trading());
    assert!(!MarketState::Closed.is_trading());
    assert!(!MarketState::Halted.is_trading());
}

#[test]
fn test_period_from_string() {
    assert_eq!(
        Period::try_from("2023Q4".to_string()).unwrap(),
        Period::Quarter {
            year: 2023,
            quarter: 4
        }
    );

    assert_eq!(
        Period::try_from("2023q1".to_string()).unwrap(),
        Period::Quarter {
            year: 2023,
            quarter: 1
        }
    );

    assert_eq!(
        Period::try_from("fy2023".to_string()).unwrap(),
        Period::Year { year: 2023 }
    );
}

#[test]
fn test_period_day_first_date_parsing() {
    let period = Period::try_from("31-12-2023".to_string()).unwrap();
    match period {
        Period::Date(dt) => {
            let expected = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
            assert_eq!(dt.date_naive(), expected);
        }
        other => panic!("Expected Period::Date variant, got {other:?}"),
    }
}

#[test]
fn test_calendar_timestamp_error_handling() {
    // Just ensure error type is accessible and used
    let invalid_dates = vec!["2023-13-01"];
    for invalid_date in invalid_dates {
        let result = Period::try_from(invalid_date.to_string());
        assert!(result.is_err());
        match result {
            Err(paft_core::error::PaftError::InvalidPeriodFormat { format }) => {
                assert_eq!(format, invalid_date);
            }
            _ => panic!("Expected InvalidPeriodFormat error"),
        }
    }
}
