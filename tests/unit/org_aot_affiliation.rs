#[test]
fn empty_affiliation_plan_needs_no_native_runtime() {
    assert!(super::project(&[], "").is_empty());
}
