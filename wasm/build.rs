use std::process::Command;

fn main() {
    let config = rust_lang_project_harness::default_rust_harness_config()
        .with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "build.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .without_verification_tasks()
            .with_rationale(
                "orgize-wasm owns the browser demo bindings and must mount the Rust project harness from its build script so filtered cargo test runs cannot bypass blocking policy",
            ),
        );
    rust_lang_project_harness::assert_rust_project_harness_build_clean_from_env_with_config(
        &config,
    );

    {
        let output = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .unwrap();

        let git_hash = String::from_utf8(output.stdout).unwrap();

        println!("cargo:rustc-env=CARGO_GIT_HASH={}", git_hash);
    }

    {
        let output = Command::new("date").args(["-R"]).output().unwrap();

        let git_hash = String::from_utf8(output.stdout).unwrap();

        println!("cargo:rustc-env=CARGO_BUILD_TIME={}", git_hash);
    }
}
