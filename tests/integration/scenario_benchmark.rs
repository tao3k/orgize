#[test]
fn orgize_rule_fixtures_have_scenario_benchmarks() {
    asp_rust::assert_rule_fixture_scenario_benchmarks(env!("CARGO_MANIFEST_DIR"));
}
