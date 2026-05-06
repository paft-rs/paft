//! Direct unit tests for `Decimal128Encode::try_to_i128_mantissa`.
//!
//! Verifies the rescale + half-even rounding contract on both the
//! `rust_decimal` and `bigdecimal` backends, plus the shared
//! `|mantissa| < 10^38` rejection band that matches polars's
//! `dec128_fits` precision check for `Decimal(38, _)` columns.

#![cfg(feature = "dataframe")]

use paft_utils::Decimal128Encode;

// 10^38 — the smallest mantissa that polars's `Decimal(38, _)` rejects.
const MAX_I128_MANTISSA: i128 = 10_i128.pow(38);

// ===================================================================
// rust_decimal backend
// ===================================================================

#[cfg(not(feature = "bigdecimal"))]
mod rust_decimal_backend {
    use super::{Decimal128Encode, MAX_I128_MANTISSA};
    use rust_decimal::Decimal;

    // ---- source_scale == target_scale ----

    #[test]
    fn same_scale_small_positive() {
        let d = Decimal::from_i128_with_scale(123_456, 3);
        assert_eq!(d.try_to_i128_mantissa(3), Some(123_456));
    }

    #[test]
    fn same_scale_small_negative() {
        let d = Decimal::from_i128_with_scale(-123_456, 3);
        assert_eq!(d.try_to_i128_mantissa(3), Some(-123_456));
    }

    #[test]
    fn same_scale_zero() {
        let d = Decimal::from_i128_with_scale(0, 4);
        assert_eq!(d.try_to_i128_mantissa(4), Some(0));
    }

    #[test]
    fn same_scale_large_mantissa() {
        // Largest mantissa rust_decimal can hold (96-bit) — well below the
        // 10^38 boundary, so it must round-trip cleanly.
        let m: i128 = 79_228_162_514_264_337_593_543_950_335; // 2^96 - 1
        let d = Decimal::from_i128_with_scale(m, 5);
        assert_eq!(d.try_to_i128_mantissa(5), Some(m));
    }

    // ---- scale-up (rescale to higher scale) ----

    #[test]
    fn scale_up_one_to_ten() {
        // 1.5 with scale=1 has mantissa 15.
        // Rescaled to scale=10 → 15 * 10^9 = 15_000_000_000.
        let d = Decimal::from_i128_with_scale(15, 1);
        assert_eq!(d.try_to_i128_mantissa(10), Some(15_000_000_000));
    }

    // ---- scale-down half-even ties ----

    #[test]
    fn scale_down_half_even_round_down_positive() {
        // 1.25 → at target_scale=1, ties to even → 1.2 → mantissa 12.
        let d = Decimal::from_i128_with_scale(125, 2);
        assert_eq!(d.try_to_i128_mantissa(1), Some(12));
    }

    #[test]
    fn scale_down_half_even_round_up_positive() {
        // 1.35 → at target_scale=1, ties to even → 1.4 → mantissa 14.
        let d = Decimal::from_i128_with_scale(135, 2);
        assert_eq!(d.try_to_i128_mantissa(1), Some(14));
    }

    #[test]
    fn scale_down_half_even_round_down_negative() {
        // -1.25 → -1.2 → mantissa -12.
        let d = Decimal::from_i128_with_scale(-125, 2);
        assert_eq!(d.try_to_i128_mantissa(1), Some(-12));
    }

    #[test]
    fn scale_down_half_even_round_up_negative() {
        // -1.35 → -1.4 → mantissa -14.
        let d = Decimal::from_i128_with_scale(-135, 2);
        assert_eq!(d.try_to_i128_mantissa(1), Some(-14));
    }

    // ---- scale-down non-tie rounding ----

    #[test]
    fn scale_down_above_half_rounds_up() {
        // 1.250001 (scale=6) at target_scale=1 → 13 (above the 1.25 tie).
        let d = Decimal::from_i128_with_scale(1_250_001, 6);
        assert_eq!(d.try_to_i128_mantissa(1), Some(13));
    }

    #[test]
    fn scale_down_below_half_rounds_down() {
        // 1.249999 (scale=6) at target_scale=1 → 12 (below the 1.25 tie).
        let d = Decimal::from_i128_with_scale(1_249_999, 6);
        assert_eq!(d.try_to_i128_mantissa(1), Some(12));
    }

    // ---- scale-up overflow (rejects via checked_mul) ----

    #[test]
    fn scale_up_overflow_returns_none() {
        // rust_decimal's mantissa is 96-bit, capped at 2^96 - 1 ≈ 7.92e28,
        // so we cannot construct `i128::MAX / 10 + 1` directly. The same
        // `checked_mul` overflow path is exercised by any mantissa whose
        // |m| × 10^diff exceeds i128::MAX (~1.7e38).
        //
        // 2^96 - 1 multiplied by 10^10 ≈ 7.92e38 — well past i128::MAX, so
        // `checked_mul` returns None.
        let m: i128 = 79_228_162_514_264_337_593_543_950_335;
        let d = Decimal::from_i128_with_scale(m, 0);
        assert_eq!(d.try_to_i128_mantissa(10), None);
    }

    // ---- precision band guard (the new behaviour) ----
    //
    // The previous implementation accepted any rescaled mantissa that fit
    // in i128, which meant values in `[10^38, i128::MAX]` could slip past
    // the `Decimal(38, _)` precision contract polars requires. The guard
    // now rejects that band. Note that constructing a rust_decimal value
    // that *naturally* lands in this band is impossible: the 96-bit
    // mantissa cap (~7.92e28) means scale-down stays under 10^38, and
    // every scale-up large enough to enter the band overflows
    // `checked_mul` first. The guard is therefore a defence-in-depth
    // check that becomes load-bearing only if rust_decimal ever widens
    // its mantissa.
    //
    // We still pin the boundaries on `bigdecimal` (where the band is
    // reachable), and we sanity-check that values comfortably under the
    // band continue to round-trip.

    #[test]
    fn rescaled_mantissa_just_below_10pow38_is_accepted() {
        // 7.92e28 * 10^9 ≈ 7.92e37, comfortably under 10^38.
        let m: i128 = 79_228_162_514_264_337_593_543_950_335;
        let d = Decimal::from_i128_with_scale(m, 0);
        let got = d.try_to_i128_mantissa(9).expect("should fit");
        assert!(got.unsigned_abs() < MAX_I128_MANTISSA.cast_unsigned());
    }

    #[test]
    fn negative_scale_up_overflow_returns_none() {
        // Same overflow path as `scale_up_overflow_returns_none` but
        // exercising the negative branch.
        let m: i128 = -79_228_162_514_264_337_593_543_950_335;
        let d = Decimal::from_i128_with_scale(m, 0);
        assert_eq!(d.try_to_i128_mantissa(10), None);
    }
}

// ===================================================================
// bigdecimal backend
// ===================================================================

#[cfg(feature = "bigdecimal")]
mod bigdecimal_backend {
    use super::{Decimal128Encode, MAX_I128_MANTISSA};
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    // ---- source_scale == target_scale ----

    #[test]
    fn same_scale_small_positive() {
        let d = BigDecimal::from_str("123.456").unwrap();
        assert_eq!(d.try_to_i128_mantissa(3), Some(123_456));
    }

    #[test]
    fn same_scale_small_negative() {
        let d = BigDecimal::from_str("-123.456").unwrap();
        assert_eq!(d.try_to_i128_mantissa(3), Some(-123_456));
    }

    #[test]
    fn same_scale_zero() {
        let d = BigDecimal::from_str("0.0000").unwrap();
        assert_eq!(d.try_to_i128_mantissa(4), Some(0));
    }

    #[test]
    fn same_scale_large_mantissa() {
        // 10^37 — large but still safely under the 10^38 precision cap.
        let s = format!("{}", 10_i128.pow(37));
        let d = BigDecimal::from_str(&s).unwrap();
        assert_eq!(d.try_to_i128_mantissa(0), Some(10_i128.pow(37)));
    }

    // ---- scale-up (rescale to higher scale) ----

    #[test]
    fn scale_up_one_to_ten() {
        let d = BigDecimal::from_str("1.5").unwrap();
        assert_eq!(d.try_to_i128_mantissa(10), Some(15_000_000_000));
    }

    // ---- scale-down half-even ties ----

    #[test]
    fn scale_down_half_even_round_down_positive() {
        let d = BigDecimal::from_str("1.25").unwrap();
        assert_eq!(d.try_to_i128_mantissa(1), Some(12));
    }

    #[test]
    fn scale_down_half_even_round_up_positive() {
        let d = BigDecimal::from_str("1.35").unwrap();
        assert_eq!(d.try_to_i128_mantissa(1), Some(14));
    }

    #[test]
    fn scale_down_half_even_round_down_negative() {
        let d = BigDecimal::from_str("-1.25").unwrap();
        assert_eq!(d.try_to_i128_mantissa(1), Some(-12));
    }

    #[test]
    fn scale_down_half_even_round_up_negative() {
        let d = BigDecimal::from_str("-1.35").unwrap();
        assert_eq!(d.try_to_i128_mantissa(1), Some(-14));
    }

    // ---- scale-down non-tie rounding ----

    #[test]
    fn scale_down_above_half_rounds_up() {
        let d = BigDecimal::from_str("1.250001").unwrap();
        assert_eq!(d.try_to_i128_mantissa(1), Some(13));
    }

    #[test]
    fn scale_down_below_half_rounds_down() {
        let d = BigDecimal::from_str("1.249999").unwrap();
        assert_eq!(d.try_to_i128_mantissa(1), Some(12));
    }

    // ---- precision boundary (10^38) ----

    #[test]
    fn boundary_just_below_10pow38_is_accepted() {
        // 10^38 - 1 — the largest 38-digit mantissa, accepted.
        let m = MAX_I128_MANTISSA - 1;
        let s = format!("{m}");
        let d = BigDecimal::from_str(&s).unwrap();
        assert_eq!(d.try_to_i128_mantissa(0), Some(m));
    }

    #[test]
    fn boundary_at_10pow38_is_rejected() {
        // 10^38 — strictly excluded by polars's `dec128_fits`.
        let s = format!("{MAX_I128_MANTISSA}");
        let d = BigDecimal::from_str(&s).unwrap();
        assert_eq!(d.try_to_i128_mantissa(0), None);
    }

    #[test]
    fn negative_boundary_just_below_10pow38_is_accepted() {
        let m = -(MAX_I128_MANTISSA - 1);
        let s = format!("{m}");
        let d = BigDecimal::from_str(&s).unwrap();
        assert_eq!(d.try_to_i128_mantissa(0), Some(m));
    }

    #[test]
    fn negative_boundary_at_10pow38_is_rejected() {
        let s = format!("-{MAX_I128_MANTISSA}");
        let d = BigDecimal::from_str(&s).unwrap();
        assert_eq!(d.try_to_i128_mantissa(0), None);
    }

    // ---- zero with non-zero target_scale ----

    #[test]
    fn zero_at_non_zero_target_scale_succeeds() {
        let d = BigDecimal::from_str("0").unwrap();
        assert_eq!(d.try_to_i128_mantissa(5), Some(0));
    }

    // ---- negative zero ----

    #[test]
    fn negative_zero_collapses_to_positive_zero() {
        // bigdecimal accepts "-0" and `with_scale_round` normalises the
        // sign of the resulting BigInt mantissa to non-negative zero, so
        // the encoded mantissa equals the encoding of "+0".
        let pos = BigDecimal::from_str("0").unwrap();
        let neg = BigDecimal::from_str("-0").unwrap();
        let pos_m = pos.try_to_i128_mantissa(3).expect("+0 encodes");
        let neg_m = neg.try_to_i128_mantissa(3).expect("-0 encodes");
        assert_eq!(pos_m, 0);
        assert_eq!(neg_m, 0);
        assert_eq!(pos_m, neg_m);
    }
}
