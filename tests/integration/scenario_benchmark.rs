fn orgize_rule_fixtures_have_scenario_benchmarks() {
    asp_rust::assert_rule_fixture_scenario_benchmarks(env!("CARGO_MANIFEST_DIR"));
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[(
    "scenario_benchmark::orgize_rule_fixtures_have_scenario_benchmarks",
    orgize_rule_fixtures_have_scenario_benchmarks,
)];
