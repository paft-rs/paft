#[test]
fn generic_snapshot_is_a_root_export() {
    fn assert_export<T>() {}

    assert_export::<paft_aggregates::GenericSnapshot>();
}
