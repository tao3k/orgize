fn main() {
    orgize_build_support::write_builtin_lint_contract_manifest();
    orgize_build_support::write_source_revision();
    orgize_build_support::write_org_aot_functions();
    orgize_build_support::write_org_aot_events();
}
