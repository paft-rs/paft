use paft_money::decimal;
use paft_money::{Decimal, RoundingStrategy};

fn round(input: &str, scale: u32, strategy: RoundingStrategy) -> Decimal {
    let value = decimal::parse_decimal(input).expect("valid decimal literal");
    decimal::round_dp_with_strategy(&value, scale, strategy)
}

fn expect_round(input: &str, scale: u32, strategy: RoundingStrategy, expected: &str) {
    let rounded = round(input, scale, strategy);
    let expected = decimal::parse_decimal(expected).expect("expected literal");
    assert_eq!(
        rounded, expected,
        "strategy {strategy:?} failed for {input}@{scale}"
    );
}

#[test]
fn midpoint_rounding_parity() {
    // Positive midpoint cases.
    expect_round("1.005", 2, RoundingStrategy::MidpointNearestEven, "1.00");
    expect_round("1.005", 2, RoundingStrategy::MidpointAwayFromZero, "1.01");
    expect_round("1.005", 2, RoundingStrategy::MidpointTowardZero, "1.00");
    expect_round("1.005", 2, RoundingStrategy::ToZero, "1.00");
    expect_round("1.005", 2, RoundingStrategy::AwayFromZero, "1.01");
    expect_round("1.005", 2, RoundingStrategy::ToPositiveInfinity, "1.01");
    expect_round("1.005", 2, RoundingStrategy::ToNegativeInfinity, "1.00");

    // Negative midpoint cases mirror the strategy behaviour.
    expect_round("-1.005", 2, RoundingStrategy::MidpointNearestEven, "-1.00");
    expect_round("-1.005", 2, RoundingStrategy::MidpointAwayFromZero, "-1.01");
    expect_round("-1.005", 2, RoundingStrategy::MidpointTowardZero, "-1.00");
    expect_round("-1.005", 2, RoundingStrategy::ToZero, "-1.00");
    expect_round("-1.005", 2, RoundingStrategy::AwayFromZero, "-1.01");
    expect_round("-1.005", 2, RoundingStrategy::ToPositiveInfinity, "-1.00");
    expect_round("-1.005", 2, RoundingStrategy::ToNegativeInfinity, "-1.01");
}

#[test]
fn non_midpoint_rounding_boundaries() {
    expect_round("0.0049", 2, RoundingStrategy::MidpointAwayFromZero, "0.00");
    expect_round("0.0051", 2, RoundingStrategy::MidpointAwayFromZero, "0.01");
    expect_round(
        "-0.0049",
        2,
        RoundingStrategy::MidpointAwayFromZero,
        "-0.00",
    );
    expect_round(
        "-0.0051",
        2,
        RoundingStrategy::MidpointAwayFromZero,
        "-0.01",
    );

    // Check boundaries near zero for asymmetric rounding strategies.
    expect_round("0.0049", 2, RoundingStrategy::AwayFromZero, "0.01");
    expect_round("-0.0049", 2, RoundingStrategy::AwayFromZero, "-0.01");

    // Large magnitudes still obey the scale.
    expect_round(
        "12345678901234567890.555",
        2,
        RoundingStrategy::MidpointAwayFromZero,
        "12345678901234567890.56",
    );

    // Very small magnitudes keep precision.
    expect_round(
        "0.00000000000000000000000123456",
        24,
        RoundingStrategy::ToZero,
        "0.000000000000000000000001",
    );
}
