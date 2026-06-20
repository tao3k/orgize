fn main() {
    orgize_build_support::write_builtin_lint_contract_manifest();

    let mut config = rust_lang_project_harness::default_rust_harness_config()
        .with_rule_severity(
            "RUST-MOD-R002",
            rust_lang_project_harness::RustDiagnosticSeverity::Info,
        )
        .with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "build.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .without_verification_tasks()
            .with_rationale(
                "orgize parser-v2 mounts the Rust project harness during build-script execution so filtered cargo test runs cannot bypass blocking policy",
            ),
        )
        .with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lint_file_links.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PureDomainLogic],
            )
            .without_verification_tasks()
            .with_rationale(
                "orgize file-link lint owns local Org AST and path-token policy, including portable skill-package references; integration tests cover the rule without external verification skills",
            ),
        )
        .with_cargo_check_advice_allow_explanation(
            "scope=orgize parser-v2 build.rs cargo-check advisory findings; owner=orgize build harness; finding_category=agent-policy advisory findings; why_safe_now=blocking compiler errors still fail and this slice only aligns rust harness dependency revisions; cleanup_trigger=dedicated orgize parser-v2 API-compatible lint cleanup slice before promoting advisory findings",
        );
    config.ignored_dir_names.insert(".devenv".to_string());
    config.ignored_dir_names.insert(".data".to_string());

    rust_lang_project_harness::assert_rust_project_harness_cargo_check_clean_from_env_with_config(
        &config,
    );
}
