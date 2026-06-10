fn main() {
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
        .with_cargo_check_advice_allow_explanation(
            "orgize keeps existing parser-v2 public row and selector internals stable during dependency rev bumps; agent-policy advisory repairs require a dedicated API-compatible slice",
        );
    config.ignored_dir_names.insert(".devenv".to_string());
    config.ignored_dir_names.insert(".data".to_string());

    rust_lang_project_harness::assert_rust_project_harness_cargo_check_clean_from_env_with_config(
        &config,
    );
}
