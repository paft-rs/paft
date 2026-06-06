#[test]
fn generic_snapshot_is_a_root_export() {
    fn assert_export<T>() {}

    assert_export::<paft_aggregates::GenericSnapshot>();
}

#[test]
fn standard_snapshot_alias_implements_eq() {
    fn assert_eq_impl<T: Eq>() {}

    assert_eq_impl::<paft_aggregates::Snapshot>();
}

#[test]
fn generic_snapshot_accepts_partial_eq_only_metadata() {
    fn assert_partial_eq_impl<T: PartialEq>() {}

    assert_partial_eq_impl::<paft_aggregates::GenericSnapshot<f64>>();
}
