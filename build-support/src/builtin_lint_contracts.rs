//! Build-time manifest generation for built-in lint `CONTRACT_ORG` files.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Scans built-in lint `CONTRACT_ORG` files and writes the generated source manifest.
pub fn write_builtin_lint_contract_manifest() {
    let contract_dir = Path::new("contracts").join("builtin-lint");
    println!("cargo:rerun-if-changed={}", contract_dir.display());

    let contract_paths = builtin_lint_contract_paths(&contract_dir);
    let generated = builtin_lint_contract_manifest_source(contract_paths);
    let output_path = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"))
        .join("builtin_lint_contracts.rs");
    fs::write(&output_path, generated).unwrap_or_else(|error| {
        panic!(
            "failed to write builtin lint contract manifest `{}`: {error}",
            output_path.display()
        )
    });
}

fn builtin_lint_contract_paths(contract_dir: &Path) -> Vec<PathBuf> {
    let mut contract_paths = fs::read_dir(contract_dir)
        .unwrap_or_else(|error| {
            panic!(
                "failed to read builtin lint contract directory `{}`: {error}",
                contract_dir.display()
            )
        })
        .map(|entry| {
            entry
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to read builtin lint contract directory entry in `{}`: {error}",
                        contract_dir.display()
                    )
                })
                .path()
        })
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("org"))
        .collect::<Vec<_>>();

    contract_paths.sort();
    assert!(
        !contract_paths.is_empty(),
        "builtin lint contract directory `{}` must contain at least one .org file",
        contract_dir.display()
    );
    contract_paths
}

fn builtin_lint_contract_manifest_source(contract_paths: Vec<PathBuf>) -> String {
    let mut generated =
        String::from("pub const BUILTIN_LINT_CONTRACT_SOURCES: &[(&str, &str)] = &[\n");
    for path in contract_paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let name = path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_else(|| {
                panic!(
                    "builtin lint contract path `{}` must have a UTF-8 file name",
                    path.display()
                )
            });
        let source = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read builtin lint contract `{}`: {error}",
                path.display()
            )
        });

        generated.push_str("    (");
        generated.push_str(&format!("{name:?}"));
        generated.push_str(", ");
        generated.push_str(&format!("{source:?}"));
        generated.push_str("),\n");
    }
    generated.push_str("];\n");
    generated
}
