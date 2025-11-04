use paft_market::requests::history::Interval;

#[test]
fn interval_intraday_detection() {
    let intraday = [
        Interval::I1s,
        Interval::I2s,
        Interval::I3s,
        Interval::I5s,
        Interval::I6s,
        Interval::I10s,
        Interval::I15s,
        Interval::I30s,
        Interval::I90s,
        Interval::I1m,
        Interval::I2m,
        Interval::I3m,
        Interval::I5m,
        Interval::I6m,
        Interval::I10m,
        Interval::I15m,
        Interval::I30m,
        Interval::I90m,
        Interval::I1h,
        Interval::I2h,
        Interval::I3h,
        Interval::I4h,
        Interval::I6h,
        Interval::I8h,
        Interval::I12h,
    ];

    for interval in intraday {
        assert!(interval.is_intraday(), "{interval:?} should be intraday");
    }

    let non_intraday = [Interval::D1, Interval::W1, Interval::M1, Interval::Y1];

    for interval in non_intraday {
        assert!(
            !interval.is_intraday(),
            "{interval:?} should not be intraday"
        );
    }
}

#[test]
fn interval_minutes_seconds() {
    assert_eq!(Interval::I1s.minutes(), None);
    assert_eq!(Interval::I1s.seconds(), Some(1));

    assert_eq!(Interval::I90s.minutes(), None);
    assert_eq!(Interval::I90s.seconds(), Some(90));

    assert_eq!(Interval::I1m.minutes(), Some(1));
    assert_eq!(Interval::I1m.seconds(), Some(60));

    assert_eq!(Interval::I90m.minutes(), Some(90));
    assert_eq!(Interval::I90m.seconds(), Some(5_400));

    assert_eq!(Interval::I1h.minutes(), Some(60));
    assert_eq!(Interval::I1h.seconds(), Some(3_600));

    assert_eq!(Interval::I12h.minutes(), Some(720));
    assert_eq!(Interval::I12h.seconds(), Some(43_200));

    assert_eq!(Interval::D1.minutes(), None);
    assert_eq!(Interval::D1.seconds(), None);
}
