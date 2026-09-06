//! Exercise the Rust helpers against results independently computed using
//! Python's arbitrary-precision Fraction. See oracle/generate.py to regenerate.

use paft_decimal::{
    Decimal, checked_add_exact, checked_div_exact, checked_mul_exact, checked_sub_exact,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Parts(String, u32);

impl Parts {
    fn decimal(&self) -> Decimal {
        Decimal::try_from_i128_with_scale(self.0.parse().unwrap(), self.1).unwrap()
    }
}

#[derive(Deserialize)]
struct Case {
    lhs: Parts,
    rhs: Parts,
    add: Option<Parts>,
    sub: Option<Parts>,
    mul: Option<Parts>,
    div: Option<Parts>,
}

type Operation = fn(&Decimal, &Decimal) -> Option<Decimal>;

#[test]
fn rust_arithmetic_matches_independent_rational_oracle() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("oracle/cases.json")).unwrap();
    let mut successes = [0; 4];
    let mut rejections = [0; 4];
    for (index, case) in cases.iter().enumerate() {
        let lhs = case.lhs.decimal();
        let rhs = case.rhs.decimal();
        for (operation_index, (name, operation, expected)) in [
            ("add", checked_add_exact as Operation, &case.add),
            ("sub", checked_sub_exact as Operation, &case.sub),
            ("mul", checked_mul_exact as Operation, &case.mul),
            ("div", checked_div_exact as Operation, &case.div),
        ]
        .into_iter()
        .enumerate()
        {
            let expected = expected.as_ref().map(Parts::decimal);
            assert_eq!(
                operation(&lhs, &rhs),
                expected,
                "case {index}: {lhs} {name} {rhs}"
            );
            if expected.is_some() {
                successes[operation_index] += 1;
            } else {
                rejections[operation_index] += 1;
            }
        }
    }
    for index in 0..4 {
        assert!(
            successes[index] >= 100,
            "exercise exact successes for every operation"
        );
        assert!(
            rejections[index] >= 100,
            "exercise representability failures for every operation"
        );
    }
}
