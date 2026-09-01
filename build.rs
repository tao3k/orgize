fn main() {
    orgize_build_support::write_builtin_lint_contract_manifest();
    asp_rust_project_harness_policy::assert_asp_workspace_build_identity_from_env();
}
