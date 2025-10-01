use paft_domain::{DomainError, Figi, Isin};
use serde_json::{from_str, to_string};

#[test]
fn isin_normalizes_display_and_as_ref() {
    let isin = Isin::new("us0378331005").expect("valid ISIN");
    assert_eq!(isin.as_ref(), "US0378331005");
    assert_eq!(isin.to_string(), "US0378331005");
}

#[test]
fn isin_serde_roundtrip() {
    let isin = Isin::new("US0378331005").expect("valid ISIN");
    let json = to_string(&isin).expect("serialize");
    assert_eq!(json, "\"US0378331005\"");
    let back: Isin = from_str(&json).expect("deserialize");
    assert_eq!(back, isin);
}

#[cfg(feature = "isin-validate")]
mod isin_strict {
    use super::*;

    #[test]
    fn rejects_bad_checksum() {
        let err = Isin::new("US0378331006").expect_err("checksum should fail");
        assert!(matches!(err, DomainError::InvalidIsin { .. }));
    }

    #[test]
    fn accepts_loose_input_with_scrubbing() {
        let isin = Isin::new(" us-037833-1005 \t").expect("valid loose input");
        assert_eq!(isin.as_ref(), "US0378331005");
    }
}

#[cfg(not(feature = "isin-validate"))]
mod isin_lenient {
    use super::*;

    #[test]
    fn accepts_non_empty_scrubbed_input() {
        let isin = Isin::new(" invalid-value ").expect("scrubbed value allowed");
        assert_eq!(isin.as_ref(), "INVALIDVALUE");
    }

    #[test]
    fn rejects_empty_after_scrub() {
        let err = Isin::new(" --- ").expect_err("scrubbed empty should fail");
        assert!(matches!(err, DomainError::InvalidIsin { .. }));
    }
}

#[test]
fn figi_uppercases_and_trims() {
    let figi = Figi::new(" bbg000b9xry4 ").expect("valid FIGI");
    assert_eq!(figi.as_ref(), "BBG000B9XRY4");
    assert_eq!(figi.to_string(), "BBG000B9XRY4");
}

#[test]
fn figi_serde_roundtrip() {
    let figi = Figi::new("BBG000B9XRY4").expect("valid FIGI");
    let json = to_string(&figi).expect("serialize");
    assert_eq!(json, "\"BBG000B9XRY4\"");
    let back: Figi = from_str(&json).expect("deserialize");
    assert_eq!(back, figi);
}

#[test]
fn figi_rejects_invalid_length() {
    let err = Figi::new("BBG000B9XRY").expect_err("length must be 12");
    assert!(matches!(err, DomainError::InvalidFigi { .. }));
}

#[test]
fn figi_rejects_non_alphanumeric() {
    let err = Figi::new("BBG000B9XRY!").expect_err("non-alphanumeric fails");
    assert!(matches!(err, DomainError::InvalidFigi { .. }));
}

#[cfg(feature = "figi-validate")]
mod figi_strict {
    use super::*;

    #[test]
    fn rejects_bad_checksum() {
        let err = Figi::new("BBG000B9XRY5").expect_err("checksum should fail");
        assert!(matches!(err, DomainError::InvalidFigi { .. }));
    }
}

#[cfg(not(feature = "figi-validate"))]
mod figi_lenient {
    use super::*;

    #[test]
    fn allows_unchecked_checksum() {
        let figi = Figi::new("BBG000B9XRY5").expect("checksum ignored without feature");
        assert_eq!(figi.as_ref(), "BBG000B9XRY5");
    }
}
